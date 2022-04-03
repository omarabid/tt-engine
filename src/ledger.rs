use crate::{
    error::{LedgerError, Result},
    types::{Account, Signal, Transaction, TransactionState, TransactionType},
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{mpsc::Sender, Arc, Mutex},
};

/// Ledger struct
#[derive(Debug)]
pub struct Ledger {
    /// Transactions DB
    tx_db: Arc<Mutex<BTreeMap<u32, Transaction>>>,
    /// Accounts DB
    /// TODO: could be a vector?
    accounts_db: Arc<Mutex<BTreeMap<usize, Account>>>,
    /// Channel to send signals to the main thread
    sender: Option<Sender<Signal>>,
    /// Thread handle
    handle: Option<std::thread::JoinHandle<Result<()>>>,
}

impl Ledger {
    /// Create a new Ledger
    pub fn new() -> Self {
        Ledger {
            tx_db: Arc::new(Mutex::new(BTreeMap::new())),
            accounts_db: Arc::new(Mutex::new(BTreeMap::new())),
            sender: None,
            handle: None,
        }
    }

    /// Start the Ledger thread and wait for transactions
    pub fn run(mut self) -> Result<Self> {
        // Create a channel to communicate with the Ledger thread
        let (tx, rx) = std::sync::mpsc::channel();

        // Store the sender in the Ledger struct
        self.sender = Some(tx);

        // Get an Arc reference to the tx_db
        let tx_db = self.tx_db.clone();
        // Get an Arc reference to the accounts_db
        let accounts_db = self.accounts_db.clone();

        // Create the Ledger thread
        self.handle = Some(std::thread::spawn(move || {
            // Wait for transactions
            while let Ok(signal) = rx.recv() {
                match signal {
                    // Stop the Ledger thread
                    Signal::Kill => break,
                    // Process transaction
                    Signal::Transaction(transaction) => match transaction.transaction_type {
                        TransactionType::Deposit | TransactionType::Withdrawal => {
                            // Insert transaction into the tx_db
                            tx_db.lock()?.insert(transaction.tx, transaction.clone());

                            // Convert Transaction and insert into the accounts_db
                            let mut acc_lock = accounts_db.lock()?;
                            let len = acc_lock.len();
                            acc_lock.insert(len, transaction.into());
                        }
                        TransactionType::Dispute => {
                            // Check that the transaction exists
                            if tx_db.lock()?.contains_key(&transaction.tx) {
                                let mut m_tx = tx_db.lock()?;
                                let mut tx_lock = m_tx.get_mut(&transaction.tx).unwrap();

                                // Check that the transaction is not already in dispute or chargeback
                                if tx_lock.state == TransactionState::New {
                                    // Update the transaction state
                                    tx_lock.state = TransactionState::Dispute;

                                    // Update the accounts state
                                    let mut acc_lock = accounts_db.lock()?;
                                    let len = acc_lock.len();
                                    acc_lock.insert(len, tx_lock.to_owned().into());
                                }
                            }
                        }
                        TransactionType::Resolve => {
                            // Check that the transaction exists
                            if tx_db.lock()?.contains_key(&transaction.tx) {
                                let mut m_tx = tx_db.lock()?;
                                let mut tx_lock = m_tx.get_mut(&transaction.tx).unwrap();

                                // Check that the transaction is in Dispute
                                if tx_lock.state == TransactionState::Dispute {
                                    // Update the transaction state
                                    tx_lock.state = TransactionState::Resolve;

                                    // Update the accounts state
                                    let mut acc_lock = accounts_db.lock()?;
                                    let len = acc_lock.len();
                                    acc_lock.insert(len, tx_lock.to_owned().into());
                                }
                            }
                        }
                        TransactionType::Chargeback => {
                            // Check that the transaction exists
                            if tx_db.lock()?.contains_key(&transaction.tx) {
                                let mut m_tx = tx_db.lock()?;
                                let mut tx_lock = m_tx.get_mut(&transaction.tx).unwrap();

                                // Check that the transaction is in dispute
                                if tx_lock.state == TransactionState::Dispute {
                                    // Update the transaction state
                                    tx_lock.state = TransactionState::Chargeback;

                                    // Update the accounts state
                                    let mut acc_lock = accounts_db.lock()?;
                                    let len = acc_lock.len();
                                    acc_lock.insert(len, tx_lock.to_owned().into());
                                }
                            }
                        }
                    },
                }
            }
            Ok(())
        }));

        Ok(self)
    }

    /// Push a transaction to the ledger
    pub fn push_transaction(&self, transaction: Transaction) -> Result<()> {
        self.sender
            .as_ref()
            .ok_or_else(|| LedgerError::Adhoc("Failed to push transaction".to_string()))?
            .send(Signal::Transaction(transaction))?;

        Ok(())
    }

    /// Stop the Ledger thread
    pub fn stop(&mut self) -> Result<()> {
        // Send a kill signal to the Ledger thread
        self.sender
            .as_ref()
            .ok_or_else(|| LedgerError::Adhoc("Failed to send Kill Signal".to_string()))?
            .send(Signal::Kill)?;

        // Wait for the Ledger thread to finish
        self.handle
            .take()
            .ok_or_else(|| LedgerError::Adhoc("Thread handle is not set".to_string()))?
            .join()
            .map_err(|_e| LedgerError::Adhoc("Failed to join thread".to_string()))??;

        Ok(())
    }

    /// Get a snapshot of the Ledger Accounts
    pub fn get_snapshot(&self) -> Result<Vec<Account>> {
        let txs = self.accounts_db.clone();
        let txs_lock = txs.lock()?;

        let accounts = txs_lock
            .iter()
            // merge accounts
            .fold(
                HashMap::new(),
                |mut acc: HashMap<usize, Account>, (_k, v)| {
                    if acc.contains_key(&(v.client as usize)) {
                        let acc_client = acc
                            .get(&(v.client as usize))
                            .unwrap_or(&Account::new(v.client))
                            .to_owned();
                        acc.insert(v.client as usize, acc_client + v.to_owned());
                    } else {
                        acc.insert(v.client as usize, Account::new(v.client) + v.to_owned());
                    }
                    acc
                },
            )
            .into_values()
            .collect::<Vec<Account>>();

        Ok(accounts)
    }
}
