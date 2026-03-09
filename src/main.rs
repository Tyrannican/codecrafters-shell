mod shell;
use shell::Repl;

fn main() -> std::io::Result<()> {
    Repl::run()
}
