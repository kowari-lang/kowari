use clap::Parser;
use std::path::PathBuf;

/// The Kowari compiler.
#[derive(Parser, Debug)]
#[command(name = "kowaric", version, about)]
struct Args {
    /// Source file to compile.
    input: PathBuf,

    /// Output binary path.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("kowaric: input = {}, output = {:?}", args.input.display(), args.output);
    // Pipeline wiring arrives in M1a.
    Ok(())
}