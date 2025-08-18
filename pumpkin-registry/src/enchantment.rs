use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enchantment {
    // TODO: Add things :D
    description: Description,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    translate: String,
}
