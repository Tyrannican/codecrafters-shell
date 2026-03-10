use crate::shell::ShellPath;
use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShellBuiltin {
    Exit,
    Echo,
    Pwd,
    Type,
    Cd,
}

impl ShellBuiltin {
    pub fn is_builtin(cmd: impl AsRef<str>) -> Option<Self> {
        match cmd.as_ref() {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "pwd" => Some(Self::Pwd),
            "type" => Some(Self::Type),
            "cd" => Some(Self::Cd),
            _ => None,
        }
    }

    pub fn execute(&self, args: &[String], path: &ShellPath) -> Result<Vec<u8>> {
        match self {
            Self::Exit => std::process::exit(0),
            Self::Echo => {
                let mut combined_args = args.join(" ");
                combined_args.push('\n');
                return Ok(combined_args.into_bytes());
            }
            Self::Type => match Self::is_builtin(&args[0]) {
                Some(_) => Ok(format!("{} is a shell builtin\n", &args[0]).into_bytes()),
                None => match path.find(&args[0]) {
                    Some(p) => Ok(format!("{} is {}\n", &args[0], p.display()).into_bytes()),
                    None => Ok(format!("{}: not found\n", &args[0]).into_bytes()),
                },
            },
            Self::Pwd => {
                let mut cwd = std::env::current_dir()
                    .context("getting current working dir")?
                    .display()
                    .to_string();
                cwd.push('\n');
                Ok(cwd.into_bytes())
            }
            Self::Cd => {
                let dst = if args[0].starts_with("~") {
                    PathBuf::from(format!(
                        "{}/{}",
                        path.home.display(),
                        &args[0].replace("~", "")
                    ))
                } else {
                    PathBuf::from(&args[0])
                };

                if let Err(_) = std::env::set_current_dir(&dst) {
                    return Ok(
                        format!("{}: No such file or directory\n", dst.display()).into_bytes()
                    );
                };

                Ok(b"".to_vec())
            }
        }
    }
}
