use crate::shell::{
    ShellPath,
    builtin::ShellBuiltin,
    utils::{CommandOutput, OutputPair, split_args},
};
use anyhow::{Context, Result};

use std::{
    io::Write,
    process::{ChildStdout, Command, Stdio},
};

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
    history: Vec<String>,
}

impl ShellCommand {
    pub fn new(mut input: Vec<String>, history: Vec<String>) -> Self {
        let name = input.remove(0);
        Self {
            name,
            args: input,
            history,
        }
    }

    pub fn execute(&self, path: &ShellPath) -> Result<OutputPair> {
        if self.name.is_empty() {
            return Ok((CommandOutput::Empty, CommandOutput::Empty));
        }

        self.run_command(path).with_context(|| {
            format!(
                "running command '{}' with arguments {:?}",
                self.name, self.args
            )
        })
    }

    fn run_command(&self, path: &ShellPath) -> Result<OutputPair> {
        if let Some(builtin) = ShellBuiltin::is_builtin(&self.name) {
            let result = builtin
                .execute(&self.args, path, &self.history)
                .with_context(|| {
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
                    let output = Command::new(&self.name)
                        .args(&self.args)
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

#[derive(Debug)]
pub struct ShellPipeline {
    commands: Vec<Vec<String>>,
}

impl ShellPipeline {
    pub fn new(root: &[String], rest: &[String]) -> Self {
        let mut commands = Vec::new();
        commands.push(root.to_vec());

        let mut rest = rest.to_vec();
        loop {
            let (cmd, remaining) = split_args(&rest);
            commands.push(cmd.to_vec());
            match remaining {
                Some(r) => rest = r[1..].to_vec(),
                None => break,
            }
        }

        Self { commands }
    }

    pub fn execute(&self, shellpath: &ShellPath, history: &[String]) -> Result<OutputPair> {
        let mut last_output = Vec::new();
        let mut last_child: Option<ChildStdout> = None;
        let mut children = Vec::new();

        for (i, args) in self.commands.iter().enumerate() {
            let is_last = i == self.commands.len() - 1;
            match ShellBuiltin::is_builtin(&args[0]) {
                Some(builtin) => {
                    let result = builtin
                        .execute(&args[1..], shellpath, history)
                        .with_context(|| {
                            format!(
                                "executing builtin `{:?}` with args {:?}",
                                builtin,
                                &args[1..]
                            )
                        })?;
                    match result {
                        CommandOutput::Stdout(out) => last_output = out,
                        CommandOutput::Stderr(_) => return Ok((CommandOutput::Empty, result)),
                        CommandOutput::Empty => {}
                    }
                }
                None => {
                    let stdin = match last_child.take() {
                        Some(out) => Stdio::from(out),
                        None if !last_output.is_empty() => Stdio::piped(),
                        None => Stdio::inherit(),
                    };

                    let mut cmd = Command::new(&args[0])
                        .args(&args[1..])
                        .stdin(stdin)
                        .stdout(if is_last {
                            Stdio::inherit()
                        } else {
                            Stdio::piped()
                        })
                        .stderr(if is_last {
                            Stdio::inherit()
                        } else {
                            Stdio::piped()
                        })
                        .spawn()?;

                    if !last_output.is_empty() {
                        let data = std::mem::take(&mut last_output);
                        let mut stdin = cmd.stdin.take().expect("handle present");
                        std::thread::spawn(move || {
                            let _ = stdin.write_all(&data);
                        });
                    }

                    if !is_last {
                        last_child = cmd.stdout.take();
                    }
                    children.push(cmd);
                }
            }
        }

        if !last_output.is_empty() {
            return Ok((CommandOutput::Stdout(last_output), CommandOutput::Empty));
        }

        let last = children.pop().expect("should always be one");
        let output = last.wait_with_output()?;
        for mut child in children {
            drop(child.stdout.take());
            drop(child.stderr.take());
            let _ = child.wait();
            let _ = child.kill();
        }

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
}
