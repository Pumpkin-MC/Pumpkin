use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};

use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    PermissionLvl, math::vector2::Vector2, text::TextComponent, version::JavaMinecraftVersion,
};

use crate::{
    block::entities::command_block::CommandBlockEntity,
    command::{
        CommandSender, CommandSource,
        context::command_source::{ResultValueTaker, ReturnValue, ReturnValueCallable},
    },
    entity::{Entity, EntityBase, player::Player},
    world::World,
};

struct SuccessCounter {
    count: Arc<AtomicI32>,
}

impl ReturnValueCallable for SuccessCounter {
    fn call(&self, value: ReturnValue) {
        if value.success_value() {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Vanilla `MinecartCommandBlock` + `BaseCommandBlock`.
pub struct CommandBlockMinecart {
    tick_count: AtomicI32,
    last_activated: AtomicI32,
    last_execution: AtomicI64,
    update_last_execution: AtomicBool,
    success_count: AtomicI32,
    track_output: AtomicBool,
    last_output: StdMutex<Option<TextComponent>>,
    command: StdMutex<String>,
    custom_name: StdMutex<Option<TextComponent>>,
}

impl CommandBlockMinecart {
    const ACTIVATION_DELAY: i32 = 4;
    const NO_LAST_EXECUTION: i64 = -1;

    pub(super) const fn new() -> Self {
        Self {
            tick_count: AtomicI32::new(0),
            last_activated: AtomicI32::new(0),
            last_execution: AtomicI64::new(Self::NO_LAST_EXECUTION),
            update_last_execution: AtomicBool::new(true),
            success_count: AtomicI32::new(0),
            track_output: AtomicBool::new(true),
            last_output: StdMutex::new(None),
            command: StdMutex::new(String::new()),
            custom_name: StdMutex::new(None),
        }
    }

    #[must_use]
    pub fn get_command(&self) -> String {
        self.command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_command(&self, command: String) {
        *self
            .command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = command;
        self.success_count.store(0, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_last_output_or_empty(&self) -> TextComponent {
        self.last_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_else(TextComponent::empty)
    }

    pub fn set_last_output(&self, output: Option<TextComponent>) {
        *self
            .last_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = output;
    }

    #[must_use]
    pub fn is_track_output(&self) -> bool {
        self.track_output.load(Ordering::Relaxed)
    }

    pub fn set_track_output(&self, track_output: bool) {
        self.track_output.store(track_output, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_success_count(&self) -> i32 {
        self.success_count.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn get_name(&self) -> TextComponent {
        self.custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_else(|| TextComponent::text("@"))
    }

    pub fn on_updated(&self, entity: &Entity) {
        entity.set_synced_data(
            tracked_data::command_block_minecart::DATA_ID_COMMAND_NAME,
            self.get_command(),
        );
        entity.set_synced_data(
            tracked_data::command_block_minecart::DATA_ID_LAST_OUTPUT,
            self.get_last_output_or_empty(),
        );
    }

    pub(super) fn init_data_tracker(&self, entity: &Entity) {
        self.on_updated(entity);
    }

    pub(super) fn step_tick(&self) {
        self.tick_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn activate(&self, entity: &Entity) {
        let tick = self.tick_count.load(Ordering::Relaxed);
        let last = self.last_activated.load(Ordering::Relaxed);
        if tick - last >= Self::ACTIVATION_DELAY {
            let world = entity.world.load_full();
            self.perform_command(entity, &world);
            self.last_activated.store(tick, Ordering::Relaxed);
        }
    }

    pub fn perform_command(&self, entity: &Entity, world: &Arc<World>) -> bool {
        let game_time = world.get_world_age();
        if game_time == self.last_execution.load(Ordering::Relaxed) {
            return false;
        }

        let command = self.get_command();
        if command.eq_ignore_ascii_case("Searge") {
            self.set_last_output(Some(TextComponent::text("#itzlipofutzli")));
            self.success_count.store(1, Ordering::Relaxed);
            self.on_updated(entity);
            return true;
        }

        self.success_count.store(0, Ordering::Relaxed);
        if world.level_info.load().game_rules.command_blocks_work && !command.is_empty() {
            self.set_last_output(None);
            if let Some(server) = world.server.upgrade() {
                let track_output = self.is_track_output();
                let proxy = Arc::new(CommandBlockEntity::new(
                    entity.block_pos.load(),
                    track_output,
                    false,
                ));
                let output = if track_output {
                    CommandSender::CommandBlock(proxy.clone(), world.clone())
                } else {
                    CommandSender::Dummy
                };
                let counter = Arc::new(AtomicI32::new(0));
                let name = self.get_name();
                let dyn_entity: Option<Arc<dyn EntityBase>> =
                    world.get_entity_by_id(entity.entity_id);
                let source = CommandSource::new(
                    output,
                    world.clone(),
                    dyn_entity,
                    entity.pos.load(),
                    Vector2::new(entity.yaw.load(), entity.pitch.load()),
                    name.clone().get_text(),
                    name,
                    server.clone(),
                )
                .with_command_result_taker(ResultValueTaker(vec![Arc::new(SuccessCounter {
                    count: counter.clone(),
                })]));

                server
                    .command_dispatcher
                    .load()
                    .handle_command(&source, &command);

                self.success_count
                    .store(counter.load(Ordering::Relaxed), Ordering::Relaxed);
                if track_output {
                    let captured = proxy
                        .last_output
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone();
                    if !captured.is_empty() {
                        self.set_last_output(Some(TextComponent::text(captured)));
                        self.on_updated(entity);
                    }
                }
            }
        }

        let next_last_exec = if self.update_last_execution.load(Ordering::Relaxed) {
            game_time
        } else {
            Self::NO_LAST_EXECUTION
        };
        self.last_execution.store(next_last_exec, Ordering::Relaxed);
        true
    }

    pub(super) fn interact(player: &Player) -> bool {
        player.is_creative() && player.permission_lvl.load() >= PermissionLvl::Two
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Command", self.get_command());
        nbt.put_int("SuccessCount", self.get_success_count());
        if let Some(custom_name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put(
                "CustomName",
                custom_name.to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3),
            );
        }
        let track_output = self.is_track_output();
        nbt.put_bool("TrackOutput", track_output);
        if track_output
            && let Some(last_output) = self
                .last_output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
        {
            nbt.put(
                "LastOutput",
                last_output.to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3),
            );
        }
        let update_last = self.update_last_execution.load(Ordering::Relaxed);
        nbt.put_bool("UpdateLastExecution", update_last);
        let last_exec = self.last_execution.load(Ordering::Relaxed);
        if update_last && last_exec != Self::NO_LAST_EXECUTION {
            nbt.put_long("LastExecution", last_exec);
        }
    }

    pub(super) fn read_nbt(&self, entity: &Entity, nbt: &NbtCompound) {
        *self
            .command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get_string("Command").unwrap_or("").to_string();
        self.success_count
            .store(nbt.get_int("SuccessCount").unwrap_or(0), Ordering::Relaxed);
        *self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get("CustomName").map(TextComponent::from_nbt);

        let track_output = nbt.get_bool("TrackOutput").unwrap_or(true);
        self.track_output.store(track_output, Ordering::Relaxed);
        let last_output = if track_output {
            nbt.get("LastOutput").map(TextComponent::from_nbt)
        } else {
            None
        };
        self.set_last_output(last_output);

        let update_last = nbt.get_bool("UpdateLastExecution").unwrap_or(true);
        self.update_last_execution
            .store(update_last, Ordering::Relaxed);
        let last_exec = if update_last {
            nbt.get_long("LastExecution")
                .unwrap_or(Self::NO_LAST_EXECUTION)
        } else {
            Self::NO_LAST_EXECUTION
        };
        self.last_execution.store(last_exec, Ordering::Relaxed);

        self.on_updated(entity);
    }
}
