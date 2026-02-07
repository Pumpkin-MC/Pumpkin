//! Procedural crafting recipes that can't be represented as static data.
//!
//! These correspond to the 11 `crafting_special_*` recipe types in vanilla Minecraft.
//! Each recipe requires code logic because the result depends on the input items'
//! data components (dye colors, potion effects, book contents, etc.).

use pumpkin_data::item::Item;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_world::item::ItemStack;

use super::recipes::RecipeInputInventory;

/// The type of special recipe that matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRecipeType {
    ArmorDye,
    BannerDuplicate,
    BookCloning,
    FireworkRocket,
    FireworkStar,
    FireworkStarFade,
    MapCloning,
    MapExtending,
    RepairItem,
    ShieldDecoration,
    TippedArrow,
}

/// Try all special recipes against the given crafting inventory.
///
/// Returns the result `ItemStack` and the recipe type if one matches.
/// Special recipes are checked after normal `RECIPES_CRAFTING` fails.
pub async fn try_special_recipe(
    inventory: &dyn RecipeInputInventory,
) -> Option<(ItemStack, SpecialRecipeType)> {
    // Collect non-empty slots for analysis
    let mut items: Vec<ItemStack> = Vec::new();
    for i in 0..inventory.size() {
        let slot = inventory.get_stack(i).await;
        let stack = slot.lock().await;
        if !stack.is_empty() {
            items.push(stack.clone());
        }
    }

    if items.is_empty() {
        return None;
    }

    // Try each special recipe type
    // Order: most common first for fast rejection
    if let Some(result) = try_repair_item(&items) {
        return Some((result, SpecialRecipeType::RepairItem));
    }
    if let Some(result) = try_armor_dye(&items) {
        return Some((result, SpecialRecipeType::ArmorDye));
    }
    if let Some(result) = try_tipped_arrow(&items) {
        return Some((result, SpecialRecipeType::TippedArrow));
    }
    if let Some(result) = try_banner_duplicate(&items) {
        return Some((result, SpecialRecipeType::BannerDuplicate));
    }
    if let Some(result) = try_book_cloning(&items) {
        return Some((result, SpecialRecipeType::BookCloning));
    }
    if let Some(result) = try_firework_rocket(&items) {
        return Some((result, SpecialRecipeType::FireworkRocket));
    }
    if let Some(result) = try_firework_star(&items) {
        return Some((result, SpecialRecipeType::FireworkStar));
    }
    if let Some(result) = try_firework_star_fade(&items) {
        return Some((result, SpecialRecipeType::FireworkStarFade));
    }
    if let Some(result) = try_map_cloning(&items) {
        return Some((result, SpecialRecipeType::MapCloning));
    }
    if let Some(result) = try_map_extending(&items) {
        return Some((result, SpecialRecipeType::MapExtending));
    }
    if let Some(result) = try_shield_decoration(&items) {
        return Some((result, SpecialRecipeType::ShieldDecoration));
    }

    None
}

/// Repair two items of the same type by combining durability.
///
/// Takes exactly 2 damageable items of the same type. The result gets 5% bonus durability.
fn try_repair_item(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() != 2 {
        return None;
    }

    let a = &items[0];
    let b = &items[1];

    // Both must be the same item
    if a.item != b.item {
        return None;
    }

    // Both must be damageable
    let max_damage = a.get_max_damage()?;
    b.get_max_damage()?;

    // Calculate combined durability with 5% bonus
    let remaining_a = max_damage - a.get_damage();
    let remaining_b = max_damage - b.get_damage();
    let bonus = max_damage / 20; // 5%
    let new_damage = max_damage - (remaining_a + remaining_b + bonus).min(max_damage);

    let mut result = ItemStack::new(1, a.item);
    // Set damage on the result
    // TODO: Copy enchantments from both inputs (merge enchantments)
    result.set_damage(new_damage.max(0));
    Some(result)
}

/// Dye a piece of leather armor (or wolf armor) with one or more dyes.
///
/// Requires exactly 1 dyeable item and 1+ dyes in the grid.
fn try_armor_dye(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() < 2 {
        return None;
    }

    let mut dyeable_item: Option<&ItemStack> = None;
    let mut dye_count = 0u32;

    for item in items {
        if item.item.has_tag(&tag::Item::MINECRAFT_DYEABLE) {
            if dyeable_item.is_some() {
                return None; // Only one dyeable item allowed
            }
            dyeable_item = Some(item);
        } else if item.item.has_tag(&tag::Item::C_DYES) {
            dye_count += 1;
        } else {
            return None; // Invalid item in grid
        }
    }

    let armor = dyeable_item?;
    if dye_count == 0 {
        return None;
    }

    // Produce a copy of the armor item
    // TODO: Apply DyedColor component with mixed color from dyes
    // Color mixing algorithm: average RGB of existing color + all dye colors
    let result = ItemStack::new(1, armor.item);
    Some(result)
}

/// Craft tipped arrows from 8 arrows + 1 lingering potion.
///
/// Produces 8 tipped arrows with the potion effect.
fn try_tipped_arrow(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() != 9 {
        return None;
    }

    let mut arrow_count = 0u32;
    let mut potion_item: Option<&ItemStack> = None;

    for item in items {
        if item.item.has_tag(&tag::Item::MINECRAFT_ARROWS) {
            arrow_count += 1;
        } else if item.item == &Item::LINGERING_POTION {
            if potion_item.is_some() {
                return None; // Only one potion allowed
            }
            potion_item = Some(item);
        } else {
            return None;
        }
    }

    if arrow_count != 8 || potion_item.is_none() {
        return None;
    }

    // Produce 8 tipped arrows
    // TODO: Copy PotionContents from lingering potion to tipped arrows
    let result = ItemStack::new(8, &Item::TIPPED_ARROW);
    Some(result)
}

/// Duplicate a banner by placing it next to a blank banner of matching color.
///
/// Takes 1 patterned banner + 1 blank banner of the same base color.
fn try_banner_duplicate(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() != 2 {
        return None;
    }

    let a = &items[0];
    let b = &items[1];

    // Both must be banners
    if !a.item.has_tag(&tag::Item::MINECRAFT_BANNERS)
        || !b.item.has_tag(&tag::Item::MINECRAFT_BANNERS)
    {
        return None;
    }

    // Must be the same color banner
    if a.item != b.item {
        return None;
    }

    // Result is a copy of the patterned banner
    // TODO: Copy BannerPatterns component from patterned banner
    // For now, produce a plain banner of the same color
    let result = ItemStack::new(1, a.item);
    Some(result)
}

/// Clone a written book by combining it with a book and quill.
///
/// Takes 1 written book + 1-8 writable books. Produces copies equal to writable book count + 1.
fn try_book_cloning(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() < 2 {
        return None;
    }

    let mut written_book: Option<&ItemStack> = None;
    let mut writable_count = 0u32;

    for item in items {
        if item.item == &Item::WRITTEN_BOOK {
            if written_book.is_some() {
                return None; // Only one written book allowed
            }
            written_book = Some(item);
        } else if item.item == &Item::WRITABLE_BOOK {
            writable_count += 1;
        } else {
            return None;
        }
    }

    written_book?;
    if writable_count == 0 {
        return None;
    }

    // Produce copies of the written book
    // TODO: Copy WrittenBookContent component (title, author, pages, generation)
    // Generation increments by 1 (original=0, copy=1, copy of copy=2, max=2)
    let count = (writable_count + 1).min(u8::MAX as u32) as u8;
    let result = ItemStack::new(count, &Item::WRITTEN_BOOK);
    Some(result)
}

/// Craft a firework rocket from paper, gunpowder, and optional firework stars.
///
/// Requires 1 paper + 1-3 gunpowder + 0-7 firework stars.
fn try_firework_rocket(items: &[ItemStack]) -> Option<ItemStack> {
    if items.is_empty() || items.len() > 11 {
        return None;
    }

    let mut paper_count = 0u32;
    let mut gunpowder_count = 0u32;
    let mut _star_count = 0u32;

    for item in items {
        if item.item == &Item::PAPER {
            paper_count += 1;
        } else if item.item == &Item::GUNPOWDER {
            gunpowder_count += 1;
        } else if item.item == &Item::FIREWORK_STAR {
            _star_count += 1;
        } else {
            return None;
        }
    }

    if paper_count != 1 || !(1..=3).contains(&gunpowder_count) {
        return None;
    }

    // Flight duration = gunpowder count (1-3)
    // TODO: Apply Fireworks component with flight duration and explosion data from stars
    let result = ItemStack::new(3, &Item::FIREWORK_ROCKET);
    Some(result)
}

/// Craft a firework star from gunpowder + dye(s) + optional modifiers.
///
/// Requires 1 gunpowder + 1-8 dyes + optional shape/trail/twinkle modifiers.
fn try_firework_star(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() < 2 {
        return None;
    }

    let mut gunpowder = false;
    let mut dye_count = 0u32;
    let mut has_invalid = false;

    for item in items {
        if item.item == &Item::GUNPOWDER {
            if gunpowder {
                has_invalid = true;
                break;
            }
            gunpowder = true;
        } else if item.item.has_tag(&tag::Item::C_DYES) {
            dye_count += 1;
        } else if is_firework_shape_modifier(item.item)
            || is_firework_effect_modifier(item.item)
        {
            // Shape modifiers: fire charge (large ball), gold nugget (star), head (creeper),
            //                  feather (burst)
            // Effect modifiers: diamond (trail), glowstone dust (twinkle)
            continue;
        } else {
            has_invalid = true;
            break;
        }
    }

    if !gunpowder || dye_count == 0 || has_invalid {
        return None;
    }

    // TODO: Apply FireworkExplosion component with colors, shape, trail, twinkle
    let result = ItemStack::new(1, &Item::FIREWORK_STAR);
    Some(result)
}

/// Add fade colors to an existing firework star.
///
/// Takes 1 firework star + 1-8 dyes.
fn try_firework_star_fade(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() < 2 {
        return None;
    }

    let mut star: Option<&ItemStack> = None;
    let mut dye_count = 0u32;

    for item in items {
        if item.item == &Item::FIREWORK_STAR {
            if star.is_some() {
                return None;
            }
            star = Some(item);
        } else if item.item.has_tag(&tag::Item::C_DYES) {
            dye_count += 1;
        } else {
            return None;
        }
    }

    star?;
    if dye_count == 0 {
        return None;
    }

    // TODO: Copy FireworkExplosion component from input star, add fade colors from dyes
    let result = ItemStack::new(1, &Item::FIREWORK_STAR);
    Some(result)
}

/// Clone a filled map by combining it with empty maps.
///
/// Takes 1 filled map + 1-8 empty maps. Produces copies equal to empty map count + 1.
fn try_map_cloning(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() < 2 {
        return None;
    }

    let mut filled_map: Option<&ItemStack> = None;
    let mut empty_count = 0u32;

    for item in items {
        if item.item == &Item::FILLED_MAP {
            if filled_map.is_some() {
                return None;
            }
            filled_map = Some(item);
        } else if item.item == &Item::MAP {
            empty_count += 1;
        } else {
            return None;
        }
    }

    filled_map?;
    if empty_count == 0 {
        return None;
    }

    // TODO: Copy MapId and MapDecorations components from filled map
    let count = (empty_count + 1).min(u8::MAX as u32) as u8;
    let result = ItemStack::new(count, &Item::FILLED_MAP);
    Some(result)
}

/// Extend a map by surrounding it with paper.
///
/// Takes 1 filled map + 8 paper in a 3x3 grid (paper surrounding map in center).
/// Note: This checks item composition only, not grid position.
fn try_map_extending(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() != 9 {
        return None;
    }

    let mut filled_map_count = 0u32;
    let mut paper_count = 0u32;

    for item in items {
        if item.item == &Item::FILLED_MAP {
            filled_map_count += 1;
        } else if item.item == &Item::PAPER {
            paper_count += 1;
        } else {
            return None;
        }
    }

    if filled_map_count != 1 || paper_count != 8 {
        return None;
    }

    // TODO: Copy map data and increase scale level
    let result = ItemStack::new(1, &Item::FILLED_MAP);
    Some(result)
}

/// Apply a banner pattern to a shield.
///
/// Takes 1 shield + 1 banner.
fn try_shield_decoration(items: &[ItemStack]) -> Option<ItemStack> {
    if items.len() != 2 {
        return None;
    }

    let mut shield: Option<&ItemStack> = None;
    let mut _banner: Option<&ItemStack> = None;

    for item in items {
        if item.item == &Item::SHIELD {
            if shield.is_some() {
                return None;
            }
            shield = Some(item);
        } else if item.item.has_tag(&tag::Item::MINECRAFT_BANNERS) {
            if _banner.is_some() {
                return None;
            }
            _banner = Some(item);
        } else {
            return None;
        }
    }

    shield?;
    _banner?;

    // TODO: Copy BannerPatterns and BaseColor from banner to shield
    let result = ItemStack::new(1, &Item::SHIELD);
    Some(result)
}

/// Check if an item is a firework shape modifier.
fn is_firework_shape_modifier(item: &Item) -> bool {
    item == &Item::FIRE_CHARGE      // Large ball
        || item == &Item::GOLD_NUGGET   // Star shape
        || is_mob_head(item)            // Creeper shape
        || item == &Item::FEATHER       // Burst shape
}

/// Check if an item is a firework effect modifier.
fn is_firework_effect_modifier(item: &Item) -> bool {
    item == &Item::DIAMOND          // Trail effect
        || item == &Item::GLOWSTONE_DUST // Twinkle effect
}

/// Check if an item is a mob head (for creeper-shaped fireworks).
fn is_mob_head(item: &Item) -> bool {
    item == &Item::CREEPER_HEAD
        || item == &Item::ZOMBIE_HEAD
        || item == &Item::SKELETON_SKULL
        || item == &Item::WITHER_SKELETON_SKULL
        || item == &Item::PLAYER_HEAD
        || item == &Item::DRAGON_HEAD
        || item == &Item::PIGLIN_HEAD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_two_damaged_items() {
        // Two diamond pickaxes at half durability
        let mut a = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        a.set_damage(780); // half of max 1561
        let mut b = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        b.set_damage(780);

        let result = try_repair_item(&[a, b]);
        assert!(result.is_some(), "Two same-type damaged items should repair");
        let result = result.unwrap();
        assert!(result.item == &Item::DIAMOND_PICKAXE);
        // Each has 781 remaining (1561-780). Combined = 1562. Bonus = 78 (5%).
        // New remaining = min(1640, 1561) = 1561. New damage = 0.
        assert!(result.get_damage() == 0, "Fully repaired");
    }

    #[test]
    fn repair_different_items_fails() {
        let a = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        let b = ItemStack::new(1, &Item::DIAMOND_AXE);
        assert!(try_repair_item(&[a, b]).is_none());
    }

    #[test]
    fn repair_non_damageable_fails() {
        let a = ItemStack::new(1, &Item::DIAMOND);
        let b = ItemStack::new(1, &Item::DIAMOND);
        assert!(try_repair_item(&[a, b]).is_none());
    }

    #[test]
    fn armor_dye_leather_helmet() {
        let armor = ItemStack::new(1, &Item::LEATHER_HELMET);
        let dye = ItemStack::new(1, &Item::RED_DYE);
        let result = try_armor_dye(&[armor, dye]);
        assert!(result.is_some(), "Leather helmet + dye should match");
        assert!(result.unwrap().item == &Item::LEATHER_HELMET);
    }

    #[test]
    fn armor_dye_multiple_dyes() {
        let armor = ItemStack::new(1, &Item::LEATHER_CHESTPLATE);
        let dye1 = ItemStack::new(1, &Item::RED_DYE);
        let dye2 = ItemStack::new(1, &Item::BLUE_DYE);
        let result = try_armor_dye(&[armor, dye1, dye2]);
        assert!(result.is_some(), "Leather armor + multiple dyes should match");
    }

    #[test]
    fn armor_dye_non_dyeable_fails() {
        let armor = ItemStack::new(1, &Item::IRON_CHESTPLATE);
        let dye = ItemStack::new(1, &Item::RED_DYE);
        assert!(try_armor_dye(&[armor, dye]).is_none());
    }

    #[test]
    fn armor_dye_no_dye_fails() {
        let armor = ItemStack::new(1, &Item::LEATHER_HELMET);
        assert!(try_armor_dye(&[armor]).is_none());
    }

    #[test]
    fn tipped_arrow_recipe() {
        let mut items = Vec::new();
        for _ in 0..8 {
            items.push(ItemStack::new(1, &Item::ARROW));
        }
        items.push(ItemStack::new(1, &Item::LINGERING_POTION));
        let result = try_tipped_arrow(&items);
        assert!(result.is_some(), "8 arrows + lingering potion should match");
        let r = result.unwrap();
        assert!(r.item == &Item::TIPPED_ARROW);
        assert!(r.item_count == 8);
    }

    #[test]
    fn tipped_arrow_wrong_count_fails() {
        let mut items = Vec::new();
        for _ in 0..7 {
            items.push(ItemStack::new(1, &Item::ARROW));
        }
        items.push(ItemStack::new(1, &Item::LINGERING_POTION));
        assert!(try_tipped_arrow(&items).is_none(), "7 arrows + potion should not match");
    }

    #[test]
    fn firework_rocket_basic() {
        let items = vec![
            ItemStack::new(1, &Item::PAPER),
            ItemStack::new(1, &Item::GUNPOWDER),
        ];
        let result = try_firework_rocket(&items);
        assert!(result.is_some(), "Paper + gunpowder should make firework rocket");
        assert!(result.unwrap().item == &Item::FIREWORK_ROCKET);
    }

    #[test]
    fn firework_rocket_with_stars() {
        let items = vec![
            ItemStack::new(1, &Item::PAPER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::FIREWORK_STAR),
        ];
        let result = try_firework_rocket(&items);
        assert!(result.is_some(), "Paper + 2 gunpowder + star should work");
    }

    #[test]
    fn firework_star_basic() {
        let items = vec![
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::RED_DYE),
        ];
        let result = try_firework_star(&items);
        assert!(result.is_some(), "Gunpowder + dye should make firework star");
        assert!(result.unwrap().item == &Item::FIREWORK_STAR);
    }

    #[test]
    fn firework_star_with_shape_modifier() {
        let items = vec![
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::BLUE_DYE),
            ItemStack::new(1, &Item::FIRE_CHARGE), // Large ball shape
        ];
        let result = try_firework_star(&items);
        assert!(result.is_some(), "Gunpowder + dye + shape should work");
    }

    #[test]
    fn firework_star_no_dye_fails() {
        let items = vec![ItemStack::new(1, &Item::GUNPOWDER)];
        assert!(try_firework_star(&items).is_none());
    }

    #[test]
    fn firework_star_fade() {
        let items = vec![
            ItemStack::new(1, &Item::FIREWORK_STAR),
            ItemStack::new(1, &Item::GREEN_DYE),
        ];
        let result = try_firework_star_fade(&items);
        assert!(result.is_some(), "Star + dye should add fade colors");
    }

    #[test]
    fn book_cloning_basic() {
        let items = vec![
            ItemStack::new(1, &Item::WRITTEN_BOOK),
            ItemStack::new(1, &Item::WRITABLE_BOOK),
        ];
        let result = try_book_cloning(&items);
        assert!(result.is_some(), "Written book + writable book should clone");
        let r = result.unwrap();
        assert!(r.item == &Item::WRITTEN_BOOK);
        assert!(r.item_count == 2, "Should produce 2 copies (original + 1)");
    }

    #[test]
    fn map_cloning_basic() {
        let items = vec![
            ItemStack::new(1, &Item::FILLED_MAP),
            ItemStack::new(1, &Item::MAP),
        ];
        let result = try_map_cloning(&items);
        assert!(result.is_some(), "Filled map + empty map should clone");
        assert!(result.unwrap().item_count == 2);
    }

    #[test]
    fn map_extending() {
        let mut items = Vec::new();
        items.push(ItemStack::new(1, &Item::FILLED_MAP));
        for _ in 0..8 {
            items.push(ItemStack::new(1, &Item::PAPER));
        }
        let result = try_map_extending(&items);
        assert!(result.is_some(), "Filled map + 8 paper should extend");
        assert!(result.unwrap().item == &Item::FILLED_MAP);
    }

    #[test]
    fn shield_decoration() {
        let items = vec![
            ItemStack::new(1, &Item::SHIELD),
            ItemStack::new(1, &Item::WHITE_BANNER),
        ];
        let result = try_shield_decoration(&items);
        assert!(result.is_some(), "Shield + banner should decorate");
        assert!(result.unwrap().item == &Item::SHIELD);
    }

    #[test]
    fn shield_decoration_no_banner_fails() {
        let items = vec![
            ItemStack::new(1, &Item::SHIELD),
            ItemStack::new(1, &Item::DIAMOND),
        ];
        assert!(try_shield_decoration(&items).is_none());
    }

    #[test]
    fn special_recipe_type_enum_covers_all_11() {
        // Compile-time check: all 11 variants exist
        let types = [
            SpecialRecipeType::ArmorDye,
            SpecialRecipeType::BannerDuplicate,
            SpecialRecipeType::BookCloning,
            SpecialRecipeType::FireworkRocket,
            SpecialRecipeType::FireworkStar,
            SpecialRecipeType::FireworkStarFade,
            SpecialRecipeType::MapCloning,
            SpecialRecipeType::MapExtending,
            SpecialRecipeType::RepairItem,
            SpecialRecipeType::ShieldDecoration,
            SpecialRecipeType::TippedArrow,
        ];
        assert_eq!(types.len(), 11);
    }
}
