### Toy Transactions Engine

* **Problem scope**
The Toy Transactions Engine needs to handle incoming transactions while maintaining an "Accounts" state. Some of these transactions do actually reference other transactions to mutate their state. As a result the Toy Transactions Engine need to keep all of these transactions in memory.

* **Proposed Solution**
The program maintains a Thread that listens and waits for upcoming Transactions through a Multi-Producer/Single-Consumer channel. When a Transaction is received, it's processed by the thread to update the "Accounts" state. A `get_snapshot` function can be used to return the state of the accounts.

- **Technical details**
  - The main file of the program is used to: 1. read the cli argument and get the csv filename, 2. Deserialize the CSV file into a Vector of Transactions, 3. start the Thread loop and push these Transactions into the loop, 4. Stop the Thread loop and then get a snapshot of the Accounts, 5. Serialize the Accounts into csv and post it to stdout.
  - Only Deposit/Withdrawal are actually transactions. Though Dispute/Resolve/Chargeback are Deserialized into a Transaction, they are considered a mutation of another Transaction.
  - The `Ledger` struct in ledger.rs is used to run the Thread loop, keep a reference to the Channel Sender, and hold the transactions and accounts state.
  - Several types are defined in `types.rs`: Transaction(with TransactionType/TransactionState), Account and Signal (used to exit the Thread loop).
  - When a Deposit/Withdrawal transaction is received, it's added to the Transactions Database `tx_db`. When a Dispute/Resolve/Charge transaction is received, it mutates the state (if applicable) of the relevant transaction in `tx_db`.
  - The Transaction is then converted to an Account and added to the Accounts Database `accounts_db`. This Types conversion is defined in `types.rs`.
  - Each Transaction is converted to an Account. The `get_snapshot` function fold these accounts by `client_id`. Merging is done by adding two accounts together.
  - Adding two accounts together is possible by implementing the `std::ops::Add` trait to the Account type.
  - Error Handling: `thiserror` is used to create a custom Error type.

- **Shortcomings**
  - Some `unwraps` are not properly handled.
  - The input data for testing is quite basic.
  - Some refactoring is needed for some parts of the code. Documentation could also be improved.
  - More consideration could be given to data types, state transitions and conditional checks.
  - More tooling (Github CI/Linting)
