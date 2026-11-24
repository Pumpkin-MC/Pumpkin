use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use super::behavior::normalize;
use crate::entity::{Entity, EntityBase, player::Player};

/// Vanilla `MinecartFurnace`.
pub(super) struct FurnaceMinecart {
    fuel: AtomicI32,
    push: AtomicCell<Vector3<f64>>,
}

impl FurnaceMinecart {
    const FUEL_TICKS_PER_ITEM: i32 = 3_600;
    const MAX_FUEL_TICKS: i32 = 32_000;

    pub(super) const fn new() -> Self {
        Self {
            fuel: AtomicI32::new(0),
            push: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    fn set_has_fuel(entity: &Entity, fueled: bool) {
        entity.set_synced_data(
            pumpkin_data::tracked_data::furnace_minecart::ID_FUEL,
            fueled,
        );
    }

    /// Vanilla `MinecartFurnace.tick` after `super.tick()`. Smoke is client side.
    pub(super) fn tick(&self, entity: &Entity) {
        let mut fuel = self.fuel.load(Ordering::Relaxed);
        if fuel > 0 {
            fuel -= 1;
            self.fuel.store(fuel, Ordering::Relaxed);
        }
        if fuel <= 0 {
            self.push.store(Vector3::new(0.0, 0.0, 0.0));
        }
        Self::set_has_fuel(entity, fuel > 0);
    }

    /// Vanilla `MinecartFurnace.applyNaturalSlowdown` before `super`.
    pub(super) fn slowdown(&self, entity: &Entity, velocity: Vector3<f64>) -> Vector3<f64> {
        let push = self.push.load();
        if push.length_squared() <= 1.0e-7 {
            return velocity.multiply(0.98, 0.0, 0.98);
        }
        // `calculateNewPushAlong`
        let push = if push.horizontal_length_squared() > 1.0e-4
            && velocity.horizontal_length_squared() > 0.001
        {
            let projected = velocity * (push.dot(&velocity) * (1.0 / velocity.length_squared()));
            normalize(projected) * push.length()
        } else {
            push
        };
        self.push.store(push);
        let next = velocity.multiply(0.8, 0.0, 0.8).add(&push);
        if entity.is_in_water() {
            next * 0.1
        } else {
            next
        }
    }

    /// Vanilla `MinecartFurnace.interact` + `addFuel`.
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
            && fuel + Self::FUEL_TICKS_PER_ITEM <= Self::MAX_FUEL_TICKS
        {
            let fuel = fuel + Self::FUEL_TICKS_PER_ITEM;
            self.fuel.store(fuel, Ordering::Relaxed);
            if fuel > 0 {
                let away = entity.pos.load() - player.get_entity().pos.load();
                self.push.store(Vector3::new(away.x, 0.0, away.z));
            }
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
        true
    }

    pub(super) fn init_data_tracker(&self, entity: &Entity) {
        Self::set_has_fuel(entity, self.fuel.load(Ordering::Relaxed) > 0);
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
