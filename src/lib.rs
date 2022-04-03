pub use account::Account;
use anyhow::Result;
pub use processor::Processor;

mod account;
pub mod io;
mod moneys;
mod processor;
pub use account::ClientId;

/// Stream input CSV file through transaction processor. Optionally print debug info (like errors 
/// and parsed data).
pub fn run_processor<R: std::io::Read>(
    processor: &mut Processor,
    raw_reader: R,
    verbose: bool,
) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(raw_reader);

    // Run transactions through processor
    for result in reader.deserialize::<io::Command>() {
        let command = result?;
        if verbose {
            println!("command: {:?}", command);
        }
        let ret = processor.execute(&command);
        if verbose {
            println!("result: {:?}", ret);
        }
    }

    Ok(())
}

/// Serialize accounts as CSV
pub fn print_accounts<W: std::io::Write>(
    raw_writer: W,
    accounts: &[account::Account],
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().from_writer(raw_writer);
    for account in accounts {
        writer.serialize(io::Account::from(account.clone()))?;
    }
    writer.flush()?;
    Ok(())
}
