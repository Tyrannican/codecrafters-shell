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

        Ok(Self {
            stdout: io::stdout(),
            stderr: io::stderr(),
            stdin,
            shellpath,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let sh_path = ShellPath::new()?;
        let completer = Some(ShellPathCompleter::new(sh_path.clone()));
        self.stdin.set_helper(completer);
        self.stdin
            .set_completion_type(rustyline::CompletionType::List);

        loop {
            if let Some((args, redirects)) = self.input().context("reading user input")? {
                if let Some(redirects) = redirects {
                    let (op, redirect_args) = (RedirectOp::parse(&redirects[0]), &redirects[1..]);

                    match op {
                        Some(RedirectOp::Pipe) => {
                            let pipeline = ShellPipeline::new(&args, redirect_args);
                            let outputs = pipeline.execute().context("executing pipeline")?;
                            self.write(outputs).context("writing to stdout/stderr")?;
                        }
                        Some(op) => {
                            let command = ShellCommand::new(args);
                            let outputs = command.execute(&self.shellpath).with_context(|| {
                                format!(
                                    "executing command `{}` with arguments: {:?}",
                                    command.name, command.args
                                )
                            })?;
                            let outputs = file::redirect_output(op, redirect_args, outputs)?;
                            self.write(outputs).context("writing to stdout/stderr")?;
                        }
                        None => {
                            let command = ShellCommand::new(args);
                            self.execute_command(command)
                                .context("executing command - no redirect found")?;
                        }
                    }
                } else {
                    let command = ShellCommand::new(args);
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
