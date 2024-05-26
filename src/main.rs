#[allow(unused_imports)]
use std::io::{self, Write};

use anyhow::Result;

pub(crate) struct Shell {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Shell {
    pub(crate) fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        write!(self.stdout, "$ ")?;
        self.stdout.flush()?;

        let mut input = String::new();
        self.stdin.read_line(&mut input)?;
        self.process(input)?;

        Ok(())
    }

    fn process(&mut self, input: String) -> Result<()> {
        write!(self.stdout, "{input}: command not found")?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
