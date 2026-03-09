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
            Self::Exit => self.exit(),
            Self::Echo => {
                let mut combined_args = args.join(" ");
                combined_args.push('\n');
                return Ok(combined_args);
            }
            _ => todo!(),
        }

        todo!()
    }

    fn exit(&self) {
        std::process::exit(0)
    }
}
