use crate::shell::{ShellPath, builtin::ShellBuiltin, utils::CommandOutput};
use anyhow::{Context, Result};
use std::{fs::File, io::Write, path::PathBuf};

type StdPair = (CommandOutput, CommandOutput);

const REDIRECT_ARGS: [&str; 7] = ["1>", ">", "1>>", ">>", "2>", "2>>", "|"];

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
}

impl ShellCommand {
    pub fn new(mut input: Vec<String>) -> Self {
        let name = input.remove(0);
        Self { name, args: input }
    }

    pub fn execute(&self, path: &ShellPath) -> Result<CommandOutput> {
        if self.name.is_empty() {
            return Ok(CommandOutput::Empty);
        }

        let (args, redirects) = self.split_args();
        let outputs = self.run_command(path, args).with_context(|| {
            format!(
                "running command '{}' with arguments {:?}",
                self.name, self.args
            )
        })?;

        if let Some(redirect) = redirects {
            return self.redirect(redirect, outputs);
        }

        let (stdout, stderr) = outputs;
        if stdout != CommandOutput::Empty {
            Ok(stdout)
        } else if stderr != CommandOutput::Empty {
            Ok(stderr)
        } else {
            Ok(CommandOutput::Empty)
        }
    }

    fn redirect(&self, args: &[String], outputs: StdPair) -> Result<CommandOutput> {
        let op = &args[0];
        match op.as_str() {
            ">" | "1>" | ">>" | "1>>" | "2>" | "2>>" => self.redirect_output(op, &args[1], outputs),
            "|" => todo!(),
            _ => unreachable!("impossible to reach"),
        }
    }

    fn redirect_output(
        &self,
        op: impl AsRef<str>,
        file: impl AsRef<str>,
        outputs: StdPair,
    ) -> Result<CommandOutput> {
        let (stdout, stderr) = outputs;
        let path = PathBuf::from(file.as_ref());
        match op.as_ref() {
            ">" | "1>" => {
                if let CommandOutput::Stdout(out) = stdout {
                    std::fs::write(&path, &out)
                        .with_context(|| format!("redirect out - writing to {}", path.display()))?;
                }

                return Ok(stderr);
            }
            ">>" | "1>>" => {
                if let CommandOutput::Stdout(mut out) = stdout {
                    let mut content = std::fs::read(&path)
                        .with_context(|| format!("appending out - reading {}", path.display()))?;

                    content.append(&mut out);
                    std::fs::write(&path, &content).with_context(|| {
                        format!("appending out - writing to {}", path.display())
                    })?;
                }

                return Ok(stderr);
            }
            "2>" => {
                if let CommandOutput::Stderr(err) = stderr {
                    std::fs::write(&path, &err)
                        .with_context(|| format!("redirect err - writing to {}", path.display()))?;
                }

                return Ok(stdout);
            }
            "2>>" => {
                if let CommandOutput::Stderr(mut err) = stderr {
                    let mut content = std::fs::read(&path)
                        .with_context(|| format!("appending err - reading {}", path.display()))?;

                    content.append(&mut err);
                    std::fs::write(&path, &content).with_context(|| {
                        format!("appending err - writing to {}", path.display())
                    })?;
                }

                return Ok(stdout);
            }
            _ => unreachable!("impossible"),
        }
    }

    fn pipe(&self, args: &[String]) {
        todo!()
    }

    fn split_args(&self) -> (&[String], Option<&[String]>) {
        if let Some(idx) = self
            .args
            .iter()
            .rposition(|arg| REDIRECT_ARGS.contains(&arg.as_str()))
        {
            let (args, redirect) = self.args.split_at(idx);
            if redirect.len() < 2 {
                return (args, None);
            } else {
                return (args, Some(redirect));
            }
        }

        (&self.args, None)
    }

    fn run_command(&self, path: &ShellPath, args: &[String]) -> Result<StdPair> {
        if let Some(builtin) = ShellBuiltin::is_builtin(&self.name) {
            let result = builtin.execute(args, path).with_context(|| {
                format!(
                    "executing shell builtin `{:?}` with arguments: `{:?}`",
                    builtin, self.args
                )
            })?;

            match result {
                CommandOutput::Stdout(_) => Ok((result, CommandOutput::Empty)),
                CommandOutput::Stderr(_) => Ok((CommandOutput::Empty, result)),
                CommandOutput::Empty => Ok((CommandOutput::Empty, CommandOutput::Empty)),
            }
        } else {
            match path.find(&self.name) {
                Some(path) => {
                    let output = std::process::Command::new(&self.name)
                        .args(args)
                        .output()
                        .with_context(|| {
                            format!(
                                "executing command {} with args {:?}",
                                path.display(),
                                self.args
                            )
                        })?;

                    let stdout = if output.stdout.is_empty() {
                        CommandOutput::Empty
                    } else {
                        CommandOutput::Stdout(output.stdout)
                    };
                    let stderr = if output.stderr.is_empty() {
                        CommandOutput::Empty
                    } else {
                        CommandOutput::Stderr(output.stderr)
                    };

                    Ok((stdout, stderr))
                }
                None => Ok((
                    CommandOutput::Empty,
                    CommandOutput::Stderr(
                        format!("{}: command not found\n", self.name).into_bytes(),
                    ),
                )),
            }
        }
    }
}
