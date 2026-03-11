use std::io::{self, Write};
mod builtin;
mod command;
mod parser;
mod utils;

use command::ShellCommand;
pub use utils::{CommandOutput, ShellPath};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Repl {
    stdin: io::Stdin,
    stdout: io::Stdout,
    stderr: io::Stderr,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let sh_path = ShellPath::new()?;

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
                    CommandOutput::Empty => {
                        self.stdout
                            .write(b"\n")
                            .context("writing empty response to stdout")?;
                    }
                }
            }
        }
    }

    fn input(&mut self) -> Result<Option<ShellCommand>> {
        let mut input = String::new();

        self.stdout.write(b"$ ").context("writing prompt")?;
        self.stdout.flush().context("flushing stdout")?;
        self.stdin.read_line(&mut input).context("reading stdin")?;

        let (_, input) = parser::parse(input.as_bytes())
            .map_err(|e| anyhow::anyhow!("parse error: {e}"))
            .context("parsing input")?;

        if input.is_empty() {
            return Ok(None);
        } else {
            Ok(Some(ShellCommand::new(input)))
        }
    }
}
