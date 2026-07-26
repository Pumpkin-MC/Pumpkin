use super::{Entity, EntityBase};
use crate::world::{
    World,
    portal::{NetherPortal, PortalProcessor, PortalType, SourcePortalInfo},
};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, Relaxed};
use tokio::sync::Mutex;

impl Entity {
    fn default_portal_cooldown(&self) -> u32 {
        if self.entity_type == &EntityType::PLAYER {
            10
        } else {
            300
        }
    }

    pub(super) async fn tick_portal(&self, caller: &Arc<dyn EntityBase>) {
        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        let mut manager_guard = self.portal_manager.lock().await;
        let mut should_remove = false;
        if let Some(pmanager_mutex) = manager_guard.as_ref() {
            let mut portal_processor = pmanager_mutex.lock().await;
            if portal_processor.process_portal_teleportation(
                &self.world.load(),
                caller.as_ref(),
                true,
            ) {
                self.portal_cooldown
                    .store(self.default_portal_cooldown(), Ordering::Relaxed);

                let transition = portal_processor
                    .portal_type
                    .get_portal_destination(
                        &self.world.load(),
                        portal_processor.destination_world.clone(),
                        caller,
                        portal_processor.entry_position,
                        portal_processor.source_portal.clone(),
                    )
                    .await;

                drop(portal_processor);

                if let Some(transition) = transition {
                    let dest_world = transition.new_world.clone();
                    let yaw = transition.yaw;
                    let pitch = transition.pitch;
                    let teleport_pos = transition.position;

                    // Teleport the main entity
                    caller
                        .clone()
                        .teleport(teleport_pos, yaw, pitch, dest_world.clone())
                        .await;

                    // Teleport all passengers recursively along with the vehicle
                    let yaw_delta = yaw.map(|y| y - self.yaw.load());
                    Self::teleport_passengers_recursive(self, teleport_pos, yaw_delta, &dest_world)
                        .await;
                }
            } else if portal_processor.portal_time == 0 {
                should_remove = true;
            }
        }
        if should_remove {
            *manager_guard = None;
        }
    }

    /// Recursively teleports all passengers (and their passengers) to the destination
    fn teleport_passengers_recursive<'a>(
        entity: &'a Self,
        position: Vector3<f64>,
        yaw_delta: Option<f32>,
        dest_world: &'a Arc<World>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let passengers = entity.passengers.lock().await.clone();
            for passenger in passengers {
                let passenger_entity = passenger.get_entity();
                let passenger_yaw = yaw_delta.map(|delta| passenger_entity.yaw.load() + delta);
                passenger_entity.portal_cooldown.store(
                    passenger_entity.default_portal_cooldown(),
                    Ordering::Relaxed,
                );

                // Get nested passengers before teleporting
                let nested_passengers = passenger_entity.passengers.lock().await.clone();

                passenger
                    .teleport(position, passenger_yaw, None, dest_world.clone())
                    .await;

                // Recursively teleport nested passengers
                for nested in nested_passengers {
                    let nested_entity = nested.get_entity();
                    Self::teleport_passengers_recursive(
                        nested_entity,
                        position,
                        yaw_delta,
                        dest_world,
                    )
                    .await;
                }
            }
        })
    }

    pub async fn try_use_portal(
        &self,
        _portal_delay: u32,
        portal_world: Arc<World>,
        pos: BlockPos,
    ) {
        // Passengers don't teleport independently - they wait for their vehicle
        if self.has_vehicle().await {
            return;
        }

        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown
                .store(self.default_portal_cooldown(), Ordering::Relaxed);
            return;
        }

        if (portal_world.dimension == Dimension::THE_NETHER
            && !portal_world
                .server
                .upgrade()
                .unwrap()
                .basic_config
                .allow_nether)
            || (portal_world.dimension == Dimension::THE_END
                && !portal_world
                    .server
                    .upgrade()
                    .unwrap()
                    .basic_config
                    .allow_end)
        {
            return;
        }

        let mut manager = self.portal_manager.lock().await;
        let world = self.world.load();
        if manager.is_none() {
            let portal_type = if portal_world.dimension == Dimension::THE_END
                || self.world.load().dimension == Dimension::THE_END
            {
                PortalType::End
            } else {
                PortalType::Nether
            };

            let mut new_manager = PortalProcessor::new(portal_type, pos, portal_world);

            if let Some(portal) = NetherPortal::get_on_axis(
                &world,
                &pos,
                pumpkin_data::block_properties::HorizontalAxis::X,
            ) && portal.was_already_valid()
            {
                new_manager.set_source_portal(SourcePortalInfo {
                    lower_corner: portal.lower_corner(),
                    axis: portal.axis(),
                    width: portal.width(),
                    height: portal.height(),
                });
            } else if let Some(portal) = NetherPortal::get_on_axis(
                &world,
                &pos,
                pumpkin_data::block_properties::HorizontalAxis::Z,
            ) && portal.was_already_valid()
            {
                new_manager.set_source_portal(SourcePortalInfo {
                    lower_corner: portal.lower_corner(),
                    axis: portal.axis(),
                    width: portal.width(),
                    height: portal.height(),
                });
            }

            *manager = Some(Mutex::new(new_manager));
        } else if let Some(manager) = manager.as_ref() {
            let mut manager = manager.lock().await;
            manager.entry_position = pos;
            manager.inside_portal_this_tick = true;
        }
    }

    pub(super) fn teleport(
        &self,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        _world: Arc<World>,
    ) {
        // Update server-side position and bounding box
        self.set_pos(position);
        if let Some(yaw) = yaw {
            self.yaw.store(yaw);
        }
        if let Some(pitch) = pitch {
            self.set_pitch(pitch);
        }
        // Update cache so we don't send rubberbanding deltas
        self.last_sent_pos.store(position);
        if let Some(yaw) = yaw {
            self.last_sent_yaw
                .store((yaw * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
            self.last_sent_head_yaw
                .store((yaw * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
        }
        if let Some(pitch) = pitch {
            self.last_sent_pitch
                .store((pitch * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
        }
        let chunk_pos = self.chunk_pos.load();
        self.world.load().broadcast_to_chunk(
            chunk_pos,
            &CEntityPositionSync::new(
                self.entity_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw.unwrap_or(self.yaw.load()),
                pitch.unwrap_or(self.pitch.load()),
                self.on_ground.load(Ordering::SeqCst),
            ),
        );
    }
}
