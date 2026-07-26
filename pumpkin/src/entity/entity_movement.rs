use super::{Entity, EntityBase};
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::vector3::Axis;
use pumpkin_util::math::{
    boundingbox::BoundingBox, get_section_cord, position::BlockPos, vector2::Vector2,
    vector3::Vector3, wrap_degrees,
};
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, Relaxed};

impl Entity {
    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, new_position: Vector3<f64>) {
        let pos = self.pos.load();
        if pos != new_position {
            self.pos.store(new_position);
            self.bounding_box.store(BoundingBox::new_from_pos(
                new_position.x,
                new_position.y,
                new_position.z,
                &self.entity_dimension.load(),
            ));

            let floor_x = new_position.x.floor() as i32;
            let floor_y = new_position.y.floor() as i32;
            let floor_z = new_position.z.floor() as i32;

            let block_pos = self.block_pos.load();
            let block_pos_vec = block_pos.0;
            if floor_x != block_pos_vec.x
                || floor_y != block_pos_vec.y
                || floor_z != block_pos_vec.z
            {
                let new_block_pos = Vector3::new(floor_x, floor_y, floor_z);
                self.block_pos.store(BlockPos(new_block_pos));

                let chunk_pos = self.chunk_pos.load();
                if get_section_cord(floor_x) != chunk_pos.x
                    || get_section_cord(floor_z) != chunk_pos.y
                {
                    self.chunk_pos.store(Vector2::new(
                        get_section_cord(new_block_pos.x),
                        get_section_cord(new_block_pos.z),
                    ));
                }
            }
        }
    }

    /// Returns entity rotation as vector
    pub fn rotation(&self) -> Vector3<f32> {
        let pitch_rad = self.pitch.load().to_radians();
        let yaw_rad = -self.yaw.load().to_radians();

        let cos_yaw = yaw_rad.cos();
        let sin_yaw = yaw_rad.sin();
        let cos_pitch = pitch_rad.cos();
        let sin_pitch = pitch_rad.sin();

        Vector3::new(sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch)
    }

    /// Changes this entity's pitch and yaw to look at target
    pub fn look_at(&self, target: Vector3<f64>) {
        let position = self.pos.load();
        let delta = target.sub(&position);
        let root = delta.x.hypot(delta.z);
        let pitch = wrap_degrees((-delta.y.atan2(root) as f32).to_degrees());
        let yaw = wrap_degrees((delta.z.atan2(delta.x) as f32).to_degrees() - 90.0);
        self.pitch.store(pitch);
        self.yaw.store(yaw);
    }

    /// Returns the block position of the block the (non-player) entity is standing on, if any.
    pub fn get_supporting_block_pos(&self) -> Option<BlockPos> {
        // Check if the entity is on the ground
        if !self.on_ground.load(Ordering::Relaxed) {
            return None;
        }

        self.supporting_block_pos.load()
    }

    fn resolve_movement_against_collisions(
        bounding_box: BoundingBox,
        movement: Vector3<f64>,
        collisions: &[BoundingBox],
    ) -> Vector3<f64> {
        let mut resolved = Vector3::default();

        for axis in Axis::all() {
            let requested = movement.get_axis(axis);
            if requested == 0.0 {
                continue;
            }

            let mut axis_movement = Vector3::default();
            axis_movement.set_axis(axis, requested);
            let current_box = bounding_box.shift(resolved);
            let mut max_time = 1.0;

            for collision in collisions {
                if let Some(collision_time) =
                    current_box.calculate_collision_time(collision, axis_movement, axis, max_time)
                {
                    max_time = collision_time;
                }
            }

            resolved.set_axis(axis, requested * max_time);
        }

        resolved
    }

    fn find_supporting_block(
        bounding_box: BoundingBox,
        vertical_movement: f64,
        collisions: &[BoundingBox],
        collision_sources: &[BlockPos],
    ) -> Option<BlockPos> {
        if vertical_movement >= 0.0 {
            return None;
        }

        let axis_movement = Vector3 {
            y: vertical_movement,
            ..Default::default()
        };
        let mut max_time = 1.0;
        let mut supporting_block = None;

        for (collision, source) in collisions.iter().zip(collision_sources) {
            if let Some(collision_time) =
                bounding_box.calculate_collision_time(collision, axis_movement, Axis::Y, max_time)
            {
                max_time = collision_time;
                supporting_block = Some(*source);
            }
        }

        supporting_block
    }

    fn find_step_supporting_block(
        bounding_box: BoundingBox,
        movement: Vector3<f64>,
        collisions: &[BoundingBox],
        collision_sources: &[BlockPos],
    ) -> Option<BlockPos> {
        let final_box = bounding_box.shift(movement);
        let mut support = None;
        let mut support_height = f64::NEG_INFINITY;

        for (collision, source) in collisions.iter().zip(collision_sources) {
            let touches_feet = (collision.max.y - final_box.min.y).abs() <= 1.0e-7;
            let overlaps_xz = collision.min.x < final_box.max.x
                && collision.max.x > final_box.min.x
                && collision.min.z < final_box.max.z
                && collision.max.z > final_box.min.z;
            if touches_feet && overlaps_xz && collision.max.y > support_height {
                support = Some(*source);
                support_height = collision.max.y;
            }
        }

        support
    }

    #[expect(clippy::float_cmp)]
    async fn adjust_movement_for_collisions(
        &self,
        movement: Vector3<f64>,
        caller: &dyn EntityBase,
    ) -> Vector3<f64> {
        if movement.length_squared() == 0.0 {
            return movement;
        }

        let was_on_ground = self.on_ground.load(Ordering::SeqCst);
        let previous_supporting_block = self.supporting_block_pos.load();
        self.on_ground.store(false, Ordering::SeqCst);
        self.supporting_block_pos.store(None);
        self.horizontal_collision.store(false, Ordering::SeqCst);

        let bounding_box = self.bounding_box.load();
        let world = self.world.load_full();
        let (collisions, block_positions) = world
            .get_block_collisions(bounding_box.stretch(movement), caller)
            .await;

        let mut collision_sources = Vec::with_capacity(collisions.len());
        for (end, position) in block_positions {
            collision_sources.resize(end, position);
        }

        let mut adjusted_movement =
            Self::resolve_movement_against_collisions(bounding_box, movement, &collisions);
        let mut supporting_block =
            Self::find_supporting_block(bounding_box, movement.y, &collisions, &collision_sources);
        let mut on_ground = if movement.y == 0.0 {
            was_on_ground
        } else {
            supporting_block.is_some()
        };
        if movement.y == 0.0 {
            supporting_block = previous_supporting_block;
        }

        let mut horizontal_collision =
            movement.x != adjusted_movement.x || movement.z != adjusted_movement.z;
        let max_step_height = caller.get_living_entity().map_or(0.0, |living| {
            living.get_attribute_value(&Attributes::STEP_HEIGHT)
        });

        if max_step_height > 0.0 && (on_ground || was_on_ground) && horizontal_collision {
            let grounded_box = if supporting_block.is_some() {
                bounding_box.shift(Vector3::new(0.0, adjusted_movement.y, 0.0))
            } else {
                bounding_box
            };
            let mut step_search_box =
                grounded_box.expand_towards(movement.x, max_step_height, movement.z);
            if supporting_block.is_none() {
                step_search_box = step_search_box.expand_towards(0.0, -1.0e-5, 0.0);
            }

            let (step_collisions, step_positions) =
                world.get_block_collisions(step_search_box, caller).await;
            let mut step_sources = Vec::with_capacity(step_collisions.len());
            for (end, position) in step_positions {
                step_sources.resize(end, position);
            }

            let mut step_heights = Vec::with_capacity(step_collisions.len() * 2);
            for collision in &step_collisions {
                for height in [collision.min.y, collision.max.y] {
                    let relative_height = height - grounded_box.min.y;
                    if relative_height > 0.0
                        && relative_height <= max_step_height
                        && (relative_height - adjusted_movement.y).abs() > 1.0e-7
                    {
                        step_heights.push(relative_height);
                    }
                }
            }
            step_heights.sort_by(f64::total_cmp);
            step_heights.dedup_by(|a, b| (*a - *b).abs() <= 1.0e-7);

            for step_height in step_heights {
                let stepped = Self::resolve_movement_against_collisions(
                    grounded_box,
                    Vector3::new(movement.x, step_height, movement.z),
                    &step_collisions,
                );
                if stepped.horizontal_length_squared()
                    > adjusted_movement.horizontal_length_squared()
                {
                    let distance_to_ground = bounding_box.min.y - grounded_box.min.y;
                    adjusted_movement =
                        Vector3::new(stepped.x, stepped.y - distance_to_ground, stepped.z);
                    supporting_block = Self::find_step_supporting_block(
                        grounded_box,
                        stepped,
                        &step_collisions,
                        &step_sources,
                    );
                    on_ground = true;
                    horizontal_collision =
                        movement.x != adjusted_movement.x || movement.z != adjusted_movement.z;
                    break;
                }
            }
        }

        self.on_ground.store(on_ground, Ordering::SeqCst);
        self.supporting_block_pos.store(supporting_block);
        self.horizontal_collision
            .store(horizontal_collision, Ordering::SeqCst);

        adjusted_movement
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    /// `LivingEntity.takeKnockback()` — caller must already scale `strength` by
    /// `(1 - knockbackResistance)` (see damage path in `living.rs`).
    pub fn apply_knockback(&self, strength: f64, mut x: f64, mut z: f64) {
        if strength <= 0.0 {
            return;
        }

        self.velocity_dirty.store(true, Ordering::SeqCst);

        // This has some vanilla magic

        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;

            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;

        let velocity = self.velocity.load();

        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    // Part of LivingEntity.tickMovement() in yarn

    pub fn check_zero_velo(&self) {
        let mut motion = self.velocity.load();

        if self.entity_type == &EntityType::PLAYER {
            if motion.horizontal_length_squared() < 9.0E-6 {
                motion.x = 0.0;

                motion.z = 0.0;
            }
        } else {
            if motion.x.abs() < 0.003 {
                motion.x = 0.0;
            }

            if motion.z.abs() < 0.003 {
                motion.z = 0.0;
            }
        }

        if motion.y.abs() < 0.003 {
            motion.y = 0.0;
        }

        self.velocity.store(motion);
    }

    pub(super) fn get_pos_with_y_offset(
        &self,
        offset: f64,
    ) -> (
        BlockPos,
        Option<&'static Block>,
        Option<&'static BlockState>,
    ) {
        if let Some(mut supporting_block) = self.supporting_block_pos.load() {
            if offset > 1.0e-5 {
                let (block, state) = self.world.load().get_block_and_state(&supporting_block);

                // Match Entity#getOnPos: fences are kept for the small movement
                // offset, while walls and fence gates always use their support.
                if (offset <= 0.5 && block.has_tag(&tag::Block::MINECRAFT_FENCES))
                    || block.has_tag(&tag::Block::MINECRAFT_WALLS)
                    || block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES)
                {
                    return (supporting_block, Some(block), Some(state));
                }

                supporting_block.0.y = (self.pos.load().y - offset).floor() as i32;

                return (supporting_block, Some(block), Some(state));
            }

            return (supporting_block, None, None);
        }

        let mut block_pos = self.block_pos.load();

        block_pos.0.y = (self.pos.load().y - offset).floor() as i32;

        (block_pos, None, None)
    }

    pub(super) fn get_block_with_y_offset(
        &self,
        offset: f64,
    ) -> (BlockPos, &'static Block, &'static BlockState) {
        let (pos, block, state) = self.get_pos_with_y_offset(offset);

        if let (Some(b), Some(s)) = (block, state) {
            (pos, b, s)
        } else {
            let (b, s) = self.world.load().get_block_and_state(&pos);

            (pos, b, s)
        }
    }

    // Entity.updateVelocity in yarn

    pub(super) fn update_velocity_from_input(&self, movement_input: Vector3<f64>, speed: f64) {
        let final_input = self.movement_input_to_velocity(movement_input, speed);

        self.velocity.store(self.velocity.load() + final_input);
    }

    // Entity.movementInputToVelocity in yarn

    fn movement_input_to_velocity(&self, movement_input: Vector3<f64>, speed: f64) -> Vector3<f64> {
        let yaw = f64::from(self.yaw.load()).to_radians();

        let dist = movement_input.length_squared();

        if dist < 1.0e-7 {
            return Vector3::default();
        }

        let input = if dist > 1.0 {
            movement_input.normalize() * speed
        } else {
            movement_input * speed
        };

        let sin = yaw.sin();

        let cos = yaw.cos();

        Vector3::new(
            input.x.mul_add(cos, -(input.z * sin)),
            input.y,
            input.z.mul_add(cos, input.x * sin),
        )
    }

    #[expect(clippy::float_cmp)]
    fn get_velocity_multiplier(&self) -> f32 {
        let block = self.world.load().get_block(&self.block_pos.load());

        let multiplier = block.velocity_multiplier;

        if multiplier != 1.0 || block == &Block::WATER || block == &Block::BUBBLE_COLUMN {
            multiplier
        } else {
            let (_pos, block, _state) = self.get_block_with_y_offset(0.500_001);

            block.velocity_multiplier
        }
    }

    #[expect(clippy::float_cmp)]
    pub(super) fn get_jump_velocity_multiplier(&self) -> f32 {
        let f = self
            .world
            .load()
            .get_block(&self.block_pos.load())
            .jump_velocity_multiplier;

        let g = self
            .get_block_with_y_offset(0.500_001)
            .1
            .jump_velocity_multiplier;

        if f == 1f32 { g } else { f }
    }

    pub fn move_pos(&self, delta: Vector3<f64>) {
        self.set_pos(self.pos.load() + delta);
    }

    // Move by a delta, adjust for collisions, and send

    // Does not send movement. That must be done separately
    pub async fn move_entity<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        mut motion: Vector3<f64>,
    ) {
        if caller.get_player().is_some() {
            return;
        }

        if self.no_clip.load(Ordering::Relaxed) {
            self.move_pos(motion);

            return;
        }

        let movement_multiplier = self.movement_multiplier.swap(Vector3::default());

        if movement_multiplier.length_squared() > 1.0e-7 {
            motion = motion.multiply(
                movement_multiplier.x,
                movement_multiplier.y,
                movement_multiplier.z,
            );

            self.velocity.store(Vector3::default());
        }

        let final_move = self
            .adjust_movement_for_collisions(motion, caller.as_ref())
            .await;

        self.move_pos(final_move);

        let velocity_multiplier = f64::from(self.get_velocity_multiplier());

        self.velocity.store(final_move * velocity_multiplier);

        if let Some(living) = caller.get_living_entity() {
            living
                .fall(
                    caller.clone(),
                    final_move.y,
                    self.on_ground.load(Ordering::SeqCst),
                    false,
                )
                .await;
        }

        if motion.y != final_move.y {
            let world = self.world.load();
            let block = self.get_block_with_y_offset(0.2).1;
            world
                .block_registry
                .update_entity_movement_after_fall_on(block, caller.as_ref())
                .await;
        }
    }

    pub fn push_out_of_blocks(&self, center_pos: Vector3<f64>) {
        let block_pos = BlockPos::floored_v(center_pos);

        let delta = center_pos.sub(&block_pos.0.to_f64());

        let mut min_dist = f64::MAX;

        let mut direction = BlockDirection::Up;

        for dir in BlockDirection::all() {
            if dir == BlockDirection::Down {
                continue;
            }

            let offset = dir.to_offset();

            if self
                .world
                .load()
                .get_block_state(&block_pos.offset(offset))
                .is_full_cube()
            {
                continue;
            }

            let component = delta.get_axis(dir.to_axis().into());

            let dist = if dir.positive() {
                1.0 - component
            } else {
                component
            };

            if dist < min_dist {
                min_dist = dist;

                direction = dir;
            }
        }

        let amplitude = rand::random::<f64>().mul_add(0.2, 0.1);

        let axis = direction.to_axis().into();

        let sign = if direction.positive() { 1.0 } else { -1.0 };

        let mut velo = self.velocity.load();

        velo = velo * 0.75;

        velo.set_axis(axis, sign * amplitude);

        self.velocity.store(velo);
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    ///
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    pub fn knockback(&self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    pub async fn slow_movement(&self, state: &BlockState, multiplier: Vector3<f64>) {
        match self.entity_type.id {
            v if v == EntityType::PLAYER.id => {
                if let Some(player_entity) = self.get_player()
                    && player_entity.is_flying().await
                {
                    return;
                }
            }
            v if (v == EntityType::SPIDER.id || v == EntityType::CAVE_SPIDER.id)
                && Block::from_state_id(state.id).id == Block::COBWEB.id =>
            {
                return;
            }
            v if v == EntityType::WITHER.id => {
                return;
            }
            _ => {}
        }
        if let Some(living) = self.get_living_entity() {
            living.fall_distance.store(0f32);
        }
        self.movement_multiplier.store(multiplier);
    }
}
