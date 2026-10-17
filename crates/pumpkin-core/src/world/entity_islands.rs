//! Splits a tick's entities into islands that touch disjoint state. Each island ticks in list
//! order on its own thread, which gives the same result as ticking the whole list in order.

use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_data::{Block, attributes::Attributes, tag, tag::Taggable};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::entity::EntityBase;
use crate::world::World;

/// Reach of same-tick neighbour updates: 15 wire, the powered block and one component.
pub const BLOCK_UPDATE_REACH: f64 = 17.0;

/// Extra reach of unaudited entities.
pub const CONSERVATIVE_MARGIN: f64 = 16.0;

/// Grid cell size for large boxes.
const CELL_SIZE: f64 = 16.0;

/// Grid cell size for bodies.
const FINE_CELL_SIZE: f64 = 4.0;

/// Largest speed one `Entity.push` adds.
const PUSH_SPEED: f64 = 0.05 * std::f64::consts::SQRT_2;

/// Extra motion covered by the contact block scan.
const SCAN_SLACK: f64 = 0.5;

/// Pushes assumed for every entity in the first round.
const PUSH_SLACK: f64 = 3.0 * PUSH_SPEED;

/// Speed a hit from an unaudited entity can add, well above melee, explosions and wind charges.
const KNOCKBACK_MOTION: f64 = 4.0;

const PUSH_ROUNDS: usize = 16;

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
        let (min, max) = (area.min_block_pos().0, area.max_block_pos().0);
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
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

impl EntityReach {
    #[must_use]
    pub fn of(entity: &dyn EntityBase) -> Self {
        if let Some(reach) = entity.audited_reach() {
            return Self::Audited(reach);
        }
        let base = entity.get_entity();
        let radius = entity.get_living_entity().map_or_else(
            || base.velocity.load().length() + CONSERVATIVE_MARGIN,
            |living| {
                living
                    .get_attribute_value(&Attributes::FOLLOW_RANGE)
                    .max(0.0)
                    + CONSERVATIVE_MARGIN
            },
        );
        Self::Conservative(TickReach::conservative(
            &base.bounding_box.load(),
            radius,
            links_of(entity),
        ))
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
    for other in [&base.vehicle, &base.leashed_to] {
        if let Some(other) = other
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            links.push(other.get_entity().entity_id);
        }
    }
    links.extend(entity.get_owner_id());
    if let Some(mob) = entity.get_mob()
        && let Some(target) = mob.get_mob_entity().get_target()
    {
        links.push(target.get_entity().entity_id);
    }
    if let Some(living) = entity.get_living_entity() {
        // Kill credit on death.
        for attacker in [
            living.last_hurt_by_player_id.load(Relaxed),
            living.last_hurt_by_mob_id.load(Relaxed),
        ] {
            if attacker != 0 {
                links.push(attacker);
            }
        }
    }
    links
}

/// Turns reports into reaches. Audited bodies grow by the pushes they can receive, and by
/// `KNOCKBACK_MOTION` within reach of an unaudited entity.
#[must_use]
pub fn resolve(reports: Vec<EntityReach>) -> Vec<TickReach> {
    let boxes: Vec<BoundingBox> = reports
        .iter()
        .map(|report| match report {
            EntityReach::Audited(audited) => audited.bounding_box,
            EntityReach::Conservative(reach) => reach.body,
        })
        .collect();
    let threats: Vec<BoundingBox> = reports
        .iter()
        .filter_map(|report| match report {
            EntityReach::Conservative(reach) => Some(reach.entities),
            EntityReach::Audited(_) => None,
        })
        .collect();
    let audited = |index: usize| matches!(reports[index], EntityReach::Audited(_));
    let bodies_with = |extra: &[f64]| -> Vec<BoundingBox> {
        reports
            .iter()
            .zip(extra)
            .map(|(report, extra)| match report {
                EntityReach::Audited(audited) => grown_body(audited, *extra),
                EntityReach::Conservative(reach) => reach.body,
            })
            .collect()
    };

    // Count neighbours as if every body grew by `PUSH_SLACK`, then recount the ones that grew
    // more. Still growing after the last round means the conservative margin.
    let mut extra: Vec<f64> = vec![0.0; reports.len()];
    for round in 0..=PUSH_ROUNDS {
        let counted: Vec<f64> = extra.iter().map(|extra| extra.max(PUSH_SLACK)).collect();
        let bodies = bodies_with(&counted);
        let hits = overlap_counts(&threats, &bodies);
        let pushers = overlap_counts(&bodies, &boxes);
        let mut outgrown = false;
        for index in 0..reports.len() {
            if !audited(index) {
                continue;
            }
            let knockback = if hits[index] > 0 {
                KNOCKBACK_MOTION
            } else {
                0.0
            };
            // Minus itself.
            let pushes = PUSH_SPEED * pushers[index].saturating_sub(1) as f64;
            extra[index] = knockback + pushes;
            if extra[index] > counted[index] {
                outgrown = true;
                if round == PUSH_ROUNDS {
                    extra[index] = CONSERVATIVE_MARGIN;
                }
            }
        }
        if !outgrown {
            break;
        }
    }

    reports
        .into_iter()
        .zip(extra)
        .map(|(report, extra)| match report {
            EntityReach::Conservative(reach) => reach,
            EntityReach::Audited(audited) => finish(audited, extra),
        })
        .collect()
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

fn overlap_counts(sources: &[BoundingBox], targets: &[BoundingBox]) -> Vec<usize> {
    let mut fine: FxHashMap<(i32, i32), Vec<usize>> = FxHashMap::default();
    let mut coarse: FxHashMap<(i32, i32), Vec<usize>> = FxHashMap::default();
    for (index, source) in sources.iter().enumerate() {
        let small = source.max.x - source.min.x <= FINE_CELL_SIZE
            && source.max.z - source.min.z <= FINE_CELL_SIZE;
        let (cells, size) = if small {
            (&mut fine, FINE_CELL_SIZE)
        } else {
            (&mut coarse, CELL_SIZE)
        };
        for x in cells_of(source.min.x, source.max.x, size) {
            for z in cells_of(source.min.z, source.max.z, size) {
                cells.entry((x, z)).or_default().push(index);
            }
        }
    }
    targets
        .par_iter()
        .map(|target| {
            let mut seen: Vec<usize> = Vec::new();
            for (cells, size) in [(&fine, FINE_CELL_SIZE), (&coarse, CELL_SIZE)] {
                if cells.is_empty() {
                    continue;
                }
                for x in cells_of(target.min.x, target.max.x, size) {
                    for z in cells_of(target.min.z, target.max.z, size) {
                        seen.extend(
                            cells
                                .get(&(x, z))
                                .into_iter()
                                .flatten()
                                .filter(|&&source| overlaps(&sources[source], target)),
                        );
                    }
                }
            }
            seen.sort_unstable();
            seen.dedup();
            seen.len()
        })
        .collect()
}

pub struct Islands {
    /// Node indices in list order.
    pub islands: Vec<Vec<usize>>,
    pub island_of: Arc<IslandMap>,
}

/// Island of each entity and player.
pub type IslandMap = FxHashMap<i32, usize>;

/// Splits `nodes` (ticking entities in list order) into islands. Idle entities and players only
/// connect islands.
#[must_use]
pub fn partition(
    nodes: &[(i32, TickReach)],
    idle_entities: &[(i32, BoundingBox)],
    players: &[(i32, BoundingBox)],
) -> Islands {
    if nodes.iter().any(|(_, reach)| reach.global) {
        let island_of = nodes
            .iter()
            .map(|(id, _)| *id)
            .chain(idle_entities.iter().map(|(id, _)| *id))
            .chain(players.iter().map(|(id, _)| *id))
            .map(|id| (id, 0))
            .collect();
        return Islands {
            islands: vec![(0..nodes.len()).collect()],
            island_of: Arc::new(island_of),
        };
    }

    let mut sets = DisjointSets::new(nodes.len() + idle_entities.len() + players.len());
    let mut slots: FxHashMap<i32, usize> = FxHashMap::default();
    let bodies = nodes
        .iter()
        .map(|(id, reach)| (*id, reach.body))
        .chain(idle_entities.iter().copied());
    let mut entity_grid = Grid::default();
    for (slot, (id, body)) in bodies.enumerate() {
        slots.insert(id, slot);
        entity_grid.insert(slot, body);
    }
    let first_player = nodes.len() + idle_entities.len();
    for (offset, (id, _)) in players.iter().enumerate() {
        slots.insert(*id, first_player + offset);
    }

    let mut query = Query::new(sets.len());
    for (slot, (_, reach)) in nodes.iter().enumerate() {
        entity_grid.union_overlapping(&mut sets, &mut query, slot, &reach.entities);
        for (offset, (_, body)) in players.iter().enumerate() {
            if overlaps(&reach.players, body) {
                sets.union(slot, first_player + offset);
            }
        }
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
            block_grid.insert(slot, reach.block_reads);
        }
        let mut query = Query::new(sets.len());
        for (slot, writes) in writers {
            block_grid.union_overlapping(&mut sets, &mut query, slot, &writes);
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
    let island_of = slots
        .into_iter()
        .filter_map(|(id, slot)| {
            island_of_root
                .get(&sets.find(slot))
                .map(|island| (id, *island))
        })
        .collect();
    Islands {
        islands,
        island_of: Arc::new(island_of),
    }
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

fn cell_range(min: f64, max: f64) -> std::ops::RangeInclusive<i32> {
    cells_of(min, max, CELL_SIZE)
}

fn cells_of(min: f64, max: f64, size: f64) -> std::ops::RangeInclusive<i32> {
    ((min / size).floor() as i32)..=((max / size).floor() as i32)
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
    cells: FxHashMap<(i32, i32), Cell>,
    boxes: Vec<BoundingBox>,
}

impl Grid {
    fn insert(&mut self, slot: usize, area: BoundingBox) {
        debug_assert_eq!(slot, self.boxes.len());
        for x in cell_range(area.min.x, area.max.x) {
            for z in cell_range(area.min.z, area.max.z) {
                let cell = self.cells.entry((x, z)).or_insert_with(|| Cell {
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
        self.boxes.push(area);
    }

    fn union_overlapping(
        &mut self,
        sets: &mut DisjointSets,
        query: &mut Query,
        slot: usize,
        area: &BoundingBox,
    ) {
        query.next();
        for x in cell_range(area.min.x, area.max.x) {
            for z in cell_range(area.min.z, area.max.z) {
                let Some(cell) = self.cells.get_mut(&(x, z)) else {
                    continue;
                };
                let (min_x, min_z) = (f64::from(x) * CELL_SIZE, f64::from(z) * CELL_SIZE);
                // `area` spans the column and every member's height, so all members overlap it.
                let covered = area.min.x <= min_x
                    && area.max.x >= min_x + CELL_SIZE
                    && area.min.z <= min_z
                    && area.max.z >= min_z + CELL_SIZE
                    && area.min.y <= cell.lowest_top
                    && area.max.y >= cell.highest_bottom;
                if covered {
                    if cell.merged {
                        sets.union(slot, cell.members[0]);
                    } else {
                        for &member in &cell.members {
                            sets.union(slot, member);
                        }
                        cell.merged = true;
                    }
                    continue;
                }
                for &member in &cell.members {
                    if query.visit(member) && overlaps(area, &self.boxes[member]) {
                        sets.union(slot, member);
                    }
                }
            }
        }
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
    use pumpkin_util::math::vector3::Vector3;

    fn islands(
        nodes: &[(i32, TickReach)],
        idle_entities: &[(i32, BoundingBox)],
        players: &[(i32, BoundingBox)],
    ) -> Vec<Vec<usize>> {
        partition(nodes, idle_entities, players).islands
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
        assert_eq!(islands(&nodes, &[], &[]), [vec![0, 1]]);
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

    #[test]
    fn neighbours_that_can_push_grow_the_body() {
        let alone = resolve(vec![audited(0.0)]);
        assert_box(&alone[0].body, &cube(0.0, 1.0).expand(0.25, 0.5, 0.25));
        let crowded = resolve(vec![audited(0.0), audited(1.2), audited(-1.2)]);
        let pushed = 0.25 + 2.0 * PUSH_SPEED;
        assert_box(
            &crowded[0].body,
            &cube(0.0, 1.0).expand(pushed, 0.5 + 2.0 * PUSH_SPEED, pushed),
        );
    }

    #[test]
    fn unaudited_neighbours_can_throw_audited_entities_around() {
        let zombie =
            EntityReach::Conservative(TickReach::conservative(&cube(10.0, 1.0), 10.0, Vec::new()));
        let reaches = resolve(vec![audited(0.0), zombie, audited(40.0)]);
        // The zombie can both hit and push it.
        let thrown = KNOCKBACK_MOTION + PUSH_SPEED;
        assert_box(
            &reaches[0].body,
            &cube(0.0, 1.0).expand(0.25 + thrown, 0.5 + thrown, 0.25 + thrown),
        );
        assert!(reaches[0].block_writes.is_some());
        assert_box(&reaches[2].body, &cube(40.0, 1.0).expand(0.25, 0.5, 0.25));
    }
}
