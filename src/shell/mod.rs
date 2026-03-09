use std::io::{self, Write};
mod builtin;
mod command;

use command::ShellCommand;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Repl {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let command = self.input().context("reading user input")?;
            let result = command.execute().with_context(|| {
                format!(
                    "executing command `{}` with arguments: `{:?}`",
                    command.name, command.args
                )
            })?;
            self.stdout
                .write(result.as_bytes())
                .context("writing to stdout")?;
        }
    }

    fn input(&mut self) -> Result<ShellCommand> {
        let mut input = String::new();

        self.stdout.write(b"$ ").context("writing prompt")?;
        self.stdout.flush().context("flushing stdout")?;
        self.stdin.read_line(&mut input).context("reading stdin")?;
        Ok(ShellCommand::new(input.trim().to_string()))
    }
}
