pub mod state;

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Tx {
    Transfer {
        from: Account,
        to: Account,
        value: u64,
    },
    Generate {
        to: Account,
        value: u64,
    },
}

impl Display for Tx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Tx::Transfer { from, to, value } => {
                write!(
                    f,
                    "[TXN] \"{}\" transferred \"{}\" coins to \"{}\" account",
                    from, value, to
                )
            }
            Tx::Generate { to, value } => {
                write!(
                    f,
                    "[GEN] generated \"{}\" coins on \"{}\" account",
                    value, to
                )
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Account(String);

impl Account {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
