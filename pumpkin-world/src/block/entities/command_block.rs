use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use tokio::sync::Mutex;

use super::BlockEntity;

// todo: CustomName, LastExecution, UpdateLastExecution
pub struct CommandBlockEntity {
    pub position: BlockPos,
    pub powered: AtomicBool,
    pub condition_met: AtomicBool,
    pub auto: AtomicBool,
    pub dirty: AtomicBool,
    pub command: Mutex<String>,
    pub last_output: Mutex<String>,
    pub track_output: AtomicBool,
    pub success_count: AtomicU32,
}

impl CommandBlockEntity {
    pub const ID: &'static str = "minecraft:command_block";
    pub fn new(position: BlockPos, track_output: bool, is_chain: bool) -> Self {
        Self {
            position,
            powered: AtomicBool::new(false),
            condition_met: AtomicBool::new(false),
            auto: AtomicBool::new(is_chain),
            dirty: AtomicBool::new(false),
            command: Mutex::new(String::new()),
            last_output: Mutex::new(String::new()),
            track_output: AtomicBool::new(track_output),
            success_count: AtomicU32::new(0),
        }
    }
}

impl BlockEntity for CommandBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }
    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let condition_met = AtomicBool::new(nbt.get_bool("conditionMet").unwrap_or(false));
        let auto = AtomicBool::new(nbt.get_bool("auto").unwrap_or(false));
        let powered = AtomicBool::new(nbt.get_bool("powered").unwrap_or(false));
        let command = Mutex::new(nbt.get_string("Command").unwrap_or("").to_string());
        let last_output = Mutex::new(nbt.get_string("LastOutput").unwrap_or("").to_string());
        let track_output = AtomicBool::new(nbt.get_bool("TrackOutput").unwrap_or(false));
        let success_count =
            AtomicU32::new(nbt.get_int("SuccessCount").unwrap_or(0).cast_unsigned());

        Self {
            position,
            condition_met,
            auto,
            powered,
            command,
            last_output,
            track_output,
            success_count,
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            nbt.put_bool("auto", self.auto.load(Ordering::SeqCst));
            nbt.put_string("Command", self.command.lock().await.to_string());
            nbt.put_bool("conditionMet", self.condition_met.load(Ordering::SeqCst));
            nbt.put_string("LastOutput", self.last_output.lock().await.to_string());
            nbt.put_bool("powered", self.powered.load(Ordering::SeqCst));
            nbt.put_bool("TrackOutput", self.track_output.load(Ordering::SeqCst));
            nbt.put_bool("UpdateLastExecution", false);
            nbt.put_int(
                "SuccessCount",
                self.success_count.load(Ordering::SeqCst).cast_signed(),
            );
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        futures::executor::block_on(async {
            self.write_nbt(&mut nbt).await;
        });
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
