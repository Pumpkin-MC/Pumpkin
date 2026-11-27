use pumpkin_data::{dye_color::DyeColor, item_stack::ItemStack};
use pumpkin_util::Hand;

use crate::GameTestResult;

/// A world entity accessible to a function test.
pub trait GameTestEntity: Send + Sync {
    fn entity_id(&self) -> i32;

    fn set_no_ai(&self, no_ai: bool) -> GameTestResult<()>;

    fn held_item(&self, hand: Hand) -> GameTestResult<ItemStack>;

    fn custom_name(&self) -> GameTestResult<Option<String>>;

    fn is_leashed(&self) -> GameTestResult<bool>;

    fn set_health(&self, health: f32) -> GameTestResult<()>;

    fn set_color(&self, color: DyeColor) -> GameTestResult<()>;

    fn set_sheared(&self, sheared: bool) -> GameTestResult<()>;

    fn is_sheared(&self) -> GameTestResult<bool>;
}
