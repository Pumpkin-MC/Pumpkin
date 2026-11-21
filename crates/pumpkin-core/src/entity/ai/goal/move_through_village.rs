use std::collections::VecDeque;
use std::f64::consts::FRAC_PI_2;

use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{PointOfInterestType, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::Occupancy;

use super::{Controls, Goal};
use crate::block::blocks::doors::DoorBlock;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::entity::mob::Mob;
use crate::world::World;

const POI_SEARCH_RADIUS: i32 = 10;
const VILLAGE_SECTION_DISTANCE: i32 = 6;
const MAX_VISITED: usize = 15;

/// Walks to occupied village POIs it has not visited yet.
pub struct MoveThroughVillageGoal {
    speed_modifier: f64,
    only_at_night: bool,
    distance_to_poi: i32,
    can_deal_with_doors: fn(&dyn Mob) -> bool,
    path: Option<Path>,
    poi_pos: BlockPos,
    visited: VecDeque<BlockPos>,
}

impl MoveThroughVillageGoal {
    #[must_use]
    pub const fn new(
        speed_modifier: f64,
        only_at_night: bool,
        distance_to_poi: i32,
        can_deal_with_doors: fn(&dyn Mob) -> bool,
    ) -> Self {
        Self {
            speed_modifier,
            only_at_night,
            distance_to_poi,
            can_deal_with_doors,
            path: None,
            poi_pos: BlockPos::new(0, 0, 0),
            visited: VecDeque::new(),
        }
    }

    fn has_not_visited(&self, poi: BlockPos) -> bool {
        !self.visited.contains(&poi)
    }

    fn find_unvisited_poi(&self, world: &World, center: BlockPos) -> Option<BlockPos> {
        world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .find_closest(
                &is_village_poi,
                &|pos| self.has_not_visited(pos),
                center,
                POI_SEARCH_RADIUS,
                Occupancy::IsOccupied,
            )
    }

    fn create_path(&self, mob: &dyn Mob, destination: Vector3<f64>) -> Option<Path> {
        let can_deal_with_doors = (self.can_deal_with_doors)(mob);
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.set_can_open_doors(can_deal_with_doors);
        let path = navigator.create_path(&steered.mob_entity().living_entity, destination, 0);
        navigator.set_can_open_doors(true);
        path
    }
}

fn is_village_poi(poi_type: PoiType) -> bool {
    poi_type.has_tag(&PointOfInterestType::MINECRAFT_VILLAGE)
}

impl Goal for MoveThroughVillageGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !mob
            .get_mob_entity()
            .steered()
            .navigator()
            .can_navigate_ground()
        {
            return false;
        }
        if self.visited.len() > MAX_VISITED {
            self.visited.pop_front();
        }
        let entity = mob.get_entity();
        let world = entity.world.load();
        if self.only_at_night && world.is_bright_outside() {
            return false;
        }
        let pos = entity.block_pos.load();
        if !world.is_close_to_village(&pos, VILLAGE_SECTION_DISTANCE) {
            return false;
        }

        let Some(land_pos) = land_random_pos::get_pos_weighted(mob, 15, 7, |candidate| {
            if !world.is_village(&candidate) {
                return f64::NEG_INFINITY;
            }
            self.find_unvisited_poi(&world, candidate)
                .map_or(f64::NEG_INFINITY, |poi| {
                    -f64::from(poi.squared_distance(&pos))
                })
        }) else {
            return false;
        };
        let Some(poi_pos) = self.find_unvisited_poi(&world, BlockPos::floored_v(land_pos)) else {
            return false;
        };
        self.poi_pos = poi_pos;

        let mut path = self.create_path(mob, poi_pos.to_f64());
        if path.is_none() {
            let Some(partial_step) = default_random_pos::get_pos_towards(
                mob,
                10,
                7,
                Vector3::new(
                    f64::from(poi_pos.0.x) + 0.5,
                    f64::from(poi_pos.0.y),
                    f64::from(poi_pos.0.z) + 0.5,
                ),
                FRAC_PI_2,
            ) else {
                return false;
            };
            path = self.create_path(mob, partial_step);
        }
        let Some(mut path) = path else {
            return false;
        };

        let door_node = path.get_nodes().iter().find_map(|node| {
            let door_pos = node.pos.up();
            DoorBlock::is_wooden_door(&world, &door_pos).then_some(node.pos)
        });
        if let Some(node_pos) = door_node {
            let steered = mob.get_mob_entity().steered();
            let Some(door_path) = steered.navigator().create_path(
                &steered.mob_entity().living_entity,
                node_pos.to_f64(),
                0,
            ) else {
                self.path = None;
                return false;
            };
            path = door_path;
        }
        self.path = Some(path);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if mob.is_navigator_idle() {
            return false;
        }
        let entity = mob.get_entity();
        let distance = f64::from(entity.width()) + f64::from(self.distance_to_poi);
        let pos = entity.pos.load();
        self.poi_pos.dist_to_center_sqr(pos.x, pos.y, pos.z) >= distance * distance
    }

    fn start(&mut self, mob: &dyn Mob) {
        let steered = mob.get_mob_entity().steered();
        steered.navigator().move_to_path(
            self.path.take(),
            self.speed_modifier,
            &steered.mob_entity().living_entity,
        );
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let pos = mob.get_entity().pos.load();
        let distance = f64::from(self.distance_to_poi);
        if mob.is_navigator_idle()
            || self.poi_pos.dist_to_center_sqr(pos.x, pos.y, pos.z) < distance * distance
        {
            self.visited.push_back(self.poi_pos);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
