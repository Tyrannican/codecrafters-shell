use anyhow::{Context, Result};

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

    pub fn execute(&self, args: &[String]) -> Result<String> {
        match self {
            Self::Exit => std::process::exit(0),
            Self::Echo => {
                let mut combined_args = args.join(" ");
                combined_args.push('\n');
                return Ok(combined_args);
            }
            Self::Type => match Self::is_builtin(&args[0]) {
                Some(_) => Ok(format!("{} is a shell builtin\n", &args[0])),
                None => Ok(format!("{}: not found\n", &args[0])),
            },
            _ => todo!(),
        }
    }
}
