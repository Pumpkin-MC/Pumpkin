use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Generator {
    Test,
    BeautifulPlains,
    Superflat,
    Void,
    Custom,
}
