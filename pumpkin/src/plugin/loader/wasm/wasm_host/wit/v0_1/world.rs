use pumpkin_data::BlockDirection as InternalBlockDirection;
use pumpkin_data::block_properties::NoteblockInstrument as InternalNoteblockInstrument;
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::chunk::io::Dirtiable;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use wasmtime::component::Resource;

use crate::block::entities::chest::ChestBlockEntity as InternalChestBlockEntity;
use crate::block::entities::command_block::CommandBlockEntity as InternalCommandBlockEntity;
use crate::block::entities::jukebox::JukeboxBlockEntity as InternalJukeboxBlockEntity;
use crate::block::entities::mob_spawner::MobSpawnerBlockEntity as InternalMobSpawnerBlockEntity;
use crate::block::entities::sign::SignBlockEntity as InternalSignBlockEntity;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::{
    BlockDirection as WitBlockDirection, BlockEntity, BlockEntityType, BlockFlags as WitBlockFlags,
    BlockPos as WitBlockPos, BlockState as WitBlockState, BoundingBox as WitBoundingBox,
    Chunk as WitChunk, NoteblockInstrument as WitNoteblockInstrument,
    PistonBehavior as WitPistonBehavior, WorldBorder as WitWorldBorder,
};
use crate::plugin::loader::wasm::wasm_host::{
    state::{
        ChunkResource, PluginHostState, TextComponentResource, WorldBorderResource, WorldResource,
    },
    wit::v0_1::pumpkin::{self, plugin::world::World},
};
use crate::world::explosion::Explosion;

pub(crate) const fn to_wasm_block_direction(dir: InternalBlockDirection) -> WitBlockDirection {
    match dir {
        InternalBlockDirection::Down => WitBlockDirection::Down,
        InternalBlockDirection::Up => WitBlockDirection::Up,
        InternalBlockDirection::North => WitBlockDirection::North,
        InternalBlockDirection::South => WitBlockDirection::South,
        InternalBlockDirection::West => WitBlockDirection::West,
        InternalBlockDirection::East => WitBlockDirection::East,
    }
}

pub(crate) const fn to_wit_noteblock_instrument(
    instr: InternalNoteblockInstrument,
) -> WitNoteblockInstrument {
    match instr {
        InternalNoteblockInstrument::Harp => WitNoteblockInstrument::Harp,
        InternalNoteblockInstrument::Basedrum => WitNoteblockInstrument::Basedrum,
        InternalNoteblockInstrument::Snare => WitNoteblockInstrument::Snare,
        InternalNoteblockInstrument::Hat => WitNoteblockInstrument::Hat,
        InternalNoteblockInstrument::Bass => WitNoteblockInstrument::Bass,
        InternalNoteblockInstrument::Flute => WitNoteblockInstrument::Flute,
        InternalNoteblockInstrument::Bell => WitNoteblockInstrument::Bell,
        InternalNoteblockInstrument::Guitar => WitNoteblockInstrument::Guitar,
        InternalNoteblockInstrument::Chime => WitNoteblockInstrument::Chime,
        InternalNoteblockInstrument::Xylophone => WitNoteblockInstrument::Xylophone,
        InternalNoteblockInstrument::IronXylophone => WitNoteblockInstrument::IronXylophone,
        InternalNoteblockInstrument::CowBell => WitNoteblockInstrument::CowBell,
        InternalNoteblockInstrument::Didgeridoo => WitNoteblockInstrument::Didgeridoo,
        InternalNoteblockInstrument::Bit => WitNoteblockInstrument::Bit,
        InternalNoteblockInstrument::Banjo => WitNoteblockInstrument::Banjo,
        InternalNoteblockInstrument::Pling => WitNoteblockInstrument::Pling,
        InternalNoteblockInstrument::Trumpet => WitNoteblockInstrument::Trumpet,
        InternalNoteblockInstrument::TrumpetExposed => WitNoteblockInstrument::TrumpetExposed,
        InternalNoteblockInstrument::TrumpetOxidized => WitNoteblockInstrument::TrumpetOxidized,
        InternalNoteblockInstrument::TrumpetWeathered => WitNoteblockInstrument::TrumpetWeathered,
        InternalNoteblockInstrument::Zombie => WitNoteblockInstrument::Zombie,
        InternalNoteblockInstrument::Skeleton => WitNoteblockInstrument::Skeleton,
        InternalNoteblockInstrument::Creeper => WitNoteblockInstrument::Creeper,
        InternalNoteblockInstrument::Dragon => WitNoteblockInstrument::Dragon,
        InternalNoteblockInstrument::WitherSkeleton => WitNoteblockInstrument::WitherSkeleton,
        InternalNoteblockInstrument::Piglin => WitNoteblockInstrument::Piglin,
        InternalNoteblockInstrument::CustomHead => WitNoteblockInstrument::CustomHead,
    }
}

pub(crate) const fn to_wit_bounding_box(
    bb: pumpkin_util::math::boundingbox::BoundingBox,
) -> WitBoundingBox {
    WitBoundingBox {
        min: (bb.min.x, bb.min.y, bb.min.z),
        max: (bb.max.x, bb.max.y, bb.max.z),
    }
}

// --- Trapping Helpers ---
impl PluginHostState {
    fn get_world_res(&self, res: &Resource<World>) -> wasmtime::Result<&WorldResource> {
        self.resource_table
            .get::<WorldResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_chunk_res(&self, res: &Resource<WitChunk>) -> wasmtime::Result<&ChunkResource> {
        self.resource_table
            .get::<ChunkResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_world_border_res(
        &self,
        res: &Resource<WitWorldBorder>,
    ) -> wasmtime::Result<&WorldBorderResource> {
        self.resource_table
            .get::<WorldBorderResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    pub(crate) fn get_text_provider(
        &self,
        res: &Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<pumpkin_util::text::TextComponent> {
        Ok(self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)?
            .provider
            .clone())
    }

    fn get_wit_biome(
        biome: &pumpkin_data::biome::Biome,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let mut names: Vec<String> = serde_json::from_str::<
            std::collections::BTreeMap<String, serde_json::Value>,
        >(&std::fs::read_to_string("assets/biome.json")?)?
        .keys()
        .cloned()
        .collect();
        names.sort();

        let index = names
            .iter()
            .position(|n| n.strip_prefix("minecraft:").unwrap_or(n) == biome.registry_id)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown biome: {}", biome.registry_id)))?;

        // Safety: The WIT enum is generated from the sorted keys of assets/biome.json.
        Ok(unsafe { std::mem::transmute::<u8, pumpkin::plugin::biomes::Biome>(index as u8) })
    }

    fn get_wit_block_entity(
        &mut self,
        block_entity: Arc<dyn crate::block::entities::BlockEntity>,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let be = block_entity;
        if be
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .is_some()
        {
            let res: Resource<BlockEntity> = self.add_block_entity(be)?;
            Ok(Some(BlockEntityType::CommandBlockEntity(
                Resource::new_own(res.rep()),
            )))
        } else if be
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .is_some()
        {
            let res: Resource<BlockEntity> = self.add_block_entity(be)?;
            Ok(Some(BlockEntityType::SignBlockEntity(Resource::new_own(
                res.rep(),
            ))))
        } else if be
            .as_any()
            .downcast_ref::<InternalJukeboxBlockEntity>()
            .is_some()
        {
            let res: Resource<BlockEntity> = self.add_block_entity(be)?;
            Ok(Some(BlockEntityType::JukeboxBlockEntity(
                Resource::new_own(res.rep()),
            )))
        } else if be
            .as_any()
            .downcast_ref::<InternalChestBlockEntity>()
            .is_some()
        {
            let res: Resource<BlockEntity> = self.add_block_entity(be)?;
            Ok(Some(BlockEntityType::ChestBlockEntity(Resource::new_own(
                res.rep(),
            ))))
        } else if be
            .as_any()
            .downcast_ref::<InternalMobSpawnerBlockEntity>()
            .is_some()
        {
            let res: Resource<BlockEntity> = self.add_block_entity(be)?;
            Ok(Some(BlockEntityType::MobSpawnerBlockEntity(
                Resource::new_own(res.rep()),
            )))
        } else {
            Ok(None)
        }
    }
}

impl pumpkin::plugin::world::Host for PluginHostState {}
impl pumpkin::plugin::particles::Host for PluginHostState {}
impl pumpkin::plugin::sounds::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_world_name()
            .to_string())
    }

    async fn get_world_border(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Resource<WitWorldBorder>> {
        let world_res = self.get_world_res(&world)?;
        self.add_world_border(world_res.provider.clone())
    }

    async fn get_chunk(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<Option<Resource<WitChunk>>> {
        let world_res = self.get_world_res(&world)?;
        let world_provider = world_res.provider.clone();
        let pos = pumpkin_util::math::vector2::Vector2::new(x, z);

        let chunk = world_provider
            .level
            .loaded_chunks
            .get(&pos)
            .map(|c| c.value().clone());
        if let Some(chunk) = chunk {
            let res = self.add_chunk(world_provider, std::sync::Arc::downgrade(&chunk))?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_block_state_id(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref.provider.get_block_state_id(&internal_pos))
    }

    async fn get_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlockState> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world_ref.provider.get_block_state(&internal_pos);

        Ok(WitBlockState {
            id: state.id,
            luminance: state.luminance,
            opacity: state.opacity,
            hardness: state.hardness,
            is_air: state.is_air(),
            is_liquid: state.is_liquid(),
            is_solid: state.is_solid(),
            is_full_cube: state.is_full_cube(),
            has_random_ticks: state.has_random_ticks(),
            piston_behavior: match state.piston_behavior {
                PistonBehavior::Normal => WitPistonBehavior::Normal,
                PistonBehavior::Destroy => WitPistonBehavior::Destroy,
                PistonBehavior::Block => WitPistonBehavior::Block,
                PistonBehavior::Ignore => WitPistonBehavior::Ignore,
                PistonBehavior::PushOnly => WitPistonBehavior::PushOnly,
            },
            burnable: state.burnable(),
            tool_required: state.tool_required(),
            sided_transparency: state.sided_transparency(),
            replaceable: state.replaceable(),
            is_solid_block: state.is_solid_block(),
            block_entity_type: state.block_entity_type,
            instrument: to_wit_noteblock_instrument(state.instrument),
            collision_shapes: state
                .get_block_collision_shapes()
                .map(to_wit_bounding_box)
                .collect(),
            outline_shapes: state
                .get_block_outline_shapes()
                .map(to_wit_bounding_box)
                .collect(),
            down_side_solid: state.is_side_solid(InternalBlockDirection::Down),
            up_side_solid: state.is_side_solid(InternalBlockDirection::Up),
            north_side_solid: state.is_side_solid(InternalBlockDirection::North),
            south_side_solid: state.is_side_solid(InternalBlockDirection::South),
            west_side_solid: state.is_side_solid(InternalBlockDirection::West),
            east_side_solid: state.is_side_solid(InternalBlockDirection::East),
            down_center_solid: state.is_center_solid(InternalBlockDirection::Down),
            up_center_solid: state.is_center_solid(InternalBlockDirection::Up),
            map_color: pumpkin_data::Block::from_state_id(state.id).map_color,
        })
    }

    async fn set_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        state: u16,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        let mut internal_flags = BlockFlags::empty();
        if update_flags.contains(WitBlockFlags::NOTIFY_NEIGHBORS) {
            internal_flags |= BlockFlags::NOTIFY_NEIGHBORS;
        }
        if update_flags.contains(WitBlockFlags::NOTIFY_LISTENERS) {
            internal_flags |= BlockFlags::NOTIFY_LISTENERS;
        }
        if update_flags.contains(WitBlockFlags::FORCE_STATE) {
            internal_flags |= BlockFlags::FORCE_STATE;
        }
        if update_flags.contains(WitBlockFlags::SKIP_DROPS) {
            internal_flags |= BlockFlags::SKIP_DROPS;
        }
        if update_flags.contains(WitBlockFlags::MOVED) {
            internal_flags |= BlockFlags::MOVED;
        }
        if update_flags.contains(WitBlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT) {
            internal_flags |= BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ADDED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ADDED_CALLBACK;
        }

        world_ref
            .provider
            .clone()
            .set_block_state(&internal_pos, state, internal_flags)
            .await;
        Ok(())
    }

    async fn get_time_of_day(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_time_of_day().await as u64)
    }

    async fn set_time_of_day(&mut self, world: Resource<World>, time: u64) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_time_of_day(time as i64)
            .await;
        Ok(())
    }

    async fn get_world_age(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_world_age().await as u64)
    }

    async fn get_dimension(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .dimension
            .minecraft_name
            .to_string())
    }

    async fn get_top_block_y(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_top_block(pumpkin_util::math::vector2::Vector2::new(x, z)))
    }

    async fn get_motion_blocking_height(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.get_heightmap_height(
            ChunkHeightmapType::MotionBlocking,
            x,
            z,
        ))
    }

    async fn is_raining(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_raining().await)
    }

    async fn set_raining(&mut self, world: Resource<World>, raining: bool) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_raining(raining)
            .await;
        Ok(())
    }

    async fn is_thundering(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_thundering().await)
    }

    async fn set_thundering(
        &mut self,
        world: Resource<World>,
        thundering: bool,
    ) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_thundering(thundering)
            .await;
        Ok(())
    }

    async fn broadcast_system_message(
        &mut self,
        world: Resource<World>,
        message: Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let msg = self.get_text_provider(&message)?;
        self.get_world_res(&world)?
            .provider
            .broadcast_system_message(&msg, overlay)
            .await;
        Ok(())
    }

    async fn get_scoreboard(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::scoreboard::Scoreboard>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        self.add_scoreboard(world_provider)
    }

    async fn play_sound(
        &mut self,
        world: Resource<World>,
        sound: pumpkin::plugin::sounds::Sound,
        category: pumpkin::plugin::world::SoundCategory,
        pos: pumpkin::plugin::common::Position,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let sound_name = format!("{sound:?}").to_lowercase().replace('_', ".");
        let sound_data = pumpkin_data::sound::Sound::from_name(&sound_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown sound: {sound_name}")))?;

        let internal_category = match category {
            pumpkin::plugin::world::SoundCategory::Master => {
                pumpkin_data::sound::SoundCategory::Master
            }
            pumpkin::plugin::world::SoundCategory::Music => {
                pumpkin_data::sound::SoundCategory::Music
            }
            pumpkin::plugin::world::SoundCategory::Records => {
                pumpkin_data::sound::SoundCategory::Records
            }
            pumpkin::plugin::world::SoundCategory::Weather => {
                pumpkin_data::sound::SoundCategory::Weather
            }
            pumpkin::plugin::world::SoundCategory::Blocks => {
                pumpkin_data::sound::SoundCategory::Blocks
            }
            pumpkin::plugin::world::SoundCategory::Hostile => {
                pumpkin_data::sound::SoundCategory::Hostile
            }
            pumpkin::plugin::world::SoundCategory::Neutral => {
                pumpkin_data::sound::SoundCategory::Neutral
            }
            pumpkin::plugin::world::SoundCategory::Players => {
                pumpkin_data::sound::SoundCategory::Players
            }
            pumpkin::plugin::world::SoundCategory::Ambient => {
                pumpkin_data::sound::SoundCategory::Ambient
            }
            pumpkin::plugin::world::SoundCategory::Voice => {
                pumpkin_data::sound::SoundCategory::Voice
            }
        };

        world_ref.provider.play_sound_raw(
            sound_data as u16,
            internal_category,
            &pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            volume,
            pitch,
        );
        Ok(())
    }

    async fn spawn_particle(
        &mut self,
        world: Resource<World>,
        particle: pumpkin::plugin::particles::Particle,
        pos: pumpkin::plugin::common::Position,
        offset: pumpkin::plugin::common::Position,
        max_speed: f32,
        count: i32,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let particle_name = format!("{particle:?}").to_lowercase().replace('_', "-");
        let particle_data = pumpkin_data::particle::Particle::from_name(&particle_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown particle: {particle_name}")))?;

        world_ref.provider.spawn_particle(
            pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            pumpkin_util::math::vector3::Vector3::new(
                offset.0 as f32,
                offset.1 as f32,
                offset.2 as f32,
            ),
            max_speed,
            count,
            particle_data,
        );
        Ok(())
    }

    async fn create_explosion(
        &mut self,
        world: Resource<World>,
        pos: pumpkin::plugin::common::Position,
        power: f32,
        _create_fire: bool,
        _interaction: pumpkin::plugin::world::ExplosionInteraction,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        // Currently Explosion only supports power and position in this codebase
        let explosion = Explosion::new(
            power,
            pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
        );
        explosion.explode(&world_ref.provider).await;
        Ok(())
    }

    async fn get_sea_level(&mut self, world: Resource<World>) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.sea_level)
    }

    async fn get_min_y(&mut self, world: Resource<World>) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.min_y)
    }

    async fn get_sky_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref.provider.get_sky_light_level(&internal_pos))
    }

    async fn set_sky_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        level: u8,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        world_ref.provider.set_sky_light_level(&internal_pos, level);
        Ok(())
    }

    async fn get_block_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref
            .provider
            .get_block_light_level(&internal_pos)
            .unwrap_or(0))
    }

    async fn set_block_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        level: u8,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        world_ref
            .provider
            .set_block_light_level(&internal_pos, level);
        Ok(())
    }

    async fn get_biome(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let biome = world_ref.provider.get_biome(&internal_pos);

        Self::get_wit_biome(biome)
    }

    async fn spawn_entity(
        &mut self,
        world: Resource<World>,
        entity_type: pumpkin::plugin::entity_types::EntityType,
        pos: pumpkin::plugin::common::Position,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::world::Entity>> {
        let world_ref = self.get_world_res(&world)?;
        let world_provider = world_ref.provider.clone();

        let mut names: Vec<String> = serde_json::from_str::<
            std::collections::BTreeMap<String, serde_json::Value>,
        >(&std::fs::read_to_string("assets/entities.json")?)?
        .keys()
        .cloned()
        .collect();
        names.sort();

        let type_name = names.get(entity_type as usize).ok_or_else(|| {
            wasmtime::Error::msg(format!("Invalid entity type index: {}", entity_type as u8))
        })?;

        let internal_type = pumpkin_data::entity::EntityType::from_name(type_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Invalid entity type: {type_name}")))?;

        let internal_pos = pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2);
        let entity = crate::entity::r#type::from_type(
            internal_type,
            internal_pos,
            &world_provider,
            uuid::Uuid::new_v4(),
        );

        world_provider.spawn_entity(entity.clone()).await;

        self.add_entity(entity)
    }

    async fn get_entities(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::entity::Entity>>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let mut entities = Vec::new();

        // Add players as entities
        for player in world_provider.players.load().iter() {
            entities.push(self.add_entity(player.clone() as Arc<dyn crate::entity::EntityBase>)?);
        }

        // Add other entities
        for entity in world_provider.entities.load().iter() {
            entities.push(self.add_entity(entity.clone())?);
        }

        Ok(entities)
    }

    async fn get_block_entity(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let block_entity = world_provider.get_block_entity(&internal_pos);

        block_entity.map_or_else(|| Ok(None), |be| self.get_wit_block_entity(be))
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<WorldResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::world::HostChunk for PluginHostState {
    async fn get_x(&mut self, chunk: Resource<WitChunk>) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.x)
    }

    async fn get_z(&mut self, chunk: Resource<WitChunk>) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.z)
    }

    async fn get_block_state_id(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data
            .section
            .get_block_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(0))
    }

    async fn get_block_state(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlockState> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let id = chunk_data
            .section
            .get_block_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(0);
        let state = pumpkin_data::block_state::BlockState::from_id(id);

        Ok(WitBlockState {
            id: state.id,
            luminance: state.luminance,
            opacity: state.opacity,
            hardness: state.hardness,
            is_air: state.is_air(),
            is_liquid: state.is_liquid(),
            is_solid: state.is_solid(),
            is_full_cube: state.is_full_cube(),
            has_random_ticks: state.has_random_ticks(),
            piston_behavior: match state.piston_behavior {
                PistonBehavior::Normal => WitPistonBehavior::Normal,
                PistonBehavior::Destroy => WitPistonBehavior::Destroy,
                PistonBehavior::Block => WitPistonBehavior::Block,
                PistonBehavior::Ignore => WitPistonBehavior::Ignore,
                PistonBehavior::PushOnly => WitPistonBehavior::PushOnly,
            },
            burnable: state.burnable(),
            tool_required: state.tool_required(),
            sided_transparency: state.sided_transparency(),
            replaceable: state.replaceable(),
            is_solid_block: state.is_solid_block(),
            block_entity_type: state.block_entity_type,
            instrument: to_wit_noteblock_instrument(state.instrument),
            collision_shapes: state
                .get_block_collision_shapes()
                .map(to_wit_bounding_box)
                .collect(),
            outline_shapes: state
                .get_block_outline_shapes()
                .map(to_wit_bounding_box)
                .collect(),
            down_side_solid: state.is_side_solid(InternalBlockDirection::Down),
            up_side_solid: state.is_side_solid(InternalBlockDirection::Up),
            north_side_solid: state.is_side_solid(InternalBlockDirection::North),
            south_side_solid: state.is_side_solid(InternalBlockDirection::South),
            west_side_solid: state.is_side_solid(InternalBlockDirection::West),
            east_side_solid: state.is_side_solid(InternalBlockDirection::East),
            down_center_solid: state.is_center_solid(InternalBlockDirection::Down),
            up_center_solid: state.is_center_solid(InternalBlockDirection::Up),
            map_color: pumpkin_data::Block::from_state_id(state.id).map_color,
        })
    }

    async fn set_block_state(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
        state: u16,
    ) -> wasmtime::Result<()> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (world, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };

        let replaced =
            chunk_data.set_block_absolute_y(pos.x as usize, pos.y, pos.z as usize, state);

        if replaced != state {
            chunk_data.mark_dirty(true);
            let absolute_pos =
                BlockPos::new(chunk_data.x * 16 + pos.x, pos.y, chunk_data.z * 16 + pos.z);
            world.register_block_change(absolute_pos, state).await;
        }

        Ok(())
    }

    async fn get_biome(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let id = chunk_data
            .section
            .get_rough_biome_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(0);
        let biome = pumpkin_data::biome::Biome::from_id(id)
            .unwrap_or(&pumpkin_data::biome::Biome::THE_VOID);

        Self::get_wit_biome(biome)
    }

    async fn get_block_entity(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (world, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let absolute_pos =
            BlockPos::new(chunk_data.x * 16 + pos.x, pos.y, chunk_data.z * 16 + pos.z);
        let block_entity = world.get_block_entity(&absolute_pos);

        block_entity.map_or_else(|| Ok(None), |be| self.get_wit_block_entity(be))
    }

    async fn get_top_block_y(
        &mut self,
        chunk: Resource<WitChunk>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.heightmap.lock().unwrap().get(
            ChunkHeightmapType::WorldSurface,
            x,
            z,
            chunk_data.section.min_y,
        ))
    }

    async fn get_sky_light(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let section_index = (pos.y - chunk_data.section.min_y) as usize / 16;
        Ok(chunk_data
            .light_engine
            .lock()
            .unwrap()
            .sky_light
            .get(section_index)
            .map_or(0, |c| {
                c.get(pos.x as usize, pos.y as usize % 16, pos.z as usize)
            }))
    }

    async fn get_block_light(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let section_index = (pos.y - chunk_data.section.min_y) as usize / 16;
        Ok(chunk_data
            .light_engine
            .lock()
            .unwrap()
            .block_light
            .get(section_index)
            .map_or(0, |c| {
                c.get(pos.x as usize, pos.y as usize % 16, pos.z as usize)
            }))
    }

    async fn drop(&mut self, rep: Resource<WitChunk>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ChunkResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::world::HostWorldBorder for PluginHostState {
    async fn get_center_x(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.center_x)
    }

    async fn get_center_z(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.center_z)
    }

    async fn set_center(
        &mut self,
        border: Resource<WitWorldBorder>,
        x: f64,
        z: f64,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world.worldborder.lock().await.set_center(&world, x, z);
        Ok(())
    }

    async fn get_diameter(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.new_diameter)
    }

    async fn set_diameter(
        &mut self,
        border: Resource<WitWorldBorder>,
        diameter: f64,
        speed: Option<u64>,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .await
            .set_diameter(&world, diameter, speed.map(|s| s as i64));
        Ok(())
    }

    async fn get_warning_distance(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i32> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.warning_blocks)
    }

    async fn set_warning_distance(
        &mut self,
        border: Resource<WitWorldBorder>,
        distance: i32,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .await
            .set_warning_distance(&world, distance);
        Ok(())
    }

    async fn get_warning_delay(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i32> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.warning_time)
    }

    async fn set_warning_delay(
        &mut self,
        border: Resource<WitWorldBorder>,
        delay: i32,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .await
            .set_warning_delay(&world, delay);
        Ok(())
    }

    async fn contains(
        &mut self,
        border: Resource<WitWorldBorder>,
        x: f64,
        z: f64,
    ) -> wasmtime::Result<bool> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res.provider.worldborder.lock().await.contains(x, z))
    }

    async fn drop(&mut self, rep: Resource<WitWorldBorder>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<WorldBorderResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
