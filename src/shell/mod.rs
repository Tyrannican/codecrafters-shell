use std::io::{self, Write};
mod builtin;
mod command;
mod parser;
mod utils;

use command::ShellCommand;
use rustyline::{Editor, config::Configurer, error::ReadlineError, history::FileHistory};
pub use utils::{CommandOutput, ShellPath, ShellPathCompleter};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Repl {
    stdout: io::Stdout,
    stderr: io::Stderr,
    stdin: Editor<ShellPathCompleter, FileHistory>,
}

impl Repl {
    pub fn new() -> Self {
        let stdin = Editor::<ShellPathCompleter, FileHistory>::new().expect("error loading reader");

        Self {
            stdout: io::stdout(),
            stderr: io::stderr(),
            stdin,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let sh_path = ShellPath::new()?;
        let completer = Some(ShellPathCompleter::new(sh_path.clone()));
        self.stdin.set_helper(completer);
        self.stdin
            .set_completion_type(rustyline::CompletionType::List);

        loop {
            if let Some(command) = self.input().context("reading user input")? {
                let result = command.execute(&sh_path).with_context(|| {
                    format!(
                        "executing command `{}` with arguments: `{:?}`",
                        command.name, command.args
                    )
                })?;
                match result {
                    CommandOutput::Stdout(out) => {
                        self.stdout.write(&out).context("writing to stdout")?;
                    }
                    CommandOutput::Stderr(err) => {
                        self.stderr.write(&err).context("writing to stderr")?;
                    }
                    CommandOutput::Empty => {}
                }
            }
        }
    }

    fn input(&mut self) -> Result<Option<ShellCommand>> {
        let line = self.stdin.readline("$ ");
        match line {
            Ok(input) => {
                let (_, input) = parser::parse(input.as_bytes())
                    .map_err(|e| anyhow::anyhow!("parse error: {e}"))
                    .context("parsing input")?;

                if input.is_empty() {
                    return Ok(None);
                } else {
                    Ok(Some(ShellCommand::new(input)))
                }
            }
            Err(ReadlineError::Interrupted) => {
                std::process::exit(0);
                // println!("^C");
                // return Ok(None);
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                return Ok(None);
            }
            _ => panic!("error parsing input"),
        }
    }
}
