use super::BlockEntity;
use crate::block::blocks::campfire::CampfireBlock;
use crate::entity::ageable::AgeableMob;
use crate::entity::passive::bee::BeeEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::block_properties::BeeNestLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{chunk::io::Dirtiable, world::BlockFlags};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering::Relaxed},
};

const MAX_OCCUPANTS: usize = 3;
const MIN_TICKS_IN_HIVE: i32 = 600;
const MIN_TICKS_IN_HIVE_WITH_NECTAR: i32 = 2400;
const MAX_HONEY_LEVEL: u8 = 5;
/// Vanilla `BeehiveBlockEntity.MIN_TICKS_BEFORE_REENTERING_HIVE`.
const SEDATED_STAY_OUT_TICKS: i32 = 400;
const IGNORED_BEE_TAGS: &[&str] = &[
    "Air",
    "drop_chances",
    "equipment",
    "Brain",
    "CanPickUpLoot",
    "DeathTime",
    "fall_distance",
    "FallFlying",
    "Fire",
    "HurtTime",
    "LeftHanded",
    "Motion",
    "NoGravity",
    "OnGround",
    "PortalCooldown",
    "Pos",
    "Rotation",
    "sleeping_pos",
    "CannotEnterHiveTicks",
    "TicksSincePollination",
    "CropsGrownSincePollination",
    "hive_pos",
    "Passengers",
    "leash",
    "UUID",
];

pub struct BeehiveBlockEntity {
    pub position: BlockPos,
    pub bees: Mutex<Option<Vec<NbtTag>>>,
    pub flower_pos: Mutex<Option<BlockPos>>,
    /// Set by fire next to the hive; the next tick empties it, the neighbour update has no `Arc<World>`.
    fire_release_pending: AtomicBool,
}

impl BlockEntity for BeehiveBlockEntity {
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
        let bees = nbt
            .get_list("bees")
            .or_else(|| nbt.get_list("Bees"))
            .map(<[_]>::to_vec);
        let flower_pos = nbt
            .get_int_array("flower_pos")
            .and_then(|arr| (arr.len() == 3).then(|| BlockPos::new(arr[0], arr[1], arr[2])))
            .or_else(|| {
                nbt.get_compound("flower_pos")
                    .or_else(|| nbt.get_compound("FlowerPos"))
                    .map(|c| {
                        BlockPos::new(
                            c.get_int("X").or_else(|| c.get_int("x")).unwrap_or(0),
                            c.get_int("Y").or_else(|| c.get_int("y")).unwrap_or(0),
                            c.get_int("Z").or_else(|| c.get_int("z")).unwrap_or(0),
                        )
                    })
            });
        Self {
            position,
            bees: Mutex::new(bees),
            flower_pos: Mutex::new(flower_pos),
            fire_release_pending: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(b) = self.bees.lock()
            && let Some(b) = b.as_ref()
        {
            nbt.put_list("bees", b.clone());
        }
        if let Ok(fp) = self.flower_pos.lock()
            && let Some(fp) = fp.as_ref()
        {
            nbt.put(
                "flower_pos",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![fp.0.x, fp.0.y, fp.0.z]),
            );
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(bees) = self.bees.try_lock()
            && let Some(ref b) = *bees
        {
            nbt.put_list("bees", b.clone());
        }
        if let Ok(flower_pos) = self.flower_pos.try_lock()
            && let Some(ref fp) = *flower_pos
        {
            nbt.put(
                "flower_pos",
                pumpkin_nbt::tag::NbtTag::IntArray(vec![fp.0.x, fp.0.y, fp.0.z]),
            );
        }
        Some(nbt)
    }

    fn tick(&self, world: &Arc<World>) {
        let emergency = self.fire_release_pending.swap(false, Relaxed);
        if self.is_empty() {
            return;
        }
        let (block, state) = world.get_block_and_state_id(&self.position);
        if !block.has_tag(&tag::Block::MINECRAFT_BEEHIVES) {
            return;
        }
        let has_bees = self.release_occupants(world, state, emergency, None);
        if has_bees && rand::random::<f64>() < 0.005 {
            let mut center = self.position.to_centered_f64();
            center.y -= 0.5;
            world.play_sound(Sound::BlockBeehiveWork, SoundCategory::Blocks, &center);
        }
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(bees) = self.bees.lock()
            && let Some(bees) = bees.as_ref()
            && !bees.is_empty()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::BeesImpl {
                bees: bees.clone(),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        if let Some(bees) =
            stack.get_data_component::<pumpkin_data::data_component_impl::BeesImpl>()
            && !bees.bees.is_empty()
        {
            *self
                .bees
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(bees.bees.clone());
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_player_destroyed(
        &self,
        world: &Arc<World>,
        state: BlockStateId,
        player: &Arc<crate::entity::player::Player>,
        tool: &pumpkin_data::item_stack::ItemStack,
    ) {
        if !tool.has_enchantment_tag(&tag::Enchantment::MINECRAFT_PREVENTS_BEE_SPAWNS_WHEN_MINING) {
            self.release_occupants(world, state, true, Some(player));
            crate::block::blocks::beehive::BeehiveBlock::anger_nearby_bees(world, &self.position);
        }
    }
}

impl BeehiveBlockEntity {
    pub const ID: &'static str = "minecraft:beehive";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            bees: Mutex::new(None),
            flower_pos: Mutex::new(None),
            fire_release_pending: AtomicBool::new(false),
        }
    }

    /// Vanilla `BeehiveBlock.updateShape`: fire next to the hive drives every bee out.
    pub fn release_for_fire(&self) {
        self.fire_release_pending.store(true, Relaxed);
    }

    pub fn is_full(&self) -> bool {
        self.bees
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|bees| bees.len() >= MAX_OCCUPANTS)
    }

    pub fn is_empty(&self) -> bool {
        self.bees
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_none_or(Vec::is_empty)
    }

    pub fn is_fire_nearby(&self, world: &World) -> bool {
        BlockPos::iterate_outwards(self.position, 1, 1, 1)
            .any(|pos| world.get_block(&pos) == &Block::FIRE)
    }

    pub fn add_occupant(&self, world: &World, bee: &BeeEntity) {
        if self.is_full() || bee.get_entity().removed.load(Relaxed) {
            return;
        }
        let entity = bee.get_entity();
        if let Some(vehicle) = entity.get_vehicle() {
            vehicle.get_entity().remove_passenger_sync(entity.entity_id);
        }
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            entity.remove_passenger_sync(passenger.get_entity().entity_id);
        }
        entity.drop_leash();
        let mut data = NbtCompound::new();
        bee.write_nbt(&mut data);
        data.child_tags
            .retain(|key, _| !IGNORED_BEE_TAGS.contains(&key.as_ref()));
        data.put_string("id", "minecraft:bee".to_string());
        let mut occupant = NbtCompound::new();
        occupant.put_compound("entity_data", data);
        occupant.put_int("ticks_in_hive", 0);
        occupant.put_int(
            "min_ticks_in_hive",
            if bee.has_nectar() {
                MIN_TICKS_IN_HIVE_WITH_NECTAR
            } else {
                MIN_TICKS_IN_HIVE
            },
        );
        let mut stored = self
            .bees
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bees = stored.get_or_insert_with(Vec::new);
        if bees.len() >= MAX_OCCUPANTS {
            return;
        }
        bees.push(NbtTag::Compound(occupant));
        drop(stored);
        if let Some(pos) = bee.saved_flower_pos() {
            let mut flower = self
                .flower_pos
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if flower.is_none() || rand::random::<bool>() {
                *flower = Some(pos);
            }
        }
        world.play_sound(
            Sound::BlockBeehiveEnter,
            SoundCategory::Blocks,
            &self.position.to_f64(),
        );
        world.emit_game_event(GameEvent::BlockChange.name(), self.position.to_f64());
        world.remove_entity(bee);
        self.mark_chunk_dirty(world);
    }

    pub fn release_occupants(
        &self,
        world: &Arc<World>,
        state: BlockStateId,
        emergency: bool,
        player: Option<&Arc<crate::entity::player::Player>>,
    ) -> bool {
        let block = Block::from_state_id(state);
        let mut props = BeeNestLikeProperties::from_state_id(state);
        let facing = props.facing.to_offset();
        let entrance = self.position.offset(facing);
        let front_blocked = !world.get_block_state(&entrance).collision_shapes.is_empty();
        let can_release = emergency || !front_blocked && !world.bees_stay_in_hive(&self.position);
        let Some(candidates) = self.take_ready_occupants(can_release, emergency) else {
            return false;
        };
        let flower = *self
            .flower_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut failed = Vec::new();
        let honey_before = props.honey_level;
        let sedated = player.is_some() && CampfireBlock::is_smokey_pos(world, &self.position);
        for occupant in candidates {
            let Some(bee) = occupant
                .extract_compound()
                .and_then(|data| self.restore_occupant(world, data, flower))
            else {
                failed.push(occupant);
                continue;
            };
            let entity = bee.get_entity();
            let exit_distance = if front_blocked {
                0.0
            } else {
                0.55 + f64::from(entity.width() / 2.0)
            };
            let pos = self.position.to_centered_f64().add_raw(
                f64::from(facing.x) * exit_distance,
                -f64::from(entity.height() / 2.0),
                f64::from(facing.z) * exit_distance,
            );
            entity.set_pos(pos);
            if !emergency && bee.has_nectar() {
                bee.drop_off_nectar();
                // Every bee reads the hive state from before this tick, so the last one sets the level.
                if honey_before < MAX_HONEY_LEVEL {
                    let increase = if rand::random_range(0..100) == 0 {
                        2
                    } else {
                        1
                    };
                    props.honey_level = (honey_before + increase).min(MAX_HONEY_LEVEL);
                }
            }
            world.play_sound(
                Sound::BlockBeehiveExit,
                SoundCategory::Blocks,
                &self.position.to_f64(),
            );
            world.emit_game_event(GameEvent::BlockChange.name(), self.position.to_f64());
            if !world.spawn_entity(bee.clone()) {
                failed.push(occupant);
                continue;
            }
            if let Some(player) = player
                && pos.squared_distance_to_vec(&player.get_entity().pos.load()) <= 16.0
            {
                if sedated {
                    bee.cannot_enter_hive_ticks
                        .store(SEDATED_STAY_OUT_TICKS, Relaxed);
                } else {
                    use crate::entity::mob::Mob;
                    bee.set_mob_target(
                        Some(player.clone()),
                        crate::plugin::api::events::entity::entity_target::TargetReason::Angry,
                    );
                }
            }
        }

        if props.honey_level != honey_before {
            world.set_block_state_if(
                &self.position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
                |current| current == state,
            );
        }
        self.mark_chunk_dirty(world);

        let mut bees = self
            .bees
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bees = bees.get_or_insert_with(Vec::new);
        bees.extend(failed);
        !bees.is_empty()
    }

    fn take_ready_occupants(&self, can_release: bool, emergency: bool) -> Option<Vec<NbtTag>> {
        let mut bees = self
            .bees
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bees = bees.as_mut().filter(|bees| !bees.is_empty())?;
        Some(
            bees.extract_if(.., |entry| {
                let NbtTag::Compound(occupant) = entry else {
                    return false;
                };
                let old_ticks = occupant
                    .get_int("ticks_in_hive")
                    .or_else(|| occupant.get_int("TicksInHive"))
                    .unwrap_or_default();
                let minimum = occupant
                    .get_int("min_ticks_in_hive")
                    .or_else(|| occupant.get_int("MinOccupationTicks"))
                    .unwrap_or(MIN_TICKS_IN_HIVE);
                if !emergency {
                    occupant.put_int("ticks_in_hive", old_ticks.saturating_add(1));
                }
                can_release && (emergency || old_ticks > minimum)
            })
            .collect(),
        )
    }

    fn restore_occupant(
        &self,
        world: &Arc<World>,
        occupant: &NbtCompound,
        flower: Option<BlockPos>,
    ) -> Option<Arc<BeeEntity>> {
        let mut data = occupant
            .get_compound("entity_data")
            .or_else(|| occupant.get_compound("EntityData"))?
            .clone();
        if data
            .get_string("id")
            .is_some_and(|id| id != "minecraft:bee" && id != "bee")
        {
            return None;
        }
        data.child_tags
            .retain(|key, _| !IGNORED_BEE_TAGS.contains(&key.as_ref()));
        let ticks = occupant
            .get_int("ticks_in_hive")
            .or_else(|| occupant.get_int("TicksInHive"))
            .unwrap_or_default()
            .max(0);
        let bee = BeeEntity::new(Entity::new(
            world.clone(),
            self.position.to_centered_f64(),
            &EntityType::BEE,
        ));
        bee.read_nbt_non_mut(&data);
        bee.set_hive_pos(self.position);
        bee.get_entity().set_has_no_gravity(true);
        if !bee.is_age_locked() {
            let age = bee.get_entity().age.load(Relaxed);
            bee.set_age(if age < 0 {
                age.saturating_add(ticks).min(0)
            } else {
                age.saturating_sub(ticks).max(0)
            });
        }
        let mob = &bee.mob_entity;
        mob.love_ticks.store(
            mob.love_ticks.load(Relaxed).saturating_sub(ticks).max(0),
            Relaxed,
        );
        if let Some(flower) = flower
            && bee.saved_flower_pos().is_none()
            && rand::random::<f32>() < 0.9
        {
            bee.set_saved_flower_pos(flower);
        }
        Some(bee)
    }

    fn mark_chunk_dirty(&self, world: &World) {
        world
            .level
            .read_chunk_sync(&self.position.chunk_position(), |chunk| {
                chunk.mark_dirty(true);
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn nectar_occupant_waits_for_an_open_exit() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let pos = BlockPos::new(8, 64, 8);
        let flower = pos.up();
        world.level.loaded_chunks.insert(
            pos.chunk_position(),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
        world.set_time_of_day(6000);
        let mut props = BeeNestLikeProperties::default(&Block::BEEHIVE);
        props.honey_level = MAX_HONEY_LEVEL - 1;
        let state = props.to_state_id(&Block::BEEHIVE);
        world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
        let exit = pos.offset(props.facing.to_offset());
        world.set_block_state(&exit, Block::STONE.default_state.id, BlockFlags::NOTIFY_ALL);
        let hive = BeehiveBlockEntity::new(pos);
        let bee = BeeEntity::new(Entity::new(
            world.clone(),
            pos.to_centered_f64(),
            &EntityType::BEE,
        ));
        bee.set_has_nectar(true);
        bee.set_saved_flower_pos(flower);
        world.spawn_entity(bee.clone());
        hive.add_occupant(&world, &bee);

        assert!(world.get_entity_by_id(bee.get_entity().entity_id).is_none());
        let mut saved = NbtCompound::new();
        hive.write_nbt(&mut saved);
        let Some(NbtTag::Compound(mut occupant)) = saved
            .get_list("bees")
            .and_then(|bees| bees.first())
            .cloned()
        else {
            return Err("hive did not save the bee".into());
        };
        assert_eq!(occupant.get_int("min_ticks_in_hive"), Some(2400));
        occupant.put_int("ticks_in_hive", 2401);
        saved.put_list("bees", vec![NbtTag::Compound(occupant)]);
        let hive = BeehiveBlockEntity::from_nbt(&saved, pos);
        hive.tick(&world);
        assert!(!hive.is_empty());
        assert_eq!(world.get_block_state_id(&pos), state);

        world.set_block_state(&exit, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
        hive.tick(&world);
        assert!(hive.is_empty());
        assert_eq!(
            BeeNestLikeProperties::from_state_id(world.get_block_state_id(&pos)).honey_level,
            5
        );
        let bounds = pumpkin_util::math::boundingbox::BoundingBox::new(
            pos.to_f64().add_raw(-2.0, -2.0, -2.0),
            pos.to_f64().add_raw(3.0, 3.0, 3.0),
        );
        let released = world.get_entities_at_box(&bounds);
        let [released] = released.as_slice() else {
            return Err("expected one released bee".into());
        };
        let released = released
            .cast_any()
            .downcast_ref::<BeeEntity>()
            .ok_or("expected a bee")?;
        assert_eq!(released.saved_flower_pos(), Some(flower));
        assert!(!released.has_nectar());
        world.level.shutdown().await;
        Ok(())
    }
}
