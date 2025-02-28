use crate::Container;
use crate::crafting::check_if_matches_crafting;
use crate::player::PlayerInventory;
use pumpkin_data::screen::WindowType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::ItemStack;
use rand::random;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct ContainerHolder {
    pub containers_by_id: HashMap<usize, OpenContainer>,
    pub location_to_container_id: HashMap<BlockPos, usize>,
}

impl ContainerHolder {
    pub async fn destroy(
        &mut self,
        id: usize,
        player_inventory: &mut PlayerInventory,
        carried_item: &mut Option<ItemStack>,
    ) -> Vec<Uuid> {
        if let Some(container) = self.containers_by_id.remove(&id) {
            let unique = container.unique;
            let players = container.players;
            let mut container = container.container.lock().await;
            container.destroy_container(player_inventory, carried_item, unique);
            players
        } else {
            vec![]
        }
    }

    pub async fn destroy_by_location(
        &mut self,
        location: &BlockPos,
        player_inventory: &mut PlayerInventory,
        carried_item: &mut Option<ItemStack>,
    ) -> Vec<Uuid> {
        if let Some(id) = self.location_to_container_id.remove(location) {
            self.destroy(id, player_inventory, carried_item).await
        } else {
            vec![]
        }
    }

    pub fn get_by_location(&self, location: &BlockPos) -> Option<&OpenContainer> {
        self.containers_by_id
            .get(self.location_to_container_id.get(location)?)
    }

    pub fn get_mut_by_location(&mut self, location: &BlockPos) -> Option<&mut OpenContainer> {
        self.containers_by_id
            .get_mut(self.location_to_container_id.get(location)?)
    }

    pub fn new_by_location<C: Container + Default + 'static>(
        &mut self,
        player_id: Uuid,
        location: BlockPos,
        block: Option<Block>,
    ) -> Option<&mut OpenContainer> {
        if self.location_to_container_id.contains_key(&location) {
            return None;
        }
        let id = self.new_container::<C>(player_id, block, false);
        self.location_to_container_id.insert(location, id);
        self.containers_by_id.get_mut(&id)
    }

    pub fn new_container<C: Container + Default + 'static>(
        &mut self,
        player_id: Uuid,
        block: Option<Block>,
        unique: bool,
    ) -> usize {
        let mut id: usize = random();
        let mut new_container =
            OpenContainer::new_empty_container::<C>(player_id, None, block, unique);
        while let Some(container) = self.containers_by_id.insert(id, new_container) {
            new_container = container;
            id = random();
        }
        id
    }

    pub fn new_unique<C: Container + Default + 'static>(
        &mut self,
        block: Option<Block>,
        player_id: Uuid,
    ) -> usize {
        let id = self.new_container::<C>(player_id, block, true);
        let container = self.containers_by_id.get_mut(&id).expect("just created it");
        container.players.push(player_id);
        id
    }
}

pub struct OpenContainer {
    pub unique: bool,
    pub id: usize,
    container: Arc<Mutex<Box<dyn Container>>>,
    players: Vec<Uuid>,
    location: Option<BlockPos>,
    block: Option<Block>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: Uuid) -> Option<&Arc<Mutex<Box<dyn Container>>>> {
        if !self.players.contains(&player_id) {
            log::debug!("couldn't open container");
            return None;
        }
        let container = &self.container;
        Some(container)
    }

    pub fn add_player(&mut self, player_id: Uuid) {
        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }
    }

    pub fn remove_player(&mut self, player_id: Uuid) {
        if let Some(index) = self.players.iter().enumerate().find_map(|(index, id)| {
            if *id == player_id { Some(index) } else { None }
        }) {
            self.players.remove(index);
        }
    }

    pub fn new_empty_container<C: Container + Default + 'static>(
        player_id: Uuid,
        location: Option<BlockPos>,
        block: Option<Block>,
        unique: bool,
    ) -> Self {
        Self {
            unique,
            players: vec![player_id],
            container: Arc::new(Mutex::new(Box::new(C::default()))),
            location,
            block,
            id: 0,
        }
    }

    pub fn is_location(&self, try_position: BlockPos) -> bool {
        if let Some(location) = self.location {
            location == try_position
        } else {
            false
        }
    }

    pub fn clear_all_players(&mut self) {
        self.players.clear();
    }

    pub fn all_player_ids(&self) -> Vec<Uuid> {
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

    pub async fn window_type(&self) -> &'static WindowType {
        let container = self.container.lock().await;
        container.window_type()
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
