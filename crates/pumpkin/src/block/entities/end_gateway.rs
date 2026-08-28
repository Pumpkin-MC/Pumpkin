use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct EndGatewayBlockEntity {
    pub position: BlockPos,
    pub age: Mutex<i64>,
    pub exact_teleport: Mutex<bool>,
    pub exit_portal: Mutex<Option<BlockPos>>,
}

impl BlockEntity for EndGatewayBlockEntity {
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
        let age = nbt.get_long("Age").unwrap_or(0);
        let exact_teleport = nbt.get_bool("ExactTeleport").unwrap_or(false);
        let exit_portal = nbt.get_compound("ExitPortal").map(|c| {
            BlockPos::new(
                c.get_int("X").unwrap_or(0),
                c.get_int("Y").unwrap_or(0),
                c.get_int("Z").unwrap_or(0),
            )
        });
        Self {
            position,
            age: Mutex::new(age),
            exact_teleport: Mutex::new(exact_teleport),
            exit_portal: Mutex::new(exit_portal),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(age) = self.age.lock() {
            nbt.put_long("Age", *age);
        }
        if let Ok(exact_teleport) = self.exact_teleport.lock() {
            nbt.put_bool("ExactTeleport", *exact_teleport);
        }
        if let Ok(exit_portal) = self.exit_portal.lock()
            && let Some(exit) = exit_portal.as_ref()
        {
            let mut exit_nbt = NbtCompound::new();
            exit_nbt.put_int("X", exit.0.x);
            exit_nbt.put_int("Y", exit.0.y);
            exit_nbt.put_int("Z", exit.0.z);
            nbt.put_compound("ExitPortal", exit_nbt);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_long("Age", *self.age.try_lock().ok()?);
        nbt.put_bool("ExactTeleport", *self.exact_teleport.try_lock().ok()?);
        if let Ok(exit) = self.exit_portal.try_lock()
            && let Some(ref exit) = *exit
        {
            let mut exit_nbt = NbtCompound::new();
            exit_nbt.put_int("X", exit.0.x);
            exit_nbt.put_int("Y", exit.0.y);
            exit_nbt.put_int("Z", exit.0.z);
            nbt.put_compound("ExitPortal", exit_nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl EndGatewayBlockEntity {
    pub const ID: &'static str = "minecraft:end_gateway";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            age: Mutex::new(0),
            exact_teleport: Mutex::new(false),
            exit_portal: Mutex::new(None),
        }
    }
}
