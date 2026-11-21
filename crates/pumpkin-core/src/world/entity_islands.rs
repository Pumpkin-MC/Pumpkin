//! Splits a tick's entities into islands that touch disjoint state. Each island ticks in list
//! order on its own thread, which gives the same result as ticking the whole list in order.

use std::collections::VecDeque;

use std::sync::Arc;

use pumpkin_data::{
    Block,
    attributes::Attributes,
    data_component_impl::{ConsumeEffect, DeathProtectionImpl},
    entity::EntityType,
    tag,
    tag::Taggable,
};
use pumpkin_util::Hand;
use pumpkin_util::math::{
    boundingbox::BoundingBox, get_section_cord, position::BlockPos, vector2::Vector2,
    vector3::Vector3,
};
use rustc_hash::FxHashMap;

use pumpkin_config::world::EntityIslandsConfig;
use pumpkin_world::chunk::CHUNK_WIDTH;

use crate::block::blocks::redstone::SIGNAL_MAX;
use crate::entity::{
    DelayedLeash, EntityBase, PUSH_STRENGTH, passive::frog::FrogEntity,
    projectile::wind_charge::WindChargeEntity, tnt::TNTEntity,
};
use crate::net::ClientPlatform;
use crate::plugin::api::events::{
    Payload,
    block::{
        BellRingEvent, BlockBreakEvent, BlockDropItemEvent, BlockExpEvent, BlockFormEvent,
        BlockPhysicsEvent, BlockRedstoneEvent, CauldronLevelChangeEvent, EntityBlockFormEvent,
        NotePlayEvent, SpongeAbsorbEvent, TNTPrimeEvent,
    },
    entity::{
        CreatureSpawnEvent, EntityBreedEvent, EntityCombustEvent, EntityDamageByEntityEvent,
        EntityDamageEvent, EntityDeathEvent, EntityDismountEvent, EntityDropItemEvent,
        EntityExhaustionEvent, EntityExplodeEvent, EntityKnockbackByEntityEvent,
        EntityKnockbackEvent, EntityMountEvent, EntityPortalEvent, EntityPoseChangeEvent,
        EntityPotionEffectEvent, EntityResurrectEvent, EntitySpawnEvent, EntityTargetEvent,
        EntityTeleportEvent, EntityToggleSitEvent, EntityUnleashEvent, ExplosionPrimeEvent,
        ItemSpawnEvent, PigZombieAngerEvent, PlayerDeathEvent,
        entity_regain_health::EntityRegainHealthEvent,
    },
    player::{
        PlayerAdvancementDoneEvent, PlayerExpChangeEvent, PlayerItemBreakEvent,
        PlayerItemDamageEvent, PlayerLevelChangeEvent, PlayerStatisticIncrementEvent,
        PlayerUnleashEntityEvent, PlayerVelocityEvent, player_teleport::PlayerTeleportEvent,
    },
    vehicle::{VehicleDamageEvent, VehicleDestroyEvent, VehicleEnterEvent, VehicleExitEvent},
    world::{GenericGameEvent, loot_generate::LootGenerateEvent},
};
use crate::plugin::{
    PluginManager, api::events::vehicle::vehicle_update::VehicleUpdateEvent,
    server::packet::PacketSentEvent,
};
use crate::server::Server;
use crate::world::{World, chunker::is_within_chebyshev_distance};

/// Reach of same-tick neighbour updates: a full wire line, the powered block and one component.
const BLOCK_UPDATE_REACH: f64 = SIGNAL_MAX as f64 + 2.0;

// A burst can travel six blocks from impact, then trigger a redstone update and a game event.
// The extra block covers the face offset and block rounding.
const WIND_CHARGE_TICK_MARGIN: f64 = 6.0 + BLOCK_UPDATE_REACH + 16.0 + 1.0;

// Block effects can send sounds two chunks away, independent of the player's watched chunks.
const WIND_CHARGE_PACKET_MARGIN: f64 = WIND_CHARGE_TICK_MARGIN + 3.0 * CHUNK_WIDTH as f64;

/// Grid cell size for large boxes.
const CELL_SIZE: f64 = 16.0;
const SMALL_CELL_SIZE: f64 = 4.0;
const MIN_COLUMN_DENSITY: usize = 32;

/// Largest speed one `Entity.push` adds.
const PUSH_SPEED: f64 = PUSH_STRENGTH * std::f64::consts::SQRT_2;

/// Extra motion covered by the contact block scan.
const SCAN_SLACK: f64 = 0.5;

/// Minimum probe margin for conservative neighbour effects.
const PUSH_SLACK: f64 = 3.0 * PUSH_SPEED;

// Larger regions retain the conservative contact-write fallback.
const MAX_CONTACT_SECTIONS: i64 = 64;
const MAX_CONTACT_BLOCKS: usize = 4096;

// Cell operations and candidate visits allowed in each planning phase.
const MAX_PLANNING_WORK: usize = 262_144;

struct PlanningBudget {
    remaining: usize,
}

impl Default for PlanningBudget {
    fn default() -> Self {
        Self {
            remaining: MAX_PLANNING_WORK,
        }
    }
}

impl PlanningBudget {
    fn spend(&mut self, work: usize) -> Option<()> {
        self.remaining = self.remaining.checked_sub(work)?;
        Some(())
    }

    fn cells(
        &mut self,
        area: &BoundingBox,
        size: f64,
    ) -> Option<[std::ops::RangeInclusive<i32>; 2]> {
        // Block reads intentionally span all Y levels.
        if area.min.y.is_nan() || area.max.y.is_nan() || area.min.y > area.max.y {
            return None;
        }
        let x = cells_of(area.min.x, area.max.x, size)?;
        let z = cells_of(area.min.z, area.max.z, size)?;
        self.spend(cell_count(&x)?.checked_mul(cell_count(&z)?)?)?;
        Some([x, z])
    }
}

const MIN_PARALLEL_ENTITIES: usize = 16;
const SERIAL_PROBE_INTERVAL: u8 = 20;
const PARALLEL_SHARE_DIVISOR: usize = 4;

#[derive(Default)]
pub(super) struct Schedule {
    entity_count: usize,
    ticks_until_probe: u8,
}

impl Schedule {
    pub(super) const fn should_plan(&mut self, entity_count: usize) -> bool {
        if self.entity_count != entity_count {
            self.entity_count = entity_count;
            self.ticks_until_probe = 0;
        }
        if entity_count < MIN_PARALLEL_ENTITIES {
            return false;
        }
        if self.ticks_until_probe > 0 {
            self.ticks_until_probe -= 1;
            return false;
        }
        true
    }

    pub(super) const fn record_plan(&mut self, largest_island: usize) -> bool {
        // A dominant island leaves too little parallel work to pay for planning.
        let parallel =
            largest_island < self.entity_count - self.entity_count / PARALLEL_SHARE_DIVISOR;
        // Only the serial preference is cached; parallel ticks always use a fresh plan.
        self.ticks_until_probe = if parallel {
            0
        } else {
            SERIAL_PROBE_INTERVAL - 1
        };
        parallel
    }
}

/// A plugin listens to an event an entity tick can fire. Its handler may touch anything,
/// so the tick runs in list order.
#[must_use]
pub fn plugins_observe_entity_ticks(server: &Server) -> bool {
    let plugins = &server.plugin_manager;
    plugins.has_handlers::<EntityTargetEvent>()
        || plugins.has_handlers::<EntityBreedEvent>()
        || plugins.has_handlers::<PigZombieAngerEvent>()
        || plugins.has_handlers::<EntityToggleSitEvent>()
        || plugins.has_handlers::<EntityBlockFormEvent>()
}

fn plugins_observe_shared_brain_tick(entity: &dyn EntityBase, movement: f64) -> bool {
    let base = entity.get_entity();
    let entity_type = base.entity_type;
    let camel = entity_type == &EntityType::CAMEL || entity_type == &EntityType::CAMEL_HUSK;
    let frog = entity_type == &EntityType::FROG;
    if !camel && !frog && entity_type != &EntityType::VILLAGER {
        return false;
    }
    let world = base.world.load();
    let Some(server) = world.server.upgrade() else {
        return false;
    };
    let plugins = &server.plugin_manager;
    if plugins.has_handlers::<EntityPoseChangeEvent>()
        || camel && plugins.has_handlers::<GenericGameEvent>()
    {
        return true;
    }
    let observes_packets = plugins.has_handlers::<PacketSentEvent>();
    // A bite can produce damage, drops and death effects such as a wind burst. A viewer
    // of the retained prey or a player credited for the kill can also receive packets.
    if frog
        && (plugins_observe_combat_or_explosion(plugins)
            || observes_packets && server.has_n_players(1))
        && entity
            .cast_any()
            .downcast_ref::<FrogEntity>()
            .is_some_and(FrogEntity::may_bite_with_tongue)
    {
        return true;
    }
    if !observes_packets {
        return false;
    }
    if world
        .entity_tracker
        .get_tracked_entity(base.entity_id)
        .is_some_and(|tracked| tracked.has_viewers())
    {
        return true;
    }

    let unbounded = !movement.is_finite() || base.has_vehicle();
    // Include movement and the nearby bed blocks changed during a Villager pose transition.
    let area = base
        .bounding_box
        .load()
        .expand(movement + 3.0, 0.0, movement + 3.0);
    let min_chunk = area.min_block_pos().chunk_position();
    let max_chunk = area.max_block_pos().chunk_position();
    world.players.load().iter().any(|player| {
        if unbounded {
            return true;
        }
        let nearest_chunk = |chunk: Vector2<i32>| {
            Vector2::new(
                chunk.x.clamp(min_chunk.x, max_chunk.x),
                chunk.y.clamp(min_chunk.y, max_chunk.y),
            )
        };
        let watch = player.watched_section.load();
        let nearest_watched = nearest_chunk(watch.center);
        let chunk = player.get_entity().chunk_pos.load();
        watch.is_within_distance(nearest_watched.x, nearest_watched.y)
            || (camel || frog)
                && matches!(player.client.as_ref(), ClientPlatform::Java(_))
                && is_within_chebyshev_distance(nearest_chunk(chunk), chunk, 2)
    })
}

fn plugins_observe_wind_charge_tick(
    plugins: &PluginManager,
    world: &World,
    charge: &WindChargeEntity,
    movement: f64,
) -> bool {
    (plugins.has_handlers::<PacketSentEvent>()
        // Breeze charge flight and bursts both send to live audiences.
        && (charge.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE
            || wind_charge_has_packet_recipients(world, charge, movement)))
        // A translated legacy client can arrive after the player snapshot and take damage immediately.
        || plugins.has_handlers::<PlayerTeleportEvent>()
        || plugins_observe_combat_or_explosion(plugins)
}

fn plugins_observe_combat_or_explosion(plugins: &PluginManager) -> bool {
    plugins.has_handlers_for(&[
        // Projectile travel, impact and explosion.
        EntityCombustEvent::get_name_static(),
        EntityPortalEvent::get_name_static(),
        GenericGameEvent::get_name_static(),
        EntityExplodeEvent::get_name_static(),
        // Damage, death, equipment and vehicle cleanup.
        EntityDamageEvent::get_name_static(),
        EntityDamageByEntityEvent::get_name_static(),
        EntityKnockbackEvent::get_name_static(),
        EntityKnockbackByEntityEvent::get_name_static(),
        EntityExhaustionEvent::get_name_static(),
        PlayerItemDamageEvent::get_name_static(),
        PlayerItemBreakEvent::get_name_static(),
        EntityResurrectEvent::get_name_static(),
        EntityRegainHealthEvent::get_name_static(),
        EntityPotionEffectEvent::get_name_static(),
        EntityPoseChangeEvent::get_name_static(),
        EntityTeleportEvent::get_name_static(),
        EntityDeathEvent::get_name_static(),
        PlayerDeathEvent::get_name_static(),
        VehicleDamageEvent::get_name_static(),
        VehicleDestroyEvent::get_name_static(),
        EntityDismountEvent::get_name_static(),
        VehicleExitEvent::get_name_static(),
        EntityUnleashEvent::get_name_static(),
        PlayerUnleashEntityEvent::get_name_static(),
        EntityMountEvent::get_name_static(),
        VehicleEnterEvent::get_name_static(),
        CreatureSpawnEvent::get_name_static(),
        EntitySpawnEvent::get_name_static(),
        EntityDropItemEvent::get_name_static(),
        ItemSpawnEvent::get_name_static(),
        PlayerAdvancementDoneEvent::get_name_static(),
        PlayerExpChangeEvent::get_name_static(),
        PlayerLevelChangeEvent::get_name_static(),
        PlayerStatisticIncrementEvent::get_name_static(),
        PlayerVelocityEvent::get_name_static(),
        // Block impact, contact and synchronous neighbour updates.
        BlockBreakEvent::get_name_static(),
        BlockDropItemEvent::get_name_static(),
        BlockExpEvent::get_name_static(),
        BlockPhysicsEvent::get_name_static(),
        BlockRedstoneEvent::get_name_static(),
        LootGenerateEvent::get_name_static(),
        BlockFormEvent::get_name_static(),
        SpongeAbsorbEvent::get_name_static(),
        CauldronLevelChangeEvent::get_name_static(),
        TNTPrimeEvent::get_name_static(),
        ExplosionPrimeEvent::get_name_static(),
        NotePlayEvent::get_name_static(),
        BellRingEvent::get_name_static(),
    ])
}

fn wind_charge_has_packet_recipients(
    world: &World,
    charge: &WindChargeEntity,
    movement: f64,
) -> bool {
    let entity = charge.get_entity();
    // Hit credit can reach owners in other dimensions. Tracking survives until unpairing.
    let tracker = &world.entity_tracker;
    if charge
        .get_projectile_owner()
        .is_some_and(|owner| owner.get_player(world).is_some())
        || tracker
            .get_tracked_entity(entity.entity_id)
            .is_some_and(|tracked| tracked.has_viewers())
    {
        return true;
    }

    let players = world.players.load();
    // A victim's credited player can still be connected in another dimension.
    if players.is_empty()
        && world
            .server
            .upgrade()
            .is_none_or(|server| !server.has_n_players(1))
    {
        return false;
    }
    if !movement.is_finite() {
        return true;
    }

    let body = entity.bounding_box.load();
    let area = body.expand(
        movement + WIND_CHARGE_TICK_MARGIN,
        movement + WIND_CHARGE_TICK_MARGIN,
        movement + WIND_CHARGE_TICK_MARGIN,
    );
    let bounds = body.expand(
        movement + WIND_CHARGE_PACKET_MARGIN,
        0.0,
        movement + WIND_CHARGE_PACKET_MARGIN,
    );
    let min_chunk = area.min_block_pos().chunk_position();
    let max_chunk = area.max_block_pos().chunk_position();
    if players.iter().any(|player| {
        let position = player.position();
        let watch = player.watched_section.load();
        // This also covers the 64-block explosion broadcast, which doesn't use watchers.
        ((bounds.min.x..=bounds.max.x).contains(&position.x)
            && (bounds.min.z..=bounds.max.z).contains(&position.z))
            || watch.is_within_distance(
                watch.center.x.clamp(min_chunk.x, max_chunk.x),
                watch.center.y.clamp(min_chunk.y, max_chunk.y),
            )
    }) {
        return true;
    }

    let has_java_player = players
        .iter()
        .any(|player| matches!(player.client.as_ref(), ClientPlatform::Java(_)));
    // A hit can reach distant viewers or kill credit. Squid ink is sent to all Java players.
    world.get_entities_at_box(&area).iter().any(|other| {
        let kind = other.get_entity().entity_type;
        other.get_entity().entity_id != entity.entity_id
            && (has_java_player && (kind == &EntityType::SQUID || kind == &EntityType::GLOW_SQUID)
                || tracker
                    .get_tracked_entity(other.get_entity().entity_id)
                    .is_some_and(|tracked| tracked.has_viewers())
                || other.get_living_entity().is_some_and(|living| {
                    living.is_blocking() || living.player_kill_credit_id().is_some()
                }))
    })
}

fn wind_charge_can_reach_portal(world: &World, contact: &BoundingBox) -> bool {
    let min = contact.min_block_pos().0;
    let max = contact.max_block_pos().0;
    let mut x = get_section_cord(min.x)..=get_section_cord(max.x);
    let y = get_section_cord(min.y.max(world.get_bottom_y()))
        ..=get_section_cord(max.y.min(world.get_top_y()));
    let z = get_section_cord(min.z)..=get_section_cord(max.z);
    if y.is_empty() {
        return false;
    }
    if [&x, &y, &z]
        .into_iter()
        .try_fold(1, |count: usize, range| {
            count.checked_mul(cell_count(range)?)
        })
        .is_none_or(|count| count > MAX_CONTACT_SECTIONS as usize)
    {
        return true;
    }
    x.any(|x| {
        z.clone().any(|z| {
            y.clone().any(|y| {
                section_may_contain_block(world, x, y, z, |block| {
                    block == &Block::END_GATEWAY
                        || block == &Block::END_PORTAL
                        || block == &Block::NETHER_PORTAL
                })
                .unwrap_or(false)
            })
        })
    })
}

/// What one entity can read or change next tick.
#[derive(Clone, Debug)]
pub struct TickReach {
    /// Space it can occupy during the tick.
    pub body: BoundingBox,
    /// Touches entities whose body overlaps this.
    pub entities: BoundingBox,
    /// Touches players whose body overlaps this.
    pub players: BoundingBox,
    pub block_reads: BoundingBox,
    pub block_writes: Option<BoundingBox>,
    /// Can act anywhere in the world.
    pub global: bool,
    /// Entities and players touched at any distance.
    pub links: Vec<i32>,
}

impl TickReach {
    /// Anything within `radius` of the box.
    #[must_use]
    pub fn conservative(bounding_box: &BoundingBox, radius: f64, links: Vec<i32>) -> Self {
        let body = bounding_box.expand(radius, radius, radius);
        let block_writes = body.expand(BLOCK_UPDATE_REACH, BLOCK_UPDATE_REACH, BLOCK_UPDATE_REACH);
        Self {
            body,
            entities: body,
            players: body,
            block_reads: block_writes,
            block_writes: Some(block_writes),
            global: false,
            links,
        }
    }
}

/// Reach of an audited entity, as distances from its body.
#[derive(Clone, Debug)]
pub struct AuditedReach {
    pub bounding_box: BoundingBox,
    /// Horizontal and vertical distance it can move by itself.
    pub motion: (f64, f64),
    pub entities: f64,
    pub players: f64,
    pub block_reads: f64,
    /// Area where block changes make it recompute its path.
    pub route: Option<BoundingBox>,
    pub changes_blocks: bool,
    pub global: bool,
    pub links: Vec<i32>,
}

impl AuditedReach {
    /// Flags blocks it can touch that change or teleport on contact.
    pub fn scan_contact_blocks(&mut self, world: &World) {
        let area = self
            .bounding_box
            .expand(
                self.motion.0 + SCAN_SLACK,
                self.motion.1 + SCAN_SLACK,
                self.motion.0 + SCAN_SLACK,
            )
            // Blocks stepped and landed on.
            .expand_towards(0.0, -1.0, 0.0);
        let blocks = (|| {
            let x = cells_of(area.min.x, area.max.x, 1.0)?;
            let y = cells_of(area.min.y, area.max.y, 1.0)?;
            let z = cells_of(area.min.z, area.max.z, 1.0)?;
            cell_count(&x)?
                .checked_mul(cell_count(&y)?)?
                .checked_mul(cell_count(&z)?)
        })();
        if blocks.is_none_or(|blocks| blocks > MAX_CONTACT_BLOCKS) {
            self.global = true;
            return;
        }
        let (min, max) = (area.min_block_pos().0, area.max_block_pos().0);
        for x in get_section_cord(min.x)..=get_section_cord(max.x) {
            for z in get_section_cord(min.z)..=get_section_cord(max.z) {
                for y in get_section_cord(min.y.max(world.get_bottom_y()))
                    ..=get_section_cord(max.y.min(world.get_top_y()))
                {
                    if !contact_section_may_change_blocks(world, x, y, z) {
                        continue;
                    }
                    let start = Vector3::new(x, y, z) * CHUNK_WIDTH as i32;
                    let end = start.add_raw(
                        CHUNK_WIDTH as i32 - 1,
                        CHUNK_WIDTH as i32 - 1,
                        CHUNK_WIDTH as i32 - 1,
                    );
                    for x in min.x.max(start.x)..=max.x.min(end.x) {
                        for y in min.y.max(start.y)..=max.y.min(end.y) {
                            for z in min.z.max(start.z)..=max.z.min(end.z) {
                                let block = world.get_block(&BlockPos::new(x, y, z));
                                if block.id == Block::END_GATEWAY.id {
                                    self.global = true;
                                } else if changes_on_contact(block) {
                                    self.changes_blocks = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Blocks whose contact, step or landing callbacks change blocks or change dimension.
fn changes_on_contact(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_PRESSURE_PLATES)
        || [
            &Block::DETECTOR_RAIL,
            &Block::TRIPWIRE,
            &Block::REDSTONE_ORE,
            &Block::DEEPSLATE_REDSTONE_ORE,
            &Block::BIG_DRIPLEAF,
            &Block::TURTLE_EGG,
            &Block::POWDER_SNOW,
            &Block::SCULK_SHRIEKER,
            &Block::SCULK_SENSOR,
            &Block::CALIBRATED_SCULK_SENSOR,
            &Block::LILY_PAD,
            &Block::FARMLAND,
            &Block::NETHER_PORTAL,
            &Block::END_PORTAL,
        ]
        .iter()
        .any(|candidate| candidate.id == block.id)
}

pub enum EntityReach {
    Audited(AuditedReach),
    Conservative(TickReach),
}

#[derive(Clone, Copy)]
pub(crate) enum BlockWriteCheck {
    Unconditional,
    Contact,
    DetectorRail(BlockPos),
}

impl EntityReach {
    #[must_use]
    pub(crate) fn block_write_check(&self, entity: &dyn EntityBase) -> BlockWriteCheck {
        match self {
            Self::Audited(audited) if !audited.changes_blocks => BlockWriteCheck::Contact,
            Self::Conservative(reach)
                if entity.get_entity().entity_type == &EntityType::CHEST_MINECART
                    && entity
                        .cast_any()
                        .is::<crate::entity::vehicle::minecart::MinecartEntity>()
                    && reach.links.is_empty() =>
            {
                let base = entity.get_entity();
                let world = base.world.load();
                if world.server.upgrade().is_some_and(|server| {
                    server.plugin_manager.has_handlers::<VehicleUpdateEvent>()
                        || server.plugin_manager.has_handlers::<PacketSentEvent>()
                }) {
                    return BlockWriteCheck::Unconditional;
                }
                // Chest carts only activate the rail inspected before movement. Their tick
                // does not run Entity's general contact callbacks.
                BlockWriteCheck::DetectorRail(BlockPos::floored_v(base.pos.load()))
            }
            _ => BlockWriteCheck::Unconditional,
        }
    }

    #[must_use]
    pub fn of(entity: &dyn EntityBase, config: &EntityIslandsConfig) -> Self {
        if let Some(reach) = entity.audited_reach() {
            return Self::Audited(reach);
        }
        let base = entity.get_entity();
        let wind_charge = entity.cast_any().downcast_ref::<WindChargeEntity>();
        let movement = base.velocity.load().length()
            + wind_charge.map_or(0.0, |charge| charge.acceleration_power().abs())
            + entity
                .get_mob()
                .map_or(0.0, crate::entity::mob::Mob::additional_tick_movement);
        let margin = if wind_charge.is_some() {
            config.unaudited_reach.max(WIND_CHARGE_TICK_MARGIN)
        } else {
            config.unaudited_reach
        };
        let radius = movement
            + margin
            + entity.get_living_entity().map_or(0.0, |living| {
                living
                    .get_attribute_value(&Attributes::FOLLOW_RANGE)
                    .max(0.0)
            });
        let mut reach =
            TickReach::conservative(&base.bounding_box.load(), radius, links_of(entity));
        if let Some(mob) = entity.get_mob() {
            let player_radius = radius.max(mob.player_tick_range() + movement);
            reach.players =
                base.bounding_box
                    .load()
                    .expand(player_radius, player_radius, player_radius);
            let block_radius = mob.conservative_block_read_radius() + movement;
            if block_radius > radius + BLOCK_UPDATE_REACH {
                reach.block_reads =
                    base.bounding_box
                        .load()
                        .expand(block_radius, block_radius, block_radius);
            }
        }
        // Blast power and chained block effects can exceed the configured margins.
        reach.global = entity
            .get_mob()
            .is_some_and(crate::entity::mob::Mob::has_global_tick_reach)
            || plugins_observe_shared_brain_tick(entity, movement)
            || wind_charge.is_some_and(|charge| {
                let world = base.world.load();
                // Block contact precedes impact, so a gateway can move the burst to its exit.
                if !movement.is_finite()
                    || base
                        .portal_manager
                        .lock()
                        .map_or(true, |manager| manager.is_some())
                    || wind_charge_can_reach_portal(
                        &world,
                        &base
                            .bounding_box
                            .load()
                            .expand(movement, movement, movement),
                    )
                {
                    return true;
                }
                world.server.upgrade().is_some_and(|server| {
                    plugins_observe_wind_charge_tick(
                        &server.plugin_manager,
                        &world,
                        charge,
                        movement,
                    )
                })
            })
            || entity
                .cast_any()
                .downcast_ref::<TNTEntity>()
                .is_some_and(TNTEntity::is_about_to_explode);
        if !reach.global && (wind_charge.is_some() || base.entity_type == &EntityType::FROG) {
            include_death_teleports(&mut reach, entity, wind_charge.is_some());
        }
        Self::Conservative(reach)
    }
}

fn include_death_teleports(reach: &mut TickReach, source: &dyn EntityBase, wind_charge: bool) {
    let world = source.get_entity().world.load();
    if wind_charge {
        let entities = world.get_entities_at_box(&reach.entities);
        let players = world.players.load();
        for victim in entities.iter().map(Arc::as_ref).chain(
            players
                .iter()
                .map(|player| player.as_ref() as &dyn EntityBase),
        ) {
            include_death_teleport(reach, &world, victim);
            if reach.global {
                return;
            }
        }
    } else if let Some(prey) = source
        .cast_any()
        .downcast_ref::<FrogEntity>()
        .and_then(FrogEntity::get_tongue_target)
    {
        include_death_teleport(reach, &world, prey.as_ref());
    }
}

fn include_death_teleport(reach: &mut TickReach, world: &World, victim: &dyn EntityBase) {
    let Some(living) = victim.get_living_entity() else {
        return;
    };
    let mut radius = None;
    for hand in Hand::all() {
        let held = living.get_stack_in_hand(victim, hand);
        if let Some(protection) = held.get_data_component::<DeathProtectionImpl>() {
            for effect in &*protection.death_effects {
                if let ConsumeEffect::TeleportRandomly(diameter) = effect {
                    radius = Some(radius.unwrap_or(0.0) + f64::from(diameter.abs()) * 0.5);
                }
            }
        }
    }
    let Some(radius) = radius else {
        return;
    };

    let victim_entity = victim.get_entity();
    let movement = victim_entity.velocity.load().length()
        + victim
            .get_mob()
            .map_or(0.0, crate::entity::mob::Mob::additional_tick_movement);
    let radius = radius + movement;
    let destination = victim_entity
        .bounding_box
        .load()
        .expand(radius, 0.0, radius);
    if !radius.is_finite()
        || destination.min.x < reach.body.min.x
        || destination.max.x > reach.body.max.x
        || destination.min.z < reach.body.min.z
        || destination.max.z > reach.body.max.z
    {
        reach.global = true;
        return;
    }

    // The search can follow a destination column down to the world's bottom.
    let min_y = f64::from(world.get_bottom_y()) - BLOCK_UPDATE_REACH;
    let max_y =
        f64::from(world.get_top_y()) + f64::from(victim_entity.height()) + BLOCK_UPDATE_REACH;
    for area in [
        &mut reach.body,
        &mut reach.entities,
        &mut reach.players,
        &mut reach.block_reads,
    ]
    .into_iter()
    .chain(reach.block_writes.as_mut())
    {
        area.min.y = area.min.y.min(min_y);
        area.max.y = area.max.y.max(max_y);
    }
}

/// Vehicle, passengers, leash holder, owner, target and recent attackers.
#[must_use]
pub fn links_of(entity: &dyn EntityBase) -> Vec<i32> {
    let base = entity.get_entity();
    let mut links: Vec<i32> = base
        .passengers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .map(|passenger| passenger.get_entity().entity_id)
        .collect();
    for other in [base.get_vehicle(), base.get_leash_holder()]
        .into_iter()
        .flatten()
    {
        links.push(other.get_entity().entity_id);
    }
    if let Some(DelayedLeash::Entity(uuid)) = *base
        .delayed_leash
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        let world = base.world.load();
        links.extend(
            world
                .get_entity_by_uuid(uuid)
                .map(|holder| holder.get_entity().entity_id)
                .or_else(|| {
                    world
                        .get_player_by_uuid(uuid)
                        .map(|player| player.entity_id())
                }),
        );
    }
    if let Some(owner) = entity.get_owner_entity() {
        links.push(owner.get_entity().entity_id);
    }
    if let Some(fish) = entity.get_mob().and_then(|mob| mob.as_schooling_fish())
        && let Some(leader) = fish.leader()
    {
        links.push(leader.get_entity().entity_id);
    }
    if let Some(mob) = entity.get_mob() {
        links.extend(mob.tick_links());
        if let Some(target) = mob.get_mob_entity().get_target() {
            links.push(target.get_entity().entity_id);
        }
    }
    if let Some(living) = entity.get_living_entity() {
        // Kill credit on death.
        links.extend(living.player_kill_credit_id());
        links.extend(living.get_last_hurt_by_mob_id().filter(|id| *id != 0));
    }
    links
}

/// Turns reports into reaches.
///
/// Audited bodies grow by the pushes they can receive, and by
/// `knockback_reach` within reach of an unaudited entity.
/// Returns `None` when the tick needs serial execution or planning exceeds its budget.
#[must_use]
pub fn resolve(reports: Vec<EntityReach>, config: &EntityIslandsConfig) -> Option<Vec<TickReach>> {
    if reports.iter().any(|report| match report {
        EntityReach::Audited(reach) => reach.global,
        EntityReach::Conservative(reach) => reach.global,
    }) {
        return None;
    }
    let mut budget = PlanningBudget::default();
    budget.spend(reports.len())?;
    let mut pushers = Grid::default();
    let mut threats = Grid::default();
    for report in &reports {
        if let EntityReach::Conservative(reach) = report {
            pushers.insert(pushers.boxes.len(), reach.body, &mut budget)?;
            threats.insert(threats.boxes.len(), reach.entities, &mut budget)?;
        }
    }
    let mut query = Query::new(reports.len());
    let mut reaches = Vec::with_capacity(reports.len());
    for report in reports {
        match report {
            EntityReach::Conservative(reach) => reaches.push(reach),
            EntityReach::Audited(audited) => {
                let mut count = 0;
                pushers.for_each_overlapping(
                    &mut query,
                    &audited.bounding_box,
                    &mut budget,
                    |_| count += 1,
                )?;
                let pushes = PUSH_SPEED * count as f64;
                let probe = grown_body(&audited, pushes + config.knockback_reach.max(PUSH_SLACK));
                let mut threatened = false;
                threats
                    .for_each_overlapping(&mut query, &probe, &mut budget, |_| threatened = true)?;
                let extra = pushes
                    + if threatened {
                        config.knockback_reach
                    } else {
                        0.0
                    };
                let reach = finish(audited, extra);
                // Travel precedes pushing. Later audited turns can only change this body's
                // velocity for the next tick, so they do not grow its current movement bound.
                pushers.insert(pushers.boxes.len(), reach.body, &mut budget)?;
                reaches.push(reach);
            }
        }
    }
    Some(reaches)
}

/// Removes contact-write fallbacks that no writer can make necessary.
pub(crate) fn prune_block_writes(
    world: &World,
    reaches: &mut [TickReach],
    checks: &[BlockWriteCheck],
) -> Option<()> {
    if reaches.iter().any(|reach| reach.global) {
        return Some(());
    }
    let mut budget = PlanningBudget::default();
    budget.spend(reaches.len())?;
    let mut candidates = Vec::new();
    let mut contacts = Grid::default();
    let mut sections = FxHashMap::default();
    let mut active: Vec<bool> = reaches
        .iter()
        .map(|reach| reach.block_writes.is_some())
        .collect();
    for (slot, reach) in reaches.iter().enumerate() {
        if !active[slot] {
            continue;
        }
        let contact = match checks[slot] {
            BlockWriteCheck::Unconditional => continue,
            BlockWriteCheck::Contact => {
                let area = reach.body.expand_towards(0.0, -1.0, 0.0);
                let contact = BoundingBox::new(
                    area.min_block_pos().0.to_f64(),
                    area.max_block_pos().0.to_f64() + Vector3::new(1.0, 1.0, 1.0),
                );
                if contact_may_change_blocks(world, &contact, &mut sections) {
                    continue;
                }
                contact
            }
            BlockWriteCheck::DetectorRail(pos) => {
                if !world.level.is_chunk_loaded(&pos.chunk_position())
                    || [pos, pos.down()]
                        .iter()
                        .any(|pos| world.get_block(pos).id == Block::DETECTOR_RAIL.id)
                {
                    continue;
                }
                BoundingBox::from_block(&pos).expand_towards(0.0, -1.0, 0.0)
            }
        };
        active[slot] = false;
        contacts.insert(candidates.len(), contact, &mut budget)?;
        candidates.push(slot);
    }
    if candidates.is_empty() {
        return Some(());
    }
    let mut pending: VecDeque<_> = active
        .iter()
        .enumerate()
        .filter_map(|(slot, active)| active.then_some(slot))
        .collect();
    let mut query = Query::new(candidates.len());
    while let Some(slot) = pending.pop_front() {
        let Some(writes) = reaches[slot].block_writes else {
            continue;
        };
        contacts.for_each_overlapping(&mut query, &writes, &mut budget, |candidate| {
            let slot = candidates[candidate];
            if !active[slot] {
                active[slot] = true;
                pending.push_back(slot);
            }
        })?;
    }
    for slot in candidates {
        if !active[slot] {
            reaches[slot].block_writes = None;
        }
    }
    Some(())
}

fn contact_may_change_blocks(
    world: &World,
    contact: &BoundingBox,
    sections: &mut FxHashMap<(i32, i32, i32), bool>,
) -> bool {
    if [
        contact.min.x,
        contact.min.y,
        contact.min.z,
        contact.max.x,
        contact.max.y,
        contact.max.z,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return true;
    }
    let bottom = world.dimension.min_y;
    let top = bottom + world.dimension.height;
    let min_y = (contact.min.y as i32).max(bottom);
    let max_y = ((contact.max.y - 1.0) as i32).min(top - 1);
    if min_y > max_y {
        return false;
    }
    let min = [
        get_section_cord(contact.min.x as i32),
        get_section_cord(min_y),
        get_section_cord(contact.min.z as i32),
    ];
    let max = [
        get_section_cord((contact.max.x - 1.0) as i32),
        get_section_cord(max_y),
        get_section_cord((contact.max.z - 1.0) as i32),
    ];
    let volume = (0..3).fold(1i64, |volume, axis| {
        volume.saturating_mul(i64::from(max[axis]) - i64::from(min[axis]) + 1)
    });
    if volume > MAX_CONTACT_SECTIONS {
        return true;
    }
    for x in min[0]..=max[0] {
        for z in min[2]..=max[2] {
            for y in min[1]..=max[1] {
                if *sections
                    .entry((x, y, z))
                    .or_insert_with(|| contact_section_may_change_blocks(world, x, y, z))
                {
                    return true;
                }
            }
        }
    }
    false
}

fn contact_section_may_change_blocks(world: &World, x: i32, y: i32, z: i32) -> bool {
    section_may_contain_block(world, x, y, z, |block| {
        block.id == Block::END_GATEWAY.id || changes_on_contact(block)
    })
    .unwrap_or(true)
}

fn section_may_contain_block(
    world: &World,
    x: i32,
    y: i32,
    z: i32,
    predicate: impl Fn(&Block) -> bool,
) -> Option<bool> {
    world.level.read_chunk_sync(&Vector2::new(x, z), |chunk| {
        let blocks = chunk
            .section
            .block_sections
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = y - get_section_cord(chunk.section.min_y);
        usize::try_from(index)
            .ok()
            .and_then(|index| blocks.get(index))
            .is_none_or(|palette| {
                palette.contains_matching(|state| predicate(Block::from_state_id(state)))
            })
    })
}

fn grown_body(audited: &AuditedReach, extra: f64) -> BoundingBox {
    let (horizontal, vertical) = audited.motion;
    audited
        .bounding_box
        .expand(horizontal + extra, vertical + extra, horizontal + extra)
}

fn finish(audited: AuditedReach, extra: f64) -> TickReach {
    let body = grown_body(&audited, extra);
    let around = |distance: f64| body.expand(distance, distance, distance);
    // Player pickup range in `World::tick`.
    let players = body.expand(
        audited.players.max(1.0),
        audited.players.max(0.5),
        audited.players.max(1.0),
    );
    let changes_blocks = audited.changes_blocks || extra > SCAN_SLACK;
    let block_writes = changes_blocks.then(|| around(BLOCK_UPDATE_REACH));
    let mut block_reads = around(if changes_blocks {
        audited.block_reads.max(BLOCK_UPDATE_REACH)
    } else {
        audited.block_reads
    });
    if let Some(route) = audited.route {
        block_reads = union(&block_reads, &route);
    }
    // Searches follow water columns down to the bottom of the world.
    block_reads.min.y = f64::NEG_INFINITY;
    block_reads.max.y = f64::INFINITY;
    TickReach {
        body,
        entities: around(audited.entities),
        players,
        block_reads,
        block_writes,
        global: audited.global,
        links: audited.links,
    }
}

const fn union(a: &BoundingBox, b: &BoundingBox) -> BoundingBox {
    BoundingBox::new(
        pumpkin_util::math::vector3::Vector3::new(
            a.min.x.min(b.min.x),
            a.min.y.min(b.min.y),
            a.min.z.min(b.min.z),
        ),
        pumpkin_util::math::vector3::Vector3::new(
            a.max.x.max(b.max.x),
            a.max.y.max(b.max.y),
            a.max.z.max(b.max.z),
        ),
    )
}

pub struct Islands {
    /// Node indices in list order.
    pub islands: Vec<Vec<usize>>,
    #[cfg(debug_assertions)]
    pub island_of: Arc<IslandMap>,
}

/// Island of each entity and player.
pub type IslandMap = FxHashMap<i32, usize>;

/// Splits `nodes` (ticking entities in list order) into islands. Idle entities and players only
/// connect islands.
/// Returns `None` when the tick needs serial execution or planning exceeds its budget.
#[must_use]
pub fn partition(
    nodes: &[(i32, TickReach)],
    idle_entities: &[(i32, BoundingBox)],
    players: &[(i32, BoundingBox)],
) -> Option<Islands> {
    if nodes.iter().any(|(_, reach)| reach.global) {
        return None;
    }
    let mut budget = PlanningBudget::default();
    budget.spend(nodes.len())?;
    budget.spend(idle_entities.len())?;
    budget.spend(players.len())?;

    let mut sets = DisjointSets::new(nodes.len() + idle_entities.len() + players.len());
    let mut slots: FxHashMap<i32, usize> = FxHashMap::default();
    let bodies = nodes
        .iter()
        .map(|(id, reach)| (*id, reach.body))
        .chain(idle_entities.iter().copied());
    let mut entity_grid = Grid::default();
    for (slot, (id, body)) in bodies.enumerate() {
        slots.insert(id, slot);
        entity_grid.insert(slot, body, &mut budget)?;
    }
    let first_player = nodes.len() + idle_entities.len();
    for (offset, (id, _)) in players.iter().enumerate() {
        slots.insert(*id, first_player + offset);
    }

    let mut query = Query::new(sets.len());
    for (slot, (_, reach)) in nodes.iter().enumerate() {
        entity_grid.union_overlapping(&mut sets, &mut query, slot, &reach.entities, &mut budget)?;
        budget.spend(players.len())?;
        for (offset, (_, body)) in players.iter().enumerate() {
            if overlaps(&reach.players, body) {
                sets.union(slot, first_player + offset);
            }
        }
        budget.spend(reach.links.len())?;
        for link in &reach.links {
            let other = *slots.entry(*link).or_insert_with(|| sets.push());
            sets.union(slot, other);
        }
    }

    let writers: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter_map(|(slot, (_, reach))| reach.block_writes.map(|writes| (slot, writes)))
        .collect();
    if !writers.is_empty() {
        let mut block_grid = Grid::default();
        for (slot, (_, reach)) in nodes.iter().enumerate() {
            block_grid.insert(slot, reach.block_reads, &mut budget)?;
        }
        let mut query = Query::new(sets.len());
        for (slot, writes) in writers {
            block_grid.union_overlapping(&mut sets, &mut query, slot, &writes, &mut budget)?;
        }
    }

    let mut island_of_root: FxHashMap<usize, usize> = FxHashMap::default();
    let mut islands: Vec<Vec<usize>> = Vec::new();
    for slot in 0..nodes.len() {
        let root = sets.find(slot);
        let island = *island_of_root.entry(root).or_insert_with(|| {
            islands.push(Vec::new());
            islands.len() - 1
        });
        islands[island].push(slot);
    }
    #[cfg(debug_assertions)]
    let island_of = slots
        .into_iter()
        .filter_map(|(id, slot)| {
            island_of_root
                .get(&sets.find(slot))
                .map(|island| (id, *island))
        })
        .collect();
    Some(Islands {
        islands,
        #[cfg(debug_assertions)]
        island_of: Arc::new(island_of),
    })
}

#[cfg(debug_assertions)]
thread_local! {
    static TICKING: std::cell::RefCell<Option<(i32, Arc<IslandMap>)>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(debug_assertions)]
static REPORTED_ESCAPES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Marks `entity_id` as ticking on this thread until [`leave`].
#[cfg(debug_assertions)]
pub fn enter(islands: &Islands, entity_id: i32) {
    TICKING.set(Some((entity_id, islands.island_of.clone())));
}

#[cfg(debug_assertions)]
pub fn leave() {
    TICKING.set(None);
}

/// Warns in debug builds when the ticking entity reaches `entity_id` in another island.
#[cfg_attr(
    not(debug_assertions),
    expect(
        clippy::missing_const_for_fn,
        reason = "The debug implementation accesses thread-local state"
    )
)]
pub fn check_reached(entity_id: i32) {
    #[cfg(debug_assertions)]
    {
        let escaped = TICKING.with_borrow(|ticking| {
            ticking.as_ref().and_then(|(ticking, island_of)| {
                let own = island_of.get(ticking)?;
                let other = island_of.get(&entity_id)?;
                (own != other).then_some(*ticking)
            })
        });
        if let Some(ticking) = escaped
            && REPORTED_ESCAPES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 20
        {
            tracing::warn!(
                "Entity {ticking} reached entity {entity_id} outside its tick island; its tick reach is too small"
            );
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = entity_id;
}

/// Touching faces count as overlapping.
fn overlaps(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min.x <= b.max.x
        && a.max.x >= b.min.x
        && a.min.y <= b.max.y
        && a.max.y >= b.min.y
        && a.min.z <= b.max.z
        && a.max.z >= b.min.z
}

fn cells_of(min: f64, max: f64, size: f64) -> Option<std::ops::RangeInclusive<i32>> {
    if !min.is_finite() || !max.is_finite() || min > max {
        return None;
    }
    Some(((min / size).floor() as i32)..=((max / size).floor() as i32))
}

fn cell_count(range: &std::ops::RangeInclusive<i32>) -> Option<usize> {
    usize::try_from(i64::from(*range.end()) - i64::from(*range.start()) + 1).ok()
}

struct Cell {
    members: Vec<usize>,
    lowest_top: f64,
    highest_bottom: f64,
    /// Every member is already in one set.
    merged: bool,
}

/// Boxes bucketed by column. Slots are inserted in order from zero.
#[derive(Default)]
struct Grid {
    refined: bool,
    small_cells: FxHashMap<(i32, i32), Cell>,
    cells: FxHashMap<(i32, i32), Cell>,
    boxes: Vec<BoundingBox>,
}

impl Grid {
    fn insert(
        &mut self,
        slot: usize,
        area: BoundingBox,
        budget: &mut PlanningBudget,
    ) -> Option<()> {
        debug_assert_eq!(slot, self.boxes.len());
        self.insert_cells(slot, area, budget)?;
        self.boxes.push(area);
        if !self.refined
            && !self.cells.is_empty()
            && self.boxes.len() / MIN_COLUMN_DENSITY >= self.cells.len()
        {
            // Sparse grids retain the cheaper coarse-only query path.
            self.refined = true;
            self.cells.clear();
            for slot in 0..self.boxes.len() {
                self.insert_cells(slot, self.boxes[slot], budget)?;
            }
        }
        Some(())
    }

    fn insert_cells(
        &mut self,
        slot: usize,
        area: BoundingBox,
        budget: &mut PlanningBudget,
    ) -> Option<()> {
        let (cells, size) = if self.refined
            && area.max.x - area.min.x <= SMALL_CELL_SIZE
            && area.max.z - area.min.z <= SMALL_CELL_SIZE
        {
            (&mut self.small_cells, SMALL_CELL_SIZE)
        } else {
            (&mut self.cells, CELL_SIZE)
        };
        let [xs, zs] = budget.cells(&area, size)?;
        for x in xs {
            for z in zs.clone() {
                let cell = cells.entry((x, z)).or_insert_with(|| Cell {
                    members: Vec::new(),
                    lowest_top: f64::INFINITY,
                    highest_bottom: f64::NEG_INFINITY,
                    merged: false,
                });
                cell.members.push(slot);
                cell.lowest_top = cell.lowest_top.min(area.max.y);
                cell.highest_bottom = cell.highest_bottom.max(area.min.y);
            }
        }
        Some(())
    }

    fn for_each_overlapping(
        &self,
        query: &mut Query,
        area: &BoundingBox,
        budget: &mut PlanningBudget,
        mut visit: impl FnMut(usize),
    ) -> Option<()> {
        query.next();
        for (cells, size) in [
            (&self.small_cells, SMALL_CELL_SIZE),
            (&self.cells, CELL_SIZE),
        ] {
            if cells.is_empty() {
                continue;
            }
            let [xs, zs] = budget.cells(area, size)?;
            for x in xs {
                for z in zs.clone() {
                    if let Some(cell) = cells.get(&(x, z)) {
                        budget.spend(cell.members.len())?;
                        for &member in &cell.members {
                            if query.visit(member) && overlaps(area, &self.boxes[member]) {
                                visit(member);
                            }
                        }
                    }
                }
            }
        }
        Some(())
    }

    fn union_overlapping(
        &mut self,
        sets: &mut DisjointSets,
        query: &mut Query,
        slot: usize,
        area: &BoundingBox,
        budget: &mut PlanningBudget,
    ) -> Option<()> {
        query.next();
        for (cells, size) in [
            (&mut self.small_cells, SMALL_CELL_SIZE),
            (&mut self.cells, CELL_SIZE),
        ] {
            if cells.is_empty() {
                continue;
            }
            let [xs, zs] = budget.cells(area, size)?;
            for x in xs {
                for z in zs.clone() {
                    let Some(cell) = cells.get_mut(&(x, z)) else {
                        continue;
                    };
                    let (min_x, min_z) = (f64::from(x) * size, f64::from(z) * size);
                    // `area` spans the column and every member's height, so all members overlap it.
                    let covered = area.min.x <= min_x
                        && area.max.x >= min_x + size
                        && area.min.z <= min_z
                        && area.max.z >= min_z + size
                        && area.min.y <= cell.lowest_top
                        && area.max.y >= cell.highest_bottom;
                    if covered {
                        if cell.merged {
                            budget.spend(1)?;
                            sets.union(slot, cell.members[0]);
                        } else {
                            budget.spend(cell.members.len())?;
                            for &member in &cell.members {
                                sets.union(slot, member);
                            }
                            cell.merged = true;
                        }
                        continue;
                    }
                    budget.spend(cell.members.len())?;
                    for &member in &cell.members {
                        if query.visit(member) && overlaps(area, &self.boxes[member]) {
                            sets.union(slot, member);
                        }
                    }
                }
            }
        }
        Some(())
    }
}

/// Visits each member once per query.
struct Query {
    stamps: Vec<u32>,
    current: u32,
}

impl Query {
    fn new(len: usize) -> Self {
        Self {
            stamps: vec![0; len],
            current: 0,
        }
    }

    const fn next(&mut self) {
        self.current += 1;
    }

    fn visit(&mut self, member: usize) -> bool {
        let stamp = &mut self.stamps[member];
        let first = *stamp != self.current;
        *stamp = self.current;
        first
    }
}

struct DisjointSets {
    parent: Vec<usize>,
}

impl DisjointSets {
    fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
        }
    }

    const fn len(&self) -> usize {
        self.parent.len()
    }

    fn push(&mut self) -> usize {
        self.parent.push(self.parent.len());
        self.parent.len() - 1
    }

    fn find(&mut self, mut index: usize) -> usize {
        while self.parent[index] != index {
            self.parent[index] = self.parent[self.parent[index]];
            index = self.parent[index];
        }
        index
    }

    fn union(&mut self, a: usize, b: usize) {
        let a = self.find(a);
        let b = self.find(b);
        if a != b {
            // Keep the lower index as root so it does not depend on union order.
            self.parent[a.max(b)] = a.min(b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity::{
            Entity,
            ai::test_support,
            mob::{Mob, slime::SlimeEntity},
            passive::frog::{FrogEntity, prepare_test_tongue_bite},
            r#type::from_type,
        },
        plugin::{EventHandler, EventPriority, player::PlayerChatEvent},
    };
    use futures::future::BoxFuture;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::vector3::Vector3;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    };

    struct ObserveBite {
        prey: i32,
        fired: AtomicBool,
    }

    impl EventHandler<EntityDamageEvent> for ObserveBite {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityDamageEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_id == self.prey
                    && event.damage_type == pumpkin_data::damage::DamageType::MOB_ATTACK
                {
                    self.fired.store(true, Relaxed);
                    event.cancelled = true;
                }
            })
        }
    }

    struct IgnoreEvent;
    impl<E: Payload> EventHandler<E> for IgnoreEvent {}

    fn force_separated_chunk(world: &World, index: usize) -> Vector2<i32> {
        let chunk = Vector2::new(index as i32 * 32, 0);
        world.level.loaded_chunks.insert(
            chunk,
            pumpkin_world::chunk::ChunkData::empty_sync(chunk.x, chunk.y),
        );
        world
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(chunk);
        chunk
    }

    fn prepare_separated_islands(world: &Arc<World>) {
        for index in 0..MIN_PARALLEL_ENTITIES {
            let chunk = force_separated_chunk(world, index);
            if index > 0 {
                world.add_entity_silent(from_type(
                    &EntityType::ARMOR_STAND,
                    Vector3::new(f64::from(chunk.x * 16) + 0.5, 100.0, 0.5),
                    world,
                    uuid::Uuid::new_v4(),
                ));
            }
        }
    }

    fn separated_frog_and_slime(world: &Arc<World>) -> (Arc<FrogEntity>, Arc<SlimeEntity>) {
        prepare_separated_islands(world);
        let frog = FrogEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 100.0, 2.5),
            &EntityType::FROG,
        ));
        let slime = SlimeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.5, 100.0, 2.5),
            &EntityType::SLIME,
        ));
        slime.set_size(1, true);
        slime.get_mob_entity().set_no_ai(true);
        world.add_entity_silent(frog.clone());
        world.add_entity_silent(slime.clone());
        (frog, slime)
    }

    fn entity_schedule(world: &World) -> (usize, bool) {
        let schedule = world
            .island_schedule
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (schedule.entity_count, schedule.ticks_until_probe > 0)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_frog_bites_in_serial_if_a_plugin_observes_damage()
    -> Result<(), Box<dyn std::error::Error>> {
        for observe_damage in [false, true] {
            let folder = tempfile::tempdir()?;
            let server = test_support::server(&folder).await?;
            let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
            let (frog, prey) = separated_frog_and_slime(&world);

            let observer = Arc::new(ObserveBite {
                prey: prey.get_entity().entity_id,
                fired: AtomicBool::new(false),
            });
            if observe_damage {
                server.plugin_manager.register::<EntityDamageEvent, _>(
                    observer.clone(),
                    EventPriority::Normal,
                    true,
                );
            } else {
                server.plugin_manager.register::<PlayerChatEvent, _>(
                    Arc::new(IgnoreEvent),
                    EventPriority::Normal,
                    true,
                );
            }
            world.tick(&server);
            let ticking_entities = MIN_PARALLEL_ENTITIES + 1;
            assert_eq!(
                entity_schedule(&world),
                (ticking_entities, false),
                "idle frog"
            );
            prepare_test_tongue_bite(&frog, prey);
            world.tick(&server);

            assert_eq!(
                observer.fired.load(Relaxed),
                observe_damage,
                "bite dispatcher"
            );
            assert_eq!(
                world.get_entity_by_id(observer.prey).is_some(),
                observe_damage,
                "the prey should survive only when the handler cancels the bite"
            );
            assert_eq!(
                entity_schedule(&world),
                (ticking_entities, observe_damage),
                "planner mode on the bite tick"
            );
            server.stop();
            server.shutdown().await;
        }
        Ok(())
    }

    #[tokio::test]
    async fn a_frog_and_its_distant_tongue_target_share_an_island()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let frog = FrogEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 100.0, 2.5),
            &EntityType::FROG,
        ));
        let prey = from_type(
            &EntityType::SLIME,
            Vector3::new(512.5, 100.0, 2.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        let separate = from_type(
            &EntityType::ARMOR_STAND,
            Vector3::new(1024.5, 100.0, 2.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        frog.set_tongue_target(prey.as_ref());
        assert!(frog.get_mob_entity().stored_target().is_none());
        world.add_entity_silent(frog.clone());
        world.add_entity_silent(prey.clone());

        let actors = [frog as Arc<dyn EntityBase>, prey, separate];
        let config = EntityIslandsConfig::default();
        let reports = actors
            .iter()
            .map(|actor| EntityReach::of(actor.as_ref(), &config))
            .collect();
        let reaches =
            resolve(reports, &config).ok_or("the reach reports should permit parallel islands")?;
        let nodes: Vec<_> = actors
            .iter()
            .map(|actor| actor.get_entity().entity_id)
            .zip(reaches)
            .collect();
        let islands = partition(&nodes, &[], &[]).ok_or("the planner should return islands")?;
        assert_eq!(islands.islands, vec![vec![0, 1], vec![2]]);
        world.level.shutdown().await;
        Ok(())
    }

    const CHEST_LOOT: &str = "minecraft:chests/simple_dungeon";

    struct ObserveChestLoot(AtomicBool);

    impl EventHandler<LootGenerateEvent> for ObserveChestLoot {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut LootGenerateEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.loot_table == CHEST_LOOT {
                    self.0.store(true, Relaxed);
                    event.cancelled = true;
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_frog_bite_serializes_a_wind_burst_that_unpacks_chest_loot()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::block::entities::{
            BlockEntity, chest::ChestBlockEntity, comparator::ComparatorBlockEntity,
        };
        use crate::world::BlockFlags;
        use pumpkin_data::{
            block_properties::{
                AttachFace, ComparatorLikeProperties, HorizontalFacing, LeverLikeProperties,
            },
            potion::Potion,
        };
        use pumpkin_nbt::compound::NbtCompound;

        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (frog, prey) = separated_frog_and_slime(&world);
        world.tick(&server);
        assert_eq!(entity_schedule(&world), (MIN_PARALLEL_ENTITIES + 1, false));

        // Suppress neighbour updates during setup so the comparator reads the chest on the bite tick.
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        let lever = BlockPos::new(3, 100, 2);
        let comparator = BlockPos::new(4, 100, 2);
        let chest_pos = BlockPos::new(5, 100, 2);
        for pos in [lever, comparator, chest_pos] {
            world.set_block_state(&pos.down(), Block::STONE.default_state.id, flags);
        }
        let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
        lever_props.face = AttachFace::Floor;
        lever_props.powered = false;
        world.set_block_state(&lever, lever_props.to_state_id(&Block::LEVER), flags);
        let mut comparator_props = ComparatorLikeProperties::default(&Block::COMPARATOR);
        comparator_props.facing = HorizontalFacing::East;
        world.set_block_state(
            &comparator,
            comparator_props.to_state_id(&Block::COMPARATOR),
            flags,
        );
        world.add_block_entity(Arc::new(ComparatorBlockEntity::new(comparator)));
        world.set_block_state(&chest_pos, Block::CHEST.default_state.id, flags);
        let mut chest_nbt = NbtCompound::new();
        chest_nbt.put_string("LootTable", CHEST_LOOT.to_owned());
        let chest = Arc::new(ChestBlockEntity::from_nbt(&chest_nbt, chest_pos));
        world.add_block_entity(chest.clone());
        assert!(!world.is_block_tick_scheduled(&comparator, &Block::COMPARATOR));
        assert_eq!(chest.loot_table_key().as_deref(), Some(CHEST_LOOT));

        for effect in Potion::WIND_CHARGED.effects {
            prey.get_mob_entity()
                .living_entity
                .add_effect(effect.clone());
        }
        let observer = Arc::new(ObserveChestLoot(AtomicBool::new(false)));
        server.plugin_manager.register::<LootGenerateEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        prepare_test_tongue_bite(&frog, prey);
        world.tick(&server);

        let lever_state = world.get_block_state(&lever);
        assert!(LeverLikeProperties::from_state_id(lever_state.id).powered);
        assert!(
            observer.0.load(Relaxed),
            "the comparator must unpack loot during the bite tick"
        );
        assert!(chest.loot_table_key().is_none());
        assert_eq!(entity_schedule(&world), (MIN_PARALLEL_ENTITIES + 1, true));
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    impl EventHandler<EntityRegainHealthEvent> for ObserveBite {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityRegainHealthEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_id == self.prey && event.amount > 0.0 {
                    self.fired.store(true, Relaxed);
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_frog_bite_serializes_healing_from_death_protection()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::borrow::Cow;

        use pumpkin_data::{
            data_component_impl::{
                ConsumeEffect, DeathProtectionImpl, EquipmentSlot, StatusEffectInstance,
            },
            item::Item,
            item_stack::ItemStack,
        };

        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (frog, prey) = separated_frog_and_slime(&world);
        let living = &prey.get_mob_entity().living_entity;
        let effect = StatusEffectInstance {
            effect_id: Cow::Borrowed("minecraft:instant_health"),
            amplifier: 0,
            duration: 1,
            ambient: false,
            show_particles: true,
            show_icon: true,
        };
        let mut protection = ItemStack::new(1, &Item::TOTEM_OF_UNDYING);
        protection.set_data_component(DeathProtectionImpl {
            death_effects: Cow::Owned(vec![ConsumeEffect::ApplyEffects((
                Cow::Owned(vec![effect]),
                1.0,
            ))]),
        });
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .equipment
            .insert(EquipmentSlot::MAIN_HAND, protection);
        let observer = Arc::new(ObserveBite {
            prey: prey.get_entity().entity_id,
            fired: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register::<EntityRegainHealthEvent, _>(observer.clone(), EventPriority::Normal, true);

        world.tick(&server);
        assert_eq!(entity_schedule(&world), (MIN_PARALLEL_ENTITIES + 1, false));
        assert!(!living.held_item_from_equipment().is_empty());
        assert!(!observer.fired.load(Relaxed));

        prepare_test_tongue_bite(&frog, prey.clone());
        world.tick(&server);

        assert!(
            living.held_item_from_equipment().is_empty(),
            "the bite must consume death protection"
        );
        assert!(prey.get_entity().is_alive());
        assert!(
            observer.fired.load(Relaxed),
            "Instant Health must dispatch the callback"
        );
        assert_eq!(entity_schedule(&world), (MIN_PARALLEL_ENTITIES + 1, true));
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    fn spawn_teleport_test_player(
        world: &Arc<World>,
        server: &Arc<Server>,
        arrives_after_planning: bool,
    ) -> Result<Arc<crate::entity::player::Player>, Box<dyn std::error::Error>> {
        use crate::entity::player::Player;
        use crate::net::{
            GameProfile, PacketRateLimiter, PlayerConfig,
            java::{JavaClient, pending::PendingConnection},
        };
        use pumpkin_util::GameMode;

        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "Notch".to_owned(),
            properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let config = PlayerConfig::default();
        let (read, write) = tokio::io::split(tokio::io::duplex(1024).0);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            "127.0.0.1:0".parse()?,
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(server),
        );
        if arrives_after_planning {
            pending
                .version
                .store(pumpkin_util::version::JavaMinecraftVersion::V_1_21_2);
        }
        let client = Arc::new(ClientPlatform::Java(JavaClient::from_pending(
            pending,
            profile.clone(),
            config.clone(),
        )));
        let player = Arc::new(Player::new(
            client,
            profile,
            config,
            world,
            GameMode::Survival,
        ));
        player.get_entity().set_pos(Vector3::new(4.5, 100.25, 2.5));
        player.set_client_loaded(!arrives_after_planning);
        Ok(player)
    }

    struct InsertPlayerDuringTick {
        entity: Entity,
        player: Arc<crate::entity::player::Player>,
    }

    impl EntityBase for InsertPlayerDuringTick {
        fn get_entity(&self) -> &Entity {
            &self.entity
        }

        fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
            None
        }

        fn cast_any(&self) -> &dyn std::any::Any {
            self
        }

        fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
            assert!(self.entity.world.load().add_player(&self.player).is_ok());
        }
    }

    struct ObserveDeathProtectionTeleport(AtomicBool);

    impl EventHandler<PlayerTeleportEvent> for ObserveDeathProtectionTeleport {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerTeleportEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.0.store(true, Relaxed);
                event.cancelled = true;
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_wind_charge_serializes_death_protection_that_teleports_a_player()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::borrow::Cow;

        use crate::world::BlockFlags;
        use pumpkin_data::{
            data_component_impl::{ConsumeEffect, DeathProtectionImpl},
            item::Item,
            item_stack::ItemStack,
        };
        use pumpkin_util::Hand;

        for (observe_teleport, arrives_after_planning) in
            [(false, false), (true, false), (true, true)]
        {
            let folder = tempfile::tempdir()?;
            let server = test_support::server(&folder).await?;
            let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
            prepare_separated_islands(&world);
            for x in 3..=5 {
                for z in 1..=3 {
                    world.set_block_state(
                        &BlockPos::new(x, 99, z),
                        Block::STONE.default_state.id,
                        BlockFlags::UPDATE_KNOWN_SHAPE,
                    );
                }
            }
            let player = spawn_teleport_test_player(&world, &server, arrives_after_planning)?;
            if arrives_after_planning {
                assert!(world.players.load().is_empty());
                assert!(
                    player.has_client_loaded(),
                    "the legacy version ignores the transfer loading flag"
                );
                world.add_entity_silent(Arc::new(InsertPlayerDuringTick {
                    entity: Entity::new(
                        world.clone(),
                        Vector3::new(5.5, 100.0, 4.5),
                        &EntityType::ARMOR_STAND,
                    ),
                    player: player.clone(),
                }));
            } else {
                world.add_player(&player)?;
            }
            player.set_health(1.0);
            let mut protection = ItemStack::new(1, &Item::TOTEM_OF_UNDYING);
            protection.set_data_component(DeathProtectionImpl {
                death_effects: Cow::Borrowed(&[ConsumeEffect::TeleportRandomly(0.25)]),
            });
            player.inventory.set_stack_in_hand(Hand::Right, protection);
            let observer = Arc::new(ObserveDeathProtectionTeleport(AtomicBool::new(false)));
            if observe_teleport {
                server.plugin_manager.register::<PlayerTeleportEvent, _>(
                    observer.clone(),
                    EventPriority::Normal,
                    true,
                );
            } else {
                server.plugin_manager.register::<PlayerChatEvent, _>(
                    Arc::new(IgnoreEvent),
                    EventPriority::Normal,
                    true,
                );
            }
            let charge = from_type(
                &EntityType::WIND_CHARGE,
                Vector3::new(2.5, 100.75, 2.5),
                &world,
                uuid::Uuid::new_v4(),
            );
            charge
                .get_entity()
                .set_velocity(Vector3::new(3.0, 0.0, 0.0));
            world.add_entity_silent(charge.clone());

            world.tick(&server);

            assert!(charge.get_entity().is_removed(), "the charge must hit");
            assert!(player.inventory.get_stack_in_hand(Hand::Right).is_empty());
            assert_eq!(
                observer.0.load(Relaxed),
                observe_teleport,
                "death protection must reach the teleport dispatcher"
            );
            assert_eq!(
                entity_schedule(&world),
                (
                    MIN_PARALLEL_ENTITIES + usize::from(arrives_after_planning),
                    observe_teleport
                )
            );
            server.stop();
            server.shutdown().await;
        }
        Ok(())
    }

    struct ObserveSquidInk(AtomicBool);

    impl EventHandler<PacketSentEvent> for ObserveSquidInk {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            use pumpkin_data::{packet::CURRENT_MC_VERSION, particle::Particle};
            use pumpkin_protocol::{
                MultiVersionJavaPacket, java::client::play::CParticle, ser::NetworkReadExt,
            };

            Box::pin(async move {
                if event.packet_id == CParticle::to_id(CURRENT_MC_VERSION)
                    // Java 26.3 prefixes the particle ID before the coordinates.
                    && event.payload.as_ref().get_var_int()
                        .is_ok_and(|id| id.0 == Particle::SquidInk as i32)
                {
                    self.0.store(true, Relaxed);
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn breeze_charge_ink_serializes_for_distant_java_packet_observers()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(check_breeze_charge_ink(false).await?);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn breeze_charge_ink_serializes_before_a_java_player_arrives()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(
            check_breeze_charge_ink(true).await?,
            "the particle callback for the late arrival ran in parallel"
        );
        Ok(())
    }

    async fn check_breeze_charge_ink(
        arrives_after_planning: bool,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use pumpkin_protocol::ConnectionState;
        use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
        use std::num::NonZero;

        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        prepare_separated_islands(&world);

        let player = spawn_teleport_test_player(&world, &server, false)?;
        player
            .get_entity()
            .set_pos(Vector3::new(16384.5, 100.0, 2.5));
        player
            .watched_section
            .store(Cylindrical::new(Vector2::new(1024, 0), NonZero::<u8>::MIN));
        let ClientPlatform::Java(client) = player.client.as_ref() else {
            return Err("expected a Java client".into());
        };
        client.connection_state.store(ConnectionState::Play);
        client.set_player(player.clone());
        if arrives_after_planning {
            assert!(world.players.load().is_empty());
            player.set_client_loaded(false);
            assert!(!player.has_client_loaded());
            world.add_entity_silent(Arc::new(InsertPlayerDuringTick {
                entity: Entity::new(
                    world.clone(),
                    Vector3::new(5.5, 100.0, 4.5),
                    &EntityType::ARMOR_STAND,
                ),
                player: player.clone(),
            }));
        } else {
            world.add_player(&player)?;
        }

        let observer = Arc::new(ObserveSquidInk(AtomicBool::new(false)));
        server.plugin_manager.register::<PacketSentEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let prey = from_type(
            &EntityType::SQUID,
            Vector3::new(4.5, 100.0, 2.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        let charge = from_type(
            &EntityType::BREEZE_WIND_CHARGE,
            Vector3::new(2.5, 100.6, 2.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        world.add_entity_silent(charge.clone());
        let owner = from_type(
            &EntityType::BREEZE,
            // Keep the owner resolvable for ink, but outside every ticking chunk.
            Vector3::new(-1023.5, 100.0, 0.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        charge
            .get_projectile_owner()
            .ok_or("expected a charge owner record")?
            .set_uuid(Some(owner.get_entity().entity_uuid));
        world.add_entity_silent(owner.clone());
        charge
            .get_entity()
            .set_velocity(Vector3::new(3.0, 0.0, 0.0));
        world.add_entity_silent(prey);

        world.tick(&server);

        client.clear_player();
        let (ticking, serial) = entity_schedule(&world);
        let saw_ink = observer.0.load(Relaxed);
        server.stop();
        server.shutdown().await;
        assert_eq!(owner.get_entity().tick_count.load(Relaxed), 0);
        assert!(charge.get_entity().is_removed(), "the charge must hit");
        assert!(
            saw_ink,
            "the distant Java player must receive the squid ink"
        );
        assert_eq!(
            ticking,
            MIN_PARALLEL_ENTITIES + 1 + usize::from(arrives_after_planning),
        );
        Ok(serial)
    }

    #[derive(Clone, Copy)]
    enum BreezePacket {
        Burst,
        Flight,
        ChargeSound,
    }

    struct ObserveBreezePacket {
        kind: BreezePacket,
        sent: AtomicBool,
    }

    impl EventHandler<PacketSentEvent> for ObserveBreezePacket {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            use pumpkin_data::{packet::CURRENT_MC_VERSION, sound::Sound};
            use pumpkin_protocol::{
                MultiVersionJavaPacket,
                java::client::play::{CEntitySoundEffect, CEntityVelocity, CExplosion},
                ser::NetworkReadExt,
            };

            Box::pin(async move {
                let expected = match self.kind {
                    BreezePacket::Burst => CExplosion::to_id(CURRENT_MC_VERSION),
                    BreezePacket::Flight => CEntityVelocity::to_id(CURRENT_MC_VERSION),
                    BreezePacket::ChargeSound => CEntitySoundEffect::to_id(CURRENT_MC_VERSION),
                };
                if event.packet_id == expected
                    && (!matches!(self.kind, BreezePacket::ChargeSound)
                        || event
                            .payload
                            .as_ref()
                            .get_var_int()
                            .is_ok_and(|id| id.0 == Sound::EntityBreezeCharge as i32 + 1))
                {
                    self.sent.store(true, Relaxed);
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn breeze_burst_serializes_before_a_nearby_java_player_arrives()
    -> Result<(), Box<dyn std::error::Error>> {
        check_breeze_packets_for_a_late_player(BreezePacket::Burst).await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn breeze_flight_serializes_before_a_nearby_java_player_arrives()
    -> Result<(), Box<dyn std::error::Error>> {
        check_breeze_packets_for_a_late_player(BreezePacket::Flight).await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn breeze_charge_sound_serializes_before_a_nearby_java_player_arrives()
    -> Result<(), Box<dyn std::error::Error>> {
        check_breeze_packets_for_a_late_player(BreezePacket::ChargeSound).await
    }

    fn prepare_breeze_long_jump(
        world: &Arc<World>,
        breeze: &dyn EntityBase,
    ) -> Result<(), &'static str> {
        use crate::entity::ai::brain::memory::types;
        use pumpkin_data::entity::EntityPose;
        use pumpkin_data::environment_attribute::Activity;

        let target = from_type(
            &EntityType::IRON_GOLEM,
            Vector3::new(9.5, 100.0, 0.5),
            world,
            uuid::Uuid::new_v4(),
        );
        let mob = breeze.get_mob().ok_or("expected a Breeze mob")?;
        mob.get_mob_entity().set_target(Some(target.clone()));
        mob.get_mob_entity().with_brain(mob, |brain| {
            brain.set(types::ATTACK_TARGET, target.clone());
            brain.set(types::BREEZE_JUMP_TARGET, BlockPos::new(8, 100, 0));
            brain.set_active_activity_if_possible(Activity::Fight);
        });
        breeze.get_entity().on_ground.store(true, Relaxed);
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Standing);
        world.add_entity_silent(target);
        Ok(())
    }

    async fn check_breeze_packets_for_a_late_player(
        kind: BreezePacket,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::world::BlockFlags;
        use pumpkin_data::entity::EntityPose;
        use pumpkin_protocol::ConnectionState;
        use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
        use std::num::NonZero;

        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        prepare_separated_islands(&world);
        if matches!(kind, BreezePacket::Burst) {
            world.set_block_state(
                &BlockPos::new(5, 100, 2),
                Block::STONE.default_state.id,
                BlockFlags::UPDATE_KNOWN_SHAPE,
            );
        }

        let player = spawn_teleport_test_player(&world, &server, false)?;
        player.get_entity().set_pos(Vector3::new(5.5, 100.0, 12.5));
        // A changing-dimension client can still track the same chunk coordinates.
        let watched = Cylindrical::new(
            Vector2::new(0, 0),
            NonZero::<u8>::new(2).ok_or("expected a nonzero view distance")?,
        );
        assert!(watched.is_within_distance(0, 0));
        player.watched_section.store(watched);
        let ClientPlatform::Java(client) = player.client.as_ref() else {
            return Err("expected a Java client".into());
        };
        client.connection_state.store(ConnectionState::Play);
        client.set_player(player.clone());
        player.set_client_loaded(false);
        assert!(world.players.load().is_empty());
        assert!(!player.has_client_loaded());
        world.add_entity_silent(Arc::new(InsertPlayerDuringTick {
            entity: Entity::new(
                world.clone(),
                Vector3::new(5.5, 100.0, 4.5),
                &EntityType::ARMOR_STAND,
            ),
            player: player.clone(),
        }));

        let observer = Arc::new(ObserveBreezePacket {
            kind,
            sent: AtomicBool::new(false),
        });
        server.plugin_manager.register::<PacketSentEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let breeze = from_type(
            &EntityType::BREEZE,
            Vector3::new(0.5, 100.0, 0.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        world.add_entity_silent(breeze.clone());
        let charge = if matches!(kind, BreezePacket::ChargeSound) {
            prepare_breeze_long_jump(&world, breeze.as_ref())?;
            None
        } else {
            let charge = from_type(
                &EntityType::BREEZE_WIND_CHARGE,
                Vector3::new(2.5, 100.6, 2.5),
                &world,
                uuid::Uuid::new_v4(),
            );
            charge
                .get_projectile_owner()
                .ok_or("expected a charge owner record")?
                .set_uuid(Some(breeze.get_entity().entity_uuid));
            charge
                .get_entity()
                .set_velocity(Vector3::new(3.0, 0.0, 0.0));
            world.add_entity_silent(charge.clone());
            // The projectile guard must work after its owner has been removed.
            breeze.get_entity().remove();
            assert!(charge.get_owner_entity().is_none());
            Some(charge)
        };

        world.tick(&server);

        client.clear_player();
        let schedule = entity_schedule(&world);
        server.stop();
        server.shutdown().await;
        let expected_ticking = MIN_PARALLEL_ENTITIES + 1 + usize::from(charge.is_none());
        if let Some(charge) = charge {
            assert_eq!(
                charge.get_entity().is_removed(),
                matches!(kind, BreezePacket::Burst)
            );
        } else {
            assert_eq!(breeze.get_entity().pose.load(), EntityPose::Inhaling);
        }
        assert!(
            observer.sent.load(Relaxed),
            "the late player must receive the expected packet"
        );
        assert_eq!(schedule, (expected_ticking, true));
        Ok(())
    }

    #[tokio::test]
    async fn death_protection_reserves_the_destination_for_a_bite_or_impact()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::borrow::Cow;

        use pumpkin_data::{
            data_component_impl::{ConsumeEffect, DeathProtectionImpl, EquipmentSlot},
            item::Item,
            item_stack::ItemStack,
        };

        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let frog = FrogEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 100.0, 2.5),
            &EntityType::FROG,
        ));
        let prey = SlimeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 100.0, 2.5),
            &EntityType::SLIME,
        ));
        let charge = from_type(
            &EntityType::WIND_CHARGE,
            Vector3::new(2.5, 100.0, 2.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        frog.set_tongue_target(prey.as_ref());
        world.add_entity_silent(prey.clone());
        let config = EntityIslandsConfig::default();
        let actors = [frog as Arc<dyn EntityBase>, charge];

        for (diameter, expect_serial) in [(1024.0, true), (0.25, false)] {
            let mut protection = ItemStack::new(1, &Item::TOTEM_OF_UNDYING);
            protection.set_data_component(DeathProtectionImpl {
                death_effects: Cow::Owned(vec![ConsumeEffect::TeleportRandomly(diameter)]),
            });
            prey.get_mob_entity()
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .equipment
                .insert(EquipmentSlot::MAIN_HAND, protection);
            for actor in &actors {
                let reports = vec![EntityReach::of(actor.as_ref(), &config)];
                let reaches = resolve(reports, &config);
                assert_eq!(reaches.is_none(), expect_serial, "{diameter}");
                if let Some(reaches) = reaches {
                    let reach = &reaches[0];
                    let floor = f64::from(world.get_bottom_y());
                    assert!(reach.body.min.y <= floor && reach.block_reads.min.y <= floor);
                }
            }
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shared_brain_pose_handlers_force_serial_ticks_for_separated_mobs()
    -> Result<(), Box<dyn std::error::Error>> {
        let pose_mobs = [
            &EntityType::FROG,
            &EntityType::CAMEL,
            &EntityType::CAMEL_HUSK,
            &EntityType::VILLAGER,
        ];
        for observe_pose in [false, true] {
            let folder = tempfile::tempdir()?;
            let server = test_support::server(&folder).await?;
            let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
            let plugins = &server.plugin_manager;
            if observe_pose {
                plugins.register::<EntityPoseChangeEvent, _>(
                    Arc::new(IgnoreEvent),
                    EventPriority::Normal,
                    true,
                );
            } else {
                plugins.register::<PlayerChatEvent, _>(
                    Arc::new(IgnoreEvent),
                    EventPriority::Normal,
                    true,
                );
            }

            for index in 0..MIN_PARALLEL_ENTITIES {
                let chunk = force_separated_chunk(&world, index);
                let entity = from_type(
                    pose_mobs
                        .get(index)
                        .copied()
                        .unwrap_or(&EntityType::ARMOR_STAND),
                    Vector3::new(f64::from(chunk.x * 16) + 0.5, 100.0, 0.5),
                    &world,
                    uuid::Uuid::new_v4(),
                );
                if index < pose_mobs.len() {
                    let config = &server.advanced_config.world.entity_islands;
                    assert_eq!(
                        resolve(vec![EntityReach::of(entity.as_ref(), config)], config).is_none(),
                        observe_pose,
                        "{:?}",
                        entity.get_entity().entity_type.resource_name
                    );
                }
                world.add_entity_silent(entity);
            }

            world.tick(&server);
            let (ticking_entities, serial) = {
                let schedule = world
                    .island_schedule
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (schedule.entity_count, schedule.ticks_until_probe > 0)
            };
            assert_eq!(ticking_entities, MIN_PARALLEL_ENTITIES);
            assert_eq!(serial, observe_pose);
            server.stop();
            server.shutdown().await;
        }
        Ok(())
    }

    #[tokio::test]
    async fn only_reachable_plugin_handlers_make_a_wind_charge_tick_global()
    -> Result<(), Box<dyn std::error::Error>> {
        struct IgnoreEvent;
        impl<E: Payload> EventHandler<E> for IgnoreEvent {}

        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let make_charge = |kind| {
            from_type(
                kind,
                Vector3::new(0.0, 64.0, 0.0),
                &world,
                uuid::Uuid::new_v4(),
            )
        };
        let ordinary = make_charge(&EntityType::WIND_CHARGE);
        let breeze = make_charge(&EntityType::BREEZE_WIND_CHARGE);
        let observe = |plugins: &PluginManager, entity: &dyn EntityBase| {
            entity
                .cast_any()
                .downcast_ref::<WindChargeEntity>()
                .map(|charge| plugins_observe_wind_charge_tick(plugins, &world, charge, 0.0))
                .ok_or("expected a wind charge")
        };
        let plugins = PluginManager::new(Vec::new());

        plugins.register::<PlayerChatEvent, _>(Arc::new(IgnoreEvent), EventPriority::Normal, true);
        assert!(!observe(&plugins, breeze.as_ref())?);

        plugins.register::<PacketSentEvent, _>(Arc::new(IgnoreEvent), EventPriority::Normal, true);
        assert!(!observe(&plugins, ordinary.as_ref())?);
        assert!(observe(&plugins, breeze.as_ref())?);

        plugins.register::<EntityExplodeEvent, _>(
            Arc::new(IgnoreEvent),
            EventPriority::Normal,
            true,
        );
        assert!(observe(&plugins, ordinary.as_ref())?);

        let plugins = PluginManager::new(Vec::new());
        plugins.register::<EntityPoseChangeEvent, _>(
            Arc::new(IgnoreEvent),
            EventPriority::Normal,
            true,
        );
        assert!(observe(&plugins, ordinary.as_ref())?);

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_wind_charge_can_reach_a_gateway_exit_without_packet_observers()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let source = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        let neighbor = pumpkin_world::chunk::ChunkData::empty_sync(1, 0);
        world.level.loaded_chunks.insert(Vector2::new(0, 0), source);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(1, 0), neighbor.clone());
        let charge = from_type(
            &EntityType::WIND_CHARGE,
            Vector3::new(14.5, 64.0, 8.5),
            &world,
            uuid::Uuid::new_v4(),
        );
        charge
            .get_entity()
            .set_velocity(Vector3::new(2.0, 0.0, 0.0));
        let config = EntityIslandsConfig::default();
        let reports = || {
            vec![
                EntityReach::of(charge.as_ref(), &config),
                EntityReach::Conservative(node(-1, 400.0, 0.0).1),
            ]
        };
        let reaches = resolve(reports(), &config).ok_or("unexpected serial fallback")?;
        let nodes = [
            (charge.get_entity().entity_id, reaches[0].clone()),
            (-1, reaches[1].clone()),
        ];
        assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1]]);

        neighbor.set_block_absolute_y(0, 64, 8, Block::END_GATEWAY.default_state.id);
        assert!(resolve(reports(), &config).is_none());

        world.level.shutdown().await;
        Ok(())
    }

    #[test]
    fn serial_schedule_retries_after_twenty_ticks_or_a_count_change() {
        let mut schedule = Schedule::default();
        assert!(!schedule.should_plan(1));
        assert!(schedule.should_plan(64));
        assert!(!schedule.record_plan(64));
        for _ in 0..19 {
            assert!(!schedule.should_plan(64));
        }
        assert!(schedule.should_plan(64));
        assert!(!schedule.record_plan(48));
        assert!(!schedule.should_plan(64));
        assert!(schedule.should_plan(65));
        assert!(!schedule.record_plan(49));
        assert!(!schedule.should_plan(65));
        assert!(schedule.should_plan(66));
        assert!(schedule.record_plan(32));
        assert!(schedule.should_plan(66));
        assert!(schedule.should_plan(66));
        assert!(!schedule.should_plan(1));
        assert!(!schedule.should_plan(1));
        assert!(schedule.should_plan(65));
        assert!(schedule.record_plan(48));
        assert!(schedule.should_plan(65));
    }

    #[expect(
        clippy::expect_used,
        reason = "Ordinary fixtures must fit the planning budget."
    )]
    fn islands(
        nodes: &[(i32, TickReach)],
        idle_entities: &[(i32, BoundingBox)],
        players: &[(i32, BoundingBox)],
    ) -> Vec<Vec<usize>> {
        partition(nodes, idle_entities, players)
            .expect("ordinary fixture exceeded the planning budget")
            .islands
    }

    fn cube(x: f64, size: f64) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(x, 64.0, 0.0),
            Vector3::new(x + size, 64.0 + size, size),
        )
    }

    fn node(id: i32, x: f64, range: f64) -> (i32, TickReach) {
        let body = cube(x, 1.0);
        let area = body.expand(range, range, range);
        (
            id,
            TickReach {
                body,
                entities: area,
                players: area,
                block_reads: body,
                block_writes: None,
                global: false,
                links: Vec::new(),
            },
        )
    }

    #[test]
    fn incremental_grid_queries_match_all_inserted_boxes() -> Result<(), &'static str> {
        let mut grid = Grid::default();
        let mut query = Query::new(128);
        let mut budget = PlanningBudget::default();
        for slot in 0..128 {
            let min = if slot < 32 {
                Vector3::new(
                    f64::from(slot % 8) * 0.5 - 8.0,
                    f64::from(slot % 3) * 16.0,
                    f64::from(slot / 8) * 0.5 - 8.0,
                )
            } else {
                Vector3::new(
                    f64::from(slot % 19) * 4.0 - 36.0,
                    f64::from(slot % 5) * 16.0,
                    f64::from(slot / 19) * 4.0 - 16.0,
                )
            };
            let size = if slot == 0 {
                4.25
            } else if slot < 32 {
                0.25
            } else {
                [0.0, 0.25, 4.0, 4.25, 17.0][slot as usize % 5]
            };
            let mut body = BoundingBox::new(min, min + Vector3::new(size, 2.0, size));
            if slot % 9 == 0 {
                body.min.y = f64::NEG_INFINITY;
                body.max.y = f64::INFINITY;
            }
            grid.insert(slot as usize, body, &mut budget)
                .ok_or("grid budget exhausted")?;
            for area in [body, body.expand(16.0, 0.0, 16.0)] {
                let expected: Vec<_> = grid
                    .boxes
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, body)| overlaps(&area, body).then_some(slot))
                    .collect();
                let mut found = Vec::new();
                grid.for_each_overlapping(&mut query, &area, &mut budget, |slot| found.push(slot))
                    .ok_or("query budget exhausted")?;
                found.sort_unstable();
                assert_eq!(found, expected);
            }
        }
        Ok(())
    }

    #[test]
    fn partition_matches_pairwise_connected_components() {
        use rand::{RngExt, SeedableRng, rngs::StdRng};

        fn bounds(random: &mut StdRng) -> BoundingBox {
            let min = Vector3::new(
                f64::from(random.random_range(-16..16)) * 4.0,
                f64::from(random.random_range(-4..4)) * 16.0,
                f64::from(random.random_range(-16..16)) * 4.0,
            );
            let size = [0.25, 4.0, 4.25, 16.0, 32.0][random.random_range(0..5)];
            BoundingBox::new(min, min + Vector3::new(size, 2.0, size))
        }

        let mut random = StdRng::seed_from_u64(42);
        for _ in 0..32 {
            let nodes: Vec<_> = (0..64)
                .map(|id| {
                    let body = if id < 48 {
                        let min = Vector3::new(
                            f64::from(id % 6) * 0.5,
                            f64::from(id / 24) * 16.0,
                            f64::from(id / 6) * 0.5,
                        );
                        BoundingBox::new(min, min + Vector3::new(0.25, 2.0, 0.25))
                    } else {
                        bounds(&mut random)
                    };
                    let mut reads = bounds(&mut random);
                    reads.min.y = f64::NEG_INFINITY;
                    reads.max.y = f64::INFINITY;
                    (
                        id,
                        TickReach {
                            body,
                            entities: body.expand(0.5, 0.5, 0.5),
                            players: body.expand(4.0, 4.0, 4.0),
                            block_reads: reads,
                            block_writes: (id % 4 == 0).then(|| bounds(&mut random)),
                            global: false,
                            links: if id % 7 == 0 { vec![1000] } else { Vec::new() },
                        },
                    )
                })
                .collect();
            let idle: Vec<_> = (100..104).map(|id| (id, bounds(&mut random))).collect();
            let players: Vec<_> = (200..203).map(|id| (id, bounds(&mut random))).collect();
            let bodies: Vec<_> = nodes
                .iter()
                .map(|(id, reach)| (*id, reach.body))
                .chain(idle.iter().copied())
                .chain(players.iter().copied())
                .collect();
            // The extra vertex represents an external link shared by several nodes.
            let mut edges = vec![vec![false; bodies.len() + 1]; bodies.len() + 1];
            for (slot, (_, reach)) in nodes.iter().enumerate() {
                for (other, (_, body)) in bodies.iter().enumerate() {
                    let area = if other < nodes.len() + idle.len() {
                        &reach.entities
                    } else {
                        &reach.players
                    };
                    if overlaps(area, body) {
                        edges[slot][other] = true;
                        edges[other][slot] = true;
                    }
                }
                if !reach.links.is_empty() {
                    edges[slot][bodies.len()] = true;
                    edges[bodies.len()][slot] = true;
                }
                if let Some(writes) = reach.block_writes {
                    for (other, (_, reader)) in nodes.iter().enumerate() {
                        if overlaps(&writes, &reader.block_reads) {
                            edges[slot][other] = true;
                            edges[other][slot] = true;
                        }
                    }
                }
            }
            let mut visited = vec![false; edges.len()];
            let mut expected = Vec::new();
            for start in 0..nodes.len() {
                if visited[start] {
                    continue;
                }
                let mut pending = VecDeque::from([start]);
                let mut component = Vec::new();
                visited[start] = true;
                while let Some(slot) = pending.pop_front() {
                    if slot < nodes.len() {
                        component.push(slot);
                    }
                    for (other, connected) in edges[slot].iter().enumerate() {
                        if *connected && !visited[other] {
                            visited[other] = true;
                            pending.push_back(other);
                        }
                    }
                }
                component.sort_unstable();
                expected.push(component);
            }
            assert_eq!(islands(&nodes, &idle, &players), expected);
        }
    }

    #[test]
    fn entities_out_of_reach_tick_apart_in_original_order() {
        let nodes = [
            node(10, 0.0, 0.5),
            node(11, 100.0, 0.5),
            node(12, 1.5, 0.5),
            node(13, 200.0, 0.5),
        ];
        assert_eq!(islands(&nodes, &[], &[]), [vec![0, 2], vec![1], vec![3]]);
    }

    #[test]
    fn one_sided_reach_joins_both_entities() {
        let nodes = [node(10, 0.0, 30.0), node(11, 25.0, 0.0)];
        assert_eq!(islands(&nodes, &[], &[]), [vec![0, 1]]);
    }

    #[tokio::test]
    async fn detonating_tnt_preserves_world_tick_order() -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{Entity, tnt::TNTEntity};
        use pumpkin_nbt::compound::NbtCompound;

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let tnt = TNTEntity::new(
            Entity::new(
                world.clone(),
                Vector3::new(0.0, 64.0, 0.0),
                &EntityType::TNT,
            ),
            TNTEntity::DEFAULT_POWER,
            TNTEntity::DEFAULT_FUSE,
        );
        let config = EntityIslandsConfig::default();
        for fuse in [2, 1, 0] {
            let mut nbt = NbtCompound::new();
            nbt.put_short("fuse", fuse);
            nbt.put_float("explosion_power", 128.0);
            tnt.read_custom_nbt(&nbt);
            let reaches = resolve(vec![EntityReach::of(&tnt, &config)], &config);
            if fuse <= 1 {
                assert!(reaches.is_none());
                continue;
            }
            let reaches = reaches.ok_or("unexpected serial fallback")?;
            let nodes = [
                (tnt.get_entity().entity_id, reaches[0].clone()),
                node(-1, 150.0, 0.0),
                node(-2, 400.0, 0.0),
            ];
            assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1], vec![2]]);
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn unaudited_living_motion_joins_entities_at_the_destination()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{Entity, mob::zombie::zombie::ZombieEntity};

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let zombie = ZombieEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::ZOMBIE,
        ));
        assert!(zombie.audited_reach().is_none());
        let config = EntityIslandsConfig::default();
        for speed in [0.0, -96.0, 96.0] {
            zombie
                .get_entity()
                .velocity
                .store(Vector3::new(speed, 0.0, 0.0));
            let reaches = resolve(vec![EntityReach::of(zombie.as_ref(), &config)], &config)
                .ok_or("unexpected serial fallback")?;
            let nodes = [
                (zombie.get_entity().entity_id, reaches[0].clone()),
                node(-1, 96.0, 0.0),
                node(-2, -96.0, 0.0),
            ];
            if speed == 0.0 {
                assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1], vec![2]]);
            } else {
                let destination = if speed > 0.0 { 1 } else { 2 };
                assert!(islands(&nodes, &[], &[])[0].contains(&destination));
            }
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[test]
    fn cells_covered_by_a_large_reach_still_respect_height() {
        let mut low = node(10, 0.0, 0.0);
        low.1.body = BoundingBox::new(Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 1.0, 2.0));
        let mut wide = node(11, 40.0, 0.0);
        wide.1.entities = BoundingBox::new(
            Vector3::new(-64.0, 60.0, -64.0),
            Vector3::new(64.0, 70.0, 64.0),
        );
        assert_eq!(islands(&[low, wide], &[], &[]), [vec![0], vec![1]]);
    }

    #[test]
    fn idle_entities_players_and_links_bridge_islands() {
        let nodes = [node(10, 0.0, 1.0), node(11, 4.0, 1.0)];
        assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1]]);
        assert_eq!(islands(&nodes, &[(20, cube(2.0, 1.0))], &[]), [vec![0, 1]]);
        assert_eq!(islands(&nodes, &[], &[(30, cube(2.0, 1.0))]), [vec![0, 1]]);
        let mut linked = nodes;
        linked[0].1.links.push(40);
        linked[1].1.links.push(40);
        assert_eq!(islands(&linked, &[], &[]), [vec![0, 1]]);
    }

    #[test]
    fn block_changes_join_entities_reading_those_blocks() {
        let mut nodes = [node(10, 0.0, 0.0), node(11, 30.0, 0.0)];
        assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1]]);
        nodes[0].1.block_writes = Some(nodes[0].1.body.expand(17.0, 17.0, 17.0));
        nodes[1].1.block_reads = nodes[1].1.body.expand(10.0, 10.0, 10.0);
        assert_eq!(islands(&nodes, &[], &[]), [vec![0], vec![1]]);
        nodes[1].1.block_reads = nodes[1].1.body.expand(15.0, 15.0, 15.0);
        assert_eq!(islands(&nodes, &[], &[]), [vec![0, 1]]);
    }

    #[test]
    fn a_global_entity_serializes_the_tick() {
        let mut nodes = [node(10, 0.0, 0.0), node(11, 300.0, 0.0)];
        nodes[1].1.global = true;
        assert!(partition(&nodes, &[], &[]).is_none());
    }

    fn assert_box(actual: &BoundingBox, expected: &BoundingBox) {
        assert_eq!((actual.min, actual.max), (expected.min, expected.max));
    }

    fn audited(x: f64) -> EntityReach {
        EntityReach::Audited(AuditedReach {
            bounding_box: cube(x, 1.0),
            motion: (0.25, 0.5),
            entities: 0.0,
            players: 0.0,
            block_reads: 0.0,
            route: None,
            changes_blocks: false,
            global: false,
            links: Vec::new(),
        })
    }

    #[tokio::test]
    async fn large_contact_scans_request_serial_ticking() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let EntityReach::Audited(mut reach) = audited(0.0) else {
            return Err("expected audited reach".into());
        };
        for motion in [8.0, f64::INFINITY, f64::NAN] {
            reach.motion = (motion, motion);
            reach.global = false;
            reach.scan_contact_blocks(&world);
            assert!(reach.global);
            assert!(
                resolve(
                    vec![EntityReach::Audited(reach.clone())],
                    &EntityIslandsConfig::default()
                )
                .is_none()
            );
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn large_follow_ranges_exhaust_planning_without_changing_attributes()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{
            Entity,
            mob::{Mob, zombie::zombie::ZombieEntity},
        };

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let zombie = ZombieEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::ZOMBIE,
        ));
        zombie
            .get_mob_entity()
            .living_entity
            .set_attribute_base(&Attributes::FOLLOW_RANGE, 1000.0);
        let config = EntityIslandsConfig::default();
        let reports = (0..16)
            .map(|_| EntityReach::of(zombie.as_ref(), &config))
            .collect();
        assert!(resolve(reports, &config).is_none());
        assert_eq!(
            zombie
                .get_mob_entity()
                .living_entity
                .get_attribute_value(&Attributes::FOLLOW_RANGE),
            1000.0
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[test]
    fn broad_queries_exhaust_the_budget_without_clipping_the_query() -> Result<(), &'static str> {
        let mut grid = Grid::default();
        let mut budget = PlanningBudget::default();
        for slot in 0..64 {
            grid.insert(slot, cube((slot % 8) as f64 * 0.5, 0.25), &mut budget)
                .ok_or("insertion refused")?;
        }
        grid.insert(64, cube(900.0, 1.0), &mut budget)
            .ok_or("distant insertion refused")?;
        assert!(grid.refined);
        let mut query = Query::new(65);
        let area = cube(0.0, 1.0).expand(1000.0, 0.0, 1000.0);
        let mut found = Vec::new();
        grid.for_each_overlapping(&mut query, &area, &mut budget, |slot| found.push(slot))
            .ok_or("first query refused")?;
        found.sort_unstable();
        assert_eq!(found, (0..65).collect::<Vec<_>>());
        assert!(
            grid.for_each_overlapping(&mut query, &area, &mut budget, |_| {})
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn oversized_and_nonfinite_partition_bounds_request_serial_ticking() {
        for bound in [1.0e20, f64::INFINITY, f64::NAN] {
            let mut nodes = [node(0, 0.0, 0.0), node(1, 20.0, 0.0)];
            nodes[0].1.entities = nodes[0].1.entities.expand(bound, 0.0, bound);
            assert!(partition(&nodes, &[], &[]).is_none());
            nodes[0].1.entities = nodes[0].1.body;
            nodes[0].1.block_reads = nodes[0].1.body.expand(bound, 0.0, bound);
            nodes[1].1.block_writes = Some(nodes[1].1.body);
            assert!(partition(&nodes, &[], &[]).is_none());
        }
        let nodes: Vec<_> = (0..2000)
            .map(|id| node(id, f64::from(id) * 4.0, 0.5))
            .collect();
        assert!(partition(&nodes, &[], &[]).is_some());
    }

    #[tokio::test]
    async fn oversized_write_queries_leave_pruning_results_unused()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        world.level.loaded_chunks.insert(
            Vector2::new(0, 0),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
        let mut reaches = vec![node(0, 1.0, 0.0).1, node(1, 32.0, 0.0).1];
        reaches[0].block_writes = Some(reaches[0].body);
        reaches[1].block_writes = Some(reaches[1].body.expand(10_000.0, 0.0, 10_000.0));
        assert!(
            prune_block_writes(
                &world,
                &mut reaches,
                &[BlockWriteCheck::Contact, BlockWriteCheck::Unconditional]
            )
            .is_none()
        );
        assert!(reaches.iter().all(|reach| reach.block_writes.is_some()));
        world.level.shutdown().await;
        Ok(())
    }

    #[test]
    fn only_earlier_pushers_grow_current_turn() -> Result<(), &'static str> {
        let alone = resolve(vec![audited(0.0)], &EntityIslandsConfig::default())
            .ok_or("unexpected serial fallback")?;
        assert_box(&alone[0].body, &cube(0.0, 1.0).expand(0.25, 0.5, 0.25));
        let crowded = resolve(
            vec![audited(0.0), audited(0.0), audited(0.0)],
            &EntityIslandsConfig::default(),
        )
        .ok_or("unexpected serial fallback")?;
        assert_box(&crowded[0].body, &alone[0].body);
        assert!(crowded[1].body.min.x < crowded[0].body.min.x);
        assert!(crowded[2].body.min.x < crowded[1].body.min.x);
        assert!(crowded[2].body.max.x > crowded[1].body.max.x);
        Ok(())
    }

    #[test]
    fn unaudited_neighbours_can_throw_audited_entities_around() -> Result<(), &'static str> {
        let zombie =
            EntityReach::Conservative(TickReach::conservative(&cube(10.0, 1.0), 10.0, Vec::new()));
        let reaches = resolve(
            vec![audited(0.0), zombie, audited(40.0)],
            &EntityIslandsConfig::default(),
        )
        .ok_or("unexpected serial fallback")?;
        // The zombie can both hit and push it.
        let thrown = EntityIslandsConfig::default().knockback_reach + PUSH_SPEED;
        assert_box(
            &reaches[0].body,
            &cube(0.0, 1.0).expand(0.25 + thrown, 0.5 + thrown, 0.25 + thrown),
        );
        assert!(reaches[0].block_writes.is_some());
        assert_box(&reaches[2].body, &cube(40.0, 1.0).expand(0.25, 0.5, 0.25));
        Ok(())
    }

    #[tokio::test]
    async fn contact_writers_propagate_through_a_reverse_ordered_chain()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        for x in 0..=4 {
            world.level.loaded_chunks.insert(
                Vector2::new(x, 0),
                pumpkin_world::chunk::ChunkData::empty_sync(x, 0),
            );
        }
        let mut reaches: Vec<_> = [30.0, 20.0, 10.0, 60.0]
            .into_iter()
            .map(|x| node(0, x, 0.0).1)
            .collect();
        // The seed reaches the second candidate, whose writes reach the first.
        reaches[0].block_writes = Some(cube(30.0, 1.0));
        reaches[1].block_writes = Some(cube(30.0, 1.0));
        reaches[2].block_writes = Some(cube(20.0, 1.0));
        reaches[3].block_writes = Some(cube(60.0, 1.0));
        let body = reaches[3].body;
        let reads = reaches[3].block_reads;
        prune_block_writes(
            &world,
            &mut reaches,
            &[
                BlockWriteCheck::Contact,
                BlockWriteCheck::Contact,
                BlockWriteCheck::Unconditional,
                BlockWriteCheck::Contact,
            ],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(
            reaches[..3]
                .iter()
                .all(|reach| reach.block_writes.is_some())
        );
        assert!(reaches[3].block_writes.is_none());
        assert_box(&reaches[3].body, &body);
        assert_box(&reaches[3].block_reads, &reads);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn final_contact_region_keeps_hazards_and_missing_chunks_conservative()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), chunk.clone());
        let mut reach = node(0, 10.0, 0.0).1;
        reach.body = BoundingBox::new(Vector3::new(10.0, 64.0, 1.0), Vector3::new(18.0, 65.0, 2.0));
        reach.block_writes = Some(reach.body.expand(
            BLOCK_UPDATE_REACH,
            BLOCK_UPDATE_REACH,
            BLOCK_UPDATE_REACH,
        ));
        let original = reach.clone();
        prune_block_writes(
            &world,
            std::slice::from_mut(&mut reach),
            &[BlockWriteCheck::Contact],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(reach.block_writes.is_some());
        let neighbor = pumpkin_world::chunk::ChunkData::empty_sync(1, 0);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(1, 0), neighbor.clone());
        neighbor.set_block_absolute_y(1, 63, 1, Block::STONE_PRESSURE_PLATE.default_state.id);
        prune_block_writes(
            &world,
            std::slice::from_mut(&mut reach),
            &[BlockWriteCheck::Contact],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(reach.block_writes.is_some());
        neighbor.set_block_absolute_y(1, 63, 1, Block::AIR.default_state.id);
        prune_block_writes(
            &world,
            std::slice::from_mut(&mut reach),
            &[BlockWriteCheck::Contact],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(reach.block_writes.is_none());
        assert_box(&reach.body, &original.body);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn chest_cart_writes_only_bridge_elevated_readers_when_a_rail_can_activate()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{Entity, vehicle::minecart::MinecartEntity};

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), chunk.clone());
        let pos = BlockPos::new(8, 40, 8);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 40.0, 8.5),
            &EntityType::CHEST_MINECART,
        ));
        let config = EntityIslandsConfig::default();
        let custom = Entity::new(
            world.clone(),
            cart.get_entity().pos.load(),
            &EntityType::CHEST_MINECART,
        );
        assert!(matches!(
            EntityReach::of(&custom, &config).block_write_check(&custom),
            BlockWriteCheck::Unconditional
        ));
        let report = EntityReach::of(&cart, &config);
        assert!(matches!(report, EntityReach::Conservative(_)));
        let check = report.block_write_check(&cart);
        let mut reader = node(1, 8.0, 0.0).1;
        reader.body =
            BoundingBox::new(Vector3::new(8.0, 200.0, 8.0), Vector3::new(9.0, 201.0, 9.0));
        reader.entities = reader.body;
        reader.players = reader.body;
        reader.block_reads = reader.body;
        reader.block_reads.min.y = f64::NEG_INFINITY;
        reader.block_reads.max.y = f64::INFINITY;
        let mut original = resolve(vec![report], &config).ok_or("unexpected serial fallback")?;
        original.push(reader);
        let partitioned = |reaches: &[TickReach]| {
            let nodes: Vec<_> = reaches
                .iter()
                .cloned()
                .enumerate()
                .map(|(id, reach)| (id as i32, reach))
                .collect();
            islands(&nodes, &[], &[])
        };
        assert_eq!(partitioned(&original), [vec![0, 1]]);
        let mut reaches = original.clone();
        prune_block_writes(
            &world,
            &mut reaches,
            &[check, BlockWriteCheck::Unconditional],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert_eq!(partitioned(&reaches), [vec![0], vec![1]]);
        assert_box(&reaches[0].body, &original[0].body);
        assert_box(&reaches[0].entities, &original[0].entities);
        assert_box(&reaches[0].block_reads, &original[0].block_reads);

        chunk.set_block_absolute_y(8, 39, 8, Block::DETECTOR_RAIL.default_state.id);
        let mut reaches = original.clone();
        prune_block_writes(
            &world,
            &mut reaches,
            &[check, BlockWriteCheck::Unconditional],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert_eq!(partitioned(&reaches), [vec![0, 1]]);

        chunk.set_block_absolute_y(8, 39, 8, Block::AIR.default_state.id);
        let mut reaches = original.clone();
        let mut writer = node(2, 100.0, 0.0).1;
        writer.block_writes = Some(BoundingBox::from_block(&pos.down()));
        reaches.push(writer);
        prune_block_writes(
            &world,
            &mut reaches,
            &[
                check,
                BlockWriteCheck::Unconditional,
                BlockWriteCheck::Unconditional,
            ],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(reaches[0].block_writes.is_some());

        world.level.loaded_chunks.remove(&Vector2::new(0, 0));
        let mut reaches = original;
        prune_block_writes(
            &world,
            &mut reaches,
            &[check, BlockWriteCheck::Unconditional],
        )
        .ok_or("unexpected pruning budget exhaustion")?;
        assert!(reaches[0].block_writes.is_some());
        world.level.shutdown().await;
        Ok(())
    }
}
