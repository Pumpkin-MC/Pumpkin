use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Generator {
    Test,
    Plains,
    Superflat,
    Void,
    Custom,
}
