use crate::plugin::{
    entity::{
        entity_combust::EntityCombustEvent,
        entity_damage::EntityDamageEvent,
        entity_death::{EntityDeathEvent, PlayerDeathEvent},
        entity_regain_health::EntityRegainHealthEvent,
        entity_spawn::EntitySpawnEvent,
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_player, consume_text_component, consume_world,
                from_wasm_block_position, to_wasm_block_position, to_wasm_position,
            },
            pumpkin::plugin::event::{
                EntityAirChangeEventData, EntityBreedEventData, EntityCombustEventData,
                EntityDamageEventData, EntityDeathEventData, EntityDismountEventData,
                EntityDyeEventData, EntityEnterLoveModeEventData, EntityExplodeEventData,
                EntityMountEventData, EntityPickupItemEventData, EntityPortalEventData,
                EntityRegainHealthEventData, EntityResurrectEventData, EntityShootBowEventData,
                EntitySpawnEventData, EntityTameEventData, EntityTargetEventData,
                EntityTeleportEventData, EntityToggleGlideEventData, EntityTransformEventData,
                Event, PlayerDeathEventData,
            },
        },
    },
};

impl ToFromWasmEvent for EntityDamageEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityDamageEvent(EntityDamageEventData {
            entity_id: self.entity_id,
            damage: self.damage,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDamageEvent(data) => Self {
                entity_id: data.entity_id,
                damage: data.damage,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityDeathEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityDeathEvent(EntityDeathEventData {
            entity_id: self.entity_id,
            dropped_exp: self.dropped_exp,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDeathEvent(data) => Self {
                entity_id: data.entity_id,
                dropped_exp: data.dropped_exp,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerDeathEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let death_message = state
            .add_text_component(self.death_message.clone())
            .expect("failed to add text component resource");

        Event::PlayerDeathEvent(PlayerDeathEventData {
            player,
            death_message,
            dropped_exp: self.dropped_exp,
            keep_inventory: self.keep_inventory,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerDeathEvent(data) => Self {
                player: consume_player(state, &data.player),
                death_message: consume_text_component(state, &data.death_message),
                dropped_exp: data.dropped_exp,
                keep_inventory: data.keep_inventory,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntitySpawnEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::EntitySpawnEvent(EntitySpawnEventData {
            entity_id: self.entity_id,
            entity_type: self.entity_type.clone(),
            position: to_wasm_position(self.position),
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::EntitySpawnEvent(data) => {
                let world = consume_world(state, &data.target_world);
                Self {
                    entity_id: data.entity_id,
                    entity_type: data.entity_type,
                    position: pumpkin_util::math::vector3::Vector3::new(
                        data.position.0,
                        data.position.1,
                        data.position.2,
                    ),
                    world,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityCombustEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityCombustEvent(EntityCombustEventData {
            entity_id: self.entity_id,
            duration_secs: self.duration_secs,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityCombustEvent(data) => Self {
                entity_id: data.entity_id,
                duration_secs: data.duration_secs,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityRegainHealthEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityRegainHealthEvent(EntityRegainHealthEventData {
            entity_id: self.entity_id,
            amount: self.amount,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityRegainHealthEvent(data) => Self {
                entity_id: data.entity_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityAirChangeEvent(EntityAirChangeEventData {
            entity_id: self.entity_id,
            amount: self.amount,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityAirChangeEvent(data) => Self {
                entity_id: data.entity_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_breed::EntityBreedEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityBreedEvent(EntityBreedEventData {
            father_id: self.father_id,
            mother_id: self.mother_id,
            child_id: self.child_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityBreedEvent(data) => Self {
                father_id: data.father_id,
                mother_id: data.mother_id,
                child_id: data.child_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_dismount::EntityDismountEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityDismountEvent(EntityDismountEventData {
            entity_id: self.entity_id,
            dismounted_id: self.dismounted_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDismountEvent(data) => Self {
                entity_id: data.entity_id,
                dismounted_id: data.dismounted_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_dye::EntityDyeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = self
            .player
            .as_ref()
            .and_then(|p| state.add_player(p.clone()).ok());
        Event::EntityDyeEvent(EntityDyeEventData {
            entity_id: self.entity_id,
            color: format!("{:?}", self.color),
            player,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDyeEvent(data) => Self {
                entity_id: data.entity_id,
                color: crate::plugin::api::events::entity::entity_dye::DyeColor::White,
                player: data.player.map(|p| consume_player(state, &p)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_enter_love_mode::EntityEnterLoveModeEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityEnterLoveModeEvent(EntityEnterLoveModeEventData {
            entity_id: self.entity_id,
            human_entity_id: self.human_entity_id,
            ticks_in_love: self.ticks_in_love,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityEnterLoveModeEvent(data) => Self {
                entity_id: data.entity_id,
                human_entity_id: data.human_entity_id,
                ticks_in_love: data.ticks_in_love,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_explode::EntityExplodeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityExplodeEvent(EntityExplodeEventData {
            entity_id: self.entity_id,
            position: to_wasm_position(self.position),
            yield_rate: self.yield_rate,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityExplodeEvent(data) => Self {
                entity_id: data.entity_id,
                position: pumpkin_util::math::vector3::Vector3::new(
                    data.position.0,
                    data.position.1,
                    data.position.2,
                ),
                yield_rate: data.yield_rate,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_mount::EntityMountEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityMountEvent(EntityMountEventData {
            entity_id: self.entity_id,
            mounted_id: self.mount_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityMountEvent(data) => Self {
                entity_id: data.entity_id,
                mount_id: data.mounted_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_pickup_item::EntityPickupItemEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityPickupItemEvent(EntityPickupItemEventData {
            entity_id: self.entity_id,
            item_name: self.item_name.clone(),
            count: self.count,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityPickupItemEvent(data) => Self {
                entity_id: data.entity_id,
                item_name: data.item_name,
                count: data.count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_portal::EntityPortalEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityPortalEvent(EntityPortalEventData {
            entity_id: self.entity_id,
            portal_pos: to_wasm_block_position(self.portal_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityPortalEvent(data) => Self {
                entity_id: data.entity_id,
                portal_pos: from_wasm_block_position(data.portal_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_resurrect::EntityResurrectEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityResurrectEvent(EntityResurrectEventData {
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityResurrectEvent(data) => Self {
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_shoot_bow::EntityShootBowEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityShootBowEvent(EntityShootBowEventData {
            entity_id: self.entity_id,
            weapon_name: self.weapon_name.clone(),
            force: self.force,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityShootBowEvent(data) => Self {
                entity_id: data.entity_id,
                weapon_name: data.weapon_name,
                force: data.force,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_tame::EntityTameEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let owner = state
            .add_player(self.owner.clone())
            .expect("failed to add player resource");
        Event::EntityTameEvent(EntityTameEventData {
            entity_id: self.entity_id,
            owner,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityTameEvent(data) => Self {
                entity_id: data.entity_id,
                owner: consume_player(state, &data.owner),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_target::EntityTargetEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityTargetEvent(EntityTargetEventData {
            entity_id: self.entity_id,
            target_id: self.target_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityTargetEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::entity::entity_teleport::EntityTeleportEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityTeleportEvent(EntityTeleportEventData {
            entity_id: self.entity_id,
            from_position: to_wasm_position(self.from_position),
            to_position: to_wasm_position(self.to_position),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityTeleportEvent(data) => Self {
                entity_id: data.entity_id,
                from_position: pumpkin_util::math::vector3::Vector3::new(
                    data.from_position.0,
                    data.from_position.1,
                    data.from_position.2,
                ),
                to_position: pumpkin_util::math::vector3::Vector3::new(
                    data.to_position.0,
                    data.to_position.1,
                    data.to_position.2,
                ),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityToggleGlideEvent(EntityToggleGlideEventData {
            entity_id: self.entity_id,
            is_gliding: self.is_gliding,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityToggleGlideEvent(data) => Self {
                entity_id: data.entity_id,
                is_gliding: data.is_gliding,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent
    for crate::plugin::api::events::entity::entity_transform::EntityTransformEvent
{
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityTransformEvent(EntityTransformEventData {
            entity_id: self.entity_id,
            new_entity_id: self.new_entity_id,
            transform_reason: self.transform_reason.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityTransformEvent(data) => Self {
                entity_id: data.entity_id,
                new_entity_id: data.new_entity_id,
                transform_reason: data.transform_reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
