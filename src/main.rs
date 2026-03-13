mod shell;
use shell::Repl;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut repl = Repl::new().context("loading repl shell")?;
    repl.run()
}
