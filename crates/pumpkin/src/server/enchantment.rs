use pumpkin_data::enchantment::AttributeModifierSlot;
use pumpkin_util::text::TextComponent;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct CustomEnchantmentEntry {
    pub id: String,
    pub description: TextComponent,
    pub max_level: u32,
    pub anvil_cost: u32,
    pub supported_items: String,
    pub weight: u32,
    pub slots: Vec<AttributeModifierSlot>,
    pub exclusive_set: Vec<String>,
}

pub struct EnchantmentManager {
    custom_enchantments: RwLock<HashMap<String, CustomEnchantmentEntry>>,
}

impl Default for EnchantmentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnchantmentManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_enchantments: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, enchantment: CustomEnchantmentEntry) -> Result<(), String> {
        let mut map = self.custom_enchantments.write().await;
        if map.contains_key(&enchantment.id) {
            return Err(format!(
                "Enchantment '{}' is already registered",
                enchantment.id
            ));
        }
        map.insert(enchantment.id.clone(), enchantment);
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Option<CustomEnchantmentEntry> {
        let map = self.custom_enchantments.read().await;
        map.get(id).cloned()
    }

    pub async fn has(&self, id: &str) -> bool {
        let map = self.custom_enchantments.read().await;
        map.contains_key(id)
    }

    pub async fn get_all_ids(&self) -> Vec<String> {
        let map = self.custom_enchantments.read().await;
        map.keys().cloned().collect()
    }
}
