use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[expect(dead_code)]
pub struct ItemEntry {
    name: String,
}
