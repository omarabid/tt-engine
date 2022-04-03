use crate::error::{LedgerError, Result};
use ledger::Ledger;
use std::env;
use types::Transaction;

mod error;
mod ledger;
mod types;

fn main() -> Result<()> {
    // read the command line arguments
    let args: Vec<String> = env::args().collect();

    // get the second argument or throw an error
    let file_name = args
        .get(1)
        .ok_or_else(|| LedgerError::Adhoc("please provide a filename".to_string()))?;

    // Create CSV Reader
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(file_name)?;

    // Create a Ledger loop
    let mut ledger = Ledger::new().run()?;

    // Loop through the CSV file and deserialize values
    let collection = reader
        .deserialize()
        .map(|result| {
            let transaction: Result<Transaction> =
                result.map_err(|err| LedgerError::Adhoc(format!("{}", err)));
            transaction
        })
        .collect::<Result<Vec<Transaction>>>()?;

    // Loop through the transactions and add them to the ledger
    for transaction in collection {
        ledger.push_transaction(transaction.to_owned())?;
    }

    // Stop the Ledger thread
    ledger.stop()?;

    // Get a snapshot of the Ledger accounts
    let accounts = ledger.get_snapshot()?;

    // Create CSV Writer
    let mut wrt = csv::WriterBuilder::new().from_writer(std::io::stdout());

    // Serialize the accounts to CSV Writer
    for account in accounts {
        wrt.serialize(account)?;
    }

    // Flush the CSV Writer to stdout
    wrt.flush()?;

    // everything is fine!
    Ok(())
}
