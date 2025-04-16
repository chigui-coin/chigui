use std::cell::RefCell;
use std::fs::read_to_string;
use std::{collections::HashMap, path::Path};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::{Account, Tx};

#[derive(Debug, Serialize, Deserialize)]
pub struct Genesis {
    genesis_time: String,
    chain_id: String,
    balances: HashMap<Account, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub balances: RefCell<HashMap<Account, u64>>,
    pub txs: Vec<Tx>,
    genesis: Genesis,
}

impl State {
    pub fn open<P: AsRef<Path>>(dbdir: P) -> Result<Self> {
        let genesis_path = dbdir.as_ref().join("genesis.json");
        let tx_db_path = dbdir.as_ref().join("tx.db");
        let genesis_json = read_to_string(genesis_path)?;
        let tx_db = read_to_string(tx_db_path)?;
        let genesis = Self::parse_genesis(&genesis_json)?;
        let txs = Self::parse_txs(&tx_db)?;
        let state = State::from_parts(genesis, txs)?;

        Ok(state)
    }

    pub fn get_balance(&self, acct: &Account) -> Option<u64> {
        let balances = self.balances.borrow();
        balances.get(acct).cloned()
    }

    fn apply(&mut self, tx: &Tx) -> Result<()> {
        match tx {
            Tx::Transfer { from, to, value } => {
                let balances = self.balances.get_mut();
                let [Some(from_balance), Some(to_balance)] = balances.get_disjoint_mut([from, to])
                else {
                    return Err(Error::msg("Account not found."));
                };

                if *value > *from_balance {
                    return Err(Error::msg("Insufficient balance."));
                }

                *to_balance += value;
                *from_balance -= value;

                Ok(())
            }
            Tx::Generate { to, value } => {
                let mut balances = self.balances.borrow_mut();
                let to = balances
                    .get_mut(to)
                    .ok_or(Error::msg("[To] Account not found."))?;

                *to += value;
                Ok(())
            }
        }
    }

    /// Create a new [`State`] instance from the given [`Genesis`] and a collection of [`Tx`] instances.
    fn from_parts(genesis: Genesis, txs: Vec<Tx>) -> Result<State> {
        let balances = genesis.balances.clone();
        let mut state = State {
            balances: RefCell::new(balances),
            txs,
            genesis,
        };
        let txs = state.txs.clone();

        for tx in txs.iter() {
            state.apply(tx)?;
        }

        Ok(state)
    }

    /// Parse the `genesis.json` file into a [`Genesis`] instance.
    fn parse_genesis(genesis_json: &str) -> Result<Genesis> {
        let genesis =
            serde_json::from_str::<Genesis>(genesis_json).context("Failed to parse genesis.")?;
        Ok(genesis)
    }

    /// Parse the `tx.db` file which is basically a JSONL file into a collection of [`Tx`] instances.
    fn parse_txs(tx_db_str: &str) -> Result<Vec<Tx>> {
        let lines = tx_db_str.lines().collect::<Vec<&str>>();
        let txs = lines
            .iter()
            .map(|line| serde_json::from_str::<Tx>(line).context("Failed to parse transaction."))
            .collect::<Result<Vec<Tx>>>()?;

        Ok(txs)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn apply_transfer_tx() -> Result<()> {
        let genesis = Genesis {
            genesis_time: String::from("2021-01-01T00:00:00Z"),
            chain_id: String::from("testnet"),
            balances: {
                let mut map = HashMap::new();
                map.insert(Account(String::from("alice")), 1000);
                map.insert(Account(String::from("bob")), 1000);
                map
            },
        };
        let mut state = State::from_parts(genesis, Vec::default())?;

        state.apply(&Tx::Transfer {
            from: Account(String::from("alice")),
            to: Account(String::from("bob")),
            value: 10,
        })?;

        assert_eq!(
            state.get_balance(&Account(String::from("bob"))).unwrap(),
            1010
        );
        assert_eq!(
            state.get_balance(&Account(String::from("alice"))).unwrap(),
            990
        );

        Ok(())
    }

    #[test]
    fn generate_coins() -> Result<()> {
        let genesis = Genesis {
            genesis_time: String::from("2021-01-01T00:00:00Z"),
            chain_id: String::from("testnet"),
            balances: {
                let mut map = HashMap::new();
                map.insert(Account::new("alice"), 1000);
                map.insert(Account::new("bob"), 1000);
                map
            },
        };
        let mut state = State::from_parts(genesis, Vec::default())?;

        state.apply(&Tx::Generate {
            to: Account::new("bob"),
            value: 10,
        })?;

        assert_eq!(state.get_balance(&Account::new("bob")).unwrap(), 1010);
        assert_eq!(state.get_balance(&Account::new("alice")).unwrap(), 1000);

        Ok(())
    }
}
