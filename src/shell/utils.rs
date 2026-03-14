use anyhow::{Context, Result};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

// Docs on this one sucked so had to call in Claude (eugh)...
use rustyline::{
    Context as RustyContext, Helper, Highlighter, Hinter, Validator,
    completion::{Completer, FilenameCompleter, Pair},
};

pub const REDIRECT_OPS: [&str; 7] = ["1>", ">", "1>>", ">>", "2>", "2>>", "|"];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RedirectOp {
    RedirectOut,
    AppendOut,
    RedirectErr,
    AppendErr,
    Pipe,
}

impl RedirectOp {
    pub fn parse(op: impl AsRef<str>) -> Option<Self> {
        match op.as_ref() {
            ">" | "1>" => Some(Self::RedirectOut),
            ">>" | "1>>" => Some(Self::AppendOut),
            "2>" => Some(Self::RedirectErr),
            "2>>" => Some(Self::AppendErr),
            "|" => Some(Self::Pipe),
            _ => None,
        }
    }
}

pub type OutputPair = (CommandOutput, CommandOutput);

#[derive(Debug, Clone, PartialEq)]
pub enum CommandOutput {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Empty,
}

#[derive(Debug, Clone)]
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
    let mut path = std::env::var("PATH")
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

    // Required for built-in completion
    path.push(PathBuf::from("exit"));
    path.push(PathBuf::from("echo"));
    path.push(PathBuf::from("pwd"));
    path.push(PathBuf::from("type"));
    path.push(PathBuf::from("cd"));

    Ok(path)
}

#[derive(Helper, Highlighter, Hinter, Validator)]
pub struct ShellPathCompleter {
    pub shellpath: ShellPath,
    pub filenames: FilenameCompleter,
}

impl ShellPathCompleter {
    pub fn new(shellpath: ShellPath) -> Self {
        Self {
            shellpath,
            filenames: FilenameCompleter::new(),
        }
    }
}

impl Completer for ShellPathCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &RustyContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let prefix = &line[start..pos];
        let is_command_position = start == 0;

        if is_command_position {
            let cmd_matches = self
                .shellpath
                .path
                .iter()
                .filter_map(|p| {
                    let name = p.file_name()?.to_str()?;
                    name.starts_with(prefix)
                        .then(|| format!("{} ", name.to_string()))
                })
                .collect::<std::collections::BTreeSet<String>>();

            let cmd_matches = cmd_matches
                .into_iter()
                .map(|m| Pair {
                    display: m.clone(),
                    replacement: m,
                })
                .collect::<Vec<Pair>>();

            Ok((start, cmd_matches))
        } else {
            let (start, candidates) = self.filenames.complete(line, pos, ctx)?;

            let thing = candidates
                .into_iter()
                .map(|mut path| {
                    if !path.replacement.ends_with('/') {
                        path.replacement.push(' ');
                    } else {
                        if !path.display.ends_with('/') {
                            path.display.push('/');
                        }
                    }

                    path
                })
                .collect();

            Ok((start, thing))
        }
    }
}

pub fn split_args(args: &[String]) -> (Vec<String>, Option<Vec<String>>) {
    if let Some(idx) = args
        .iter()
        .position(|arg| REDIRECT_OPS.contains(&arg.as_str()))
    {
        let (args, redirect) = args.split_at(idx);
        if redirect.len() < 2 {
            return (args.to_vec(), None);
        } else {
            return (args.to_vec(), Some(redirect.to_vec()));
        }
    }

    (args.to_vec(), None)
}
