#[allow(unused_imports)]
use std::io::{self, Write};

use anyhow::Result;

pub(crate) struct Command {
    name: String,
    args: Vec<String>,
}

impl Command {
    pub(crate) fn new(input: String) -> Self {
        let input = input.trim();

        let (name, args) = match input.split_once(' ') {
            Some((name, rest)) => {
                let args = rest
                    .split(' ')
                    .filter_map(|arg| {
                        let arg = arg.to_string();
                        if arg.is_empty() {
                            return None;
                        }
                        Some(arg)
                    })
                    .collect::<Vec<String>>();

                (name, args)
            }
            None => (input, vec![]),
        };

        Self {
            name: name.to_string(),
            args,
        }
    }
}

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
        loop {
            write!(self.stdout, "$ ")?;
            self.stdout.flush()?;

            let mut input = String::new();
            self.stdin.read_line(&mut input)?;

            let command = Command::new(input);
            self.process(command)?;
        }
    }

    fn process(&mut self, command: Command) -> Result<()> {
        match command.name.as_str() {
            "exit" => {
                let code = command.args[0].parse::<i32>()?;
                std::process::exit(code);
            }
            "echo" => {
                let output = command.args.join(" ");
                writeln!(self.stdout, "{output}")?;
            }
            _ => writeln!(self.stdout, "{}: command not found", command.name)?,
        }
        self.stdout.flush()?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
