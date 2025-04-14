use crate::Container;
use pumpkin_data::block::Block;
use pumpkin_data::screen::WindowType;
use pumpkin_util::math::position::BlockPos;
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
pub struct ChestContainer([Option<ItemStack>; 27]);

impl ChestContainer {
    pub fn new() -> Self {
        Self([const { None }; 27])
    }
}
impl Container for ChestContainer {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x3
    }

    fn window_name(&self) -> &'static str {
        "Chest"
    }
    fn all_slots(&mut self) -> Box<[&mut Option<ItemStack>]> {
        self.0.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Box<[Option<&ItemStack>]> {
        self.0.iter().map(|slot| slot.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct CraftingTable {
    input: [[Option<ItemStack>; 3]; 3],
    output: Option<ItemStack>,
}

impl CraftingTable {
    const SLOT_OUTPUT: usize = 0;
    const SLOT_INPUT_START: usize = 1;
    const SLOT_INPUT_END: usize = 9;
}

impl Container for CraftingTable {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Crafting
    }

    fn window_name(&self) -> &'static str {
        "Crafting Table"
    }
    fn all_slots(&mut self) -> Box<[&mut Option<ItemStack>]> {
        let slots = vec![&mut self.output];

        slots
            .into_iter()
            .chain(self.input.iter_mut().flatten())
            .collect()
    }

    fn all_slots_ref(&self) -> Box<[Option<&ItemStack>]> {
        let slots = vec![self.output.as_ref()];

        slots
            .into_iter()
            .chain(self.input.iter().flatten().map(|i| i.as_ref()))
            .collect()
    }

    fn all_combinable_slots(&self) -> Box<[Option<&ItemStack>]> {
        self.input.iter().flatten().map(|s| s.as_ref()).collect()
    }

    fn all_combinable_slots_mut(&mut self) -> Box<[&mut Option<ItemStack>]> {
        self.input.iter_mut().flatten().collect()
    }

    fn craft(&mut self) -> bool {
        // TODO: Is there a better way to do this?
        let _check = [
            [
                self.input[0][0].as_ref(),
                self.input[0][1].as_ref(),
                self.input[0][2].as_ref(),
            ],
            [
                self.input[1][0].as_ref(),
                self.input[1][1].as_ref(),
                self.input[1][2].as_ref(),
            ],
            [
                self.input[2][0].as_ref(),
                self.input[2][1].as_ref(),
                self.input[2][2].as_ref(),
            ],
        ];

        let new_output = None; //check_if_matches_crafting(check);
        let result = new_output != self.output
            || self.input.iter().flatten().any(|s| s.is_some())
            || new_output.is_some();

        self.output = new_output;
        result
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(Self::SLOT_OUTPUT)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        (Self::SLOT_INPUT_START..=Self::SLOT_INPUT_END).contains(slot)
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
    fn all_slots(&mut self) -> Box<[&mut Option<ItemStack>]> {
        Box::new([&mut self.cook, &mut self.fuel, &mut self.output])
    }

    fn all_slots_ref(&self) -> Box<[Option<&ItemStack>]> {
        Box::new([self.cook.as_ref(), self.fuel.as_ref(), self.output.as_ref()])
    }
}
