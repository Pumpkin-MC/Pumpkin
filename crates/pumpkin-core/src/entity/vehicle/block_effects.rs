use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::{
    Block, BlockId, BlockState, BlockStateId, damage::DamageType, fluid::Fluid, sound::Sound,
};
use pumpkin_util::math::{
    boundingbox::BoundingBox,
    position::BlockPos,
    vector3::{Axis, Vector3},
};
use rand::RngExt;

use super::movement_history::{BLOCK_INTERSECTION_EPSILON, VehicleMovement};
use crate::block::blocks::{cauldron, fire::FireBlockBase, powder_snow};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use crate::world::World;

const EPSILON: f64 = 1.0e-7;
const MAX_BLOCK_ITERATIONS_ALONG_TRAVEL_PER_TICK: usize = 16;
// Both Java entity fire effects use the fizz sample.
const BEDROCK_FIRE_SOUND: &str = "random.fizz";

/// Applies Java's remaining lava damage after ignition has been handled.
pub(super) fn hurt_vehicle_in_lava(carrier: &dyn EntityBase) {
    let entity = carrier.get_entity();
    if !entity.is_invulnerable_to(&DamageType::LAVA, None)
        && entity.damage(carrier, 4.0, DamageType::LAVA)
        && !entity.is_silent()
    {
        let pitch = 2.0 + rand::rng().random::<f32>() * 0.4;
        let world = entity.world.load_full();
        let position = entity.pos.load();
        carrier.play_entity_sound(Sound::EntityGenericBurn, 0.4, pitch);
        world.play_bedrock_named_sound(BEDROCK_FIRE_SOUND, &position, 0.4, pitch);
    }
}

#[derive(Default)]
struct ContactEffects {
    fire_damage: Vec<f32>,
    lava_contacts: usize,
    freezing: bool,
    before_extinguish: Vec<BeforeExtinguish>,
    extinguish: bool,
}

impl ContactEffects {
    fn record_block(
        &mut self,
        world: &Arc<World>,
        carrier: &dyn EntityBase,
        pos: BlockPos,
        block: &Block,
        state: &BlockState,
        server: &Server,
    ) {
        match block.id {
            BlockId::POWDER_SNOW => {
                carrier
                    .get_entity()
                    .slow_movement(state, Vector3::new(0.9, 1.5, 0.9));
                self.freezing = true;
                self.before_extinguish
                    .push(BeforeExtinguish::MeltPowderSnow(pos));
                self.extinguish = true;
            }
            BlockId::FIRE => self.fire_damage.push(1.0),
            BlockId::SOUL_FIRE => self.fire_damage.push(2.0),
            BlockId::LAVA_CAULDRON => self.lava_contacts += 1,
            BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                self.before_extinguish
                    .push(BeforeExtinguish::ConsumeCauldronLayer(pos, state.id));
                self.extinguish = true;
            }
            _ => world
                .block_registry
                .on_entity_collision(block, world, carrier, &pos, state, server),
        }
    }
}

enum BeforeExtinguish {
    MeltPowderSnow(BlockPos),
    ConsumeCauldronLayer(BlockPos, BlockStateId),
}

#[derive(Default)]
struct ContactEffectSteps {
    step: Option<usize>,
    current: ContactEffects,
    collected: Vec<ContactEffects>,
}

impl ContactEffectSteps {
    fn advance(&mut self, step: usize) {
        if self.step != Some(step) {
            self.finish_step();
            self.step = Some(step);
        }
    }

    fn finish_step(&mut self) {
        if !self.current.fire_damage.is_empty()
            || self.current.lava_contacts != 0
            || self.current.freezing
            || self.current.extinguish
        {
            self.collected.push(std::mem::take(&mut self.current));
        }
    }

    fn apply(
        mut self,
        world: &Arc<World>,
        carrier: &dyn EntityBase,
        is_current: impl Fn() -> bool,
    ) {
        self.finish_step();
        let entity = carrier.get_entity();
        for effects in self.collected {
            if !is_current() {
                break;
            }
            if effects.freezing {
                entity.is_in_powder_snow.store(true, Relaxed);
                let frozen = entity.get_frozen_ticks();
                if entity.can_freeze(carrier) && frozen != Entity::MAX_FROZEN_TICKS {
                    entity.set_frozen_ticks(frozen.saturating_add(1));
                    if !is_current() {
                        break;
                    }
                }
            }
            if (!effects.fire_damage.is_empty() || effects.lava_contacts != 0)
                && entity.get_frozen_ticks() != 0
            {
                entity.set_frozen_ticks(0);
                if !is_current() {
                    break;
                }
            }
            if !effects.fire_damage.is_empty() && !entity.is_fire_immune() {
                FireBlockBase::ignite_from_contact(carrier);
                for damage in effects.fire_damage {
                    if !is_current() {
                        return;
                    }
                    if !entity.is_invulnerable_to(&DamageType::IN_FIRE, None) {
                        entity.damage(carrier, damage, DamageType::IN_FIRE);
                    }
                }
            }
            if effects.lava_contacts != 0 && is_current() && !entity.is_fire_immune() {
                carrier.set_on_fire_for(15.0);
                for _ in 0..effects.lava_contacts {
                    if !is_current() {
                        return;
                    }
                    hurt_vehicle_in_lava(carrier);
                }
            }
            for effect in effects.before_extinguish {
                if !is_current() {
                    return;
                }
                match effect {
                    BeforeExtinguish::MeltPowderSnow(pos) => {
                        powder_snow::melt_if_burning(world, carrier, &pos);
                    }
                    BeforeExtinguish::ConsumeCauldronLayer(pos, state) => {
                        // Vehicles use Entity.mayInteract, which allows them to consume a layer.
                        if world.get_block_state_id(&pos) == state {
                            cauldron::consume_extinguishing_layer(
                                world,
                                &pos,
                                state,
                                carrier,
                                || is_current() && world.get_block_state_id(&pos) == state,
                            );
                        }
                    }
                }
            }
            if effects.extinguish && is_current() {
                entity.fire_ticks.fetch_min(0, Relaxed);
            }
        }
    }
}

impl Entity {
    /// Applies accepted vehicle crossings, with a fallback to the vehicle's last position.
    pub fn apply_recorded_vehicle_block_effects(
        &self,
        carrier: &dyn EntityBase,
        previous: Vector3<f64>,
    ) -> bool {
        let position = self.pos.load();
        let Some(history) = self.vehicle_movements.as_ref() else {
            return true;
        };
        let mut movements = {
            let mut history = history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut history.pending)
        };
        let from = movements.back().map_or(previous, |movement| movement.to);
        let epsilon = f64::from(BLOCK_INTERSECTION_EPSILON * BLOCK_INTERSECTION_EPSILON);
        if movements.is_empty() || (position - from).length_squared() > epsilon {
            movements.push_back(VehicleMovement {
                from,
                to: position,
                requested: None,
            });
        }
        self.apply_vehicle_block_effects(carrier, movements)
    }

    pub(super) fn apply_vehicle_block_effects_from(
        &self,
        carrier: &dyn EntityBase,
        from: Vector3<f64>,
    ) -> bool {
        self.apply_vehicle_block_effects(
            carrier,
            [VehicleMovement {
                from,
                to: self.pos.load(),
                requested: None,
            }],
        )
    }

    /// Applies contact at the current position and clears any recorded path.
    pub fn apply_vehicle_destination_block_effects(&self, carrier: &dyn EntityBase) -> bool {
        let current = self.apply_vehicle_block_effects_from(carrier, self.pos.load());
        self.clear_vehicle_movement();
        current
    }

    fn apply_vehicle_block_effects(
        &self,
        carrier: &dyn EntityBase,
        movements: impl IntoIterator<Item = VehicleMovement>,
    ) -> bool {
        if !self.is_affected_by_blocks() {
            return !self.is_removed();
        }
        let world = self.world.load_full();
        let Some(server) = world.server.upgrade() else {
            return true;
        };
        let position = self.pos.load();
        let bounds = self.bounding_box.load();
        let epoch = self.vehicle_motion_epoch();
        let is_current = || {
            self.is_affected_by_blocks()
                && self.pos.load() == position
                && self.vehicle_motion_epoch() == epoch
                && Arc::ptr_eq(&world, &self.world.load())
        };
        if self.on_ground.load(Relaxed) {
            let pos = self.get_pos_with_y_offset(0.2).0;
            let (block, state) = world.get_block_and_state(&pos);
            world
                .block_registry
                .on_entity_step(block, &world, carrier, &pos, state, false);
        }
        let previous_fire_ticks = self.fire_ticks.load(Relaxed);
        let was_on_fire = previous_fire_ticks > 0 && !self.is_fire_immune();
        let was_freezing = self.get_frozen_ticks() > 0;
        let mut visited = HashSet::new();
        let mut effects = ContactEffectSteps::default();
        for movement in movements {
            if !is_current() {
                break;
            }
            for_each_block_along_movement(
                bounds.shift(movement.from - position),
                movement.to - movement.from,
                movement.requested,
                |pos, precise, start, delta, step| {
                    if !is_current() || visited.contains(&pos) {
                        return;
                    }
                    let (block, state) = world.get_block_and_state(&pos);
                    if state.is_air() {
                        return;
                    }
                    let inside =
                        intersects_block_contact(&world, carrier, block, state, pos, start, delta);
                    let (fluid, fluid_state) = world.get_fluid_and_fluid_state(&pos);
                    let inside_fluid = fluid != &Fluid::EMPTY && {
                        let fluid_height =
                            f64::from(world.get_fluid_height(&pos, fluid, &fluid_state));
                        let fluid_box =
                            BoundingBox::new_array([0.0, 0.0, 0.0], [1.0, fluid_height, 1.0]);
                        intersects_swept(start, delta, fluid_box.at_pos(pos))
                    };
                    if !inside && !inside_fluid {
                        return;
                    }
                    visited.insert(pos);
                    effects.advance(step);
                    if inside && (block != &Block::BUBBLE_COLUMN || precise) {
                        effects
                            .current
                            .record_block(&world, carrier, pos, block, state, &server);
                    }
                    if inside_fluid && is_current() {
                        if fluid == &Fluid::WATER || fluid == &Fluid::FLOWING_WATER {
                            effects.current.extinguish = true;
                        } else if fluid == &Fluid::LAVA || fluid == &Fluid::FLOWING_LAVA {
                            effects.current.lava_contacts += 1;
                        } else {
                            world
                                .block_registry
                                .on_entity_collision_fluid(fluid, carrier);
                        }
                    }
                },
            );
        }
        effects.apply(&world, carrier, is_current);
        self.finish_vehicle_contact_fire(
            carrier,
            previous_fire_ticks,
            was_on_fire,
            was_freezing,
            is_current,
        )
    }

    fn finish_vehicle_contact_fire(
        &self,
        carrier: &dyn EntityBase,
        previous_fire_ticks: i32,
        was_on_fire: bool,
        was_freezing: bool,
        is_current: impl Fn() -> bool,
    ) -> bool {
        if !is_current() {
            return false;
        }
        if self.is_in_rain() {
            self.fire_ticks.fetch_min(0, Relaxed);
        }
        let fire_ticks = self.fire_ticks.load(Relaxed);
        let on_fire = fire_ticks > 0 && !self.is_fire_immune();
        if (was_on_fire && !on_fire) || (was_freezing && self.get_frozen_ticks() == 0) {
            let mut rng = rand::rng();
            let pitch = 1.6 + (rng.random::<f32>() - rng.random::<f32>()) * 0.4;
            let world = self.world.load_full();
            let position = self.pos.load();
            world.play_sound_fine(
                Sound::EntityGenericExtinguishFire,
                carrier.sound_source(),
                &position,
                0.7,
                pitch,
            );
            world.play_bedrock_named_sound(BEDROCK_FIRE_SOUND, &position, 0.7, pitch);
        }
        if !is_current() {
            return false;
        }
        let fire_ticks = self.fire_ticks.load(Relaxed);
        let on_fire = fire_ticks > 0 && !self.is_fire_immune();
        if !on_fire && fire_ticks <= previous_fire_ticks {
            self.fire_ticks.store(0, Relaxed);
        }
        self.set_on_fire(on_fire);
        is_current()
    }
}

fn intersects_block_contact(
    world: &World,
    carrier: &dyn EntityBase,
    block: &Block,
    state: &BlockState,
    pos: BlockPos,
    start: BoundingBox,
    delta: Vector3<f64>,
) -> bool {
    let shape = if block == &Block::POWDER_SNOW {
        powder_snow::inside_collision_shape_for_entity(carrier, &pos)
    } else {
        world
            .block_registry
            .get_inside_collision_shape(block, world, state, &pos)
    };
    let filled_cauldron = matches!(
        block.id,
        BlockId::LAVA_CAULDRON | BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON
    );
    (shape.min == Vector3::default() && shape.max == Vector3::new(1.0, 1.0, 1.0))
        || intersects_swept(start, delta, shape.at_pos(pos))
        || (filled_cauldron
            && state
                .get_block_collision_shapes_at(&pos)
                .any(|shape| intersects_swept(start, delta, shape.at_pos(pos))))
}

fn intersects_swept(start: BoundingBox, motion: Vector3<f64>, shape: BoundingBox) -> bool {
    let half_size = (start.max - start.min) * 0.5;
    let center = start.min + half_size;
    let to = center + motion;
    let expanded = shape.expand(
        half_size.x - EPSILON,
        half_size.y - EPSILON,
        half_size.z - EPSILON,
    );
    expanded.contains(center) || expanded.contains(to) || expanded.clip(center, to).is_some()
}

fn for_each_block_along_movement(
    bounds: BoundingBox,
    movement: Vector3<f64>,
    requested_movement: Option<Vector3<f64>>,
    mut visit: impl FnMut(BlockPos, bool, BoundingBox, Vector3<f64>, usize),
) {
    // Entity.checkInsideBlocks follows the collision axes, with 16 steps per movement.
    let mut bounds = bounds;
    let destination = bounds
        .shift(movement)
        .expand_all(-f64::from(BLOCK_INTERSECTION_EPSILON));
    let initial_bounds = bounds;
    let mut remaining_steps = MAX_BLOCK_ITERATIONS_ALONG_TRAVEL_PER_TICK;
    let Some(requested_movement) = requested_movement.filter(|_| movement != Vector3::default())
    else {
        remaining_steps = visit_movement_blocks(bounds, movement, remaining_steps, &mut visit);
        if remaining_steps == 0 {
            for pos in BlockPos::iterate(
                destination.min_block_pos(),
                BlockPos::floored_v(destination.max),
            ) {
                visit(
                    pos,
                    intersects_block(&destination, pos),
                    initial_bounds.shift(movement),
                    Vector3::default(),
                    0,
                );
            }
        }
        return;
    };
    for axis in axis_step_order(requested_movement) {
        let distance = movement.get_axis(axis);
        if distance == 0.0 {
            continue;
        }
        let mut offset = Vector3::default();
        offset.set_axis(axis, distance);
        remaining_steps = visit_movement_blocks(bounds, offset, remaining_steps, &mut visit);
        if remaining_steps == 0 {
            break;
        }
        bounds = bounds.shift(offset);
    }
    if remaining_steps == 0 {
        for pos in BlockPos::iterate(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
        ) {
            visit(
                pos,
                intersects_block(&destination, pos),
                initial_bounds.shift(movement),
                Vector3::default(),
                0,
            );
        }
    }
}

fn visit_movement_blocks(
    bounds: BoundingBox,
    movement: Vector3<f64>,
    remaining_steps: usize,
    visit: &mut impl FnMut(BlockPos, bool, BoundingBox, Vector3<f64>, usize),
) -> usize {
    let destination = bounds
        .shift(movement)
        .expand_all(-f64::from(BLOCK_INTERSECTION_EPSILON));
    // Vanilla sends the block-position precision flag on movement longer than the
    // deflated block length, or when the final bounds actually touch that block.
    let moved_far =
        movement.length_squared() > (1.0 - f64::from(BLOCK_INTERSECTION_EPSILON)).powi(2);
    let mut visit = |pos, step| {
        visit(
            pos,
            moved_far || intersects_block(&destination, pos),
            bounds,
            movement,
            step,
        );
    };
    if movement.length_squared()
        < f64::from(BLOCK_INTERSECTION_EPSILON * BLOCK_INTERSECTION_EPSILON)
    {
        for pos in BlockPos::iterate(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
        ) {
            visit(pos, 0);
        }
        return remaining_steps - 1;
    }
    let mut visited = HashSet::new();
    let mut last_step = 0;
    let mut visit_step = |pos, iteration| {
        if !visited.insert(pos) {
            return true;
        }
        if iteration >= remaining_steps {
            return false;
        }
        last_step = iteration;
        visit(pos, iteration);
        true
    };
    let start = destination.shift(movement * -1.0);
    for pos in between_corners_in_direction(
        start.min_block_pos(),
        BlockPos::floored_v(start.max),
        movement,
    ) {
        visit_step(pos, 0);
    }
    if let Some(steps) = add_collisions_along_travel(destination, movement, &mut visit_step) {
        for pos in between_corners_in_direction(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
            movement,
        ) {
            if !visit_step(pos, steps + 1) {
                break;
            }
        }
    }
    remaining_steps - (last_step + 1)
}

fn intersects_block(bounds: &BoundingBox, pos: BlockPos) -> bool {
    bounds.intersects(&BoundingBox::full_block().at_pos(pos))
}

fn add_collisions_along_travel(
    destination: BoundingBox,
    movement: Vector3<f64>,
    visit: &mut impl FnMut(BlockPos, usize) -> bool,
) -> Option<usize> {
    let corner = furthest_corner(movement);
    let size = destination.max - destination.min;
    let mut to = (destination.min + destination.max) * 0.5;
    for component in Axis::all() {
        to.set_axis(
            component,
            to.get_axis(component) + size.get_axis(component) * 0.5 * corner.get_axis(component),
        );
    }
    let from = to - movement;
    let mut cell = BlockPos::floored_v(from);
    let (step, interval, mut next) = traversal_steps(from, movement);
    let mut steps = 0;
    while next.x <= 1.0 || next.y <= 1.0 || next.z <= 1.0 {
        let axis = next_traversal_axis(next);
        cell.0
            .set_axis(axis, cell.0.get_axis(axis) + step.get_axis(axis));
        next.set_axis(axis, next.get_axis(axis) + interval.get_axis(axis));
        let Some(point) = BoundingBox::from_block(&cell).clip(from, to) else {
            continue;
        };
        steps += 1;
        let opposite = opposite_corner(point, cell, size, corner);
        for pos in between_corners_in_direction(cell, opposite, movement) {
            if !visit(pos, steps) {
                return None;
            }
        }
    }
    Some(steps)
}

fn furthest_corner(movement: Vector3<f64>) -> Vector3<f64> {
    let sign = |value| if value >= 0.0 { 1.0 } else { -1.0 };
    let (x, y, z) = (sign(movement.x), sign(movement.y), sign(movement.z));
    if movement.x.abs() <= movement.y.abs() && movement.x.abs() <= movement.z.abs() {
        Vector3::new(-x, -z, y)
    } else if movement.y.abs() <= movement.z.abs() {
        Vector3::new(z, -y, -x)
    } else {
        Vector3::new(-y, x, -z)
    }
}

fn traversal_steps(
    from: Vector3<f64>,
    movement: Vector3<f64>,
) -> (Vector3<i32>, Vector3<f64>, Vector3<f64>) {
    let mut steps = Vector3::default();
    let mut intervals = Vector3::default();
    let mut next = Vector3::default();
    for axis in Axis::all() {
        let distance = movement.get_axis(axis);
        let step = if distance == 0.0 {
            0
        } else if distance > 0.0 {
            1
        } else {
            -1
        };
        let interval = if step == 0 {
            f64::MAX
        } else {
            f64::from(step) / distance
        };
        let fraction = from.get_axis(axis) - from.get_axis(axis).floor();
        steps.set_axis(axis, step);
        intervals.set_axis(axis, interval);
        next.set_axis(
            axis,
            interval * if step > 0 { 1.0 - fraction } else { fraction },
        );
    }
    (steps, intervals, next)
}

fn next_traversal_axis(next: Vector3<f64>) -> Axis {
    if next.x < next.y {
        if next.x < next.z { Axis::X } else { Axis::Z }
    } else if next.y < next.z {
        Axis::Y
    } else {
        Axis::Z
    }
}

fn opposite_corner(
    point: Vector3<f64>,
    cell: BlockPos,
    size: Vector3<f64>,
    corner: Vector3<f64>,
) -> BlockPos {
    let mut opposite = Vector3::default();
    let epsilon = f64::from(BLOCK_INTERSECTION_EPSILON);
    for axis in Axis::all() {
        let start = f64::from(cell.0.get_axis(axis));
        opposite.set_axis(
            axis,
            point
                .get_axis(axis)
                .clamp(start + epsilon, start + 1.0 - epsilon)
                - size.get_axis(axis) * corner.get_axis(axis),
        );
    }
    BlockPos::floored_v(opposite)
}

fn axis_step_order(movement: Vector3<f64>) -> [Axis; 3] {
    if movement.x.abs() < movement.z.abs() {
        [Axis::Y, Axis::Z, Axis::X]
    } else {
        Axis::all()
    }
}

fn between_corners_in_direction(
    first: BlockPos,
    second: BlockPos,
    movement: Vector3<f64>,
) -> impl Iterator<Item = BlockPos> {
    let mut min = Vector3::default();
    let mut max = Vector3::default();
    for axis in Axis::all() {
        min.set_axis(axis, first.0.get_axis(axis).min(second.0.get_axis(axis)));
        max.set_axis(axis, first.0.get_axis(axis).max(second.0.get_axis(axis)));
    }
    let [slow, middle, fast] = axis_step_order(movement);
    let size = max - min;
    let end = BlockPos::new(
        size.get_axis(fast),
        size.get_axis(middle),
        size.get_axis(slow),
    );
    BlockPos::iterate(BlockPos::ZERO, end).map(move |offset| {
        let mut pos = BlockPos::ZERO;
        for (axis, offset) in [(fast, offset.0.x), (middle, offset.0.y), (slow, offset.0.z)] {
            pos.0.set_axis(
                axis,
                if movement.get_axis(axis) < 0.0 {
                    max.get_axis(axis) - offset
                } else {
                    min.get_axis(axis) + offset
                },
            );
        }
        pos
    })
}
