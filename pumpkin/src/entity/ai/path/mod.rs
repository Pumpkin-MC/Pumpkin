use pumpkin_core::math::vector3::Vector3;
use pumpkin_protocol::client::play::CUpdateEntityPos;

use crate::entity::living::LivingEntity;

pub struct Navigator {
    current_goal: Option<NavigatorGoal>,
}

pub struct NavigatorGoal {
    pub current_progress: Vector3<f64>,
    pub destination: Vector3<f64>,
}

impl Navigator {
    pub fn new() -> Self {
        Self { current_goal: None }
    }

    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.current_goal = Some(goal);
    }

    pub fn cancel(&mut self) {
        self.current_goal = None;
    }

    pub async fn tick(&mut self, entity: &LivingEntity) {
        if let Some(goal) = &mut self.current_goal {
            // first lets check if we reached destination
            if goal.current_progress == goal.destination {
                // if yes, we are done here
                self.current_goal = None;
                return;
            }

            // lets figuire out that is less expensive, minus or plus x
            let mut current_expense = f64::MAX;
            let mut pos = 0;
            for x in -1..2 {
                let node = Node::new(Vector3::new(
                    x as f64,
                    goal.current_progress.y,
                    goal.current_progress.z,
                ));
                let expense = node.get_expense(goal.destination);
                if expense <= current_expense {
                    current_expense = expense;
                    pos = x;
                }
            }
            dbg!(pos);
            let mut current_expense = f64::MAX;
            goal.current_progress.x += pos as f64;
            
            for z in -1..2 {
                let node = Node::new(Vector3::new(
                    goal.current_progress.x as f64,
                    goal.current_progress.y,
                    z as f64,
                ));
                let expense = node.get_expense(goal.destination);
                if expense <= current_expense {
                    current_expense = expense;
                    pos = z;
                }
            }
            
            goal.current_progress.z += pos as f64;

            // now lets move
            entity.set_pos(goal.current_progress);
            let pos = entity.entity.pos.load();
            let last_pos = entity.last_pos.load();
            entity
                .entity
                .world
                .broadcast_packet_all(&CUpdateEntityPos::new(
                    entity.entity.entity_id.into(),
                    Vector3::new(
                        pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                        pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                        pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                    ),
                    entity
                        .entity
                        .on_ground
                        .load(std::sync::atomic::Ordering::Relaxed),
                ))
                .await;
        }
    }
}

pub struct Node {
    pub location: Vector3<f64>,
}

impl Node {
    pub fn new(location: Vector3<f64>) -> Self {
        Self { location }
    }
    /// How expensive is it to go to a location
    ///
    /// Returns a f64, Higher = More Expensive
    pub fn get_expense(&self, end: Vector3<f64>) -> f64 {
        self.location.squared_distance_to_vec(end).sqrt()
    }
}
