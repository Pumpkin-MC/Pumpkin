use rustc_hash::FxHashMap;

use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use crate::entity::ai::pathfinder::{
    amphibious_node_evaluator::AmphibiousNodeEvaluator,
    node::{Node, PATH_TYPE_COUNT, PathType, Target},
    pathfinding_context::PathfindingContext,
    swim_node_evaluator::SwimNodeEvaluator,
    walk_node_evaluator::WalkNodeEvaluator,
};

pub trait NodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData);
    fn done(&mut self);
    fn get_start(&mut self) -> impl std::future::Future<Output = Option<Node>> + Send;
    fn get_target(&mut self, pos: BlockPos) -> Target;
    fn get_neighbors(
        &mut self,
        current: &Node,
        out: &mut Vec<Node>,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> impl std::future::Future<Output = PathType> + Send;
    fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> impl std::future::Future<Output = PathType> + Send;
    fn set_can_pass_doors(&mut self, can_pass: bool);
    fn set_can_open_doors(&mut self, can_open: bool);
    fn set_can_float(&mut self, can_float: bool);
    fn set_can_walk_over_fences(&mut self, can_walk: bool);
    fn can_pass_doors(&self) -> bool;
    fn can_open_doors(&self) -> bool;
    fn can_float(&self) -> bool;
    fn can_walk_over_fences(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct MobData {
    pub position: Vector3<f64>,
    pub width: f32,
    pub height: f32,
    pub max_step_height: f32,
    /// Vanilla `LivingEntity.getMaxFallDistance` default:
    /// `getComfortableFallDistance(0)` = 3 (`LivingEntity.java:1658-1663`).
    /// Gap: `Mob.getMaxFallDistance` (`Mob.java:860-869`) raises this while the
    /// mob has an attack target, based on health and difficulty; Pumpkin does
    /// not wire target/health into pathfinding yet.
    pub max_fall_distance: f32,
    pub can_swim: bool,
    pub can_walk_on_water: bool,
    pub avoids_fire: bool,
    pub avoids_water: bool,
    pub on_ground: bool,
    /// Vanilla `Entity.isInWater` (`wasTouchingWater`), read by
    /// `WalkNodeEvaluator.getStart` (`WalkNodeEvaluator.java:80`).
    pub is_in_water: bool,
    pub path_type_malus: [Option<f32>; PATH_TYPE_COUNT],
}

impl MobData {
    #[must_use]
    pub const fn new(
        position: Vector3<f64>,
        width: f32,
        height: f32,
        max_step_height: f32,
    ) -> Self {
        Self {
            position,
            width,
            height,
            max_step_height,
            max_fall_distance: 3.0,
            can_swim: false,
            can_walk_on_water: false,
            avoids_fire: true,
            avoids_water: false,
            on_ground: true,
            is_in_water: false,
            path_type_malus: [None; PATH_TYPE_COUNT],
        }
    }

    #[must_use]
    pub fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.path_type_malus[path_type as usize].unwrap_or_else(|| path_type.get_malus())
    }

    pub const fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.path_type_malus[path_type as usize] = Some(malus);
    }

    /// The mob's axis-aligned bounding box, reconstructed the way vanilla
    /// `Entity.getBoundingBox` derives it from position and dimensions.
    #[must_use]
    pub fn bounding_box(&self) -> BoundingBox {
        let half_width = f64::from(self.width) / 2.0;
        BoundingBox::new(
            Vector3::new(
                self.position.x - half_width,
                self.position.y,
                self.position.z - half_width,
            ),
            Vector3::new(
                self.position.x + half_width,
                self.position.y + f64::from(self.height),
                self.position.z + half_width,
            ),
        )
    }

    #[must_use]
    pub const fn block_position(&self) -> (i32, i32, i32) {
        (
            self.position.x.floor() as i32,
            self.position.y.floor() as i32,
            self.position.z.floor() as i32,
        )
    }

    #[must_use]
    pub fn get_bb_width(&self) -> i32 {
        (self.width + 1.0).floor() as i32
    }

    #[must_use]
    pub fn get_bb_height(&self) -> i32 {
        (self.height + 1.0).floor() as i32
    }
}

pub struct BaseNodeEvaluator {
    pub context: Option<PathfindingContext>,
    pub mob_data: Option<MobData>,
    pub nodes: FxHashMap<Vector3<i32>, Node>,
    pub entity_width: i32,
    pub entity_height: i32,
    pub entity_depth: i32, // Same as width?
    pub can_pass_doors: bool,
    pub can_open_doors: bool,
    pub can_float: bool,
    pub can_walk_over_fences: bool,
}

impl Default for BaseNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: None,
            mob_data: None,
            nodes: FxHashMap::default(),
            entity_width: 1,
            entity_height: 2,
            entity_depth: 1,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
            can_walk_over_fences: false,
        }
    }

    pub fn get_node(&mut self, pos: BlockPos) -> Node {
        if let Some(node) = self.nodes.get(&pos.0) {
            *node
        } else {
            let node = Node::new(pos);
            self.nodes.insert(pos.0, node);
            node
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.context = None;
        self.mob_data = None;
    }

    #[must_use]
    pub fn is_position_in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        self.mob_data.as_ref().is_none_or(|mob_data| {
            let mob_pos = mob_data.block_position();
            let dx = (x - mob_pos.0).abs();
            let dy = (y - mob_pos.1).abs();
            let dz = (z - mob_pos.2).abs();

            dx <= self.entity_width / 2 && dy <= self.entity_height && dz <= self.entity_depth / 2
        })
    }
}

/// Which node evaluator a mob's navigation uses, mirroring vanilla's
/// `Mob.createNavigation` overrides (`Mob.java:196-198` defaults to
/// `GroundPathNavigation`, i.e. the walk evaluator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvaluatorKind {
    /// Vanilla `GroundPathNavigation` / `WalkNodeEvaluator` — the default for
    /// every mob without a `createNavigation` override.
    #[default]
    Walk,
    /// Vanilla `WaterBoundPathNavigation` / `SwimNodeEvaluator`
    /// (`WaterBoundPathNavigation.java:23-29`); `allow_breaching` is true only
    /// for dolphins (`WaterBoundPathNavigation.java:25`).
    Swim { allow_breaching: bool },
    /// Vanilla `AmphibiousPathNavigation` / `AmphibiousNodeEvaluator(false)`
    /// (`AmphibiousPathNavigation.java:20-24`).
    Amphibious,
}

impl EvaluatorKind {
    /// Vanilla `PathNavigation.canUpdatePath` per navigation type:
    /// - `WaterBoundPathNavigation.java:31-34`:
    ///   `allowBreaching || mob.isInLiquid()`.
    /// - `AmphibiousPathNavigation.java:26-29`: always `true`.
    /// - Walk: `true`, preserving Pumpkin's pre-existing behavior for every
    ///   land mob (vanilla `GroundPathNavigation.canUpdatePath` also gates on
    ///   `onGround || isInLiquid || isPassenger`, which is not modeled here).
    #[must_use]
    pub const fn can_update_path(self, is_in_liquid: bool) -> bool {
        match self {
            Self::Walk | Self::Amphibious => true,
            Self::Swim { allow_breaching } => allow_breaching || is_in_liquid,
        }
    }
}

/// Concrete evaluator storage for the [`EvaluatorKind`] selection. The
/// [`NodeEvaluator`] trait returns `impl Future` and is therefore not
/// dyn-compatible, so dispatch happens by matching this enum.
pub enum AnyNodeEvaluator {
    Walk(Box<WalkNodeEvaluator>),
    Swim(Box<SwimNodeEvaluator>),
    Amphibious(Box<AmphibiousNodeEvaluator>),
}

macro_rules! dispatch {
    ($self:expr, $e:ident => $call:expr) => {
        match $self {
            AnyNodeEvaluator::Walk($e) => $call,
            AnyNodeEvaluator::Swim($e) => $call,
            AnyNodeEvaluator::Amphibious($e) => $call,
        }
    };
}

impl AnyNodeEvaluator {
    /// Builds the evaluator a vanilla navigation would construct:
    /// - Swim: `new SwimNodeEvaluator(allowBreaching)` with
    ///   `setCanPassDoors(false)` (`WaterBoundPathNavigation.java:25-27`).
    /// - Amphibious: `new AmphibiousNodeEvaluator(false)`
    ///   (`AmphibiousPathNavigation.java:22`).
    #[must_use]
    pub fn from_kind(kind: EvaluatorKind) -> Self {
        match kind {
            EvaluatorKind::Walk => Self::Walk(Box::default()),
            EvaluatorKind::Swim { allow_breaching } => {
                let mut evaluator = SwimNodeEvaluator::new(allow_breaching);
                evaluator.set_can_pass_doors(false);
                Self::Swim(Box::new(evaluator))
            }
            EvaluatorKind::Amphibious => {
                Self::Amphibious(Box::new(AmphibiousNodeEvaluator::new(false)))
            }
        }
    }
}

impl Default for AnyNodeEvaluator {
    fn default() -> Self {
        Self::from_kind(EvaluatorKind::Walk)
    }
}

impl NodeEvaluator for AnyNodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        dispatch!(self, e => e.prepare(context, mob_data));
    }

    fn done(&mut self) {
        dispatch!(self, e => e.done());
    }

    async fn get_start(&mut self) -> Option<Node> {
        dispatch!(self, e => e.get_start().await)
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        dispatch!(self, e => e.get_target(pos))
    }

    async fn get_neighbors(&mut self, current: &Node, out: &mut Vec<Node>) {
        dispatch!(self, e => e.get_neighbors(current, out).await);
    }

    async fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        dispatch!(self, e => e.get_path_type_of_mob(context, pos, mob_data).await)
    }

    async fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> PathType {
        dispatch!(self, e => e.get_path_type(context, pos).await)
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        dispatch!(self, e => e.set_can_pass_doors(can_pass));
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        dispatch!(self, e => e.set_can_open_doors(can_open));
    }

    fn set_can_float(&mut self, can_float: bool) {
        dispatch!(self, e => e.set_can_float(can_float));
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        dispatch!(self, e => e.set_can_walk_over_fences(can_walk));
    }

    fn can_pass_doors(&self) -> bool {
        dispatch!(self, e => e.can_pass_doors())
    }

    fn can_open_doors(&self) -> bool {
        dispatch!(self, e => e.can_open_doors())
    }

    fn can_float(&self) -> bool {
        dispatch!(self, e => e.can_float())
    }

    fn can_walk_over_fences(&self) -> bool {
        dispatch!(self, e => e.can_walk_over_fences())
    }
}

#[cfg(test)]
mod tests {
    use super::EvaluatorKind;

    #[test]
    fn default_kind_is_walk() {
        // Mob.java:196-198: the default navigation is GroundPathNavigation.
        assert_eq!(EvaluatorKind::default(), EvaluatorKind::Walk);
    }

    #[test]
    fn can_update_path_matches_vanilla_navigations() {
        // WaterBoundPathNavigation.java:31-34: allowBreaching || isInLiquid.
        assert!(
            !EvaluatorKind::Swim {
                allow_breaching: false
            }
            .can_update_path(false)
        );
        assert!(
            EvaluatorKind::Swim {
                allow_breaching: false
            }
            .can_update_path(true)
        );
        assert!(
            EvaluatorKind::Swim {
                allow_breaching: true
            }
            .can_update_path(false)
        );
        // AmphibiousPathNavigation.java:26-29: always true.
        assert!(EvaluatorKind::Amphibious.can_update_path(false));
        // Walk preserves the pre-existing always-on behavior.
        assert!(EvaluatorKind::Walk.can_update_path(false));
    }
}
