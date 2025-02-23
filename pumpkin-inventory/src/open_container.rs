use crate::Container;
use crate::crafting::check_if_matches_crafting;
use pumpkin_data::screen::WindowType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct OpenContainer {
    // TODO: unique id should be here
    // TODO: should this be uuid?
    players: Vec<i32>,
    container: Arc<Mutex<Box<dyn Container>>>,
    location: Option<BlockPos>,
    block: Option<Block>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: i32) -> Option<&Arc<Mutex<Box<dyn Container>>>> {
        if !self.players.contains(&player_id) {
            log::debug!("couldn't open container");
            return None;
        }
        let container = &self.container;
        Some(container)
    }

    pub fn add_player(&mut self, player_id: i32) {
        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }
    }

    pub fn remove_player(&mut self, player_id: i32) {
        if let Some(index) = self.players.iter().enumerate().find_map(|(index, id)| {
            if *id == player_id { Some(index) } else { None }
        }) {
            self.players.remove(index);
        }
    }

    pub fn new_empty_container<C: Container + Default + 'static>(
        player_id: i32,
        location: Option<BlockPos>,
        block: Option<Block>,
    ) -> Self {
        Self {
            players: vec![player_id],
            container: Arc::new(Mutex::new(Box::new(C::default()))),
            location,
            block,
        }
    }

    pub fn is_location(&self, try_position: BlockPos) -> bool {
        if let Some(location) = self.location {
            location == try_position
        } else {
            false
        }
    }

    pub async fn clear_all_slots(&self) {
        self.container.lock().await.clear_all_slots();
    }

    pub fn clear_all_players(&mut self) {
        self.players.clear();
    }

    pub fn all_player_ids(&self) -> Vec<i32> {
        self.players.clone()
    }

    pub fn get_number_of_players(&self) -> usize {
        self.players.len()
    }

    pub fn get_location(&self) -> Option<BlockPos> {
        self.location
    }

    pub async fn set_location(&mut self, location: Option<BlockPos>) {
        self.location = location;
    }

    pub fn get_block(&self) -> Option<Block> {
        self.block.clone()
    }
}
#[derive(Default)]
pub struct Chest([Option<ItemStack>; 27]);

impl Chest {
    pub fn new() -> Self {
        Self([None; 27])
    }
}
impl Container for Chest {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x3
    }

    fn window_name(&self) -> &'static str {
        "Chest"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.0.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.0.iter().map(|slot| slot.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct CraftingTable {
    input: [[Option<ItemStack>; 3]; 3],
    output: Option<ItemStack>,
}

impl Container for CraftingTable {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Crafting
    }

    fn window_name(&self) -> &'static str {
        "Crafting Table"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let slots = vec![&mut self.output];
        let slots = slots
            .into_iter()
            .chain(self.input.iter_mut().flatten())
            .collect();
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        let slots = vec![self.output.as_ref()];
        let slots = slots
            .into_iter()
            .chain(self.input.iter().flatten().map(|i| i.as_ref()))
            .collect();
        slots
    }

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.input.iter().flatten().map(|s| s.as_ref()).collect()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.input.iter_mut().flatten().collect()
    }

    fn craft(&mut self) -> bool {
        let old_output = self.output;
        self.output = check_if_matches_crafting(self.input);
        old_output != self.output
            || self.input.iter().flatten().any(|s| s.is_some())
            || self.output.is_some()
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(0)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        (1..10).contains(slot)
    }
    fn recipe_used(&mut self) {
        self.input.iter_mut().flatten().for_each(|slot| {
            if let Some(item) = slot {
                if item.item_count > 1 {
                    item.item_count -= 1;
                } else {
                    *slot = None;
                }
            }
        })
    }
}

#[derive(Default)]
pub struct Furnace {
    cook: Option<ItemStack>,
    fuel: Option<ItemStack>,
    output: Option<ItemStack>,
}

impl Container for Furnace {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Furnace
    }

    fn window_name(&self) -> &'static str {
        "Furnace"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let mut slots = vec![&mut self.cook];
        slots.push(&mut self.fuel);
        slots.push(&mut self.output);
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        let mut slots = vec![self.cook.as_ref()];
        slots.push(self.fuel.as_ref());
        slots.push(self.output.as_ref());
        slots
    }
}

#[derive(Default)]
pub struct Anvil {
    input_left: Option<ItemStack>,
    input_right: Option<ItemStack>,
    output: Option<ItemStack>,
    _repair_cost: i32,
}

impl Container for Anvil {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Anvil
    }

    fn window_name(&self) -> &'static str {
        "Anvil"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![
            &mut self.input_left,
            &mut self.input_right,
            &mut self.output,
        ]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![
            self.input_left.as_ref(),
            self.input_right.as_ref(),
            self.output.as_ref(),
        ]
    }

    fn craft(&mut self) -> bool {
        /*
        TODO: repair logic
        let new_output = match (&self.input_left, &self.input_right) {
            (Some(left), Some(right)) if left.item.id == right.item.id => {
                let mut combined = left.clone();
                combined.item_count += right.item_count;
                combined.item_count = combined.item_count.min(64);
                Some(combined)
            }
            _ => None,
        };

        if self.output != new_output {
            self.output = new_output;
            true
        } else {
            false
        }*/

        false
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(2)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        *slot == 0 || *slot == 1
    }

    fn recipe_used(&mut self) {
        if self.output.take().is_some() {
            self.input_left.take();
            self.input_right.take();
        }
    }
}

#[derive(Default)]
pub struct Beacon {
    payment: Option<ItemStack>,
}

impl Container for Beacon {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Beacon
    }

    fn window_name(&self) -> &'static str {
        "Beacon"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![&mut self.payment]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![self.payment.as_ref()]
    }
}

#[derive(Default)]
pub struct BrewingStand {
    bottles: [Option<ItemStack>; 3],
    ingredient: Option<ItemStack>,
    fuel: Option<ItemStack>,
}

impl Container for BrewingStand {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::BrewingStand
    }

    fn window_name(&self) -> &'static str {
        "Brewing Stand"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let mut slots = self.bottles.iter_mut().collect::<Vec<_>>();
        slots.push(&mut self.ingredient);
        slots.push(&mut self.fuel);
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        let mut slots = self.bottles.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
        slots.push(self.ingredient.as_ref());
        slots.push(self.fuel.as_ref());
        slots
    }
}

#[derive(Default)]
pub struct Hopper {
    items: [Option<ItemStack>; 5],
}

impl Container for Hopper {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Hopper
    }

    fn window_name(&self) -> &'static str {
        "Hopper"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|s| s.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct ShulkerBox {
    items: [Option<ItemStack>; 27],
}

impl Container for ShulkerBox {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::ShulkerBox
    }

    fn window_name(&self) -> &'static str {
        "Shulker Box"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|s| s.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct Dispenser {
    items: [Option<ItemStack>; 9],
}

impl Container for Dispenser {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic3x3
    }

    fn window_name(&self) -> &'static str {
        "Dispenser"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|s| s.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct Dropper {
    items: [Option<ItemStack>; 9],
}

impl Container for Dropper {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic3x3
    }

    fn window_name(&self) -> &'static str {
        "Dropper"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|s| s.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct Stonecutter {
    input: Option<ItemStack>,
    output: Option<ItemStack>,
}

impl Container for Stonecutter {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Stonecutter
    }

    fn window_name(&self) -> &'static str {
        "Stonecutter"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![&mut self.input, &mut self.output]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![self.input.as_ref(), self.output.as_ref()]
    }

    fn craft(&mut self) -> bool {
        /*
        TODO: Add stonecutter craft logic

        let new_output = self.input.as_ref().and_then(|input| {
            pumpkin_registry::get_stonecutter_recipe(input.item.id)
                .map(|recipe| ItemStack::new(recipe.result, recipe.count))
        });

        if self.output != new_output {
            self.output = new_output;
            true
        } else {
            false
        }*/
        false
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(1)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        *slot == 0
    }

    fn recipe_used(&mut self) {
        if self.output.take().is_some() {
            self.input.take();
        }
    }
}

#[derive(Default)]
pub struct Loom {
    banner: Option<ItemStack>,
    dye: Option<ItemStack>,
    pattern: Option<ItemStack>,
    output: Option<ItemStack>,
}

impl Container for Loom {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Loom
    }

    fn window_name(&self) -> &'static str {
        "Loom"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![
            &mut self.banner,
            &mut self.dye,
            &mut self.pattern,
            &mut self.output,
        ]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![
            self.banner.as_ref(),
            self.dye.as_ref(),
            self.pattern.as_ref(),
            self.output.as_ref(),
        ]
    }
}

#[derive(Default)]
pub struct EnchantingTable {
    item: Option<ItemStack>,
    lapis: Option<ItemStack>,
}

impl Container for EnchantingTable {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Enchantment
    }

    fn window_name(&self) -> &'static str {
        "Enchanting Table"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![&mut self.item, &mut self.lapis]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![self.item.as_ref(), self.lapis.as_ref()]
    }
}

#[derive(Default)]
pub struct Grindstone {
    input_top: Option<ItemStack>,
    input_bottom: Option<ItemStack>,
    output: Option<ItemStack>,
}

impl Container for Grindstone {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Grindstone
    }

    fn window_name(&self) -> &'static str {
        "Grindstone"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![
            &mut self.input_top,
            &mut self.input_bottom,
            &mut self.output,
        ]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![
            self.input_top.as_ref(),
            self.input_bottom.as_ref(),
            self.output.as_ref(),
        ]
    }

    fn craft(&mut self) -> bool {
        /*
        TODO: implement grindstone logic

        let new_output = match (&self.input_top, &self.input_bottom) {
            (Some(top), Some(bottom)) if top.item.id == bottom.item.id => {
                let mut combined = top.clone();
                combined.item_count += bottom.item_count;
                combined.item_count = combined.item_count.min(64);
                Some(combined)
            }
            (Some(item), None) | (None, Some(item)) => Some(item.clone()),
            _ => None,
        };

        if self.output != new_output {
            self.output = new_output;
            true
        } else {
            false
        }*/
        false
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(2)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        *slot == 0 || *slot == 1
    }

    fn recipe_used(&mut self) {
        if self.output.take().is_some() {
            self.input_top.take();
            self.input_bottom.take();
        }
    }
}

#[derive(Default)]
pub struct Lectern {
    book: Option<ItemStack>,
}

impl Container for Lectern {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Lectern
    }

    fn window_name(&self) -> &'static str {
        "Lectern"
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        vec![&mut self.book]
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        vec![self.book.as_ref()]
    }
}
