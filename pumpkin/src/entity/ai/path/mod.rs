use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::living::LivingEntity;

#[derive(Default)]
pub struct Navigator {
    current_goal: Option<NavigatorGoal>,
}

pub struct NavigatorGoal {
    pub current_progress: Vector3<f64>,
    pub destination: Vector3<f64>,
    pub speed: f64,
}

impl Navigator {
    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.current_goal = Some(goal);
    }

    pub fn cancel(&mut self) {
        self.current_goal = None;
    }

    pub async fn tick(&mut self, living: &LivingEntity) {
        if let Some(goal) = &mut self.current_goal {
            // First, let's check if we have reached the destination
            if goal.current_progress == goal.destination {
                // If yes, we are done here.
                self.current_goal = None;
                return;
            }

            // A star algorithm
            let mut best_move = Vector3::new(0.0, 0.0, 0.0);
            let mut lowest_cost = f64::MAX;

            let world = living.entity.world.read().await;

            for x in -1..=1 {
                for z in -1..=1 {
                    let x = f64::from(x);
                    let z = f64::from(z);
                    let potential_pos = Vector3::new(
                        goal.current_progress.x + x,
                        goal.current_progress.y,
                        goal.current_progress.z + z,
                    );
                    let state = world
                        .get_block_state(&BlockPos(potential_pos.to_i32()))
                        .await;
                    let shapes = state.get_block_collision_shapes();
                    if !shapes.is_empty() {
                        continue;
                    }

                    let node = Node::new(potential_pos);
                    let cost = node.get_expense(goal.destination);

                    if cost < lowest_cost {
                        lowest_cost = cost;
                        best_move = Vector3::new(x, 0.0, z);
                    }
                }
            }

            // This is important. Firstly, this saves us many packets when we don't actually move. Secondly, this prevents division using zero
            // when normalize
            if best_move.x == 0.0 && best_move.z == 0.0 {
                return;
            }

            // Update current progress based on the best move
            goal.current_progress += best_move.normalize() * goal.speed;

            // Now let's move
            living.entity.set_pos(goal.current_progress);
            living.entity.send_pos().await;
        }
    }
}

pub struct Node {
    pub location: Vector3<f64>,
}

impl Node {
    #[must_use]
    pub fn new(location: Vector3<f64>) -> Self {
        Self { location }
    }
    /// How expensive is it to go to a location?
    ///
    /// Returns an `f64`; higher means more expensive.
    #[must_use]
    pub fn get_expense(&self, end: Vector3<f64>) -> f64 {
        self.location.squared_distance_to_vec(end).sqrt()
    }
}
