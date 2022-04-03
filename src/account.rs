//! Account management

use crate::io;
use crate::moneys::Moneys;
use anyhow::Result;

pub type ClientId = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    client: ClientId,
    available: Moneys,
    held: Moneys,
    locked: bool,
}

impl Account {
    pub fn new(client: ClientId) -> Self {
        Self {
            client,
            available: Moneys::ZERO,
            held: Moneys::ZERO,
            locked: false,
        }
    }

    pub fn client(&self) -> ClientId {
        self.client
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    #[allow(dead_code)]
    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn deposit(&self, amount: Moneys) -> Result<Self> {
        let new_available = self.available.add(amount)?;

        Ok(Self {
            client: self.client,
            available: new_available,
            held: self.held,
            locked: self.locked,
        })
    }

    pub fn withdraw(&self, amount: Moneys) -> Result<Self> {
        let new_available = self.available.sub(amount)?;

        Ok(Self {
            client: self.client,
            available: new_available,
            held: self.held,
            locked: self.locked,
        })
    }

    pub fn dispute(&self, amount: Moneys) -> Result<Self> {
        let new_available = self.available.sub(amount)?;
        let new_held = self.held.add(amount)?;

        Ok(Self {
            client: self.client,
            available: new_available,
            held: new_held,
            locked: self.locked,
        })
    }

    pub fn resolve(&self, amount: Moneys) -> Result<Self> {
        let new_available = self.available.add(amount)?;
        let new_held = self.held.sub(amount)?;

        Ok(Self {
            client: self.client,
            available: new_available,
            held: new_held,
            locked: self.locked,
        })
    }

    pub fn chargeback(&self, amount: Moneys) -> Result<Self> {
        let new_held = self.held.sub(amount)?;

        Ok(Self {
            client: self.client,
            available: self.available,
            held: new_held,
            locked: true,
        })
    }
}

impl From<Account> for io::Account {
    fn from(account: Account) -> Self {
        Self {
            client: account.client,
            available: account.available.into(),
            held: account.held.into(),
            total: f64::from(account.available) + f64::from(account.held),
            locked: account.locked,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const EMPTY_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(0),
        held: Moneys::new(0),
        locked: false,
    };
    const DEPOSITED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(300),
        held: Moneys::new(0),
        locked: false,
    };
    const LOCKED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(100),
        held: Moneys::new(0),
        locked: true,
    };
    const HELD_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(100),
        held: Moneys::new(200),
        locked: false,
    };
    const HELD_DEPOSIT_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(500),
        held: Moneys::new(200),
        locked: false,
    };
    const MAXED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::MAX,
        held: Moneys::new(0),
        locked: false,
    };
    const MAXED_DISPUTED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::new(0),
        held: Moneys::MAX,
        locked: false,
    };
    const MAXED_DISPUTED_DEPOSITED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::MAX,
        held: Moneys::MAX,
        locked: false,
    };
    const MAXED_DISPUTED_LOCKED_ACCOUNT: Account = Account {
        client: 317,
        available: Moneys::MAX,
        held: Moneys::new(0),
        locked: true,
    };

    #[test]
    fn init() {
        assert_eq!(Account::new(317), EMPTY_ACCOUNT);
    }

    #[test]
    fn withdraw() {
        // Regular withdraw
        assert_eq!(
            DEPOSITED_ACCOUNT.withdraw(Moneys::new(300)).unwrap(),
            EMPTY_ACCOUNT
        );
        assert!(DEPOSITED_ACCOUNT.withdraw(Moneys::new(301)).is_err());

        // Max limits
        assert!(DEPOSITED_ACCOUNT.withdraw(Moneys::MAX).is_err());
        assert!(EMPTY_ACCOUNT.withdraw(Moneys::MAX).is_err());
        assert!(EMPTY_ACCOUNT.withdraw(Moneys::new(1)).is_err());
        assert_eq!(MAXED_ACCOUNT.withdraw(Moneys::MAX).unwrap(), EMPTY_ACCOUNT);
    }

    #[test]
    fn deposit() {
        // Regular deposits
        assert_eq!(
            EMPTY_ACCOUNT.deposit(Moneys::new(300)).unwrap(),
            DEPOSITED_ACCOUNT
        );
        assert_eq!(EMPTY_ACCOUNT.deposit(Moneys::MAX).unwrap(), MAXED_ACCOUNT);

        // Held deposits
        assert_eq!(
            HELD_ACCOUNT.deposit(Moneys::new(400)).unwrap(),
            HELD_DEPOSIT_ACCOUNT
        );

        // Max limits
        assert!(MAXED_ACCOUNT.deposit(Moneys::new(1)).is_err());
        assert!(MAXED_ACCOUNT.deposit(Moneys::MAX).is_err());
    }

    #[test]
    fn disputes() {
        assert_eq!(
            DEPOSITED_ACCOUNT.dispute(Moneys::new(200)).unwrap(),
            HELD_ACCOUNT
        );
        assert_eq!(
            HELD_ACCOUNT.resolve(Moneys::new(200)).unwrap(),
            DEPOSITED_ACCOUNT
        );
        assert_eq!(
            HELD_ACCOUNT.chargeback(Moneys::new(200)).unwrap(),
            LOCKED_ACCOUNT
        );
        assert!(HELD_ACCOUNT.chargeback(Moneys::new(201)).is_err());
        assert!(HELD_ACCOUNT.resolve(Moneys::new(201)).is_err());
        assert_eq!(HELD_ACCOUNT.resolve(Moneys::new(0)).unwrap(), HELD_ACCOUNT);

        // Max limits
        assert_eq!(
            MAXED_ACCOUNT.dispute(Moneys::MAX).unwrap(),
            MAXED_DISPUTED_ACCOUNT
        );
        assert_eq!(
            MAXED_DISPUTED_ACCOUNT.deposit(Moneys::MAX).unwrap(),
            MAXED_DISPUTED_DEPOSITED_ACCOUNT
        );
        assert!(MAXED_DISPUTED_DEPOSITED_ACCOUNT
            .resolve(Moneys::new(1))
            .is_err());
        assert_eq!(
            MAXED_DISPUTED_DEPOSITED_ACCOUNT
                .chargeback(Moneys::MAX)
                .unwrap(),
            MAXED_DISPUTED_LOCKED_ACCOUNT
        );
    }

    #[test]
    fn locking() {
        assert!(!EMPTY_ACCOUNT.is_locked());
        assert!(LOCKED_ACCOUNT.is_locked());
        let mut account = LOCKED_ACCOUNT.clone();
        account.unlock();
        assert!(!account.is_locked());
    }
}
