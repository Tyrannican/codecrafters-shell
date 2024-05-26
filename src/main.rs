#[allow(unused_imports)]
use std::io::{self, Write};

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ShellBuiltin {
    Echo,
    Exit,
    Type,
}

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

pub(crate) fn is_builtin(name: &str) -> Option<ShellBuiltin> {
    match name {
        "echo" => Some(ShellBuiltin::Echo),
        "exit" => Some(ShellBuiltin::Exit),
        "type" => Some(ShellBuiltin::Type),
        _ => None,
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
            self.exec(command)?;
        }
    }

    fn exec(&mut self, command: Command) -> Result<()> {
        if let Some(builtin) = is_builtin(&command.name) {
            self.exec_builtin(builtin, &command)?;
        } else {
            writeln!(self.stdout, "{}: command not found", command.name)?;
        }

        self.stdout.flush()?;

        Ok(())
    }

    fn exec_builtin(&mut self, builtin: ShellBuiltin, command: &Command) -> Result<()> {
        match builtin {
            ShellBuiltin::Exit => {
                let code = command.args[0].parse::<i32>()?;
                std::process::exit(code);
            }
            ShellBuiltin::Echo => {
                let output = command.args.join(" ");
                writeln!(self.stdout, "{output}")?;
            }
            ShellBuiltin::Type => {
                let type_arg = &command.args[0];
                if is_builtin(type_arg).is_some() {
                    writeln!(self.stdout, "{type_arg} is a shell builtin")?;
                } else {
                    writeln!(self.stdout, "{type_arg} not found")?;
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
