mod shell;
use shell::Repl;

use anyhow::Result;

fn main() -> Result<()> {
    let mut repl = Repl::new();
    repl.run()
}
