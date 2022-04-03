use serde::{Deserialize, Serialize};

/// Transaction Type
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

/// Transaction State
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionState {
    New,
    Dispute,
    Resolve,
    Chargeback,
}

impl Default for TransactionState {
    fn default() -> Self {
        TransactionState::New
    }
}

/// Transaction struct
#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
    #[serde(skip_deserializing)]
    pub state: TransactionState,
}

/// Account struct
#[derive(Debug, Serialize, Clone)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    /// Create a new Account
    pub fn new(client: u16) -> Self {
        Account {
            client,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}

// Implement Add operator for account
impl std::ops::Add<Account> for Account {
    type Output = Account;

    fn add(self, other: Account) -> Account {
        // Check if the last account in locked
        if self.locked {
            return self;
        }
        // Prevent negative withdrawal
        // notice: this is sub-optimal since disputes are detected with the
        // held value
        if (self.available + other.available) < 0.0 && other.held == 0.0 {
            return self;
        }
        Account {
            client: self.client,
            available: self.available + other.available,
            held: self.held + other.held,
            total: self.total + other.total,
            locked: self.locked || other.locked,
        }
    }
}

/// Convert a Transaction to an Account
impl From<Transaction> for Account {
    fn from(t: Transaction) -> Self {
        match t.transaction_type {
            TransactionType::Deposit => match t.state {
                TransactionState::New => Account {
                    client: t.client,
                    available: t.amount.unwrap_or(0.0),
                    held: 0.0,
                    total: t.amount.unwrap_or(0.0),
                    locked: false,
                },
                TransactionState::Resolve => Account {
                    client: t.client,
                    available: t.amount.unwrap_or(0.0),
                    held: -1.0 * t.amount.unwrap_or(0.0),
                    total: 0.0,
                    locked: false,
                },
                TransactionState::Dispute => Account {
                    client: t.client,
                    available: -1.0 * t.amount.unwrap_or(0.0),
                    held: t.amount.unwrap_or(0.0),
                    total: 0.0,
                    locked: false,
                },
                TransactionState::Chargeback => Account {
                    client: t.client,
                    available: 0.0,
                    held: -1.0 * t.amount.unwrap_or(0.0),
                    total: -1.0 * t.amount.unwrap_or(0.0),
                    locked: true,
                },
            },
            TransactionType::Withdrawal => match t.state {
                TransactionState::New => Account {
                    client: t.client,
                    available: -1.0 * t.amount.unwrap_or(0.0),
                    held: 0.0,
                    total: -1.0 * t.amount.unwrap_or(0.0),
                    locked: false,
                },
                TransactionState::Resolve => Account {
                    client: t.client,
                    available: -1.0 * t.amount.unwrap_or(0.0),
                    held: 0.0,
                    total: -1.0 * t.amount.unwrap_or(0.0),
                    locked: false,
                },
                TransactionState::Dispute => Account {
                    client: t.client,
                    available: 1.0 * t.amount.unwrap_or(0.0),
                    held: -1.0 * t.amount.unwrap_or(0.0),
                    total: 0.0,
                    locked: false,
                },
                TransactionState::Chargeback => Account {
                    client: t.client,
                    available: 0.0,
                    held: 0.0,
                    total: 0.0,
                    locked: true,
                },
            },
            // TODO: use logging instead
            _ => panic!("Should not happen"),
        }
    }
}

/// Signal Enum
pub enum Signal {
    Kill,
    Transaction(Transaction),
}
