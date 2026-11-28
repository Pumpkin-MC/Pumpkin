use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{Entity, EntityBase, player::Player};

pub(super) struct FurnaceMinecart {
    fuel: AtomicI32,
    push: AtomicCell<Vector3<f64>>,
}

impl FurnaceMinecart {
    const FUEL_PER_ITEM: i32 = 3_600;
    const MAX_FUEL: i32 = 32_000;

    pub(super) const fn new() -> Self {
        Self {
            fuel: AtomicI32::new(0),
            push: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    fn set_fueled(entity: &Entity, fueled: bool) {
        entity.set_synced_data(
            pumpkin_data::tracked_data::furnace_minecart::ID_FUEL,
            fueled,
        );
    }

    pub(super) fn tick(&self, entity: &Entity) {
        let fuel = self.fuel.load(Ordering::Relaxed);
        if fuel <= 0 {
            self.push.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        }

        let remaining = fuel - 1;
        self.fuel.store(remaining, Ordering::Relaxed);
        if remaining == 0 {
            self.push.store(Vector3::new(0.0, 0.0, 0.0));
            Self::set_fueled(entity, false);
        } else if rand::rng().random_range(0..4) == 0 {
            let mut pos = entity.pos.load();
            pos.y += 0.8;
            entity.world.load().spawn_particle(
                pos,
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::LargeSmoke,
            );
        }
    }

    pub(super) fn velocity(&self, velocity: Vector3<f64>, in_water: bool) -> Vector3<f64> {
        let mut push = self.push.load();
        if push.length_squared() <= 1.0e-7 {
            return velocity.multiply(0.98, 0.0, 0.98);
        }
        if push.horizontal_length_squared() > 1.0e-4 && velocity.horizontal_length_squared() > 0.001
        {
            let projection = velocity * push.dot(&velocity) * velocity.length_squared().recip();
            let length = projection.length();
            // Java Vec3.normalize compares against the float cutoff, then divides each axis.
            push = if length < f64::from(1.0e-5f32) {
                Vector3::default()
            } else {
                (projection / length) * push.length()
            };
            self.push.store(push);
        }
        let next = velocity.multiply(0.8, 0.0, 0.8) + push;
        if in_water { next * 0.1 } else { next }
    }

    pub(super) fn interact(
        &self,
        entity: &Entity,
        player: &Player,
        item_stack: &mut ItemStack,
    ) -> bool {
        let fuel = self.fuel.load(Ordering::Relaxed);
        if item_stack
            .get_item()
            .has_tag(&tag::Item::MINECRAFT_FURNACE_MINECART_FUEL)
            && fuel <= Self::MAX_FUEL - Self::FUEL_PER_ITEM
        {
            self.fuel
                .store(fuel + Self::FUEL_PER_ITEM, Ordering::Relaxed);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            if fuel <= 0 {
                Self::set_fueled(entity, true);
            }
            let cart_pos = entity.pos.load();
            let player_pos = player.get_entity().pos.load();
            self.push.store(Vector3::new(
                cart_pos.x - player_pos.x,
                0.0,
                cart_pos.z - player_pos.z,
            ));
        }
        true
    }

    pub(super) fn init_data_tracker(&self, entity: &Entity) {
        Self::set_fueled(entity, self.fuel.load(Ordering::Relaxed) > 0);
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        let push = self.push.load();
        nbt.put_double("PushX", push.x);
        nbt.put_double("PushZ", push.z);
        nbt.put_short("Fuel", self.fuel.load(Ordering::Relaxed) as i16);
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        self.push.store(Vector3::new(
            nbt.get_double("PushX").unwrap_or(0.0),
            0.0,
            nbt.get_double("PushZ").unwrap_or(0.0),
        ));
        self.fuel.store(
            i32::from(nbt.get_short("Fuel").unwrap_or(0)),
            Ordering::Relaxed,
        );
    }
}
