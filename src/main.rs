use anyhow::{Context as _, Result};
use clap::Parser;
use std::fs;
use std::io;
use std::path;
use transaction_processor::{print_accounts, run_processor, Processor};

/// Definition of command-line arguments
#[derive(Debug, Parser)]
struct Cli {
    /// Whether to output some more information
    #[clap(short, long)]
    verbose: bool,
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let f = fs::File::open(&args.path)
        .with_context(|| format!("failed opening input file {:?}", args.path))?;
    let raw_reader = io::BufReader::new(f);

    // Build processor
    let mut processor: Processor = Default::default();
    // Run all input transactions through it (can be called multiple times)
    run_processor(&mut processor, raw_reader, args.verbose)
        .context("error in transaction runner")?;

    // Gather accounts and print them
    let accounts = processor.accounts();
    print_accounts(io::stdout(), &accounts)?;

    Ok(())
}
