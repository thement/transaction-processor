//! Types for de/serializing input and output

use crate::account;
use crate::moneys::Moneys;
use crate::processor;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// I probably wouldn't use the same struct for both passing around and for serialization, but
/// I have no time now
#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub client: account::ClientId,
    pub tx: processor::TransactionId,
    pub amount: Option<f64>,
}

impl Command {
    pub fn get_moneys(&self) -> Result<Moneys> {
        match self.amount {
            None => bail!("transaction is missing amount of money"),
            Some(amount) => Moneys::try_from(amount),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub client: account::ClientId,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}
