use serde::{Deserialize, Serialize};

/// Parameters that describe the process by which a password was generated.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct DicewareParams {
    /// The number of words used in the passphrase
    pub words: Range,
    /// The number of extra characters added to the passphrase
    pub extras: Range,
}

/// A range of integers, inclusive at both ends.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Range {
    pub low: usize,
    pub high: usize,
}
