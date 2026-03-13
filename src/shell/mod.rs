use std::io::{self, Write};
mod builtin;
mod command;
mod file;
mod parser;
mod utils;

use command::ShellCommand;
use rustyline::{Editor, config::Configurer, error::ReadlineError, history::FileHistory};
pub use utils::{CommandOutput, OutputPair, RedirectOp, ShellPath, ShellPathCompleter};

use anyhow::{Context, Result};

const REDIRECT_OPS: [&str; 7] = ["1>", ">", "1>>", ">>", "2>", "2>>", "|"];

#[derive(Debug)]
pub struct Repl {
    stdout: io::Stdout,
    stderr: io::Stderr,
    stdin: Editor<ShellPathCompleter, FileHistory>,
    shellpath: ShellPath,
}

impl Repl {
    pub fn new() -> Result<Self> {
        let mut stdin = Editor::<ShellPathCompleter, FileHistory>::new()?;
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
                let command = ShellCommand::new(args);
                if let Some(redirects) = redirects {
                    let (op, redirect_args) = (RedirectOp::parse(&redirects[0]), &redirects[1..]);

                    match op {
                        Some(RedirectOp::Pipe) => todo!(),
                        Some(op) => {
                            let outputs = command.execute(&self.shellpath).with_context(|| {
                                format!(
                                    "executing command `{}` with arguments: {:?}",
                                    command.name, command.args
                                )
                            })?;
                            let outputs = file::redirect_output(op, redirect_args, outputs)?;
                            self.write(outputs).context("writing to stdout/stderr")?;
                        }
                        None => self
                            .execute_command(command)
                            .context("executing command - no redirect found")?,
                    }
                } else {
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

fn split_args(args: &[String]) -> (Vec<String>, Option<Vec<String>>) {
    if let Some(idx) = args
        .iter()
        .rposition(|arg| REDIRECT_OPS.contains(&arg.as_str()))
    {
        let (args, redirect) = args.split_at(idx);
        if redirect.len() < 2 {
            return (args.to_vec(), None);
        } else {
            return (args.to_vec(), Some(redirect.to_vec()));
        }
    }

    (args.to_vec(), None)
}
