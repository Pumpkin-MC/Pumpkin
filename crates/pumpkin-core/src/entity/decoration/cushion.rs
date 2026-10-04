use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU32, Ordering},
};

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Block as BlockTag;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use crate::world::World;

pub struct CushionEntity {
    pub entity: Entity,
    color: AtomicU8,
    ticks_since_last_check: AtomicU32,
}

impl CushionEntity {
    const ANCHOR_DEPTH: f64 = 0.015_625;

    /// Vanilla `Cushion.canBePlacedAt`.
    pub fn can_be_placed_at(world: &World, bounding_box: &BoundingBox) -> bool {
        Self::would_survive_at(world, bounding_box) && !Self::is_anchor_buried(world, bounding_box)
    }

    /// Vanilla `Cushion.wouldSurviveAt`.
    pub fn would_survive_at(world: &World, bounding_box: &BoundingBox) -> bool {
        Self::has_anchor_below(world, bounding_box)
            && !Self::is_covered_by_suffocating_blocks(world, bounding_box)
    }

    fn has_anchor_below(world: &World, bounding_box: &BoundingBox) -> bool {
        let anchor_box = BoundingBox::new(
            Vector3::new(
                bounding_box.min.x,
                bounding_box.min.y - Self::ANCHOR_DEPTH,
                bounding_box.min.z,
            ),
            Vector3::new(
                bounding_box.max.x.next_down(),
                bounding_box.min.y,
                bounding_box.max.z.next_down(),
            ),
        );
        let mut search_box = anchor_box;
        search_box.min.y -= 0.125;

        BlockPos::iterate(search_box.min_block_pos(), search_box.max_block_pos()).any(|pos| {
            let state = world.get_block_state(&pos);
            state
                .get_block_outline_shapes_without_fluid_at(&pos)
                .reduce(|bounds, shape| {
                    BoundingBox::new(
                        Vector3::new(
                            bounds.min.x.min(shape.min.x),
                            bounds.min.y.min(shape.min.y),
                            bounds.min.z.min(shape.min.z),
                        ),
                        Vector3::new(
                            bounds.max.x.max(shape.max.x),
                            bounds.max.y.max(shape.max.y),
                            bounds.max.z.max(shape.max.z),
                        ),
                    )
                })
                .is_some_and(|bounds| bounds.at_pos(pos).intersects(&anchor_box))
        })
    }

    fn is_anchor_buried(world: &World, bounding_box: &BoundingBox) -> bool {
        let mut resting_slice = *bounding_box;
        resting_slice.max.y = resting_slice.min.y + Self::ANCHOR_DEPTH;
        let resting_slice = resting_slice.contract_all(1.0e-7);

        let mut exposed_surface = vec![resting_slice];
        for pos in BlockPos::iterate(resting_slice.min_block_pos(), resting_slice.max_block_pos()) {
            let state = world.get_block_state(&pos);
            for collider in state.get_block_collision_shapes_at(&pos) {
                let collider = collider.at_pos(pos);
                exposed_surface = exposed_surface
                    .into_iter()
                    .flat_map(|part| subtract_box(part, &collider))
                    .collect();
                if exposed_surface.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    fn is_covered_by_suffocating_blocks(world: &World, bounding_box: &BoundingBox) -> bool {
        let inner = bounding_box.contract_all(1.0e-7);
        BlockPos::iterate(inner.min_block_pos(), inner.max_block_pos()).all(|pos| {
            let (block, state) = world.get_block_and_state(&pos);
            // These blocks override vanilla's default tag/full-cube predicate.
            if [
                Block::FARMLAND.id,
                Block::DIRT_PATH.id,
                Block::SOUL_SAND.id,
                Block::MUD.id,
            ]
            .contains(&block.id)
            {
                return true;
            }
            if block.id.has_tag(BlockTag::MINECRAFT_LEAVES)
                || block.id.has_tag(BlockTag::C_GLASS_BLOCKS)
                || block.name.ends_with("copper_grate")
                || block.id == Block::MANGROVE_ROOTS.id
                || block.id == Block::MOVING_PISTON.id
            {
                return false;
            }
            if block.id.has_tag(BlockTag::MINECRAFT_SHULKER_BOXES) {
                return world.get_block_entity(&pos).is_none_or(|entity| {
                    entity
                        .as_any()
                        .downcast_ref::<ShulkerBoxBlockEntity>()
                        .is_none_or(ShulkerBoxBlockEntity::is_closed)
                });
            }
            state.is_full_cube()
                && (block.id.has_tag(BlockTag::MINECRAFT_CAUSES_SUFFOCATION)
                    || block.id == Block::PISTON.id
                    || block.id == Block::STICKY_PISTON.id)
        })
    }

    pub const fn new(entity: Entity, color: u8) -> Self {
        Self {
            entity,
            color: AtomicU8::new(color),
            ticks_since_last_check: AtomicU32::new(0),
        }
    }

    pub fn color(&self) -> u8 {
        self.color.load(Ordering::Relaxed)
    }

    pub fn set_color(&self, color: u8) {
        self.color.store(color, Ordering::Relaxed);
        self.sync_color();
    }

    fn sync_color(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        );
    }

    #[must_use]
    pub const fn item_for_color(color: u8) -> &'static Item {
        match color {
            0 => &Item::WHITE_CUSHION,
            1 => &Item::ORANGE_CUSHION,
            2 => &Item::MAGENTA_CUSHION,
            3 => &Item::LIGHT_BLUE_CUSHION,
            4 => &Item::YELLOW_CUSHION,
            5 => &Item::LIME_CUSHION,
            6 => &Item::PINK_CUSHION,
            7 => &Item::GRAY_CUSHION,
            8 => &Item::LIGHT_GRAY_CUSHION,
            9 => &Item::CYAN_CUSHION,
            10 => &Item::PURPLE_CUSHION,
            11 => &Item::BLUE_CUSHION,
            12 => &Item::BROWN_CUSHION,
            13 => &Item::GREEN_CUSHION,
            14 => &Item::RED_CUSHION,
            _ => &Item::BLACK_CUSHION,
        }
    }

    #[must_use]
    pub const fn color_from_item(item: &Item) -> u8 {
        match item.id {
            id if id == Item::WHITE_CUSHION.id => 0,
            id if id == Item::ORANGE_CUSHION.id => 1,
            id if id == Item::MAGENTA_CUSHION.id => 2,
            id if id == Item::LIGHT_BLUE_CUSHION.id => 3,
            id if id == Item::YELLOW_CUSHION.id => 4,
            id if id == Item::LIME_CUSHION.id => 5,
            id if id == Item::PINK_CUSHION.id => 6,
            id if id == Item::GRAY_CUSHION.id => 7,
            id if id == Item::LIGHT_GRAY_CUSHION.id => 8,
            id if id == Item::CYAN_CUSHION.id => 9,
            id if id == Item::PURPLE_CUSHION.id => 10,
            id if id == Item::BLUE_CUSHION.id => 11,
            id if id == Item::BROWN_CUSHION.id => 12,
            id if id == Item::GREEN_CUSHION.id => 13,
            id if id == Item::RED_CUSHION.id => 14,
            _ => 15,
        }
    }

    fn drop_and_remove(&self) {
        let entity = &self.entity;
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            entity.remove_passenger(passenger.get_entity().entity_id);
        }
        // A cancelled dismount must not leave a rider attached to a removed cushion.
        if entity.has_passengers() {
            return;
        }

        let world = entity.world.load();
        world.play_sound(
            Sound::EntityCushionBreak,
            SoundCategory::Blocks,
            &entity.pos.load(),
        );
        world.drop_stack(
            &entity.block_pos.load(),
            ItemStack::new(1, Self::item_for_color(self.color())),
        );
        entity.remove();
    }
}

/// The parts of `part` that `other` doesn't cover.
fn subtract_box(part: BoundingBox, other: &BoundingBox) -> Vec<BoundingBox> {
    if !part.intersects(other) {
        return vec![part];
    }
    let mut pieces = Vec::new();
    let mut rest = part;
    if rest.min.x < other.min.x {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(other.min.x, rest.max.y, rest.max.z),
        ));
        rest.min.x = other.min.x;
    }
    if rest.max.x > other.max.x {
        pieces.push(BoundingBox::new(
            Vector3::new(other.max.x, rest.min.y, rest.min.z),
            rest.max,
        ));
        rest.max.x = other.max.x;
    }
    if rest.min.y < other.min.y {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(rest.max.x, other.min.y, rest.max.z),
        ));
        rest.min.y = other.min.y;
    }
    if rest.max.y > other.max.y {
        pieces.push(BoundingBox::new(
            Vector3::new(rest.min.x, other.max.y, rest.min.z),
            rest.max,
        ));
        rest.max.y = other.max.y;
    }
    if rest.min.z < other.min.z {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(rest.max.x, rest.max.y, other.min.z),
        ));
        rest.min.z = other.min.z;
    }
    if rest.max.z > other.max.z {
        pieces.push(BoundingBox::new(
            Vector3::new(rest.min.x, rest.min.y, other.max.z),
            rest.max,
        ));
    }
    pieces
}

impl EntityBase for CushionEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn init_data_tracker(&self) {
        self.sync_color();
    }

    /// The colour has to ride along with the spawn packet; a client seeing the
    /// cushion for the first time never got the tracked data update.
    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let mut metadata = Vec::new();
        Metadata::new(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        )
        .write(&mut metadata, &version)
        .ok()?;
        metadata.push(255);
        Some(metadata.into_boxed_slice())
    }

    /// Vanilla BlockAttachedEntity.tick: drops the cushion once nothing holds it up.
    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        if self.ticks_since_last_check.fetch_add(1, Ordering::Relaxed) < 100 {
            return;
        }
        self.ticks_since_last_check.store(0, Ordering::Relaxed);
        let world = self.entity.world.load();
        if self.entity.is_alive()
            && !Self::would_survive_at(&world, &self.entity.bounding_box.load())
        {
            self.drop_and_remove();
        }
    }

    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("Color", self.color() as i8);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(color) = nbt.get_byte("Color") {
            self.set_color(color as u8);
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        if player.get_entity().is_sneaking() {
            return false;
        }
        // Do not replace an unacknowledged teleport with another seat switch.
        if player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            return false;
        }
        if !self
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
        {
            return false;
        }

        let world = self.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.entity.entity_id) else {
            return false;
        };
        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        // Vanilla `startRiding` leaves the current vehicle first, so this switches seats.
        let current_vehicle = player
            .get_entity()
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(current_vehicle) = current_vehicle {
            current_vehicle
                .get_entity()
                .remove_passenger(player.entity_id());
            if player.get_entity().has_vehicle() {
                return false;
            }
        }

        self.entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
        if player
            .get_entity()
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_none_or(|vehicle| vehicle.get_entity().entity_id != self.entity.entity_id)
        {
            return false;
        }

        // The player's vehicle attachment is 0.6 blocks above their feet.
        let position = self.entity.pos.load()
            + Vector3::new(
                0.0,
                f64::from(self.entity.entity_type.dimension[1]) - 0.6,
                0.0,
            );
        let mut awaiting_teleport = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        player.get_entity().set_pos(position);
        // The dismount teleport precedes the mount packet. Its acknowledgement
        // must preserve the new riding position, not restore the old seat.
        if let Some((_, target)) = awaiting_teleport.as_mut() {
            *target = position;
        }
        drop(awaiting_teleport);

        world.play_sound(
            Sound::EntityCushionSit,
            SoundCategory::Blocks,
            &self.entity.pos.load(),
        );
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        if !self.entity.is_alive() {
            return false;
        }
        self.drop_and_remove();
        true
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::entity::EntityType;
    use pumpkin_world::level::Level;
    use pumpkin_world::world_info::LevelData;
    use std::sync::Weak;

    #[tokio::test]
    async fn breaking_an_occupied_cushion_detaches_its_passenger() {
        let directory = tempfile::tempdir().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            directory.path().to_path_buf(),
            0,
            Dimension::OVERWORLD,
        );
        let world = Arc::new(World::load(
            level.clone(),
            Arc::new(ArcSwap::from_pointee(LevelData::default(
                pumpkin_util::world_seed::Seed(0),
            ))),
            Dimension::OVERWORLD,
            crate::block::registry::default_registry(),
            Weak::new(),
        ));
        let position = Vector3::new(0.5, 64.0, 0.5);
        let cushion = Arc::new(CushionEntity::new(
            Entity::new(world.clone(), position, &EntityType::CUSHION),
            0,
        ));
        let passenger = Arc::new(Entity::new(world.clone(), position, &EntityType::PIG));
        world.spawn_entity(cushion.clone());
        cushion
            .entity
            .add_passenger(cushion.clone(), passenger.clone());

        cushion.drop_and_remove();
        let passenger_detached = !passenger.has_vehicle();
        let cushion_empty = !cushion.entity.has_passengers();
        let cushion_removed = world.get_entity_by_id(cushion.entity.entity_id).is_none();
        level.shutdown().await;

        assert!(
            passenger_detached,
            "passenger still references the removed cushion"
        );
        assert!(cushion_empty, "removed cushion still owns its passenger");
        assert!(cushion_removed);
    }
}
