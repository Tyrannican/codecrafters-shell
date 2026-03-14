use std::io::{self, Write};
mod builtin;
mod command;
mod file;
mod parser;
mod utils;

use command::{ShellCommand, ShellPipeline};
use rustyline::{Editor, config::Configurer, error::ReadlineError, history::FileHistory};
pub use utils::{CommandOutput, OutputPair, RedirectOp, ShellPath, ShellPathCompleter, split_args};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Repl {
    history: Option<String>,
    stdout: io::Stdout,
    stderr: io::Stderr,
    stdin: Editor<ShellPathCompleter, FileHistory>,
    shellpath: ShellPath,
}

impl Repl {
    pub fn new() -> Result<Self> {
        let mut stdin = Editor::<ShellPathCompleter, FileHistory>::new()
            .map_err(|e| anyhow::anyhow!("rustyline editor error: {e}"))
            .context("creating rustyline editor")?;

        let shellpath = ShellPath::new().context("loading shellpath")?;
        let completer = Some(ShellPathCompleter::new(shellpath.clone()));
        stdin.set_helper(completer);
        stdin.set_completion_type(rustyline::CompletionType::List);

        let history_file = std::env::var("HISTFILE").ok();
        if let Some(ref path) = history_file {
            stdin
                .load_history(&path)
                .context("loading history from HISTFILE")?;
        }

        Ok(Self {
            history: history_file,
            stdout: io::stdout(),
            stderr: io::stderr(),
            stdin,
            shellpath,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if let Some((args, redirects)) = self.input().context("reading user input")? {
                let history = self
                    .stdin
                    .history()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                if let Some(redirects) = redirects {
                    let (op, redirect_args) = (RedirectOp::parse(&redirects[0]), &redirects[1..]);

                    if let Some(RedirectOp::Pipe) = op {
                        let pipeline = ShellPipeline::new(&args, redirect_args);
                        let outputs = pipeline
                            .execute(&self.shellpath, &history)
                            .context("executing pipeline")?;

                        self.write(outputs).context("writing to stdout/stderr")?;
                    } else {
                        let command = ShellCommand::new(args, history);
                        match op {
                            Some(op) => {
                                let outputs =
                                    command.execute(&self.shellpath).with_context(|| {
                                        format!(
                                            "executing command `{}` with arguments: {:?}",
                                            command.name, command.args
                                        )
                                    })?;
                                let outputs = file::redirect_output(op, redirect_args, outputs)?;
                                self.write(outputs).context("writing to stdout/stderr")?;
                            }
                            None => {
                                self.execute_command(command)
                                    .context("executing command - no redirect found")?;
                            }
                        }
                    }
                } else {
                    let command = ShellCommand::new(args, history);
                    self.execute_command(command)
                        .context("executing command - no redirect present")?;
                }
            }
        }
    }

    fn execute_command(&mut self, command: ShellCommand) -> Result<()> {
        let outputs = command.execute(&self.shellpath).with_context(|| {
            format!(
                "executing command `{}` with arguments: {:?}",
                command.name, command.args
            )
        })?;

        self.write(outputs).context("writing to stdout/stderr")
    }

    fn write(&mut self, outputs: OutputPair) -> Result<()> {
        let (stdout, stderr) = outputs;
        if let CommandOutput::Stdout(out) = stdout {
            self.stdout.write(&out).context("writing to stdout")?;
        }

        if let CommandOutput::Stderr(err) = stderr {
            self.stderr.write(&err).context("writing to stderr")?;
        }

        Ok(())
    }

    fn input(&mut self) -> Result<Option<(Vec<String>, Option<Vec<String>>)>> {
        match self.stdin.readline("$ ") {
            Ok(input) => {
                self.stdin
                    .add_history_entry(&input)
                    .with_context(|| format!("adding history entry: {input}"))?;

                let (_, input) = parser::parse(input.as_bytes())
                    .map_err(|e| anyhow::anyhow!("parse error: {e}"))
                    .context("parsing input")?;

                if input.is_empty() {
                    return Ok(None);
                } else {
                    let args = split_args(&input);
                    Ok(Some(args))
                }
            }
            Err(ReadlineError::Interrupted) => {
                if let Some(path) = &self.history {
                    self.stdin.save_history(&path).context("saving history")?;
                }
                std::process::exit(0);
            }
            Err(ReadlineError::Eof) => {
                eprintln!("^D");
                return Ok(None);
            }
            _ => anyhow::bail!("error parsing input"),
        }
    }
}
