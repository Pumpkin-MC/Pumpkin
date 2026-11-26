use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering::Relaxed},
};

use pumpkin_data::{
    Block, BlockState, BlockStateId,
    attributes::Attributes,
    damage::DamageType,
    entity::EntityType,
    fluid::Fluid,
    item_stack::ItemStack,
    particle::Particle,
    sound::Sound,
    tag::{Tag, Taggable},
};
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector2::Vector2, vector3::Vector3,
};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::world::BlockAccessor;
use rand::seq::SliceRandom;
use rustc_hash::FxHashSet;

use crate::{
    block::{ExplodeArgs, collect_loot},
    entity::{Entity, EntityBase, item::ItemEntity, projectile::is_redirectable, tnt::TNTEntity},
    world::loot::LootContextParameters,
};

use super::{BlockFlags, World};

/// Defines the type of explosion interaction with the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplosionInteraction {
    None,
    Block,
    Mob,
    Tnt,
    Trigger,
}

/// Defines how an explosion interacts with blocks in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockInteraction {
    /// Keeps blocks intact (no block damage, no drops).
    Keep,
    /// Destroys blocks and drops 100% of items without decay.
    Destroy,
    /// Destroys blocks and applies loot decay based on explosion radius.
    DestroyWithDecay,
    /// Triggers block effects without destroying them.
    TriggerBlock,
}

/// Defines how damage and block destruction are calculated for an explosion.
pub trait ExplosionDamageCalculator: Send + Sync {
    /// Returns the block's explosion resistance. If None, the block is treated as air/empty.
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    /// Returns whether this block should be destroyed / affected by the explosion.
    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        true
    }

    /// Returns whether the entity should take damage from the explosion.
    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        true
    }

    /// Returns knockback multiplier for the given entity (default 1.0).
    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        1.0
    }

    /// Calculates the damage amount to deal to the entity given the exposure.
    fn get_entity_damage_amount(
        &self,
        explosion: &Explosion,
        entity: &dyn EntityBase,
        exposure: f32,
    ) -> f32 {
        let radius = explosion.power as f64 * 2.0;
        let distance = (entity
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&explosion.pos))
        .sqrt()
            / radius;
        let damage_multiplier = (1.0 - distance) * exposure as f64;
        // Vanilla scales by the doubled radius, not the power.
        (f64::midpoint(damage_multiplier * damage_multiplier, damage_multiplier) * 7.0 * radius
            + 1.0) as f32
    }
}

/// Default explosion damage calculator implementing vanilla standard explosion rules.
pub struct DefaultExplosionDamageCalculator;

impl ExplosionDamageCalculator for DefaultExplosionDamageCalculator {}

/// A configurable explosion damage calculator (e.g. for wind charges, mace wind bursts).
pub struct SimpleExplosionDamageCalculator {
    pub explodes_blocks: bool,
    pub damages_entities: bool,
    pub knockback_multiplier: Option<f32>,
    pub immune_blocks: Option<&'static Tag>,
}

impl SimpleExplosionDamageCalculator {
    #[must_use]
    pub const fn new(
        explodes_blocks: bool,
        damages_entities: bool,
        knockback_multiplier: Option<f32>,
        immune_blocks: Option<&'static Tag>,
    ) -> Self {
        Self {
            explodes_blocks,
            damages_entities,
            knockback_multiplier,
            immune_blocks,
        }
    }
}

impl ExplosionDamageCalculator for SimpleExplosionDamageCalculator {
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if let Some(immune_tag) = self.immune_blocks {
            return block.has_tag(immune_tag).then_some(3_600_000.0);
        }
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        self.explodes_blocks
    }

    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        self.damages_entities
    }

    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        self.knockback_multiplier.unwrap_or(1.0)
    }
}

/// The loaded chunks around an explosion, looked up once. Its rays and exposure checks read
/// blocks through them instead of finding the chunk in the level for every block.
pub(crate) struct ExplosionChunks<'w> {
    world: &'w World,
    min_x: i32,
    min_z: i32,
    width: i32,
    depth: i32,
    chunks: Vec<Option<Arc<ChunkData>>>,
    cache: Option<BlockCache>,
}

/// Block states read during one pass, in a cube around the explosion. Each cell holds the
/// pass stamp in its high half and the state in its low half: a cell from an earlier pass is
/// stale, so starting a pass forgets everything without clearing the cube. A pass is a stretch
/// in which nothing can change a block: the ray march, or one entity's exposure.
struct BlockCache {
    origin: [i32; 3],
    side: i32,
    cells: Vec<AtomicU32>,
    stamp: AtomicU32,
}

impl BlockCache {
    /// Cubes past this side read the chunks directly, rather than allocate a large cube.
    const MAX_SIDE: i32 = 64;

    fn new(center: Vector3<f64>, radius: f64) -> Option<Self> {
        let radius = radius.ceil() as i32;
        let side = 2 * radius + 1;
        if side > Self::MAX_SIDE {
            return None;
        }
        let at = |v: f64| v.floor() as i32 - radius;
        Some(Self {
            origin: [at(center.x), at(center.y), at(center.z)],
            side,
            cells: (0..side * side * side).map(|_| AtomicU32::new(0)).collect(),
            stamp: AtomicU32::new(1),
        })
    }

    fn cell(&self, position: &BlockPos) -> Option<&AtomicU32> {
        let offset = |axis: usize, v: i32| {
            let d = v - self.origin[axis];
            (0..self.side).contains(&d).then_some(d)
        };
        let x = offset(0, position.0.x)?;
        let y = offset(1, position.0.y)?;
        let z = offset(2, position.0.z)?;
        self.cells
            .get(((y * self.side + z) * self.side + x) as usize)
    }

    fn next_pass(&self) {
        let stamp = self.stamp.load(Relaxed) + 1;
        if stamp > u32::from(u16::MAX) {
            for cell in &self.cells {
                cell.store(0, Relaxed);
            }
            self.stamp.store(1, Relaxed);
        } else {
            self.stamp.store(stamp, Relaxed);
        }
    }
}

impl<'w> ExplosionChunks<'w> {
    /// The chunks within `radius` blocks of `center`, horizontally.
    pub(crate) fn around(world: &'w World, center: Vector3<f64>, radius: f64) -> Self {
        let chunk_of = |v: f64| (v.floor() as i32) >> 4;
        let (min_x, max_x) = (chunk_of(center.x - radius), chunk_of(center.x + radius));
        let (min_z, max_z) = (chunk_of(center.z - radius), chunk_of(center.z + radius));
        let (width, depth) = (max_x - min_x + 1, max_z - min_z + 1);
        let mut chunks = Vec::with_capacity((width * depth).max(0) as usize);
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                chunks.push(
                    world
                        .level
                        .read_chunk_sync(&Vector2::new(x, z), std::clone::Clone::clone),
                );
            }
        }
        Self {
            world,
            min_x,
            min_z,
            width,
            depth,
            chunks,
            cache: BlockCache::new(center, radius),
        }
    }

    /// Starts a pass: blocks may have changed since the previous one, so none of its reads
    /// are reused.
    fn next_pass(&self) {
        if let Some(cache) = &self.cache {
            cache.next_pass();
        }
    }

    /// The block state at `position`, air outside the build limit and in unloaded chunks,
    /// like `World::get_block_state_id`.
    fn state_id(&self, position: &BlockPos) -> BlockStateId {
        let Some((cache, cell)) = self
            .cache
            .as_ref()
            .and_then(|cache| Some((cache, cache.cell(position)?)))
        else {
            return self.read_state_id(position);
        };
        let stamp = cache.stamp.load(Relaxed);
        let cached = cell.load(Relaxed);
        if cached >> 16 == stamp {
            return BlockStateId::new_or_air(cached as u16);
        }
        let id = self.read_state_id(position);
        cell.store(stamp << 16 | u32::from(id.as_u16()), Relaxed);
        id
    }

    fn read_state_id(&self, position: &BlockPos) -> BlockStateId {
        if !self.world.is_in_build_limit(*position) {
            return Block::AIR.default_state.id;
        }
        let (chunk, relative) = position.chunk_and_chunk_relative_position();
        let (dx, dz) = (chunk.x - self.min_x, chunk.y - self.min_z);
        if !(0..self.width).contains(&dx) || !(0..self.depth).contains(&dz) {
            return self.world.get_block_state_id(position);
        }
        self.chunks[(dz * self.width + dx) as usize]
            .as_ref()
            .and_then(|chunk| {
                chunk.section.get_block_absolute_y(
                    relative.x as usize,
                    position.0.y,
                    relative.z as usize,
                )
            })
            .unwrap_or(Block::AIR.default_state.id)
    }
}

impl BlockAccessor for ExplosionChunks<'_> {
    fn get_block(&self, position: &BlockPos) -> &'static Block {
        Block::from_state_id(self.state_id(position))
    }

    fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        BlockState::from_id(self.state_id(position))
    }

    fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.state_id(position)
    }

    fn get_block_and_state(&self, position: &BlockPos) -> (&'static Block, &'static BlockState) {
        BlockState::from_id_with_block(self.state_id(position))
    }
}

/// How a block slows a ray down.
#[derive(Clone, Copy)]
enum RayResistance {
    /// Air without fluid: the ray only pays its step and never explodes it.
    Empty,
    /// A block the calculator gives no resistance: free, but it can explode.
    Free,
    Resists(f32),
}

/// What a ray learned about the block it is in, reused while its steps stay in that block.
struct RayCell {
    pos: BlockPos,
    block: &'static Block,
    resistance: RayResistance,
    protects_rail: bool,
    /// Already in the exploded positions, from an earlier step of this ray.
    added: bool,
}

pub struct Explosion<'a> {
    power: f32,
    pos: Vector3<f64>,
    block_interaction: BlockInteraction,
    damage_calculator: Option<Arc<dyn ExplosionDamageCalculator>>,
    preserve_rails: bool,
    source: Option<&'a dyn EntityBase>,
    /// The entity credited with the explosion when it isn't the source itself.
    indirect_source: Option<Arc<dyn EntityBase>>,
    source_is_indirect: bool,
    damage_type: Option<DamageType>,
    small_particle: Particle,
    large_particle: Particle,
    sound: Sound,
    from_wind_charge: bool,
}

impl<'a> Explosion<'a> {
    #[must_use]
    pub const fn new(power: f32, pos: Vector3<f64>, block_interaction: BlockInteraction) -> Self {
        Self {
            power,
            pos,
            block_interaction,
            damage_calculator: None,
            preserve_rails: false,
            source: None,
            indirect_source: None,
            source_is_indirect: false,
            damage_type: None,
            small_particle: Particle::Explosion,
            large_particle: Particle::ExplosionEmitter,
            sound: Sound::EntityGenericExplode,
            from_wind_charge: false,
        }
    }

    #[must_use]
    pub fn with_damage_calculator(
        mut self,
        calculator: Arc<dyn ExplosionDamageCalculator>,
    ) -> Self {
        self.damage_calculator = Some(calculator);
        self
    }

    #[must_use]
    pub const fn preserving_rails(mut self) -> Self {
        self.preserve_rails = true;
        self
    }

    /// Sets the entity that exploded and who gets the credit for it:
    /// primed TNT credits whoever lit it, a living entity itself and a projectile its living owner.
    #[must_use]
    pub fn with_source(mut self, source: &'a dyn EntityBase) -> Self {
        self.source = Some(source);
        self.source_is_indirect = source.get_living_entity().is_some();
        self.indirect_source = if self.source_is_indirect {
            None
        } else {
            let world = source.get_entity().world.load();
            let owner = source.cast_any().downcast_ref::<TNTEntity>().map_or_else(
                || {
                    source
                        .get_projectile_owner()
                        .and_then(|owner| owner.get(&world))
                },
                |tnt| tnt.owner.get(&world),
            );
            owner.filter(|owner| owner.get_living_entity().is_some())
        };
        self
    }

    /// Credits the explosion to `cause` instead of the source's owner, for sources that pass their own damage source.
    #[must_use]
    pub fn with_indirect_source(mut self, cause: Option<Arc<dyn EntityBase>>) -> Self {
        self.source_is_indirect = false;
        self.indirect_source = cause;
        self
    }

    #[must_use]
    pub const fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = Some(damage_type);
        self
    }

    #[must_use]
    pub const fn center(&self) -> Vector3<f64> {
        self.pos
    }

    #[must_use]
    pub const fn power(&self) -> f32 {
        self.power
    }

    #[must_use]
    pub const fn source(&self) -> Option<&'a dyn EntityBase> {
        self.source
    }

    #[must_use]
    pub fn indirect_source(&self) -> Option<&dyn EntityBase> {
        if self.source_is_indirect {
            self.source
        } else {
            self.indirect_source.as_deref()
        }
    }

    /// Credited explosions with a source deal `player_explosion` damage, which changes the death message.
    #[must_use]
    pub fn damage_type(&self) -> DamageType {
        self.damage_type.unwrap_or(
            if self.source.is_some() && self.indirect_source().is_some() {
                DamageType::PLAYER_EXPLOSION
            } else {
                DamageType::EXPLOSION
            },
        )
    }

    #[must_use]
    pub const fn with_particles_and_sound(
        mut self,
        small_particle: Particle,
        large_particle: Particle,
        sound: Sound,
    ) -> Self {
        self.small_particle = small_particle;
        self.large_particle = large_particle;
        self.sound = sound;
        self
    }

    #[must_use]
    pub const fn from_wind_charge(mut self) -> Self {
        self.from_wind_charge = true;
        self
    }

    /// Mirrors `ServerExplosion.isSmall`.
    #[must_use]
    pub const fn is_small(&self) -> bool {
        self.power < 2.0 || matches!(self.block_interaction, BlockInteraction::Keep)
    }

    #[must_use]
    pub const fn particle(&self) -> Particle {
        if self.is_small() {
            self.small_particle
        } else {
            self.large_particle
        }
    }

    #[must_use]
    pub const fn sound(&self) -> Sound {
        self.sound
    }

    fn protects_rail(&self, world: &World, pos: &BlockPos, block: &Block) -> bool {
        self.preserve_rails && (Self::is_rail(block) || Self::is_rail(world.get_block(&pos.up())))
    }

    fn is_rail(block: &Block) -> bool {
        block.id == Block::RAIL.id
            || block.id == Block::POWERED_RAIL.id
            || block.id == Block::DETECTOR_RAIL.id
            || block.id == Block::ACTIVATOR_RAIL.id
    }

    /// Vanilla `calculateExplodedPositions`, shuffled like `interactWithBlocks`.
    fn get_blocks_to_destroy(&self, world: &World, blocks: &ExplosionChunks<'_>) -> Vec<BlockPos> {
        // TODO: vanilla rolls from `level.random`; same odds, not seed-reproducible.
        let positions = self.exploded_positions(world, blocks, rand::random::<f32>);
        let mut positions: Vec<_> = positions.into_iter().collect();
        // TODO: vanilla shuffles with `level.random`, same odds, not seed-reproducible.
        positions.shuffle(&mut rand::rng());
        positions
    }

    /// The ray march of `calculateExplodedPositions`; `ray_roll` gives each ray's random draw.
    ///
    /// A ray takes steps of 0.3, about three per block. A step in the block the previous step
    /// was in costs what that one did (blocks don't change while rays march), so the block,
    /// its fluid and its resistance are read once per block, not once per step, as in Horizon.
    fn exploded_positions(
        &self,
        world: &World,
        blocks: &ExplosionChunks<'_>,
        mut ray_roll: impl FnMut() -> f32,
    ) -> FxHashSet<BlockPos> {
        let mut positions = FxHashSet::default();

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };
        blocks.next_pass();

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if x > 0 && x < 15 && y > 0 && y < 15 && z > 0 && z < 15 {
                        continue;
                    }

                    let mut dir_x = f64::from(x) / 7.5 - 1.0;
                    let mut dir_y = f64::from(y) / 7.5 - 1.0;
                    let mut dir_z = f64::from(z) / 7.5 - 1.0;

                    let length = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
                    dir_x /= length;
                    dir_y /= length;
                    dir_z /= length;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y;
                    let mut pos_z = self.pos.z;

                    let random_val = ray_roll();
                    let mut h = self.power * random_val.mul_add(0.6, 0.7);
                    let mut cell: Option<RayCell> = None;

                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);

                        if !world.is_in_build_limit(block_pos) {
                            break;
                        }

                        if cell.as_ref().is_some_and(|cell| cell.pos != block_pos) {
                            cell = None;
                        }
                        let cell = cell.get_or_insert_with(|| {
                            self.explosion_cell(world, blocks, calc, block_pos)
                        });

                        if !matches!(cell.resistance, RayResistance::Empty) {
                            if let RayResistance::Resists(resistance) = cell.resistance {
                                h -= (resistance + 0.3) * 0.3;
                            }

                            if h > 0.0
                                && !cell.added
                                && !cell.protects_rail
                                && calc.should_block_explode(self, world, &block_pos, cell.block, h)
                            {
                                positions.insert(block_pos);
                                cell.added = true;
                            }
                        }

                        pos_x += dir_x * 0.3;
                        pos_y += dir_y * 0.3;
                        pos_z += dir_z * 0.3;
                        h -= 0.225_000_01;
                    }
                }
            }
        }

        positions
    }

    /// Reads the block a ray step is in: its resistance to this explosion and whether the
    /// rails rule spares it.
    fn explosion_cell(
        &self,
        world: &World,
        blocks: &ExplosionChunks<'_>,
        calc: &dyn ExplosionDamageCalculator,
        block_pos: BlockPos,
    ) -> RayCell {
        let state_id = blocks.state_id(&block_pos);
        let (block, state) = BlockState::from_id_with_block(state_id);
        let (_fluid, fluid_state) = Fluid::fluid_state_of(state_id);

        if state.is_air() && fluid_state.is_empty {
            return RayCell {
                pos: block_pos,
                block,
                resistance: RayResistance::Empty,
                protects_rail: false,
                added: false,
            };
        }

        let protects_rail = self.protects_rail(world, &block_pos, block);
        let resistance = if protects_rail {
            Some(0.0)
        } else {
            calc.get_block_explosion_resistance(self, world, &block_pos, block, &fluid_state)
        };

        // The exploding entity adjusts resistance
        // unless the explosion brought its own calculator.
        let resistance = match (self.source, &self.damage_calculator) {
            (Some(source), None) => {
                resistance.map(|r| source.get_block_explosion_resistance(block, r))
            }
            _ => resistance,
        };
        RayCell {
            pos: block_pos,
            block,
            resistance: resistance.map_or(RayResistance::Free, RayResistance::Resists),
            protects_rail,
            added: false,
        }
    }

    fn damage_entities(&self, world: &Arc<World>, blocks: &ExplosionChunks<'_>) {
        // Explosion is too small
        if self.power < 1.0e-5 {
            return;
        }

        let radius = self.power as f64 * 2.0;
        let min_x = (self.pos.x - radius - 1.0).floor() as i32;
        let max_x = (self.pos.x + radius + 1.0).floor() as i32;
        let min_y = (self.pos.y - radius - 1.0).floor() as i32;
        let max_y = (self.pos.y + radius + 1.0).floor() as i32;
        let min_z = (self.pos.z - radius - 1.0).floor() as i32;
        let max_z = (self.pos.z + radius + 1.0).floor() as i32;

        let search_box = BoundingBox::new(
            Vector3::new(min_x as f64, min_y as f64, min_z as f64),
            Vector3::new(max_x as f64, max_y as f64, max_z as f64),
        );

        let entities = world.get_all_at_box(&search_box);

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };

        // TODO vanilla also skips the source entity, checks `doTeamsAllowDamage`, gives redirectable projectiles
        // the damage source's owner and calls `onExplosionHit`. `Explosion` has no source yet.
        for entity_base in entities {
            if entity_base.is_immune_to_explosion() {
                continue;
            }

            // Skip spectators (no damage, no knockback)
            if entity_base.is_spectator() {
                continue;
            }

            let entity = entity_base.get_entity();
            if self
                .source
                .is_some_and(|source| source.get_entity().entity_id == entity.entity_id)
            {
                continue;
            }

            let distance = (entity.pos.load().squared_distance_to_vec(&self.pos)).sqrt() / radius;
            if distance > 1.0 {
                continue;
            }

            let should_damage = calc.should_damage_entity(self, entity_base.as_ref());
            let knockback_multiplier = calc.get_knockback_multiplier(entity_base.as_ref()) as f64;

            let exposure = if !should_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                // Hurting or pushing the previous entity may have changed blocks.
                blocks.next_pass();
                Self::calculate_exposure(&self.pos, entity, blocks) as f64
            };

            // No exposure still deals the formula's flat 1 damage, like vanilla.
            if should_damage
                && self
                    .source
                    .is_none_or(|source| source.do_teams_allow_damage(entity_base.as_ref()))
            {
                let damage =
                    calc.get_entity_damage_amount(self, entity_base.as_ref(), exposure as f32);
                entity_base.damage_with_context(
                    entity_base.as_ref(),
                    damage,
                    self.damage_type(),
                    None,
                    self.source,
                    self.indirect_source(),
                );
            }

            // Calculate and apply knockback
            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let direction = (dir_pos - self.pos).normalize();

            // Vanilla's `RangedAttribute` keeps this in 0..=1, Pumpkin's attribute values are unclamped.
            let knockback_resistance = entity_base.get_living_entity().map_or(0.0, |living| {
                living
                    .get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE)
                    .clamp(0.0, 1.0)
            });
            let knockback_power =
                (1.0 - distance) * exposure * knockback_multiplier * (1.0 - knockback_resistance);
            let knockback = direction * knockback_power;
            // Vanilla `ServerExplosion.hurtEntities`: creative flyers get no knockback packet.
            if !entity_base
                .get_player()
                .is_some_and(|player| player.is_creative() && player.is_flying())
            {
                // A living entity's goes through `EntityKnockbackEvent`
                let knockback = if entity_base.get_living_entity().is_some() {
                    entity.fire_knockback_events(knockback_power, knockback, None)
                } else {
                    Some(knockback)
                };
                if let Some(knockback) = knockback {
                    // TODO vanilla sends a player's knockback in `ClientboundExplodePacket`, added to the client's
                    // motion. Setting the motion here starts from the server's stale copy of a player's velocity.
                    entity.add_velocity(knockback);
                }
            }

            self.on_explosion_hit(entity_base.as_ref());
        }
    }

    /// A pushed redirectable projectile now belongs to whoever caused the explosion, and a player remembers the
    /// impulse for fall damage.
    fn on_explosion_hit(&self, entity_base: &dyn EntityBase) {
        if is_redirectable(entity_base.get_entity())
            && let Some(owner) = entity_base.get_projectile_owner()
        {
            owner.set_uuid(
                self.indirect_source()
                    .map(|cause| cause.get_entity().entity_uuid),
            );
        } else if let Some(player) = entity_base.get_player() {
            // Vanilla `ServerPlayer.onExplosionHit`: wind-charge explosions skip fall damage.
            player
                .living_entity
                .set_ignore_fall_damage_from_current_impulse(
                    self.from_wind_charge,
                    player.position(),
                );
        }
    }

    fn calculate_exposure(
        explosion_pos: &Vector3<f64>,
        entity: &Entity,
        blocks: &(impl BlockAccessor + ?Sized),
    ) -> f32 {
        let bbox = entity.bounding_box.load();

        let step_x = 1.0 / ((bbox.max.x - bbox.min.x) * 2.0 + 1.0);
        let step_y = 1.0 / ((bbox.max.y - bbox.min.y) * 2.0 + 1.0);
        let step_z = 1.0 / ((bbox.max.z - bbox.min.z) * 2.0 + 1.0);

        if step_x < 0.0 || step_y < 0.0 || step_z < 0.0 {
            return 0.0;
        }

        let offset_x = (1.0 - (1.0 / step_x).floor() * step_x) / 2.0;
        let offset_z = (1.0 - (1.0 / step_z).floor() * step_z) / 2.0;

        let mut visible_points = 0;
        let mut total_points = 0;

        let mut k = 0.0;
        while k <= 1.0 {
            let mut l = 0.0;
            while l <= 1.0 {
                let mut m = 0.0;
                while m <= 1.0 {
                    let n = bbox.min.x + (bbox.max.x - bbox.min.x) * k;
                    let o = bbox.min.y + (bbox.max.y - bbox.min.y) * l;
                    let p = bbox.min.z + (bbox.max.z - bbox.min.z) * m;

                    let vec3d = Vector3::new(n + offset_x, o, p + offset_z);

                    if pumpkin_world::clip::clip(
                        blocks,
                        &pumpkin_world::clip::ClipContext::new(
                            vec3d,
                            *explosion_pos,
                            pumpkin_world::clip::ClipBlock::Collider,
                            pumpkin_world::clip::ClipFluid::None,
                        ),
                    )
                    .is_miss
                    {
                        visible_points += 1;
                    }

                    total_points += 1;
                    m += step_z;
                }
                l += step_y;
            }
            k += step_x;
        }

        if total_points == 0 {
            return 0.0;
        }

        visible_points as f32 / total_points as f32
    }

    /// Vanilla `ServerExplosion.explode`: returns the count of affected positions, also for `Keep`.
    pub fn explode(&self, world: &Arc<World>) -> u32 {
        // TODO vanilla passes the source entity with the event. `Explosion` has no source yet.
        world.emit_game_event(
            pumpkin_data::game_event::GameEvent::Explode.name(),
            self.pos,
        );
        // Vanilla collects the positions before `hurtEntities`: a crystal set off by the damage
        // must not change which blocks this explosion breaks. `Keep` counts them too, for the packet.
        // Covers the rays (at most 1.74 × power long) and the entity search box (2 × power + 1).
        let chunks = ExplosionChunks::around(world, self.pos, f64::from(self.power) * 2.0 + 2.0);
        let blocks = self.get_blocks_to_destroy(world, &chunks);
        self.damage_entities(world, &chunks);
        drop(chunks);

        match self.block_interaction {
            BlockInteraction::Keep => blocks.len() as u32,
            BlockInteraction::TriggerBlock => {
                for pos in &blocks {
                    // Vanilla `interactWithBlocks` reads the live state.
                    let block = world.get_block(pos);
                    if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(block.id) {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }
                }
                blocks.len() as u32
            }
            BlockInteraction::Destroy | BlockInteraction::DestroyWithDecay => {
                let center_pos = BlockPos::floored(self.pos.x, self.pos.y, self.pos.z);
                let yield_rate = if self.power > 0.0 {
                    1.0 / self.power
                } else {
                    1.0
                };
                if world.cancelled_by_plugin(|| {
                    crate::plugin::api::events::block::block_explode::BlockExplodeEvent::new(
                        center_pos, yield_rate,
                    )
                }) {
                    return 0;
                }

                let decay_drops = self.block_interaction == BlockInteraction::DestroyWithDecay;
                let base_params = LootContextParameters {
                    explosion_radius: decay_drops.then_some(self.power),
                    world_time: world.level_info.load().day_time as u64,
                    is_raining: Some(world.is_raining()),
                    is_thundering: Some(world.is_thundering()),
                    // Vanilla `THIS_ENTITY` is the explosion's direct source.
                    this_entity: self.source,
                    ..Default::default()
                };
                let mut drops: Vec<(BlockPos, ItemStack)> = Vec::new();

                for pos in &blocks {
                    // Vanilla reads the live state: updates from earlier blocks may have broken or replaced this one.
                    let (block, state) = world.get_block_and_state(pos);
                    if state.is_air() {
                        continue;
                    }
                    let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

                    // TODO vanilla `onExplosionHit`: blocks drop XP (`spawnAfterBreak`) when the indirect source is a player.
                    if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                        let params = LootContextParameters {
                            block_state: Some(state),
                            // Vanilla `ORIGIN` is `Vec3.atCenterOf(pos)`.
                            position: Some(pos.to_centered_f64()),
                            ..base_params.clone()
                        };
                        for stack in collect_loot(world, block, pos, &params) {
                            add_or_append_stack(&mut drops, stack, *pos);
                        }
                    }

                    world.set_block_state(pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
                    world.close_container_screens_at(pos);

                    if let Some(pumpkin_block) = pumpkin_block {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }
                }
                for (pos, stack) in drops {
                    world.drop_stack(&pos, stack);
                }
                // TODO: fire
                blocks.len() as u32
            }
        }
    }
}

/// Vanilla `ServerExplosion.addOrAppendStack`: explosion drops merge into piles of up to 16
/// before they spawn, each pile at the first block that dropped it.
fn add_or_append_stack(
    drops: &mut Vec<(BlockPos, ItemStack)>,
    mut stack: ItemStack,
    pos: BlockPos,
) {
    // Vanilla reads an empty stack as air: it never merges and spawns nothing.
    if stack.is_empty() {
        return;
    }
    for (_, pile) in drops.iter_mut() {
        if ItemEntity::are_mergeable(pile, &stack) {
            ItemEntity::merge(pile, &mut stack, 16);
        }
        if stack.is_empty() {
            return;
        }
    }
    drops.push((pos, stack));
}

#[cfg(test)]
mod tests {
    use super::{
        BlockInteraction, Explosion, ExplosionDamageCalculator, SimpleExplosionDamageCalculator,
        add_or_append_stack,
    };
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::{Block, fluid::Fluid, particle::Particle, sound::Sound, tag};
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

    #[test]
    fn tnt_minecart_rail_protection_covers_every_rail_type() {
        for rail in [
            &Block::RAIL,
            &Block::POWERED_RAIL,
            &Block::DETECTOR_RAIL,
            &Block::ACTIVATOR_RAIL,
        ] {
            assert!(Explosion::is_rail(rail));
        }
        assert!(!Explosion::is_rail(&Block::STONE));
    }

    #[test]
    fn explosion_drops_merge_into_piles_of_16() {
        let mut drops = Vec::new();
        for x in 0..20 {
            add_or_append_stack(
                &mut drops,
                ItemStack::new(1, &Item::COBBLESTONE),
                BlockPos::new(x, 0, 0),
            );
        }
        add_or_append_stack(
            &mut drops,
            ItemStack::new(1, &Item::DIRT),
            BlockPos::new(0, 1, 0),
        );

        let counts: Vec<_> = drops
            .iter()
            .map(|(pos, stack)| (pos.0.x, stack.item.id, stack.item_count))
            .collect();
        assert_eq!(
            counts,
            [
                (0, Item::COBBLESTONE.id, 16),
                (16, Item::COBBLESTONE.id, 4),
                (0, Item::DIRT.id, 1),
            ]
        );
    }

    /// A multi-item stack tops up a pile to 16, the rest becomes one pile of any size.
    #[test]
    fn multi_item_drop_tops_up_then_stays_one_pile() {
        let mut drops = vec![(
            BlockPos::new(0, 0, 0),
            ItemStack::new(10, &Item::COBBLESTONE),
        )];

        add_or_append_stack(
            &mut drops,
            ItemStack::new(32, &Item::COBBLESTONE),
            BlockPos::new(1, 0, 0),
        );

        let counts: Vec<_> = drops.iter().map(|(_, stack)| stack.item_count).collect();
        assert_eq!(counts, [16, 26]);
    }

    /// Vanilla `areMergable` checks the whole incoming count: 10 + 60 > 64 skips a pile with room for 6.
    #[test]
    fn pile_with_room_is_skipped_when_both_exceed_the_max_stack() {
        let mut drops = vec![(
            BlockPos::new(0, 0, 0),
            ItemStack::new(10, &Item::COBBLESTONE),
        )];

        add_or_append_stack(
            &mut drops,
            ItemStack::new(60, &Item::COBBLESTONE),
            BlockPos::new(1, 0, 0),
        );

        let counts: Vec<_> = drops.iter().map(|(_, stack)| stack.item_count).collect();
        assert_eq!(counts, [10, 60]);
    }

    /// The pre-cache ray march, one block read per 0.3 step, kept as the reference.
    fn stepwise_positions(
        explosion: &Explosion<'_>,
        world: &crate::world::World,
        mut ray_roll: impl FnMut() -> f32,
    ) -> rustc_hash::FxHashSet<BlockPos> {
        use pumpkin_data::BlockState;
        let mut positions = rustc_hash::FxHashSet::default();
        let calc = super::DefaultExplosionDamageCalculator;
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if x > 0 && x < 15 && y > 0 && y < 15 && z > 0 && z < 15 {
                        continue;
                    }
                    let mut dir_x = f64::from(x) / 7.5 - 1.0;
                    let mut dir_y = f64::from(y) / 7.5 - 1.0;
                    let mut dir_z = f64::from(z) / 7.5 - 1.0;
                    let length = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
                    dir_x /= length;
                    dir_y /= length;
                    dir_z /= length;
                    let (mut pos_x, mut pos_y, mut pos_z) =
                        (explosion.pos.x, explosion.pos.y, explosion.pos.z);
                    let mut h = explosion.power * ray_roll().mul_add(0.6, 0.7);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);
                        if !world.is_in_build_limit(block_pos) {
                            break;
                        }
                        let state_id = world.get_block_state_id(&block_pos);
                        let (block, state) = BlockState::from_id_with_block(state_id);
                        let (_fluid, fluid_state) = Fluid::fluid_state_of(state_id);
                        if !state.is_air() || !fluid_state.is_empty {
                            let protects_rail = explosion.protects_rail(world, &block_pos, block);
                            let resistance = if protects_rail {
                                Some(0.0)
                            } else {
                                calc.get_block_explosion_resistance(
                                    explosion,
                                    world,
                                    &block_pos,
                                    block,
                                    &fluid_state,
                                )
                            };
                            if let Some(resistance) = resistance {
                                h -= (resistance + 0.3) * 0.3;
                            }
                            if h > 0.0
                                && !protects_rail
                                && calc.should_block_explode(explosion, world, &block_pos, block, h)
                            {
                                positions.insert(block_pos);
                            }
                        }
                        pos_x += dir_x * 0.3;
                        pos_y += dir_y * 0.3;
                        pos_z += dir_z * 0.3;
                        h -= 0.225_000_01;
                    }
                }
            }
        }
        positions
    }

    /// Reading each block once per ray, not once per step, explodes exactly the same blocks.
    #[tokio::test]
    async fn cached_ray_march_matches_the_stepwise_march() -> Result<(), Box<dyn std::error::Error>>
    {
        use crate::world::BlockFlags;
        use pumpkin_util::math::vector2::Vector2;

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        for cx in -2..=1 {
            for cz in -2..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(cx, cz),
                    pumpkin_world::chunk::ChunkData::empty_sync(cx, cz),
                );
            }
        }
        // A deterministic mix of air, weak, strong and fluid blocks, rails included.
        let palette = [
            Block::AIR.default_state.id,
            Block::AIR.default_state.id,
            Block::AIR.default_state.id,
            Block::STONE.default_state.id,
            Block::OAK_PLANKS.default_state.id,
            Block::GLASS.default_state.id,
            Block::OBSIDIAN.default_state.id,
            Block::WATER.default_state.id,
            Block::TNT.default_state.id,
            Block::RAIL.default_state.id,
            Block::DIRT.default_state.id,
        ];
        let mut seed = 7u64;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) as u32
        };
        for x in -9..9 {
            for y in 56..74 {
                for z in -9..9 {
                    let state = palette[next() as usize % palette.len()];
                    world.set_block_state(&BlockPos::new(x, y, z), state, BlockFlags::FORCE_STATE);
                }
            }
        }

        for (power, rails) in [(4.0, false), (4.0, true), (2.0, false), (6.0, false)] {
            let center = Vector3::new(0.37, 64.81, -0.52);
            let mut explosion = Explosion::new(power, center, BlockInteraction::Destroy);
            if rails {
                explosion = explosion.preserving_rails();
            }
            let rolls: Vec<f32> = (0..1352).map(|_| (next() % 1000) as f32 / 1000.0).collect();
            let mut cached_rolls = rolls.iter().copied();
            let mut stepwise_rolls = rolls.iter().copied();
            let chunks =
                super::ExplosionChunks::around(&world, center, f64::from(power) * 2.0 + 2.0);
            let cached = explosion
                .exploded_positions(&world, &chunks, || cached_rolls.next().unwrap_or(0.5));
            let stepwise =
                stepwise_positions(&explosion, &world, || stepwise_rolls.next().unwrap_or(0.5));
            assert!(!stepwise.is_empty());
            assert_eq!(cached, stepwise, "power {power}, rails {rails}");

            // Exposure through the cached chunks sees what a clip through the world sees.
            for _ in 0..40 {
                let at = Vector3::new(
                    f64::from(next() % 1400) / 100.0 - 7.0,
                    f64::from(next() % 1400) / 100.0 + 57.0,
                    f64::from(next() % 1400) / 100.0 - 7.0,
                );
                let body = crate::entity::Entity::new(
                    world.clone(),
                    at,
                    &pumpkin_data::entity::EntityType::ZOMBIE,
                );
                chunks.next_pass();
                let through_cache = Explosion::calculate_exposure(&center, &body, &chunks);
                let through_world = Explosion::calculate_exposure(&center, &body, world.as_ref());
                assert_eq!(
                    through_cache.to_bits(),
                    through_world.to_bits(),
                    "at {at:?}"
                );
            }
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn wind_burst_explosion_uses_large_gust_particle_and_immune_block_resistance()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let explosion = Explosion::new(
            3.5,
            Vector3::new(0.0, 64.0, 0.0),
            BlockInteraction::TriggerBlock,
        )
        .with_particles_and_sound(
            Particle::GustEmitterSmall,
            Particle::GustEmitterLarge,
            Sound::EntityWindChargeWindBurst,
        );
        assert!(!explosion.is_small());
        assert_eq!(explosion.particle(), Particle::GustEmitterLarge);
        assert_eq!(explosion.sound(), Sound::EntityWindChargeWindBurst);

        let calc = SimpleExplosionDamageCalculator::new(
            true,
            false,
            Some(1.2),
            Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
        );
        let pos = BlockPos::new(0, 64, 0);
        let empty_fluid = &Fluid::EMPTY.states[0];
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::BEDROCK,
                empty_fluid
            ),
            Some(3_600_000.0)
        );
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::LEVER,
                empty_fluid
            ),
            None
        );
        assert!(calc.should_block_explode(&explosion, &world, &pos, &Block::LEVER, 1.0));

        world.level.shutdown().await;
        Ok(())
    }
}
