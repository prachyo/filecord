use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub op: String,
    pub path: String,
    pub tmp_id: String,
    pub state: String,
}
