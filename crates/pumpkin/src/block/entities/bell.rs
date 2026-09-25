use crate::block::entities::BlockEntity;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::sync::Arc;

pub struct BellBlockEntity {
    pub position: BlockPos,
    pub last_side_hit: AtomicCell<Option<HorizontalFacing>>,
    pub ring_ticks: AtomicCell<i32>,
    pub ringing: AtomicCell<bool>,
    resonating: AtomicCell<bool>,
    resonance_triggered: AtomicCell<bool>,
    resonate_time: AtomicCell<i32>,
}

impl BellBlockEntity {
    pub const ID: &'static str = "minecraft:bell";
    const RAIDER_DETECTION_RANGE: f64 = 32.0;
    const RAIDER_GLOWING_RANGE: f64 = 48.0;
    const RESONANCE_TICKS: i32 = 40;
    const GLOWING_TICKS: i32 = 60;

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_side_hit: AtomicCell::new(None),
            ring_ticks: AtomicCell::new(0),
            resonate_time: AtomicCell::new(0),
            resonating: AtomicCell::new(false),
            resonance_triggered: AtomicCell::new(false),
            ringing: AtomicCell::new(false),
        }
    }
    pub fn activate(&self, direction: HorizontalFacing) {
        self.last_side_hit.store(Some(direction));
        if self.ringing.load() {
            self.ring_ticks.store(0);
        } else {
            self.ringing.store(true);
            self.resonance_triggered.store(false);
        }
    }

    fn raider_search_box(position: &BlockPos) -> BoundingBox {
        BoundingBox::from_block(position).expand_all(Self::RAIDER_DETECTION_RANGE)
    }

    fn glowing_search_box(position: &BlockPos) -> BoundingBox {
        BoundingBox::from_block(position).expand_all(Self::RAIDER_GLOWING_RANGE)
    }

    fn entity_within_range(
        position: &BlockPos,
        entity_pos: &pumpkin_util::math::vector3::Vector3<f64>,
        range: f64,
    ) -> bool {
        position
            .to_centered_f64()
            .squared_distance_to_vec(entity_pos)
            <= range * range
    }

    const fn resonance_effect() -> Effect {
        Effect {
            effect_type: &StatusEffect::GLOWING,
            duration: Self::GLOWING_TICKS,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    const fn resonance_finished(resonate_time: i32) -> bool {
        resonate_time >= Self::RESONANCE_TICKS
    }

    const fn should_start_resonance(
        ring_ticks: i32,
        resonate_time: i32,
        resonating: bool,
        resonance_triggered: bool,
    ) -> bool {
        ring_ticks >= 5 && resonate_time == 0 && !resonating && !resonance_triggered
    }

    fn raiders_hear_bell(&self, world: &World) -> bool {
        world
            .get_entities_at_box(&Self::raider_search_box(&self.position))
            .iter()
            .any(|entity| {
                entity.get_entity().is_alive()
                    && Self::entity_within_range(
                        &self.position,
                        &entity.get_entity().pos.load(),
                        Self::RAIDER_DETECTION_RANGE,
                    )
                    && entity
                        .get_mob()
                        .is_some_and(|mob| mob.as_raider().is_some())
            })
    }

    fn make_raiders_glowing(&self, world: &World) {
        let effect = Self::resonance_effect();
        for entity in world.get_entities_at_box(&Self::glowing_search_box(&self.position)) {
            if entity.get_entity().is_alive()
                && Self::entity_within_range(
                    &self.position,
                    &entity.get_entity().pos.load(),
                    Self::RAIDER_GLOWING_RANGE,
                )
                && entity
                    .get_mob()
                    .is_some_and(|mob| mob.as_raider().is_some())
                && let Some(living) = entity.get_living_entity()
            {
                living.add_effect(effect.clone());
            }
        }
    }
}

impl BlockEntity for BellBlockEntity {
    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self::new(position)
    }

    fn tick(&self, world: &Arc<World>) {
        if self.ringing.load() {
            self.ring_ticks.fetch_add(1);
        }
        if self.ring_ticks.load() >= 50 {
            self.ringing.store(false);
            self.ring_ticks.store(0);
            self.resonance_triggered.store(false);
        }
        if Self::should_start_resonance(
            self.ring_ticks.load(),
            self.resonate_time.load(),
            self.resonating.load(),
            self.resonance_triggered.load(),
        ) && self.raiders_hear_bell(world)
        {
            self.resonance_triggered.store(true);
            self.resonating.store(true);
            world.play_sound_fine(
                Sound::BlockBellResonate,
                SoundCategory::Blocks,
                &self.position.to_f64(),
                1.0,
                1.0,
            );
        }

        if self.resonating.load() {
            let resonate_time = self.resonate_time.fetch_add(1) + 1;
            if Self::resonance_finished(resonate_time) {
                self.resonating.store(false);
                self.resonate_time.store(0);
                self.make_raiders_glowing(world);
            }
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::effect::StatusEffect;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn search_boxes_and_resonance_effect_match_vanilla_ranges() {
        let position = BlockPos::new(10, 20, 30);
        let raider_box = BellBlockEntity::raider_search_box(&position);
        let glowing_box = BellBlockEntity::glowing_search_box(&position);

        assert_eq!(raider_box.min.x, -22.0);
        assert_eq!(raider_box.max.x, 43.0);
        assert_eq!(glowing_box.min.x, -38.0);
        assert_eq!(glowing_box.max.x, 59.0);

        let effect = BellBlockEntity::resonance_effect();
        assert_eq!(effect.effect_type.id, StatusEffect::GLOWING.id);
        assert_eq!(effect.duration, 60);
    }

    #[test]
    fn raider_range_is_spherical() {
        let position = BlockPos::new(0, 0, 0);
        assert!(BellBlockEntity::entity_within_range(
            &position,
            &Vector3::new(32.5, 0.5, 0.5),
            BellBlockEntity::RAIDER_DETECTION_RANGE,
        ));
        assert!(!BellBlockEntity::entity_within_range(
            &position,
            &Vector3::new(32.5, 32.5, 0.5),
            BellBlockEntity::RAIDER_DETECTION_RANGE,
        ));
    }

    #[test]
    fn resonance_finishes_at_forty_ticks() {
        assert!(!BellBlockEntity::resonance_finished(39));
        assert!(BellBlockEntity::resonance_finished(40));
    }

    #[test]
    fn resonance_latch_blocks_a_second_trigger_during_one_ring() {
        assert!(BellBlockEntity::should_start_resonance(5, 0, false, false));
        assert!(!BellBlockEntity::should_start_resonance(5, 0, false, true));
    }

    #[test]
    fn starting_a_new_ring_clears_the_resonance_latch() {
        let bell = BellBlockEntity::new(BlockPos::new(0, 0, 0));
        bell.resonance_triggered.store(true);

        bell.activate(HorizontalFacing::North);

        assert!(!bell.resonance_triggered.load());
    }
}
