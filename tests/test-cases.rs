use anyhow::{bail, Context as _, Result};
use std::fs;
use std::io;
use transaction_processor::{io::Account as IoAccount, run_processor, Processor};

/// Utility function to read accounts for testing purposes
pub fn read_accounts(path: &str) -> Result<Vec<IoAccount>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;

    let mut accounts = vec![];
    for result in reader.deserialize::<IoAccount>() {
        accounts.push(result?);
    }

    Ok(accounts)
}

fn sort_clients(a: &IoAccount, b: &IoAccount) -> std::cmp::Ordering {
    a.client.partial_cmp(&b.client).unwrap()
}

fn run_testcase(transaction_path: &str, account_path: &str) -> Result<()> {
    let f = fs::File::open(transaction_path)?;
    let raw_reader = io::BufReader::new(f);
    let mut processor: Processor = Default::default();
    run_processor(&mut processor, raw_reader, true).context("error in transaction runner")?;
    let mut accounts: Vec<_> = processor
        .accounts()
        .into_iter()
        .map(|account| IoAccount::from(account))
        .collect();
    let mut expected_accounts = read_accounts(account_path)?;

    accounts.sort_by(sort_clients);
    expected_accounts.sort_by(sort_clients);

    // Panic if the two arrays are not the same length, but only after
    // checking the whole list
    let mut fail = false;
    for i in 0..accounts.len().max(expected_accounts.len()) {
        // TODO: this test needs to test equality on floats with limited precision (approx_eq)
        if accounts[i] != expected_accounts[i] {
            println!(
                "accounts at position {} differ:\n{:?}\n{:?}",
                i, accounts[i], expected_accounts[i]
            );
            fail = true;
        }
    }
    if fail {
        bail!("Test case {} failed", transaction_path);
    }

    Ok(())
}

#[test]
fn run_all_testcases() {
    const PREFIX: &str = "tests/test-cases";
    for name in ["test1", "official"].iter() {
        let tp = format!("{}/{}.input.txt", PREFIX, name);
        let ap = format!("{}/{}.output.txt", PREFIX, name);
        run_testcase(&tp, &ap).unwrap();
    }
}
