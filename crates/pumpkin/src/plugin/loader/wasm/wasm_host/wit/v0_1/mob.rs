use std::sync::Arc;
use wasmtime::component::Resource;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::Goal;
use crate::entity::mob::Mob as InternalMob;
use crate::entity::passive::tamable::TamableAnimal;
use crate::plugin::loader::wasm::wasm_host::{
    PluginInstance, WasmPlugin,
    state::{MobResource, PluginHostState},
    wit::v0_1::entity::entity_from_resource,
    wit::v0_1::pumpkin::plugin::{
        common::Position,
        uuid::Uuid,
        world::{
            AgeableData as WitAgeableData, BlockDirection as WitBlockDirection,
            CatData as WitCatData, CreeperData as WitCreeperData, DyeColor as WitDyeColor,
            EndermanData as WitEndermanData, Entity, FoxData as WitFoxData, HostMob,
            IronGolemData as WitIronGolemData, LivingEntity as WitLivingEntity, Mob as WitMob,
            MobData as WitMobData, PathNodeType as WitPathNodeType, SheepData as WitSheepData,
            ShulkerData as WitShulkerData, SlimeData as WitSlimeData,
            VillagerData as WitVillagerData, VillagerProfession as WitVillagerProfession,
            WolfData as WitWolfData, ZombieData as WitZombieData,
        },
    },
    wit::v0_1::uuid::UuidExt,
};

pub fn mob_from_resource(
    state: &PluginHostState,
    entity: &Resource<WitMob>,
) -> wasmtime::Result<std::sync::Arc<dyn crate::entity::EntityBase>> {
    state
        .resource_table
        .get::<MobResource>(&Resource::new_own(entity.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid mob resource handle"))
        .map(|resource| resource.provider.clone())
}

#[must_use]
pub const fn from_wit_path_node_type(
    t: WitPathNodeType,
) -> crate::entity::ai::pathfinder::node::PathType {
    use crate::entity::ai::pathfinder::node::PathType;
    match t {
        WitPathNodeType::Blocked => PathType::Blocked,
        WitPathNodeType::Open => PathType::Open,
        WitPathNodeType::Walkable => PathType::Walkable,
        WitPathNodeType::WalkableDoor => PathType::WalkableDoor,
        WitPathNodeType::Trapdoor => PathType::Trapdoor,
        WitPathNodeType::PowderSnow => PathType::PowderSnow,
        WitPathNodeType::DangerPowderSnow => PathType::DangerPowderSnow,
        WitPathNodeType::Fence => PathType::Fence,
        WitPathNodeType::Lava => PathType::Lava,
        WitPathNodeType::Water => PathType::Water,
        WitPathNodeType::WaterBorder => PathType::WaterBorder,
        WitPathNodeType::Rail => PathType::Rail,
        WitPathNodeType::UnpassableRail => PathType::UnpassableRail,
        WitPathNodeType::DangerFire => PathType::DangerFire,
        WitPathNodeType::DamageFire => PathType::DamageFire,
        WitPathNodeType::DangerOther => PathType::DangerOther,
        WitPathNodeType::DamageOther => PathType::DamageOther,
        WitPathNodeType::DoorOpen => PathType::DoorOpen,
        WitPathNodeType::DoorWoodClosed => PathType::DoorWoodClosed,
        WitPathNodeType::DoorIronClosed => PathType::DoorIronClosed,
        WitPathNodeType::Breach => PathType::Breach,
        WitPathNodeType::Leaves => PathType::Leaves,
        WitPathNodeType::StickyHoney => PathType::StickyHoney,
        WitPathNodeType::Cocoa => PathType::Cocoa,
        WitPathNodeType::DamageCautious => PathType::DamageCautious,
        WitPathNodeType::DangerTrapdoor => PathType::DangerTrapdoor,
    }
}

#[must_use]
pub const fn to_wit_dye_color(color: u8) -> WitDyeColor {
    match color {
        0 => WitDyeColor::White,
        1 => WitDyeColor::Orange,
        2 => WitDyeColor::Magenta,
        3 => WitDyeColor::LightBlue,
        4 => WitDyeColor::Yellow,
        5 => WitDyeColor::Lime,
        6 => WitDyeColor::Pink,
        7 => WitDyeColor::Gray,
        8 => WitDyeColor::LightGray,
        9 => WitDyeColor::Cyan,
        10 => WitDyeColor::Purple,
        11 => WitDyeColor::Blue,
        12 => WitDyeColor::Brown,
        13 => WitDyeColor::Green,
        14 => WitDyeColor::Red,
        _ => WitDyeColor::Black,
    }
}

#[must_use]
pub const fn from_wit_dye_color(color: WitDyeColor) -> u8 {
    match color {
        WitDyeColor::White => 0,
        WitDyeColor::Orange => 1,
        WitDyeColor::Magenta => 2,
        WitDyeColor::LightBlue => 3,
        WitDyeColor::Yellow => 4,
        WitDyeColor::Lime => 5,
        WitDyeColor::Pink => 6,
        WitDyeColor::Gray => 7,
        WitDyeColor::LightGray => 8,
        WitDyeColor::Cyan => 9,
        WitDyeColor::Purple => 10,
        WitDyeColor::Blue => 11,
        WitDyeColor::Brown => 12,
        WitDyeColor::Green => 13,
        WitDyeColor::Red => 14,
        WitDyeColor::Black => 15,
    }
}

#[must_use]
pub const fn to_wit_villager_profession(prof_id: i32) -> WitVillagerProfession {
    let prof = match pumpkin_data::villager::VillagerProfession::from_i32(prof_id) {
        Some(p) => p,
        None => pumpkin_data::villager::VillagerProfession::None,
    };
    match prof {
        pumpkin_data::villager::VillagerProfession::None => WitVillagerProfession::None,
        pumpkin_data::villager::VillagerProfession::Armorer => WitVillagerProfession::Armorer,
        pumpkin_data::villager::VillagerProfession::Butcher => WitVillagerProfession::Butcher,
        pumpkin_data::villager::VillagerProfession::Cartographer => {
            WitVillagerProfession::Cartographer
        }
        pumpkin_data::villager::VillagerProfession::Cleric => WitVillagerProfession::Cleric,
        pumpkin_data::villager::VillagerProfession::Farmer => WitVillagerProfession::Farmer,
        pumpkin_data::villager::VillagerProfession::Fisherman => WitVillagerProfession::Fisherman,
        pumpkin_data::villager::VillagerProfession::Fletcher => WitVillagerProfession::Fletcher,
        pumpkin_data::villager::VillagerProfession::Leatherworker => {
            WitVillagerProfession::Leatherworker
        }
        pumpkin_data::villager::VillagerProfession::Librarian => WitVillagerProfession::Librarian,
        pumpkin_data::villager::VillagerProfession::Mason => WitVillagerProfession::Mason,
        pumpkin_data::villager::VillagerProfession::Nitwit => WitVillagerProfession::Nitwit,
        pumpkin_data::villager::VillagerProfession::Shepherd => WitVillagerProfession::Shepherd,
        pumpkin_data::villager::VillagerProfession::Toolsmith => WitVillagerProfession::Toolsmith,
        pumpkin_data::villager::VillagerProfession::Weaponsmith => {
            WitVillagerProfession::Weaponsmith
        }
    }
}

#[must_use]
pub const fn from_wit_villager_profession(
    prof: WitVillagerProfession,
) -> pumpkin_data::villager::VillagerProfession {
    match prof {
        WitVillagerProfession::None => pumpkin_data::villager::VillagerProfession::None,
        WitVillagerProfession::Armorer => pumpkin_data::villager::VillagerProfession::Armorer,
        WitVillagerProfession::Butcher => pumpkin_data::villager::VillagerProfession::Butcher,
        WitVillagerProfession::Cartographer => {
            pumpkin_data::villager::VillagerProfession::Cartographer
        }
        WitVillagerProfession::Cleric => pumpkin_data::villager::VillagerProfession::Cleric,
        WitVillagerProfession::Farmer => pumpkin_data::villager::VillagerProfession::Farmer,
        WitVillagerProfession::Fisherman => pumpkin_data::villager::VillagerProfession::Fisherman,
        WitVillagerProfession::Fletcher => pumpkin_data::villager::VillagerProfession::Fletcher,
        WitVillagerProfession::Leatherworker => {
            pumpkin_data::villager::VillagerProfession::Leatherworker
        }
        WitVillagerProfession::Librarian => pumpkin_data::villager::VillagerProfession::Librarian,
        WitVillagerProfession::Mason => pumpkin_data::villager::VillagerProfession::Mason,
        WitVillagerProfession::Nitwit => pumpkin_data::villager::VillagerProfession::Nitwit,
        WitVillagerProfession::Shepherd => pumpkin_data::villager::VillagerProfession::Shepherd,
        WitVillagerProfession::Toolsmith => pumpkin_data::villager::VillagerProfession::Toolsmith,
        WitVillagerProfession::Weaponsmith => {
            pumpkin_data::villager::VillagerProfession::Weaponsmith
        }
    }
}

#[must_use]
pub const fn to_wit_block_direction(dir: pumpkin_data::BlockDirection) -> WitBlockDirection {
    match dir {
        pumpkin_data::BlockDirection::Down => WitBlockDirection::Down,
        pumpkin_data::BlockDirection::Up => WitBlockDirection::Up,
        pumpkin_data::BlockDirection::North => WitBlockDirection::North,
        pumpkin_data::BlockDirection::South => WitBlockDirection::South,
        pumpkin_data::BlockDirection::West => WitBlockDirection::West,
        pumpkin_data::BlockDirection::East => WitBlockDirection::East,
    }
}

#[must_use]
pub const fn from_wit_block_direction(dir: WitBlockDirection) -> pumpkin_data::BlockDirection {
    match dir {
        WitBlockDirection::Down => pumpkin_data::BlockDirection::Down,
        WitBlockDirection::Up => pumpkin_data::BlockDirection::Up,
        WitBlockDirection::North => pumpkin_data::BlockDirection::North,
        WitBlockDirection::South => pumpkin_data::BlockDirection::South,
        WitBlockDirection::West => pumpkin_data::BlockDirection::West,
        WitBlockDirection::East => pumpkin_data::BlockDirection::East,
    }
}

pub struct CustomWasmGoal {
    pub plugin: Arc<WasmPlugin>,
    pub goal_id: u32,
}

fn current_mob_entity(mob: &dyn InternalMob) -> Option<Arc<dyn crate::entity::EntityBase>> {
    let entity = mob.get_entity();
    entity.world.load().get_entity_by_id(entity.entity_id)
}

impl Goal for CustomWasmGoal {
    fn can_start(&mut self, _mob: &dyn InternalMob) -> bool {
        false
    }

    fn should_continue(&self, _mob: &dyn InternalMob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn InternalMob) {
        if let Some(entity_arc) = current_mob_entity(mob) {
            let plugin = self.plugin.clone();
            let goal_id = self.goal_id;
            let world = entity_arc.get_entity().world.load();
            if let Some(server) = world.server.upgrade() {
                let run = async move {
                    let mut store = plugin.store.lock().await;
                    match plugin.plugin_instance {
                        PluginInstance::V0_1(ref plugin_inst) => {
                            let Some(server) = store.data_mut().server.clone() else {
                                return;
                            };
                            let Ok(server_res) = store.data_mut().add_server(server) else {
                                return;
                            };
                            let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                                let _ = store
                                    .data_mut()
                                    .resource_table
                                    .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                        wasmtime::component::Resource::new_own(server_res.rep()),
                                    );
                                return;
                            };
                            let server_rep = server_res.rep();
                            let entity_rep = entity_res.rep();
                            let _ = plugin_inst
                                .call_handle_ai_goal_start(
                                    &mut *store,
                                    goal_id,
                                    server_res,
                                    entity_res,
                                )
                                .await;
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                    wasmtime::component::Resource::new_own(entity_rep),
                                );
                        }
                    }
                };

                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| {
                        server.runtime.block_on(run);
                    });
                } else {
                    server.runtime.block_on(run);
                }
            }
        }
    }

    fn tick(&mut self, mob: &dyn InternalMob) {
        if let Some(entity_arc) = current_mob_entity(mob) {
            let plugin = self.plugin.clone();
            let goal_id = self.goal_id;
            let world = entity_arc.get_entity().world.load();
            if let Some(server) = world.server.upgrade() {
                let run = async move {
                    let mut store = plugin.store.lock().await;
                    match plugin.plugin_instance {
                        PluginInstance::V0_1(ref plugin_inst) => {
                            let Some(server) = store.data_mut().server.clone() else {
                                return;
                            };
                            let Ok(server_res) = store.data_mut().add_server(server) else {
                                return;
                            };
                            let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                                let _ = store
                                    .data_mut()
                                    .resource_table
                                    .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                        wasmtime::component::Resource::new_own(server_res.rep()),
                                    );
                                return;
                            };
                            let server_rep = server_res.rep();
                            let entity_rep = entity_res.rep();
                            let _ = plugin_inst
                                .call_handle_ai_goal_tick(
                                    &mut *store,
                                    goal_id,
                                    server_res,
                                    entity_res,
                                )
                                .await;
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                    wasmtime::component::Resource::new_own(entity_rep),
                                );
                        }
                    }
                };

                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| {
                        server.runtime.block_on(run);
                    });
                } else {
                    server.runtime.block_on(run);
                }
            }
        }
    }

    fn stop(&mut self, mob: &dyn InternalMob) {
        if let Some(entity_arc) = current_mob_entity(mob) {
            let plugin = self.plugin.clone();
            let goal_id = self.goal_id;
            let world = entity_arc.get_entity().world.load();
            if let Some(server) = world.server.upgrade() {
                let run = async move {
                    let mut store = plugin.store.lock().await;
                    match plugin.plugin_instance {
                        PluginInstance::V0_1(ref plugin_inst) => {
                            let Some(server) = store.data_mut().server.clone() else {
                                return;
                            };
                            let Ok(server_res) = store.data_mut().add_server(server) else {
                                return;
                            };
                            let Ok(entity_res) = store.data_mut().add_entity(entity_arc) else {
                                let _ = store
                                    .data_mut()
                                    .resource_table
                                    .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                        wasmtime::component::Resource::new_own(server_res.rep()),
                                    );
                                return;
                            };
                            let server_rep = server_res.rep();
                            let entity_rep = entity_res.rep();
                            let _ = plugin_inst
                                .call_handle_ai_goal_stop(
                                    &mut *store,
                                    goal_id,
                                    server_res,
                                    entity_res,
                                )
                                .await;
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::EntityResource>(
                                    wasmtime::component::Resource::new_own(entity_rep),
                                );
                        }
                    }
                };

                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::task::block_in_place(|| {
                        server.runtime.block_on(run);
                    });
                } else {
                    server.runtime.block_on(run);
                }
            }
        }
    }
}

impl HostMob for PluginHostState {
    async fn as_entity(&mut self, this: Resource<WitMob>) -> wasmtime::Result<Resource<Entity>> {
        let entity = mob_from_resource(self, &this)?;
        self.add_entity(entity)
    }

    async fn as_living(
        &mut self,
        this: Resource<WitMob>,
    ) -> wasmtime::Result<Resource<WitLivingEntity>> {
        let entity = mob_from_resource(self, &this)?;
        self.add_living_entity(entity)
    }

    async fn add_ai_goal(
        &mut self,
        this: Resource<WitMob>,
        priority: u8,
        goal: crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            let mob_entity = mob.get_mob_entity();
            match goal {
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::Swim => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::swim::SwimGoal::default());
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::WanderAround(speed) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::wander_around::WanderAroundGoal::new(f64::from(speed)));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::MeleeAttack(speed) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::melee_attack::MeleeAttackGoal::new(f64::from(speed), false));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::LookAtPlayer(range) => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::look_at_entity::LookAtEntityGoal::new(
                        std::sync::Weak::<crate::entity::mob::zombie::zombie::ZombieEntity>::new() as std::sync::Weak<dyn crate::entity::mob::Mob>,
                        &pumpkin_data::entity::EntityType::PLAYER,
                        range,
                        0.02,
                        false,
                    ));
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::LookAround => {
                    mob_entity.add_goal(priority, crate::entity::ai::goal::look_around::RandomLookAroundGoal::default());
                }
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::BuiltinAiGoal::EscapeDanger(speed) => {
                    mob_entity.add_goal(priority, *crate::entity::ai::goal::escape_danger::EscapeDangerGoal::new(f64::from(speed)));
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn add_custom_ai_goal(
        &mut self,
        this: Resource<WitMob>,
        priority: u8,
        goal_id: u32,
    ) -> wasmtime::Result<()> {
        let Some(plugin) = self.plugin.as_ref().and_then(std::sync::Weak::upgrade) else {
            return Err(wasmtime::Error::msg("Plugin not active"));
        };
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            let mob_entity = mob.get_mob_entity();
            mob_entity.add_goal(priority, CustomWasmGoal { plugin, goal_id });
        }
        Ok(())
    }

    async fn clear_ai_goals(&mut self, this: Resource<WitMob>) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().clear_ai_goals(mob);
        }
        Ok(())
    }

    async fn set_ai_disabled(
        &mut self,
        this: Resource<WitMob>,
        disabled: bool,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().set_no_ai(disabled);
        }
        Ok(())
    }

    async fn is_ai_disabled(&mut self, this: Resource<WitMob>) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity
            .get_mob()
            .is_none_or(|mob| mob.get_mob_entity().is_no_ai()))
    }

    async fn set_target(
        &mut self,
        this: Resource<WitMob>,
        target: Option<Resource<Entity>>,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        let target_entity = target.map(|t| entity_from_resource(self, &t)).transpose()?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity().set_target(target_entity);
        }
        Ok(())
    }

    async fn get_target(
        &mut self,
        this: Resource<WitMob>,
    ) -> wasmtime::Result<Option<Resource<Entity>>> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(target) = entity
            .get_mob()
            .and_then(|mob| mob.get_mob_entity().get_target())
        {
            return Ok(Some(self.add_entity(target)?));
        }
        Ok(None)
    }

    async fn navigate_to_pos(
        &mut self,
        this: Resource<WitMob>,
        pos: Position,
        speed: f64,
    ) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_mob().is_some_and(|mob| {
            let mob_pos = entity.get_entity().pos.load();
            let dest = Vector3::new(pos.0, pos.1, pos.2);
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(crate::entity::ai::pathfinder::NavigatorGoal::new(
                    mob_pos, dest, speed,
                ));
            true
        }))
    }

    async fn navigate_to_entity(
        &mut self,
        this: Resource<WitMob>,
        target: Resource<Entity>,
        speed: f64,
    ) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        let target_entity = entity_from_resource(self, &target)?;
        Ok(entity.get_mob().is_some_and(|mob| {
            let mob_pos = entity.get_entity().pos.load();
            let target_pos = target_entity.get_entity().pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(crate::entity::ai::pathfinder::NavigatorGoal::new(
                    mob_pos, target_pos, speed,
                ));
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity(mob, &target_entity);
            true
        }))
    }

    async fn stop_navigation(&mut self, this: Resource<WitMob>) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
        }
        Ok(())
    }

    async fn is_navigating(&mut self, this: Resource<WitMob>) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_mob().is_some_and(|mob| {
            let is_idle = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_idle
                .load(std::sync::atomic::Ordering::Relaxed);
            !is_idle
        }))
    }

    async fn has_reached_destination(&mut self, this: Resource<WitMob>) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_mob().is_none_or(|mob| {
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_idle
                .load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    async fn set_navigation_speed(
        &mut self,
        this: Resource<WitMob>,
        speed: f64,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_speed(speed);
        }
        Ok(())
    }

    async fn can_reach(
        &mut self,
        this: Resource<WitMob>,
        pos: Position,
        max_distance: f32,
    ) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_mob().is_some_and(|mob| {
            let living = &mob.get_mob_entity().living_entity;
            let dest = Vector3::new(pos.0, pos.1, pos.2);
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .can_reach_within(living, dest, max_distance)
        }))
    }

    async fn set_pathfinding_malus(
        &mut self,
        this: Resource<WitMob>,
        node_type: WitPathNodeType,
        malus: f32,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            let internal_type = from_wit_path_node_type(node_type);
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_pathfinding_malus(internal_type, malus);
        }
        Ok(())
    }

    async fn get_pathfinding_malus(
        &mut self,
        this: Resource<WitMob>,
        node_type: WitPathNodeType,
    ) -> wasmtime::Result<f32> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_mob().map_or(0.0, |mob| {
            let internal_type = from_wit_path_node_type(node_type);
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_pathfinding_malus(internal_type)
        }))
    }

    async fn look_at(&mut self, this: Resource<WitMob>, pos: Position) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at(mob, pos.0, pos.1, pos.2);
        }
        Ok(())
    }

    async fn look_at_entity(
        &mut self,
        this: Resource<WitMob>,
        target: Resource<Entity>,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        let target_entity = entity_from_resource(self, &target)?;
        if let Some(mob) = entity.get_mob() {
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity(mob, &target_entity);
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn get_mob_data(&mut self, this: Resource<WitMob>) -> wasmtime::Result<WitMobData> {
        let entity = mob_from_resource(self, &this)?;
        let any = entity.cast_any();

        if let Some(sheep) = any.downcast_ref::<crate::entity::passive::sheep::SheepEntity>() {
            return Ok(WitMobData::Sheep(WitSheepData {
                color: to_wit_dye_color(sheep.get_color()),
                is_sheared: sheep.is_sheared(),
            }));
        }

        if let Some(wolf) = any.downcast_ref::<crate::entity::passive::wolf::WolfEntity>() {
            return Ok(WitMobData::Wolf(WitWolfData {
                is_tamed: wolf.is_tame(),
                owner: wolf.get_owner().map(|u| Uuid::to_wit(&u)),
                is_sitting: wolf.is_in_sitting_pose(),
                collar_color: to_wit_dye_color(wolf.get_collar_color()),
                is_angry: false,
                is_begging: false,
            }));
        }

        if let Some(cat) = any.downcast_ref::<crate::entity::passive::cat::CatEntity>() {
            return Ok(WitMobData::Cat(WitCatData {
                is_tamed: cat.is_tame(),
                owner: cat.get_owner().map(|u| Uuid::to_wit(&u)),
                is_sitting: cat.is_in_sitting_pose(),
                collar_color: to_wit_dye_color(cat.get_collar_color()),
            }));
        }

        if let Some(villager) =
            any.downcast_ref::<crate::entity::passive::villager::VillagerEntity>()
        {
            let data = *villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            return Ok(WitMobData::Villager(WitVillagerData {
                profession: to_wit_villager_profession(data.profession.0),
                level: (data.level.0).clamp(0, 255) as u8,
                experience: villager
                    .xp
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .max(0) as u32,
            }));
        }

        if let Some(creeper) = any.downcast_ref::<crate::entity::mob::creeper::CreeperEntity>() {
            return Ok(WitMobData::Creeper(WitCreeperData {
                is_powered: creeper.is_charged(),
                fuse: creeper.get_fuse(),
                is_ignited: creeper.is_ignited(),
                explosion_radius: creeper.get_explosion_radius().clamp(0, 255) as u8,
            }));
        }

        if let Some(slime) = any.downcast_ref::<crate::entity::mob::slime::SlimeEntity>() {
            return Ok(WitMobData::Slime(WitSlimeData {
                size: slime.get_size(),
            }));
        }

        if let Some(enderman) = any.downcast_ref::<crate::entity::mob::enderman::EndermanEntity>() {
            return Ok(WitMobData::Enderman(WitEndermanData {
                carried_block_state: enderman
                    .get_carried_block()
                    .map(pumpkin_data::BlockStateId::as_u16),
                is_screaming: enderman.is_angry(),
                is_staring: enderman.is_angry(),
            }));
        }

        if let Some(iron_golem) =
            any.downcast_ref::<crate::entity::passive::iron_golem::IronGolemEntity>()
        {
            return Ok(WitMobData::IronGolem(WitIronGolemData {
                is_player_created: iron_golem.is_player_created(),
            }));
        }

        if let Some(fox) = any.downcast_ref::<crate::entity::passive::fox::FoxEntity>() {
            return Ok(WitMobData::Fox(WitFoxData {
                is_sitting: fox.is_sitting(),
                is_sleeping: fox.is_sleeping(),
                is_crouching: fox.is_crouching(),
            }));
        }

        if let Some(shulker) = any.downcast_ref::<crate::entity::mob::shulker::ShulkerEntity>() {
            return Ok(WitMobData::Shulker(WitShulkerData {
                attached_face: to_wit_block_direction(shulker.get_attach_face()),
                peek_amount: shulker.get_raw_peek(),
                color: shulker.get_color().map(to_wit_dye_color),
            }));
        }

        if let Some(zombie) = any.downcast_ref::<crate::entity::mob::zombie::zombie::ZombieEntity>()
        {
            return Ok(WitMobData::Zombie(WitZombieData {
                is_baby: zombie.is_baby(),
                can_break_doors: zombie.can_break_doors(),
            }));
        }

        if let Some(living) = entity.get_living_entity() {
            let age = living.entity.age.load(std::sync::atomic::Ordering::Relaxed);
            return Ok(WitMobData::Ageable(WitAgeableData {
                is_baby: age < 0,
                age,
                in_love_ticks: 0,
            }));
        }

        Ok(WitMobData::Generic)
    }

    #[allow(clippy::too_many_lines)]
    async fn set_mob_data(
        &mut self,
        this: Resource<WitMob>,
        data: WitMobData,
    ) -> wasmtime::Result<bool> {
        let entity = mob_from_resource(self, &this)?;
        let any = entity.cast_any();

        match data {
            WitMobData::Sheep(sheep_data) => {
                if let Some(sheep) =
                    any.downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
                {
                    sheep.set_color(from_wit_dye_color(sheep_data.color));
                    sheep.set_sheared(sheep_data.is_sheared);
                    return Ok(true);
                }
            }
            WitMobData::Wolf(wolf_data) => {
                if let Some(wolf) = any.downcast_ref::<crate::entity::passive::wolf::WolfEntity>() {
                    wolf.set_tame(wolf_data.is_tamed);
                    wolf.set_owner(wolf_data.owner.map(|u| Uuid::from_wit(&u)));
                    wolf.set_in_sitting_pose(wolf_data.is_sitting);
                    wolf.set_collar_color(from_wit_dye_color(wolf_data.collar_color));
                    return Ok(true);
                }
            }
            WitMobData::Cat(cat_data) => {
                if let Some(cat) = any.downcast_ref::<crate::entity::passive::cat::CatEntity>() {
                    cat.set_tame(
                        cat_data.is_tamed,
                        cat_data.owner.map(|u| Uuid::from_wit(&u)),
                    );
                    cat.set_sitting(cat_data.is_sitting);
                    cat.set_collar_color(from_wit_dye_color(cat_data.collar_color));
                    return Ok(true);
                }
            }
            WitMobData::Villager(villager_data) => {
                if let Some(villager) =
                    any.downcast_ref::<crate::entity::passive::villager::VillagerEntity>()
                {
                    {
                        let mut vdata = villager
                            .villager_data
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        vdata.profession = pumpkin_protocol::codec::var_int::VarInt(
                            from_wit_villager_profession(villager_data.profession) as i32,
                        );
                        vdata.level = pumpkin_protocol::codec::var_int::VarInt(i32::from(
                            villager_data.level,
                        ));
                    };
                    villager.xp.store(
                        villager_data.experience as i32,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    return Ok(true);
                }
            }
            WitMobData::Creeper(creeper_data) => {
                if let Some(creeper) =
                    any.downcast_ref::<crate::entity::mob::creeper::CreeperEntity>()
                {
                    creeper.set_charged(creeper_data.is_powered);
                    creeper.set_fuse(creeper_data.fuse);
                    creeper.set_ignited(creeper_data.is_ignited);
                    creeper.set_explosion_radius(i32::from(creeper_data.explosion_radius));
                    return Ok(true);
                }
            }
            WitMobData::Slime(slime_data) => {
                if let Some(slime) = any.downcast_ref::<crate::entity::mob::slime::SlimeEntity>() {
                    slime.set_size(slime_data.size, false);
                    return Ok(true);
                }
            }
            WitMobData::Enderman(enderman_data) => {
                if let Some(enderman) =
                    any.downcast_ref::<crate::entity::mob::enderman::EndermanEntity>()
                {
                    enderman.set_carried_block(
                        enderman_data
                            .carried_block_state
                            .and_then(pumpkin_data::BlockStateId::new),
                    );
                    enderman.set_angry(enderman_data.is_screaming || enderman_data.is_staring);
                    return Ok(true);
                }
            }
            WitMobData::IronGolem(iron_golem_data) => {
                if let Some(iron_golem) =
                    any.downcast_ref::<crate::entity::passive::iron_golem::IronGolemEntity>()
                {
                    iron_golem.set_player_created(iron_golem_data.is_player_created);
                    return Ok(true);
                }
            }
            WitMobData::Fox(fox_data) => {
                if let Some(fox) = any.downcast_ref::<crate::entity::passive::fox::FoxEntity>() {
                    fox.set_sitting(fox_data.is_sitting);
                    fox.set_sleeping(fox_data.is_sleeping);
                    fox.set_crouching(fox_data.is_crouching);
                    return Ok(true);
                }
            }
            WitMobData::Shulker(shulker_data) => {
                if let Some(shulker) =
                    any.downcast_ref::<crate::entity::mob::shulker::ShulkerEntity>()
                {
                    shulker.set_attach_face(from_wit_block_direction(shulker_data.attached_face));
                    shulker.set_raw_peek(shulker_data.peek_amount);
                    shulker.set_color(shulker_data.color.map(from_wit_dye_color));
                    return Ok(true);
                }
            }
            WitMobData::Zombie(zombie_data) => {
                if let Some(zombie) =
                    any.downcast_ref::<crate::entity::mob::zombie::zombie::ZombieEntity>()
                {
                    zombie.set_baby(zombie_data.is_baby);
                    zombie.set_can_break_doors(zombie_data.can_break_doors);
                    return Ok(true);
                }
            }
            WitMobData::Ageable(ageable_data) => {
                if let Some(living) = entity.get_living_entity() {
                    let age = if ageable_data.is_baby && ageable_data.age >= 0 {
                        -24000
                    } else {
                        ageable_data.age
                    };
                    living
                        .entity
                        .age
                        .store(age, std::sync::atomic::Ordering::Relaxed);
                    return Ok(true);
                }
            }
            WitMobData::Generic => return Ok(true),
        }

        Ok(false)
    }

    async fn set_freeze_ticks(
        &mut self,
        this: Resource<WitMob>,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        let entity = mob_from_resource(self, &this)?;
        entity.get_entity().set_frozen_ticks(ticks);
        Ok(())
    }

    async fn get_freeze_ticks(&mut self, this: Resource<WitMob>) -> wasmtime::Result<i32> {
        let entity = mob_from_resource(self, &this)?;
        Ok(entity.get_entity().get_frozen_ticks())
    }

    async fn drop(&mut self, rep: Resource<WitMob>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<MobResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
