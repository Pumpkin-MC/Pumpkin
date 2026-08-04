use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, CreakingHeartLikeProperties, CreakingHeartState,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::block::blocks::creaking_heart::{creaking_active, has_required_logs};
use crate::world::World;

use super::BlockEntity;

/// `net.minecraft.world.level.block.entity.CreakingHeartBlockEntity`.
///
/// The creaking-mob half (`spawnProtector`, `removeProtector`, `creakingHurt`,
/// `spreadResin` and the hurt-emitter particles) is deliberately not ported: it needs
/// `Creaking.setTransient`/`tearDown`/`playerIsStuckInYou` and a heart<->mob binding that
/// Pumpkin's `CreakingEntity` does not have. The persisted `creaking` UUID is still
/// round-tripped so an existing binding survives save/load once that half lands, and
/// `computeAnalogOutputSignal` returns vanilla's 0 branch for a heart with no live
/// protector -- which is every heart until then.
pub struct CreakingHeartBlockEntity {
    pub position: BlockPos,
    /// `creakingInfo`, reduced to the persisted UUID arm (`Either.right`).
    pub creaking_uuid: Mutex<Option<Uuid>>,
    pub ticker: AtomicI32,
    pub output_signal: AtomicI32,
}

impl BlockEntity for CreakingHeartBlockEntity {
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
        // loadAdditional: read("creaking", UUIDUtil.CODEC) -- four big-endian ints.
        let creaking_uuid = nbt.get_int_array("creaking").and_then(uuid_from_int_array);
        Self {
            position,
            creaking_uuid: Mutex::new(creaking_uuid),
            ticker: AtomicI32::new(0),
            output_signal: AtomicI32::new(0),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let creaking_uuid = *self.creaking_uuid.lock().await;
            if let Some(uuid) = creaking_uuid {
                nbt.put("creaking", uuid_to_int_array(uuid));
            }
        })
    }

    /// `CreakingHeartBlockEntity.serverTick`, minus the protector spawn/upkeep branch.
    /// Vanilla's `CreakingHeartBlock.getTicker` returns null while the heart is UPROOTED,
    /// so an uprooted heart does not tick here either.
    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let (block, state) = world.get_block_and_state(&self.position);
            if block.id != Block::CREAKING_HEART.id {
                return;
            }
            let mut props = CreakingHeartLikeProperties::from_state_id(state.id, block);
            if props.creaking_heart_state == CreakingHeartState::Uprooted {
                return;
            }

            let computed = self.compute_analog_output_signal();
            if self.output_signal.swap(computed, Ordering::Relaxed) != computed {
                world.update_comparators(&self.position, block).await;
            }

            // `if (entity.ticker-- < 0)`: post-decrement, so the body runs on the tick the
            // pre-decrement value is negative. Reseeds to `nextInt(5) + 20`.
            if self.ticker.fetch_sub(1, Ordering::Relaxed) >= 0 {
                return;
            }
            self.ticker
                .store(20 + rand::rng().random_range(0..5), Ordering::Relaxed);

            // updateCreakingState: an uprooted-eligible heart (no logs, no bound creaking)
            // goes UPROOTED, otherwise it tracks the creaking_active environment attribute.
            let new_state = if has_required_logs(world, &self.position, props.axis)
                || self.creaking_uuid.lock().await.is_some()
            {
                if creaking_active(world).await {
                    CreakingHeartState::Awake
                } else {
                    CreakingHeartState::Dormant
                }
            } else {
                CreakingHeartState::Uprooted
            };

            if new_state != props.creaking_heart_state {
                props.creaking_heart_state = new_state;
                world
                    .set_block_state(
                        &self.position,
                        props.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        // getUpdateTag == saveCustomOnly.
        let mut nbt = NbtCompound::new();
        let creaking_uuid = *self.creaking_uuid.try_lock().ok()?;
        if let Some(uuid) = creaking_uuid {
            nbt.put("creaking", uuid_to_int_array(uuid));
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CreakingHeartBlockEntity {
    pub const ID: &'static str = "minecraft:creaking_heart";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            creaking_uuid: Mutex::new(None),
            ticker: AtomicI32::new(0),
            output_signal: AtomicI32::new(0),
        }
    }

    /// `getAnalogOutputSignal`: the value cached by the last tick, not a fresh compute.
    pub fn get_analog_output_signal(&self) -> u8 {
        self.output_signal.load(Ordering::Relaxed).clamp(0, 15) as u8
    }

    /// `computeAnalogOutputSignal`: `15 - floor(clamp(dist, 0, 32) / 32 * 15)` for a live
    /// protector, else 0. Always the 0 branch until the creaking binding is implemented.
    pub const fn compute_analog_output_signal(&self) -> i32 {
        0
    }
}

/// `UUIDUtil.CODEC`: four big-endian ints, most-significant first.
fn uuid_from_int_array(values: &[i32]) -> Option<Uuid> {
    let &[a, b, c, d] = values else { return None };
    Some(Uuid::from_u128(
        ((a as u32 as u128) << 96)
            | ((b as u32 as u128) << 64)
            | ((c as u32 as u128) << 32)
            | (d as u32 as u128),
    ))
}

fn uuid_to_int_array(u: Uuid) -> NbtTag {
    let v = u.as_u128();
    NbtTag::IntArray(vec![
        (v >> 96) as i32,
        ((v >> 64) & 0xFFFF_FFFF) as i32,
        ((v >> 32) & 0xFFFF_FFFF) as i32,
        (v & 0xFFFF_FFFF) as i32,
    ])
}
