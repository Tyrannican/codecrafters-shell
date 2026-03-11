use anyhow::{Context, Result};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandOutput {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Empty,
}

#[derive(Debug)]
pub struct ShellPath {
    pub path: Vec<PathBuf>,
    pub home: PathBuf,
}

impl ShellPath {
    pub fn new() -> Result<Self> {
        let path = parse_path().context("parsing PATH var")?;
        let home = PathBuf::from(std::env::var("HOME").context("reading HOME dir")?);
        Ok(Self { path, home })
    }

    pub fn find(&self, cmd: impl AsRef<str>) -> Option<&PathBuf> {
        for path in self.path.iter() {
            let file = path.file_name().expect("should be fine");
            if file == cmd.as_ref() {
                return Some(path);
            }
        }

        None
    }
}

fn is_executable(path: &PathBuf) -> bool {
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn parse_path() -> Result<Vec<PathBuf>> {
    let path = std::env::var("PATH")
        .context("loading PATH var")?
        .split(':')
        .filter_map(|entry| {
            let entry = PathBuf::from(entry);
            if entry.is_dir() {
                return Some(
                    std::fs::read_dir(entry)
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter_map(|entry| {
                            let path = entry.path();
                            if path.is_file() && is_executable(&path) {
                                return Some(path);
                            } else {
                                return None;
                            }
                        })
                        .collect(),
                );
            } else {
                if is_executable(&entry) {
                    return Some(vec![entry]);
                }
            }

            None
        })
        .flatten()
        .collect::<Vec<PathBuf>>();

    Ok(path)
}
