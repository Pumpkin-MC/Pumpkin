use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::{
    BlockDirection, data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::{GameMode, Hand, PermissionLvl};
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockHitResult, registry::BlockActionResult};
use crate::entity::{
    EntityBase,
    player::{ClientMove, ClientMoveOutcome, Player},
};
use crate::plugin::player::item_held::PlayerItemHeldEvent;
use crate::server::Server;

/// The result of a simulated player action.
pub type SimulatedPlayerResult<T> = Result<T, String>;

/// Controls a player admitted to the server without a network connection.
pub struct SimulatedPlayer {
    player: Arc<Player>,
    server: Weak<Server>,
}

impl SimulatedPlayer {
    pub(crate) const fn new(player: Arc<Player>, server: Weak<Server>) -> Self {
        Self { player, server }
    }

    #[must_use]
    pub const fn player(&self) -> &Arc<Player> {
        &self.player
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.player
            .world()
            .get_player_by_uuid(self.player.gameprofile.id)
            .is_some()
    }

    #[must_use]
    pub fn entity_id(&self) -> i32 {
        self.player.entity_id()
    }

    pub fn set_visible_in_list(&self, visible: bool) {
        self.player
            .client
            .set_simulated_player_list_visible(visible);
    }

    pub fn set_visible_in_tab_list(&self, visible: bool) {
        self.player.set_tab_list_listed(visible);
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.player.gameprofile.name
    }

    #[must_use]
    pub fn position(&self) -> pumpkin_util::math::vector3::Vector3<f64> {
        self.player.position()
    }

    #[must_use]
    pub fn gamemode(&self) -> GameMode {
        self.player.gamemode.load()
    }

    pub fn set_gamemode(&self, mode: GameMode) -> SimulatedPlayerResult<()> {
        self.player.set_gamemode(mode);
        Ok(())
    }

    pub fn set_permission_level(&self, level: PermissionLvl) -> SimulatedPlayerResult<()> {
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let dispatcher = server.command_dispatcher.load();
        self.player.set_permission_lvl(&server, level, &dispatcher);
        Ok(())
    }

    pub fn run_command(&self, command: &str) -> SimulatedPlayerResult<()> {
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let dispatcher = server.command_dispatcher.load();
        let source = self.player.get_command_source(&server).with_silent();
        dispatcher
            .execute_input(command, &source)
            .map(|_| ())
            .map_err(|error| error.message.get_text())
    }

    #[must_use]
    pub fn selected_hotbar_slot(&self) -> u8 {
        self.player.inventory.get_selected_slot()
    }

    pub fn set_selected_hotbar_slot(&self, slot: u8) -> SimulatedPlayerResult<()> {
        if !self.is_connected() {
            return Err("Simulated player is disconnected".to_string());
        }
        if slot > 8 {
            return Err("Invalid held slot".to_string());
        }
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let mut event = PlayerItemHeldEvent::new(
            self.player.clone(),
            self.player.inventory.get_selected_slot(),
            slot,
        );
        server.plugin_manager.fire_blocking(&server, &mut event);
        if event.cancelled {
            return Err("Held slot change was cancelled".to_string());
        }
        self.player.inventory.set_selected_slot(slot);
        self.player.living_entity.send_equipment_changes(&[(
            EquipmentSlot::MAIN_HAND,
            self.player.inventory.held_item(),
        )]);
        Ok(())
    }

    #[must_use]
    pub fn held_item(&self, hand: Hand) -> ItemStack {
        self.player.inventory.get_stack_in_hand(hand)
    }

    pub fn set_held_item(&self, hand: Hand, stack: ItemStack) -> SimulatedPlayerResult<()> {
        if !self.is_connected() {
            return Err("Simulated player is disconnected".to_string());
        }
        self.player.inventory.set_stack_in_hand(hand, stack.clone());
        let slot = match hand {
            Hand::Right => EquipmentSlot::MAIN_HAND,
            Hand::Left => EquipmentSlot::OFF_HAND,
        };
        self.player
            .living_entity
            .send_equipment_changes(&[(slot, stack)]);
        Ok(())
    }

    pub fn first_empty_inventory_slot(&self) -> Option<usize> {
        self.player
            .inventory
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .position(ItemStack::is_empty)
    }

    pub fn inventory_item(&self, slot: usize) -> SimulatedPlayerResult<ItemStack> {
        self.player
            .inventory
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
            .cloned()
            .ok_or_else(|| "Invalid inventory slot".to_string())
    }

    #[must_use]
    pub fn inventory_count(&self, item: &Item) -> u32 {
        self.player
            .inventory
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|stack| stack.item == item)
            .map(|stack| u32::from(stack.item_count))
            .sum()
    }

    pub fn interact_with(&self, entity_id: i32, hand: Hand) -> SimulatedPlayerResult<()> {
        if !self.is_connected() {
            return Err("Simulated player is disconnected".to_string());
        }
        let world = self.player.world();
        let target: Arc<dyn EntityBase> = world
            .get_player_by_id(entity_id)
            .map(|player| player as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id))
            .ok_or_else(|| "Entity not found".to_string())?;
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let inventory = self.player.inventory();
        let mut stack = inventory.get_stack_in_hand(hand);
        let handled = target.interact_with_hand(&self.player, &mut stack, hand);
        let interacted = handled.unwrap_or_else(|| target.interact(&self.player, &mut stack));
        if !interacted {
            server
                .item_registry
                .use_on_entity(&mut stack, &self.player, target);
        }
        inventory.set_stack_in_hand(hand, stack);
        Ok(())
    }

    pub fn use_block(&self, position: &BlockPos, hand: Hand) -> SimulatedPlayerResult<()> {
        let world = self.player.world();
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let block = world.get_block(position);
        let hit = BlockHitResult {
            face: &BlockDirection::Up,
            cursor_pos: &pumpkin_util::math::vector3::Vector3::new(0.5, 0.5, 0.5),
        };
        let mut held = self.player.inventory.get_stack_in_hand(hand);
        let slot = match hand {
            Hand::Right => EquipmentSlot::MAIN_HAND,
            Hand::Left => EquipmentSlot::OFF_HAND,
        };
        let result = server.block_registry.use_with_item(
            block,
            &self.player,
            position,
            &hit,
            &mut held,
            &slot,
            &server,
            &world,
        );
        let result = if matches!(result, BlockActionResult::PassToDefaultBlockAction) {
            server
                .block_registry
                .on_use(block, &self.player, position, &hit, &server, &world)
        } else {
            result
        };
        self.player.inventory.set_stack_in_hand(hand, held);
        if result.consumes_action() {
            Ok(())
        } else {
            Err(format!("Block use was not handled at {position:?}"))
        }
    }

    pub fn break_block(&self, position: &BlockPos) -> SimulatedPlayerResult<bool> {
        let world = self.player.world();
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let (block, state) = world.get_block_and_state(position);
        let broken = world.break_block(
            position,
            Some(&self.player),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
        );
        if broken.is_some() {
            server
                .block_registry
                .broken(&world, block, &self.player, position, &server, state);
        }
        Ok(broken.is_some())
    }

    pub fn set_sprinting(&self, sprinting: bool) -> SimulatedPlayerResult<()> {
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        self.player.apply_sprint_input(&server, sprinting);
        Ok(())
    }

    #[must_use]
    pub fn is_sprinting(&self) -> bool {
        self.player.get_entity().is_sprinting()
    }

    pub fn teleport_to(
        &self,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> SimulatedPlayerResult<()> {
        self.player.get_entity().set_pos(position);
        Ok(())
    }

    pub fn move_to(
        &self,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> SimulatedPlayerResult<()> {
        if !self.is_connected() {
            return Err("Simulated player is disconnected".to_string());
        }
        let server = self
            .server
            .upgrade()
            .ok_or_else(|| "Server stopped".to_string())?;
        let movement = ClientMove {
            position: Some(position),
            yaw: None,
            pitch: None,
            head_yaw: None,
            on_ground: self
                .player
                .living_entity
                .entity
                .on_ground
                .load(Ordering::Relaxed),
            horizontal_collision: false,
        };
        match self.player.apply_client_move(&server, movement) {
            ClientMoveOutcome::Applied => Ok(()),
            ClientMoveOutcome::Rejected | ClientMoveOutcome::Ignored => {
                Err("Simulated player movement was not applied".to_string())
            }
        }
    }

    pub async fn disconnect(&self) -> SimulatedPlayerResult<()> {
        if !self.is_connected() {
            return Ok(());
        }
        self.player.remove().await;
        if let Some(server) = self.server.upgrade() {
            server.remove_player(&self.player);
        }
        Ok(())
    }

    pub fn look_at_entity(&self, entity_id: i32) -> SimulatedPlayerResult<()> {
        self.player
            .look_at_entity(entity_id)
            .map_err(|err| format!("Failed to look at entity: {err}"))?;
        Ok(())
    }
}
