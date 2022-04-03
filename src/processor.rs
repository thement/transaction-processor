//! Transaction management

use crate::account::{Account, ClientId};
use crate::io::{Command, CommandType};
use crate::moneys::Moneys;
use anyhow::{anyhow, bail, ensure, Result};
use std::collections::HashMap;

pub type TransactionId = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DepositTransactionState {
    Deposited,
    Disputed,
    ChargedBack,
}

#[derive(Debug, Clone)]
enum Transaction {
    WithdrawTransaction {
        #[allow(dead_code)]
        client: ClientId,
        #[allow(dead_code)]
        amount: Moneys,
    },
    DepositTransaction {
        client: ClientId,
        amount: Moneys,
        state: DepositTransactionState,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Processor {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl Processor {
    pub fn accounts(&self) -> Vec<Account> {
        self.accounts.iter().map(|(_k, v)| v.clone()).collect()
    }

    fn dispute_step(
        account: &Account,
        transaction: Option<Transaction>,
        expected_state: DepositTransactionState,
        next_state: DepositTransactionState,
    ) -> Result<(Moneys, Transaction)> {
        match transaction {
            None => bail!("transaction not found"),
            Some(Transaction::DepositTransaction {
                client,
                amount,
                state,
            }) => {
                ensure!(
                    client == account.client(),
                    "transaction references different client"
                );
                ensure!(
                    state == expected_state,
                    "disputed transaction is in a wrong state"
                );
                let new_transaction = Transaction::DepositTransaction {
                    client,
                    amount,
                    state: next_state,
                };
                Ok((amount, new_transaction))
            }
            Some(_) => bail!("transaction is not deposit transaction"),
        }
    }

    /// Applies command to given transaction and account, doesn't modify state
    fn apply_command(
        command: &Command,
        account: Account,
        transaction: Option<Transaction>,
    ) -> Result<(Account, Transaction)> {
        ensure!(!account.is_locked(), "locked account");

        let r = match command.command_type {
            CommandType::Withdrawal => {
                ensure!(transaction.is_none(), "transaction already exists");
                let moneys = command.get_moneys()?;
                let new_account = account.withdraw(moneys)?;
                let new_transaction = Transaction::WithdrawTransaction {
                    client: account.client(),
                    amount: moneys,
                };
                (new_account, new_transaction)
            }
            CommandType::Deposit => {
                ensure!(transaction.is_none(), "transaction already exists");
                let moneys = command.get_moneys()?;
                let new_account = account.deposit(moneys)?;
                let new_transaction = Transaction::DepositTransaction {
                    client: account.client(),
                    amount: moneys,
                    state: DepositTransactionState::Deposited,
                };
                (new_account, new_transaction)
            }
            CommandType::Dispute => {
                let (moneys, new_transaction) = Self::dispute_step(
                    &account,
                    transaction,
                    DepositTransactionState::Deposited,
                    DepositTransactionState::Disputed,
                )?;
                let new_account = account.dispute(moneys)?;
                (new_account, new_transaction)
            }
            CommandType::Resolve => {
                let (moneys, new_transaction) = Self::dispute_step(
                    &account,
                    transaction,
                    DepositTransactionState::Disputed,
                    DepositTransactionState::Deposited,
                )?;
                let new_account = account.resolve(moneys)?;
                (new_account, new_transaction)
            }
            CommandType::Chargeback => {
                let (moneys, new_transaction) = Self::dispute_step(
                    &account,
                    transaction,
                    DepositTransactionState::Disputed,
                    DepositTransactionState::ChargedBack,
                )?;
                let new_account = account.chargeback(moneys)?;
                (new_account, new_transaction)
            }
        };
        Ok(r)
    }

    #[allow(dead_code)]
    pub fn unlock(&mut self, client: ClientId) -> Result<()> {
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(anyhow!("client not found"))?;
        ensure!(account.is_locked(), "account not locked");
        account.unlock();
        Ok(())
    }

    pub fn execute(&mut self, command: &Command) -> Result<()> {
        let account = self
            .accounts
            .get(&command.client)
            .map(|account| (*account).clone())
            .unwrap_or_else(|| Account::new(command.client));
        let transaction = self
            .transactions
            .get(&command.tx)
            .map(|transaction| transaction.to_owned());

        let (new_account, new_transaction) = Self::apply_command(command, account, transaction)?;

        self.accounts.insert(command.client, new_account);
        self.transactions.insert(command.tx, new_transaction);

        Ok(())
    }
}
