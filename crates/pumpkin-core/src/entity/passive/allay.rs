use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    player::Player,
};

pub struct AllayEntity {
    pub mob_entity: MobEntity,
    pub dancing: AtomicBool,
    pub can_duplicate: AtomicBool,
    pub duplication_cooldown: AtomicI32,
    pub owner: AtomicCell<Option<Uuid>>,
}

impl AllayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let allay = Self {
            mob_entity,
            dancing: AtomicBool::new(false),
            can_duplicate: AtomicBool::new(true),
            duplication_cooldown: AtomicI32::new(0),
            owner: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(allay);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.dancing.load(Ordering::Relaxed)
    }

    pub fn set_dancing(&self, dancing: bool) {
        self.dancing.store(dancing, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::allay::DATA_DANCING, dancing);
    }

    #[must_use]
    pub fn can_duplicate(&self) -> bool {
        self.can_duplicate.load(Ordering::Relaxed)
    }

    pub fn set_can_duplicate(&self, can_duplicate: bool) {
        self.can_duplicate.store(can_duplicate, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
            can_duplicate,
        );
    }
}

impl Mob for AllayEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("CanDuplicate", self.can_duplicate());
        nbt.put_int(
            "DuplicationCooldown",
            self.duplication_cooldown.load(Ordering::Relaxed),
        );
        if let Some(owner) = self.owner.load() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(can) = nbt.get_bool("CanDuplicate") {
            self.set_can_duplicate(can);
        }
        if let Some(cd) = nbt.get_int("DuplicationCooldown") {
            self.duplication_cooldown.store(cd, Ordering::Relaxed);
        }
        if let Some(owner) = nbt.get_uuid("Owner") {
            self.owner.store(Some(owner));
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let cd = self.duplication_cooldown.load(Ordering::Relaxed);
        if cd > 0 {
            let next = cd - 1;
            self.duplication_cooldown.store(next, Ordering::Relaxed);
            if next == 0 {
                self.set_can_duplicate(true);
            }
        }

        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        if !entity.silent.load(Ordering::Relaxed) && rand::random_range(0..100) == 0 {
            entity.world.load().play_sound_fine(
                Sound::EntityAllayAmbientWithoutItem,
                SoundCategory::Neutral,
                &entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::allay::DATA_DANCING,
            self.is_dancing(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::allay::DATA_CAN_DUPLICATE,
            self.can_duplicate(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if self.is_dancing() && self.can_duplicate() && item == &Item::AMETHYST_SHARD {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.set_can_duplicate(false);
            self.duplication_cooldown.store(6000, Ordering::Relaxed);

            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                1.0,
                7,
                Particle::Heart,
            );
            world.play_sound(
                Sound::EntityAllayAmbientWithoutItem,
                SoundCategory::Neutral,
                &pos,
            );

            let new_allay = Self::new(Entity::new(world.clone(), pos, &EntityType::ALLAY));
            world.spawn_entity(new_allay);
            return true;
        }

        let current_item = {
            let equip = self
                .mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            equip
                .equipment
                .get(&pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        };

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();

        if current_item.is_empty() && !item_stack.is_empty() {
            let mut item_to_give = item_stack.clone();
            item_to_give.set_count(1);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);

            {
                let mut equip = self
                    .mob_entity
                    .living_entity
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                equip
                    .equipment
                    .insert(pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND, item_to_give.clone());
            }
            self.mob_entity.living_entity.send_equipment_changes(&[(
                pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND,
                item_to_give.clone(),
            )]);

            self.owner.store(Some(player.gameprofile.id));
            world.play_sound(Sound::EntityAllayItemGiven, SoundCategory::Neutral, &pos);

            if item_to_give.item.id == Item::CAKE.id {
                let block_pos = pos.to_block_pos();
                let mut near_noteblock = false;
                for dx in -3..=3 {
                    for dy in -3..=3 {
                        for dz in -3..=3 {
                            let check_pos = pumpkin_util::math::position::BlockPos(
                                pumpkin_util::math::vector3::Vector3::new(
                                    block_pos.0.x + dx,
                                    block_pos.0.y + dy,
                                    block_pos.0.z + dz,
                                ),
                            );
                            if world.get_block(&check_pos).id == pumpkin_data::Block::NOTE_BLOCK.id {
                                near_noteblock = true;
                                break;
                            }
                        }
                        if near_noteblock {
                            break;
                        }
                    }
                    if near_noteblock {
                        break;
                    }
                }
                if near_noteblock {
                    player.trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::AllayDeliverCakeToNoteBlock,
                    );
                }
            }

            return true;
        }

        if !current_item.is_empty() && item_stack.is_empty() {
            {
                let mut equip = self
                    .mob_entity
                    .living_entity
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                equip
                    .equipment
                    .remove(&pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND);
            }
            self.mob_entity.living_entity.send_equipment_changes(&[(
                pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND,
                ItemStack::EMPTY.clone(),
            )]);

            world.play_sound(Sound::EntityAllayItemTaken, SoundCategory::Neutral, &pos);

            let mut to_give = current_item.clone();
            let was_added = player.inventory().insert_stack_anywhere(&mut to_give);
            if !was_added && !to_give.is_empty() {
                world.drop_stack(&player.position().to_block_pos(), to_give);
            }
            player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .send_content_updates();

            player.trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::AllayDeliverItemToPlayer,
            );

            if current_item.item.id == Item::CAKE.id {
                let block_pos = pos.to_block_pos();
                let mut near_noteblock = false;
                for dx in -3..=3 {
                    for dy in -3..=3 {
                        for dz in -3..=3 {
                            let check_pos = pumpkin_util::math::position::BlockPos(
                                pumpkin_util::math::vector3::Vector3::new(
                                    block_pos.0.x + dx,
                                    block_pos.0.y + dy,
                                    block_pos.0.z + dz,
                                ),
                            );
                            if world.get_block(&check_pos).id == pumpkin_data::Block::NOTE_BLOCK.id {
                                near_noteblock = true;
                                break;
                            }
                        }
                        if near_noteblock {
                            break;
                        }
                    }
                    if near_noteblock {
                        break;
                    }
                }
                if near_noteblock {
                    player.trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::AllayDeliverCakeToNoteBlock,
                    );
                }
            }

            return true;
        }

        if self.owner.load().is_none() {
            self.owner.store(Some(player.gameprofile.id));
        }

        false
    }
}

impl CustomSound for AllayEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityAllayDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityAllayHurt)
    }
}
