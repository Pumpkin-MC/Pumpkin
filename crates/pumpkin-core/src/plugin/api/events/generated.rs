/* This file is generated from WIT. Do not edit manually. */
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::too_many_arguments
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerFishState {
    Fishing,
    CaughtFish,
    CaughtEntity,
    InGround,
    FailedAttempt,
    ReelIn,
    Bite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InteractAction {
    LeftClickBlock,
    LeftClickAir,
    RightClickAir,
    RightClickBlock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ServerLoadType {
    Startup,
    Reload,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetReason {
    ClosestPlayer,
    ClosestEntity,
    RandomTarget,
    TargetAttackedEntity,
    TargetAttackedNearbyEntity,
    TargetAttackedOwner,
    OwnerAttackedTarget,
    PigZombieTarget,
    Angry,
    ForgotTarget,
    Fleeing,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreatureSpawnReason {
    SpawnerEgg,
    DispenseEgg,
    Bucket,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerAnimationType {
    ArmSwingMain,
    ArmSwingOff,
    LeaveBed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CauldronChangeReason {
    BucketEmpty,
    BucketFill,
    BottleEmpty,
    BottleFill,
    NaturalFill,
    Extinguish,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortalType {
    Nether,
    End,
    Custom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeType {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    DarkOak,
    Mangrove,
    Cherry,
    Poplar,
    Azalea,
    BrownMushroom,
    RedMushroom,
    Custom,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerListPingAddress {
    pub host: String,
    pub port: u16,
}
impl ServerListPingAddress {
    #[must_use]
    pub const fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnchantmentOffer {
    pub cost: i32,
    pub enchantment_id: i32,
    pub enchantment_level: i32,
}
impl EnchantmentOffer {
    #[must_use]
    pub const fn new(cost: i32, enchantment_id: i32, enchantment_level: i32) -> Self {
        Self {
            cost,
            enchantment_id,
            enchantment_level,
        }
    }
}
#[derive(Clone, Copy)]
pub struct EnchantmentValue {
    pub enchantment: &'static pumpkin_data::Enchantment,
    pub level: u32,
}
impl EnchantmentValue {
    #[must_use]
    pub const fn new(enchantment: &'static pumpkin_data::Enchantment, level: u32) -> Self {
        Self { enchantment, level }
    }
}
#[derive(Clone)]
pub struct PlayerJoinEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub join_message: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl PlayerJoinEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        join_message: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player,
            join_message,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerJoinEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerJoinEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerJoinEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerJoinEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerLeaveEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub leave_message: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl PlayerLeaveEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        leave_message: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player,
            leave_message,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerLeaveEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLeaveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLeaveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerLeaveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerLoginEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub kick_message: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl PlayerLoginEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        kick_message: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player,
            kick_message,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerLoginEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLoginEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLoginEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerLoginEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerChatEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub message: String,
    pub recipients: Vec<std::sync::Arc<crate::entity::player::Player>>,
    pub signature: Option<Vec<u8>>,
    pub cancelled: bool,
}
impl PlayerChatEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        message: String,
        recipients: Vec<std::sync::Arc<crate::entity::player::Player>>,
        signature: Option<Vec<u8>>,
    ) -> Self {
        Self {
            player,
            message,
            recipients,
            signature,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerChatEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerChatEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerChatEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerChatEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerCommandSendEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub command: String,
    pub cancelled: bool,
}
impl PlayerCommandSendEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        command: String,
    ) -> Self {
        Self {
            player,
            command,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerCommandSendEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerCommandSendEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerCommandSendEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerCommandSendEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerPermissionCheckEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub permission: String,
    pub permission_result: bool,
}
impl PlayerPermissionCheckEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        permission: String,
        permission_result: bool,
    ) -> Self {
        Self {
            player,
            permission,
            permission_result,
        }
    }
}
impl super::Payload for PlayerPermissionCheckEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerPermissionCheckEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerPermissionCheckEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerMoveEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub from_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub to_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl PlayerMoveEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        from_position: pumpkin_util::math::vector3::Vector3<f64>,
        to_position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            player,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerMoveEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerMoveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerMoveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerMoveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerTeleportEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub from_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub to_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl PlayerTeleportEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        from_position: pumpkin_util::math::vector3::Vector3<f64>,
        to_position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            player,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerTeleportEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerTeleportEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerTeleportEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerTeleportEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerChangeWorldEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub previous_world: std::sync::Arc<crate::world::World>,
    pub new_world: std::sync::Arc<crate::world::World>,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub cancelled: bool,
}
impl PlayerChangeWorldEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        previous_world: std::sync::Arc<crate::world::World>,
        new_world: std::sync::Arc<crate::world::World>,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self {
            player,
            previous_world,
            new_world,
            position,
            yaw,
            pitch,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerChangeWorldEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerChangeWorldEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerChangeWorldEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerChangeWorldEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRespawnEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub previous_world: std::sync::Arc<crate::world::World>,
    pub respawned_world: std::sync::Arc<crate::world::World>,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub alive: bool,
}
impl PlayerRespawnEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        previous_world: std::sync::Arc<crate::world::World>,
        respawned_world: std::sync::Arc<crate::world::World>,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        yaw: f32,
        pitch: f32,
        alive: bool,
    ) -> Self {
        Self {
            player,
            previous_world,
            respawned_world,
            position,
            yaw,
            pitch,
            alive,
        }
    }
}
impl super::Payload for PlayerRespawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRespawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRespawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerExpChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub amount: i32,
}
impl PlayerExpChangeEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>, amount: i32) -> Self {
        Self { player, amount }
    }
}
impl super::Payload for PlayerExpChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerExpChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerExpChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerItemHeldEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub previous_slot: u8,
    pub new_slot: u8,
    pub cancelled: bool,
}
impl PlayerItemHeldEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        previous_slot: u8,
        new_slot: u8,
    ) -> Self {
        Self {
            player,
            previous_slot,
            new_slot,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerItemHeldEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerItemHeldEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerItemHeldEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerItemHeldEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerChangedMainHandEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub main_hand: pumpkin_util::Hand,
}
impl PlayerChangedMainHandEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        main_hand: pumpkin_util::Hand,
    ) -> Self {
        Self { player, main_hand }
    }
}
impl super::Payload for PlayerChangedMainHandEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerChangedMainHandEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerChangedMainHandEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerGamemodeChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub previous_gamemode: pumpkin_util::GameMode,
    pub new_gamemode: pumpkin_util::GameMode,
    pub cancelled: bool,
}
impl PlayerGamemodeChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        previous_gamemode: pumpkin_util::GameMode,
        new_gamemode: pumpkin_util::GameMode,
    ) -> Self {
        Self {
            player,
            previous_gamemode,
            new_gamemode,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerGamemodeChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerGamemodeChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerGamemodeChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerGamemodeChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerCustomPayloadEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub channel: String,
    pub data: bytes::Bytes,
}
impl PlayerCustomPayloadEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        channel: String,
        data: bytes::Bytes,
    ) -> Self {
        Self {
            player,
            channel,
            data,
        }
    }
}
impl super::Payload for PlayerCustomPayloadEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerCustomPayloadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerCustomPayloadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerFishEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub caught_uuid: Option<uuid::Uuid>,
    pub caught_type: String,
    pub hook_uuid: uuid::Uuid,
    pub state: PlayerFishState,
    pub hand: pumpkin_util::Hand,
    pub exp_to_drop: i32,
    pub cancelled: bool,
}
impl PlayerFishEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        caught_uuid: Option<uuid::Uuid>,
        caught_type: String,
        hook_uuid: uuid::Uuid,
        state: PlayerFishState,
        hand: pumpkin_util::Hand,
        exp_to_drop: i32,
    ) -> Self {
        Self {
            player,
            caught_uuid,
            caught_type,
            hook_uuid,
            state,
            hand,
            exp_to_drop,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerFishEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerFishEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerFishEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerFishEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerEggThrowEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub egg_uuid: uuid::Uuid,
    pub hatching: bool,
    pub num_hatches: u8,
    pub hatching_type: &'static pumpkin_data::entity::EntityType,
    pub cancelled: bool,
}
impl PlayerEggThrowEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        egg_uuid: uuid::Uuid,
        hatching: bool,
        num_hatches: u8,
        hatching_type: &'static pumpkin_data::entity::EntityType,
    ) -> Self {
        Self {
            player,
            egg_uuid,
            hatching,
            num_hatches,
            hatching_type,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerEggThrowEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerEggThrowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerEggThrowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerEggThrowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerInteractUnknownEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub action: pumpkin_protocol::java::server::play::ActionType,
    pub cancelled: bool,
}
impl PlayerInteractUnknownEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        action: pumpkin_protocol::java::server::play::ActionType,
    ) -> Self {
        Self {
            player,
            entity_id,
            action,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerInteractUnknownEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerInteractUnknownEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerInteractUnknownEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerInteractUnknownEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerInteractEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub target: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub action: pumpkin_protocol::java::server::play::ActionType,
    pub target_position: Option<pumpkin_util::math::vector3::Vector3<f64>>,
    pub sneaking: bool,
    pub cancelled: bool,
}
impl PlayerInteractEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        target: std::sync::Arc<dyn crate::entity::EntityBase>,
        action: pumpkin_protocol::java::server::play::ActionType,
        target_position: Option<pumpkin_util::math::vector3::Vector3<f64>>,
        sneaking: bool,
    ) -> Self {
        Self {
            player,
            target,
            action,
            target_position,
            sneaking,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerInteractEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerInteractEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerInteractEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerInteractEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerInteractEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub action: InteractAction,
    pub clicked_pos: Option<pumpkin_util::math::position::BlockPos>,
    pub block: &'static pumpkin_data::Block,
    pub cancelled: bool,
}
impl PlayerInteractEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        action: InteractAction,
        clicked_pos: Option<pumpkin_util::math::position::BlockPos>,
        block: &'static pumpkin_data::Block,
    ) -> Self {
        Self {
            player,
            action,
            clicked_pos,
            block,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerInteractEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerInteractEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerInteractEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerInteractEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerToggleSneakEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub is_sneaking: bool,
    pub cancelled: bool,
}
impl PlayerToggleSneakEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        is_sneaking: bool,
    ) -> Self {
        Self {
            player,
            is_sneaking,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerToggleSneakEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerToggleSneakEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerToggleSneakEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerToggleSneakEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerToggleFlightEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub is_flying: bool,
    pub cancelled: bool,
}
impl PlayerToggleFlightEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        is_flying: bool,
    ) -> Self {
        Self {
            player,
            is_flying,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerToggleFlightEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerToggleFlightEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerToggleFlightEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerToggleFlightEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerToggleSprintEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub is_sprinting: bool,
    pub cancelled: bool,
}
impl PlayerToggleSprintEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        is_sprinting: bool,
    ) -> Self {
        Self {
            player,
            is_sprinting,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerToggleSprintEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerToggleSprintEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerToggleSprintEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerToggleSprintEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryClickEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub window_type: Option<pumpkin_data::screen::WindowType>,
    pub click_type: pumpkin_inventory::screen_handler::ClickType,
    pub slot: i16,
    pub raw_slot: i16,
    pub clicked_item: Option<pumpkin_data::item_stack::ItemStack>,
    pub cursor: Option<pumpkin_data::item_stack::ItemStack>,
    pub hotbar_button: i32,
    pub cancelled: bool,
}
impl InventoryClickEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        window_type: Option<pumpkin_data::screen::WindowType>,
        click_type: pumpkin_inventory::screen_handler::ClickType,
        slot: i16,
        raw_slot: i16,
        clicked_item: Option<pumpkin_data::item_stack::ItemStack>,
        cursor: Option<pumpkin_data::item_stack::ItemStack>,
        hotbar_button: i32,
    ) -> Self {
        Self {
            player,
            window_type,
            click_type,
            slot,
            raw_slot,
            clicked_item,
            cursor,
            hotbar_button,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryClickEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryClickEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryClickEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryClickEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryCloseEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub window_type: Option<pumpkin_data::screen::WindowType>,
}
impl InventoryCloseEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        window_type: Option<pumpkin_data::screen::WindowType>,
    ) -> Self {
        Self {
            player,
            window_type,
        }
    }
}
impl super::Payload for InventoryCloseEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryCloseEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryCloseEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct BlockRedstoneEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub state_id: pumpkin_data::BlockStateId,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub old_current: i32,
    pub new_current: i32,
    pub cancelled: bool,
}
impl BlockRedstoneEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        state_id: pumpkin_data::BlockStateId,
        block_pos: pumpkin_util::math::position::BlockPos,
        old_current: i32,
        new_current: i32,
    ) -> Self {
        Self {
            target_world,
            state_id,
            block_pos,
            old_current,
            new_current,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockRedstoneEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockRedstoneEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockRedstoneEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockRedstoneEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockBreakEvent {
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub block: &'static pumpkin_data::Block,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub exp: u32,
    pub should_drop: bool,
    pub cancelled: bool,
}
impl BlockBreakEvent {
    #[must_use]
    pub const fn new(
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
        block: &'static pumpkin_data::Block,
        block_pos: pumpkin_util::math::position::BlockPos,
        exp: u32,
        should_drop: bool,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            exp,
            should_drop,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockBreakEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockBreakEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockBreakEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockBreakEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockBurnEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub igniting_block: &'static pumpkin_data::Block,
    pub block: &'static pumpkin_data::Block,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl BlockBurnEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        igniting_block: &'static pumpkin_data::Block,
        block: &'static pumpkin_data::Block,
        block_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            target_world,
            igniting_block,
            block,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockBurnEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockBurnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockBurnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockBurnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockCanBuildEvent {
    pub block_to_build: &'static pumpkin_data::Block,
    pub buildable: bool,
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block: &'static pumpkin_data::Block,
    pub cancelled: bool,
}
impl BlockCanBuildEvent {
    #[must_use]
    pub const fn new(
        block_to_build: &'static pumpkin_data::Block,
        buildable: bool,
        player: std::sync::Arc<crate::entity::player::Player>,
        block: &'static pumpkin_data::Block,
    ) -> Self {
        Self {
            block_to_build,
            buildable,
            player,
            block,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockCanBuildEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockCanBuildEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockCanBuildEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockCanBuildEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockGrowEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub old_block: &'static pumpkin_data::Block,
    pub old_state_id: pumpkin_data::BlockStateId,
    pub new_block: &'static pumpkin_data::Block,
    pub new_state_id: pumpkin_data::BlockStateId,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl BlockGrowEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        old_block: &'static pumpkin_data::Block,
        old_state_id: pumpkin_data::BlockStateId,
        new_block: &'static pumpkin_data::Block,
        new_state_id: pumpkin_data::BlockStateId,
        block_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            target_world,
            old_block,
            old_state_id,
            new_block,
            new_state_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockGrowEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockGrowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockGrowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockGrowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockPlaceEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_placed: &'static pumpkin_data::Block,
    pub block_placed_against: &'static pumpkin_data::Block,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub can_build: bool,
    pub cancelled: bool,
}
impl BlockPlaceEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_placed: &'static pumpkin_data::Block,
        block_placed_against: &'static pumpkin_data::Block,
        block_pos: pumpkin_util::math::position::BlockPos,
        can_build: bool,
    ) -> Self {
        Self {
            player,
            block_placed,
            block_placed_against,
            block_pos,
            can_build,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockPlaceEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockPlaceEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockPlaceEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockPlaceEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct DialogClickActionEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub id: String,
    pub payload: Option<bytes::Bytes>,
    pub cancelled: bool,
}
impl DialogClickActionEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        id: String,
        payload: Option<bytes::Bytes>,
    ) -> Self {
        Self {
            player,
            id,
            payload,
            cancelled: false,
        }
    }
}
impl super::Payload for DialogClickActionEvent {
    fn get_name_static() -> &'static str {
        stringify!(DialogClickActionEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(DialogClickActionEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for DialogClickActionEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct DialogShowEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub dialog: pumpkin_protocol::java::client::dialog::Dialog,
    pub cancelled: bool,
}
impl DialogShowEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        dialog: pumpkin_protocol::java::client::dialog::Dialog,
    ) -> Self {
        Self {
            player,
            dialog,
            cancelled: false,
        }
    }
}
impl super::Payload for DialogShowEvent {
    fn get_name_static() -> &'static str {
        stringify!(DialogShowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(DialogShowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for DialogShowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct DialogClearEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl DialogClearEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for DialogClearEvent {
    fn get_name_static() -> &'static str {
        stringify!(DialogClearEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(DialogClearEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for DialogClearEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BedrockFormResponseEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub form_id: u32,
    pub response_data: Option<String>,
}
impl BedrockFormResponseEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        form_id: u32,
        response_data: Option<String>,
    ) -> Self {
        Self {
            player,
            form_id,
            response_data,
        }
    }
}
impl super::Payload for BedrockFormResponseEvent {
    fn get_name_static() -> &'static str {
        stringify!(BedrockFormResponseEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BedrockFormResponseEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct ServerCommandEvent {
    pub command: String,
    pub cancelled: bool,
}
impl ServerCommandEvent {
    #[must_use]
    pub const fn new(command: String) -> Self {
        Self {
            command,
            cancelled: false,
        }
    }
}
impl super::Payload for ServerCommandEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerCommandEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerCommandEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ServerCommandEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ServerListPingEvent {
    pub hostname: String,
    pub address: ServerListPingAddress,
    pub motd: pumpkin_util::text::TextComponent,
    pub max_players: u32,
    pub num_players: u32,
    pub favicon: Option<String>,
}
impl ServerListPingEvent {
    #[must_use]
    pub const fn new(
        hostname: String,
        address: ServerListPingAddress,
        motd: pumpkin_util::text::TextComponent,
        max_players: u32,
        num_players: u32,
        favicon: Option<String>,
    ) -> Self {
        Self {
            hostname,
            address,
            motd,
            max_players,
            num_players,
            favicon,
        }
    }
}
impl super::Payload for ServerListPingEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerListPingEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerListPingEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct ServerLoadEvent {
    pub load_type: ServerLoadType,
}
impl ServerLoadEvent {
    #[must_use]
    pub const fn new(load_type: ServerLoadType) -> Self {
        Self { load_type }
    }
}
impl super::Payload for ServerLoadEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerLoadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerLoadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct SpawnChangeEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub previous_position: pumpkin_util::math::position::BlockPos,
    pub previous_yaw: f32,
    pub previous_pitch: f32,
    pub new_position: pumpkin_util::math::position::BlockPos,
    pub new_yaw: f32,
    pub new_pitch: f32,
}
impl SpawnChangeEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        previous_position: pumpkin_util::math::position::BlockPos,
        previous_yaw: f32,
        previous_pitch: f32,
        new_position: pumpkin_util::math::position::BlockPos,
        new_yaw: f32,
        new_pitch: f32,
    ) -> Self {
        Self {
            target_world,
            previous_position,
            previous_yaw,
            previous_pitch,
            new_position,
            new_yaw,
            new_pitch,
        }
    }
}
impl super::Payload for SpawnChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(SpawnChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SpawnChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct ServerBroadcastEvent {
    pub message: pumpkin_util::text::TextComponent,
    pub sender: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl ServerBroadcastEvent {
    #[must_use]
    pub const fn new(
        message: pumpkin_util::text::TextComponent,
        sender: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            message,
            sender,
            cancelled: false,
        }
    }
}
impl super::Payload for ServerBroadcastEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerBroadcastEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerBroadcastEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ServerBroadcastEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ServerTickStartEvent {
    pub tick: i32,
}
impl ServerTickStartEvent {
    #[must_use]
    pub const fn new(tick: i32) -> Self {
        Self { tick }
    }
}
impl super::Payload for ServerTickStartEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerTickStartEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerTickStartEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct ServerTickEndEvent {
    pub tick: i32,
    pub duration_nanos: i64,
}
impl ServerTickEndEvent {
    #[must_use]
    pub const fn new(tick: i32, duration_nanos: i64) -> Self {
        Self {
            tick,
            duration_nanos,
        }
    }
}
impl super::Payload for ServerTickEndEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServerTickEndEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServerTickEndEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct ChunkLoadEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub cancelled: bool,
}
impl ChunkLoadEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Self {
        Self {
            target_world,
            chunk_x,
            chunk_z,
            cancelled: false,
        }
    }
}
impl super::Payload for ChunkLoadEvent {
    fn get_name_static() -> &'static str {
        stringify!(ChunkLoadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ChunkLoadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ChunkLoadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ChunkSaveEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub cancelled: bool,
}
impl ChunkSaveEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Self {
        Self {
            target_world,
            chunk_x,
            chunk_z,
            cancelled: false,
        }
    }
}
impl super::Payload for ChunkSaveEvent {
    fn get_name_static() -> &'static str {
        stringify!(ChunkSaveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ChunkSaveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ChunkSaveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ChunkSendEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub cancelled: bool,
}
impl ChunkSendEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Self {
        Self {
            target_world,
            chunk_x,
            chunk_z,
            cancelled: false,
        }
    }
}
impl super::Payload for ChunkSendEvent {
    fn get_name_static() -> &'static str {
        stringify!(ChunkSendEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ChunkSendEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ChunkSendEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDamageEvent {
    pub entity_id: i32,
    pub damage: f32,
    pub damage_type: pumpkin_data::damage::DamageType,
    pub cancelled: bool,
}
impl EntityDamageEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        damage: f32,
        damage_type: pumpkin_data::damage::DamageType,
    ) -> Self {
        Self {
            entity_id,
            damage,
            damage_type,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDamageEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDamageEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDamageEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDamageEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDeathEvent {
    pub entity_id: i32,
    pub dropped_exp: i32,
}
impl EntityDeathEvent {
    #[must_use]
    pub const fn new(entity_id: i32, dropped_exp: i32) -> Self {
        Self {
            entity_id,
            dropped_exp,
        }
    }
}
impl super::Payload for EntityDeathEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDeathEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDeathEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerDeathEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub death_message: pumpkin_util::text::TextComponent,
    pub dropped_exp: i32,
    pub keep_inventory: bool,
    pub cancelled: bool,
}
impl PlayerDeathEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        death_message: pumpkin_util::text::TextComponent,
        dropped_exp: i32,
        keep_inventory: bool,
    ) -> Self {
        Self {
            player,
            death_message,
            dropped_exp,
            keep_inventory,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerDeathEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerDeathEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerDeathEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerDeathEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntitySpawnEvent {
    pub entity_id: i32,
    pub entity_type: String,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub cancelled: bool,
}
impl EntitySpawnEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: String,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        target_world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            position,
            target_world,
            cancelled: false,
        }
    }
}
impl super::Payload for EntitySpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntitySpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntitySpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntitySpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityCombustEvent {
    pub entity_id: i32,
    pub duration_secs: f32,
    pub cancelled: bool,
}
impl EntityCombustEvent {
    #[must_use]
    pub const fn new(entity_id: i32, duration_secs: f32) -> Self {
        Self {
            entity_id,
            duration_secs,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityCombustEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityCombustEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityCombustEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityCombustEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityRegainHealthEvent {
    pub entity_id: i32,
    pub amount: f32,
    pub cancelled: bool,
}
impl EntityRegainHealthEvent {
    #[must_use]
    pub const fn new(entity_id: i32, amount: f32) -> Self {
        Self {
            entity_id,
            amount,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityRegainHealthEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityRegainHealthEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityRegainHealthEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityRegainHealthEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityAirChangeEvent {
    pub entity_id: i32,
    pub amount: i32,
    pub cancelled: bool,
}
impl EntityAirChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, amount: i32) -> Self {
        Self {
            entity_id,
            amount,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityAirChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityAirChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityAirChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityAirChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityBreedEvent {
    pub father_id: i32,
    pub mother_id: i32,
    pub child_id: i32,
    pub experience: i32,
    pub cancelled: bool,
}
impl EntityBreedEvent {
    #[must_use]
    pub const fn new(father_id: i32, mother_id: i32, child_id: i32, experience: i32) -> Self {
        Self {
            father_id,
            mother_id,
            child_id,
            experience,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityBreedEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityBreedEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityBreedEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityBreedEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDismountEvent {
    pub entity_id: i32,
    pub dismounted_id: i32,
    pub cancelled: bool,
}
impl EntityDismountEvent {
    #[must_use]
    pub const fn new(entity_id: i32, dismounted_id: i32) -> Self {
        Self {
            entity_id,
            dismounted_id,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDismountEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDismountEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDismountEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDismountEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDyeEvent {
    pub entity_id: i32,
    pub color: pumpkin_data::dye_color::DyeColor,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub cancelled: bool,
}
impl EntityDyeEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        color: pumpkin_data::dye_color::DyeColor,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
    ) -> Self {
        Self {
            entity_id,
            color,
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDyeEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDyeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDyeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDyeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityEnterLoveModeEvent {
    pub entity_id: i32,
    pub human_entity_id: Option<i32>,
    pub ticks_in_love: i32,
    pub cancelled: bool,
}
impl EntityEnterLoveModeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, human_entity_id: Option<i32>, ticks_in_love: i32) -> Self {
        Self {
            entity_id,
            human_entity_id,
            ticks_in_love,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityEnterLoveModeEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityEnterLoveModeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityEnterLoveModeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityEnterLoveModeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityExplodeEvent {
    pub entity_id: i32,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub yield_rate: f32,
    pub cancelled: bool,
}
impl EntityExplodeEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        yield_rate: f32,
    ) -> Self {
        Self {
            entity_id,
            position,
            yield_rate,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityExplodeEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityExplodeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityExplodeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityExplodeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityMountEvent {
    pub entity_id: i32,
    pub mounted_id: i32,
    pub cancelled: bool,
}
impl EntityMountEvent {
    #[must_use]
    pub const fn new(entity_id: i32, mounted_id: i32) -> Self {
        Self {
            entity_id,
            mounted_id,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityMountEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityMountEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityMountEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityMountEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPickupItemEvent {
    pub entity_id: i32,
    pub item_name: String,
    pub count: u8,
    pub cancelled: bool,
}
impl EntityPickupItemEvent {
    #[must_use]
    pub const fn new(entity_id: i32, item_name: String, count: u8) -> Self {
        Self {
            entity_id,
            item_name,
            count,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPickupItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPickupItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPickupItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPickupItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPortalEvent {
    pub entity_id: i32,
    pub portal_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityPortalEvent {
    #[must_use]
    pub const fn new(entity_id: i32, portal_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            portal_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPortalEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPortalEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPortalEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPortalEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityResurrectEvent {
    pub entity_id: i32,
    pub cancelled: bool,
}
impl EntityResurrectEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityResurrectEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityResurrectEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityResurrectEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityResurrectEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityShootBowEvent {
    pub entity_id: i32,
    pub weapon_name: String,
    pub force: f32,
    pub cancelled: bool,
}
impl EntityShootBowEvent {
    #[must_use]
    pub const fn new(entity_id: i32, weapon_name: String, force: f32) -> Self {
        Self {
            entity_id,
            weapon_name,
            force,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityShootBowEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityShootBowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityShootBowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityShootBowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTameEvent {
    pub entity_id: i32,
    pub owner: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl EntityTameEvent {
    #[must_use]
    pub const fn new(entity_id: i32, owner: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            entity_id,
            owner,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTameEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTameEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTameEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTameEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTargetEvent {
    pub entity_id: i32,
    pub target_id: Option<i32>,
    pub reason: TargetReason,
    pub cancelled: bool,
}
impl EntityTargetEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: Option<i32>, reason: TargetReason) -> Self {
        Self {
            entity_id,
            target_id,
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTargetEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTargetEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTargetEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTargetEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTeleportEvent {
    pub entity_id: i32,
    pub from_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub to_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl EntityTeleportEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        from_position: pumpkin_util::math::vector3::Vector3<f64>,
        to_position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTeleportEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTeleportEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTeleportEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTeleportEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityToggleGlideEvent {
    pub entity_id: i32,
    pub is_gliding: bool,
    pub cancelled: bool,
}
impl EntityToggleGlideEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_gliding: bool) -> Self {
        Self {
            entity_id,
            is_gliding,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityToggleGlideEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityToggleGlideEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityToggleGlideEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityToggleGlideEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTransformEvent {
    pub entity_id: i32,
    pub new_entity_id: i32,
    pub transform_reason: String,
    pub cancelled: bool,
}
impl EntityTransformEvent {
    #[must_use]
    pub const fn new(entity_id: i32, new_entity_id: i32, transform_reason: String) -> Self {
        Self {
            entity_id,
            new_entity_id,
            transform_reason,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTransformEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTransformEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTransformEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTransformEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerItemConsumeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
    pub cancelled: bool,
}
impl PlayerItemConsumeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
    ) -> Self {
        Self {
            player,
            item_name,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerItemConsumeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerItemConsumeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerItemConsumeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerItemConsumeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerItemDamageEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
    pub damage: i32,
    pub cancelled: bool,
}
impl PlayerItemDamageEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
        damage: i32,
    ) -> Self {
        Self {
            player,
            item_name,
            damage,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerItemDamageEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerItemDamageEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerItemDamageEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerItemDamageEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerDropItemEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
    pub count: u8,
    pub cancelled: bool,
}
impl PlayerDropItemEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
        count: u8,
    ) -> Self {
        Self {
            player,
            item_name,
            count,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerDropItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerDropItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerDropItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerDropItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerBedEnterEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub bed_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl PlayerBedEnterEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        bed_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            player,
            bed_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerBedEnterEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerBedEnterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerBedEnterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerBedEnterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerBedLeaveEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub bed_pos: pumpkin_util::math::position::BlockPos,
}
impl PlayerBedLeaveEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        bed_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self { player, bed_pos }
    }
}
impl super::Payload for PlayerBedLeaveEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerBedLeaveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerBedLeaveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerBucketEmptyEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub bucket: String,
    pub cancelled: bool,
}
impl PlayerBucketEmptyEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        bucket: String,
    ) -> Self {
        Self {
            player,
            block_pos,
            bucket,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerBucketEmptyEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerBucketEmptyEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerBucketEmptyEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerBucketEmptyEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerBucketFillEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub bucket: String,
    pub cancelled: bool,
}
impl PlayerBucketFillEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        bucket: String,
    ) -> Self {
        Self {
            player,
            block_pos,
            bucket,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerBucketFillEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerBucketFillEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerBucketFillEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerBucketFillEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockDamageEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block: &'static pumpkin_data::Block,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub insta_break: bool,
    pub cancelled: bool,
}
impl BlockDamageEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block: &'static pumpkin_data::Block,
        block_pos: pumpkin_util::math::position::BlockPos,
        insta_break: bool,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            insta_break,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockDamageEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDamageEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDamageEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockDamageEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockIgniteEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub igniting_block: &'static pumpkin_data::Block,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub cancelled: bool,
}
impl BlockIgniteEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        igniting_block: &'static pumpkin_data::Block,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
    ) -> Self {
        Self {
            block_pos,
            igniting_block,
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockIgniteEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockIgniteEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockIgniteEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockIgniteEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockFromToEvent {
    pub from_pos: pumpkin_util::math::position::BlockPos,
    pub to_pos: pumpkin_util::math::position::BlockPos,
    pub block: &'static pumpkin_data::Block,
    pub cancelled: bool,
}
impl BlockFromToEvent {
    #[must_use]
    pub const fn new(
        from_pos: pumpkin_util::math::position::BlockPos,
        to_pos: pumpkin_util::math::position::BlockPos,
        block: &'static pumpkin_data::Block,
    ) -> Self {
        Self {
            from_pos,
            to_pos,
            block,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockFromToEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockFromToEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockFromToEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockFromToEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockFormEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub block: &'static pumpkin_data::Block,
    pub cancelled: bool,
}
impl BlockFormEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        block: &'static pumpkin_data::Block,
    ) -> Self {
        Self {
            block_pos,
            block,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockFormEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockFormEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockFormEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockFormEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockFadeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub block: &'static pumpkin_data::Block,
    pub cancelled: bool,
}
impl BlockFadeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        block: &'static pumpkin_data::Block,
    ) -> Self {
        Self {
            block_pos,
            block,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockFadeEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockFadeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockFadeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockFadeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockDispenseEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub item_name: String,
    pub cancelled: bool,
}
impl BlockDispenseEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, item_name: String) -> Self {
        Self {
            block_pos,
            item_name,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockDispenseEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDispenseEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDispenseEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockDispenseEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockExplodeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub yield_rate: f32,
    pub cancelled: bool,
}
impl BlockExplodeEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, yield_rate: f32) -> Self {
        Self {
            block_pos,
            yield_rate,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockExplodeEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockExplodeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockExplodeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockExplodeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockPhysicsEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub changed_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl BlockPhysicsEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        changed_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            block_pos,
            changed_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockPhysicsEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockPhysicsEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockPhysicsEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockPhysicsEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockPistonExtendEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub direction: String,
    pub cancelled: bool,
}
impl BlockPistonExtendEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, direction: String) -> Self {
        Self {
            block_pos,
            direction,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockPistonExtendEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockPistonExtendEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockPistonExtendEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockPistonExtendEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockPistonRetractEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub direction: String,
    pub cancelled: bool,
}
impl BlockPistonRetractEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, direction: String) -> Self {
        Self {
            block_pos,
            direction,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockPistonRetractEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockPistonRetractEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockPistonRetractEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockPistonRetractEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct NotePlayEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub instrument: String,
    pub note: u8,
    pub cancelled: bool,
}
impl NotePlayEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        instrument: String,
        note: u8,
    ) -> Self {
        Self {
            block_pos,
            instrument,
            note,
            cancelled: false,
        }
    }
}
impl super::Payload for NotePlayEvent {
    fn get_name_static() -> &'static str {
        stringify!(NotePlayEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(NotePlayEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for NotePlayEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SignChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub lines: Vec<String>,
    pub cancelled: bool,
}
impl SignChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        lines: Vec<String>,
    ) -> Self {
        Self {
            player,
            block_pos,
            lines,
            cancelled: false,
        }
    }
}
impl super::Payload for SignChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(SignChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SignChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SignChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SpongeAbsorbEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl SpongeAbsorbEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for SpongeAbsorbEvent {
    fn get_name_static() -> &'static str {
        stringify!(SpongeAbsorbEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SpongeAbsorbEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SpongeAbsorbEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct TntPrimeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub prime_reason: String,
    pub cancelled: bool,
}
impl TntPrimeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        prime_reason: String,
    ) -> Self {
        Self {
            block_pos,
            prime_reason,
            cancelled: false,
        }
    }
}
impl super::Payload for TntPrimeEvent {
    fn get_name_static() -> &'static str {
        stringify!(TntPrimeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(TntPrimeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for TntPrimeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct WeatherChangeEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub to_weather_state: bool,
    pub cancelled: bool,
}
impl WeatherChangeEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        to_weather_state: bool,
    ) -> Self {
        Self {
            target_world,
            to_weather_state,
            cancelled: false,
        }
    }
}
impl super::Payload for WeatherChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(WeatherChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WeatherChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for WeatherChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ThunderChangeEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub to_thunder_state: bool,
    pub cancelled: bool,
}
impl ThunderChangeEvent {
    #[must_use]
    pub const fn new(
        target_world: std::sync::Arc<crate::world::World>,
        to_thunder_state: bool,
    ) -> Self {
        Self {
            target_world,
            to_thunder_state,
            cancelled: false,
        }
    }
}
impl super::Payload for ThunderChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(ThunderChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ThunderChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ThunderChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct WorldLoadEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
}
impl WorldLoadEvent {
    #[must_use]
    pub const fn new(target_world: std::sync::Arc<crate::world::World>) -> Self {
        Self { target_world }
    }
}
impl super::Payload for WorldLoadEvent {
    fn get_name_static() -> &'static str {
        stringify!(WorldLoadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WorldLoadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct WorldUnloadEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
    pub cancelled: bool,
}
impl WorldUnloadEvent {
    #[must_use]
    pub const fn new(target_world: std::sync::Arc<crate::world::World>) -> Self {
        Self {
            target_world,
            cancelled: false,
        }
    }
}
impl super::Payload for WorldUnloadEvent {
    fn get_name_static() -> &'static str {
        stringify!(WorldUnloadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WorldUnloadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for WorldUnloadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct AsyncStructureGenerateEvent {
    pub world_name: String,
    pub structure_name: String,
    pub pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl AsyncStructureGenerateEvent {
    #[must_use]
    pub const fn new(
        world_name: String,
        structure_name: String,
        pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            world_name,
            structure_name,
            pos,
            cancelled: false,
        }
    }
}
impl super::Payload for AsyncStructureGenerateEvent {
    fn get_name_static() -> &'static str {
        stringify!(AsyncStructureGenerateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(AsyncStructureGenerateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for AsyncStructureGenerateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct AsyncStructureSpawnEvent {
    pub world_name: String,
    pub structure_name: String,
    pub pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl AsyncStructureSpawnEvent {
    #[must_use]
    pub const fn new(
        world_name: String,
        structure_name: String,
        pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            world_name,
            structure_name,
            pos,
            cancelled: false,
        }
    }
}
impl super::Payload for AsyncStructureSpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(AsyncStructureSpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(AsyncStructureSpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for AsyncStructureSpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ChunkPopulateEvent {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub cancelled: bool,
}
impl ChunkPopulateEvent {
    #[must_use]
    pub const fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            cancelled: false,
        }
    }
}
impl super::Payload for ChunkPopulateEvent {
    fn get_name_static() -> &'static str {
        stringify!(ChunkPopulateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ChunkPopulateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ChunkPopulateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ChunkUnloadEvent {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub cancelled: bool,
}
impl ChunkUnloadEvent {
    #[must_use]
    pub const fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            cancelled: false,
        }
    }
}
impl super::Payload for ChunkUnloadEvent {
    fn get_name_static() -> &'static str {
        stringify!(ChunkUnloadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ChunkUnloadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ChunkUnloadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntitiesLoadEvent {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub entity_count: u32,
    pub cancelled: bool,
}
impl EntitiesLoadEvent {
    #[must_use]
    pub const fn new(chunk_x: i32, chunk_z: i32, entity_count: u32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            entity_count,
            cancelled: false,
        }
    }
}
impl super::Payload for EntitiesLoadEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntitiesLoadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntitiesLoadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntitiesLoadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntitiesUnloadEvent {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub entity_count: u32,
    pub cancelled: bool,
}
impl EntitiesUnloadEvent {
    #[must_use]
    pub const fn new(chunk_x: i32, chunk_z: i32, entity_count: u32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            entity_count,
            cancelled: false,
        }
    }
}
impl super::Payload for EntitiesUnloadEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntitiesUnloadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntitiesUnloadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntitiesUnloadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct GenericGameEvent {
    pub event_id: String,
    pub pos: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl GenericGameEvent {
    #[must_use]
    pub const fn new(event_id: String, pos: pumpkin_util::math::vector3::Vector3<f64>) -> Self {
        Self {
            event_id,
            pos,
            cancelled: false,
        }
    }
}
impl super::Payload for GenericGameEvent {
    fn get_name_static() -> &'static str {
        stringify!(GenericGameEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(GenericGameEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for GenericGameEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct LootGenerateEvent {
    pub loot_table: String,
    pub cancelled: bool,
}
impl LootGenerateEvent {
    #[must_use]
    pub const fn new(loot_table: String) -> Self {
        Self {
            loot_table,
            cancelled: false,
        }
    }
}
impl super::Payload for LootGenerateEvent {
    fn get_name_static() -> &'static str {
        stringify!(LootGenerateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(LootGenerateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for LootGenerateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PortalCreateEvent {
    pub pos: pumpkin_util::math::position::BlockPos,
    pub portal_type: PortalType,
    pub cancelled: bool,
}
impl PortalCreateEvent {
    #[must_use]
    pub const fn new(pos: pumpkin_util::math::position::BlockPos, portal_type: PortalType) -> Self {
        Self {
            pos,
            portal_type,
            cancelled: false,
        }
    }
}
impl super::Payload for PortalCreateEvent {
    fn get_name_static() -> &'static str {
        stringify!(PortalCreateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PortalCreateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PortalCreateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct StructureGrowEvent {
    pub pos: pumpkin_util::math::position::BlockPos,
    pub species: TreeType,
    pub bone_meal: bool,
    pub cancelled: bool,
}
impl StructureGrowEvent {
    #[must_use]
    pub const fn new(
        pos: pumpkin_util::math::position::BlockPos,
        species: TreeType,
        bone_meal: bool,
    ) -> Self {
        Self {
            pos,
            species,
            bone_meal,
            cancelled: false,
        }
    }
}
impl super::Payload for StructureGrowEvent {
    fn get_name_static() -> &'static str {
        stringify!(StructureGrowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(StructureGrowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for StructureGrowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct TimeSkipEvent {
    pub skip_amount: i64,
    pub cancelled: bool,
}
impl TimeSkipEvent {
    #[must_use]
    pub const fn new(skip_amount: i64) -> Self {
        Self {
            skip_amount,
            cancelled: false,
        }
    }
}
impl super::Payload for TimeSkipEvent {
    fn get_name_static() -> &'static str {
        stringify!(TimeSkipEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(TimeSkipEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for TimeSkipEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct WorldInitEvent {
    pub target_world: std::sync::Arc<crate::world::World>,
}
impl WorldInitEvent {
    #[must_use]
    pub const fn new(target_world: std::sync::Arc<crate::world::World>) -> Self {
        Self { target_world }
    }
}
impl super::Payload for WorldInitEvent {
    fn get_name_static() -> &'static str {
        stringify!(WorldInitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WorldInitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct WorldSaveEvent {
    pub world_name: String,
    pub cancelled: bool,
}
impl WorldSaveEvent {
    #[must_use]
    pub const fn new(world_name: String) -> Self {
        Self {
            world_name,
            cancelled: false,
        }
    }
}
impl super::Payload for WorldSaveEvent {
    fn get_name_static() -> &'static str {
        stringify!(WorldSaveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WorldSaveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for WorldSaveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryOpenEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl InventoryOpenEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryOpenEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryOpenEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryOpenEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryOpenEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryDragEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl InventoryDragEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryDragEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryDragEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryDragEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryDragEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CraftItemEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub recipe_id: String,
    pub cancelled: bool,
}
impl CraftItemEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        recipe_id: String,
    ) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
impl super::Payload for CraftItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(CraftItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CraftItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CraftItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FurnaceSmeltEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub source_item: String,
    pub result_item: String,
    pub cancelled: bool,
}
impl FurnaceSmeltEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        source_item: String,
        result_item: String,
    ) -> Self {
        Self {
            block_pos,
            source_item,
            result_item,
            cancelled: false,
        }
    }
}
impl super::Payload for FurnaceSmeltEvent {
    fn get_name_static() -> &'static str {
        stringify!(FurnaceSmeltEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FurnaceSmeltEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FurnaceSmeltEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BrewEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub fuel_level: u8,
    pub cancelled: bool,
}
impl BrewEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, fuel_level: u8) -> Self {
        Self {
            block_pos,
            fuel_level,
            cancelled: false,
        }
    }
}
impl super::Payload for BrewEvent {
    fn get_name_static() -> &'static str {
        stringify!(BrewEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BrewEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BrewEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BrewingStandFuelEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub fuel_power: u16,
    pub cancelled: bool,
}
impl BrewingStandFuelEvent {
    #[must_use]
    pub const fn new(block_pos: pumpkin_util::math::position::BlockPos, fuel_power: u16) -> Self {
        Self {
            block_pos,
            fuel_power,
            cancelled: false,
        }
    }
}
impl super::Payload for BrewingStandFuelEvent {
    fn get_name_static() -> &'static str {
        stringify!(BrewingStandFuelEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BrewingStandFuelEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BrewingStandFuelEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FurnaceBurnEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub fuel_item: String,
    pub burn_time: u32,
    pub cancelled: bool,
}
impl FurnaceBurnEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        fuel_item: String,
        burn_time: u32,
    ) -> Self {
        Self {
            block_pos,
            fuel_item,
            burn_time,
            cancelled: false,
        }
    }
}
impl super::Payload for FurnaceBurnEvent {
    fn get_name_static() -> &'static str {
        stringify!(FurnaceBurnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FurnaceBurnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FurnaceBurnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FurnaceExtractEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub item_id: String,
    pub item_amount: u32,
    pub exp_gained: f32,
}
impl FurnaceExtractEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        item_id: String,
        item_amount: u32,
        exp_gained: f32,
    ) -> Self {
        Self {
            player,
            block_pos,
            item_id,
            item_amount,
            exp_gained,
        }
    }
}
impl super::Payload for FurnaceExtractEvent {
    fn get_name_static() -> &'static str {
        stringify!(FurnaceExtractEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FurnaceExtractEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct FurnaceStartSmeltEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub source_item: String,
    pub cooking_time: u32,
    pub cancelled: bool,
}
impl FurnaceStartSmeltEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        source_item: String,
        cooking_time: u32,
    ) -> Self {
        Self {
            block_pos,
            source_item,
            cooking_time,
            cancelled: false,
        }
    }
}
impl super::Payload for FurnaceStartSmeltEvent {
    fn get_name_static() -> &'static str {
        stringify!(FurnaceStartSmeltEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FurnaceStartSmeltEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FurnaceStartSmeltEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct HopperInventorySearchEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub search_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl HopperInventorySearchEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        search_pos: pumpkin_util::math::position::BlockPos,
    ) -> Self {
        Self {
            block_pos,
            search_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for HopperInventorySearchEvent {
    fn get_name_static() -> &'static str {
        stringify!(HopperInventorySearchEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(HopperInventorySearchEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for HopperInventorySearchEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryCreativeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub slot: i16,
    pub item_id: String,
    pub item_count: u8,
    pub cancelled: bool,
}
impl InventoryCreativeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        slot: i16,
        item_id: String,
        item_count: u8,
    ) -> Self {
        Self {
            player,
            slot,
            item_id,
            item_count,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryCreativeEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryCreativeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryCreativeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryCreativeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryInteractEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl InventoryInteractEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryInteractEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryInteractEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryInteractEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryInteractEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryMoveItemEvent {
    pub source_pos: pumpkin_util::math::position::BlockPos,
    pub target_pos: pumpkin_util::math::position::BlockPos,
    pub item_id: String,
    pub item_amount: u32,
    pub cancelled: bool,
}
impl InventoryMoveItemEvent {
    #[must_use]
    pub const fn new(
        source_pos: pumpkin_util::math::position::BlockPos,
        target_pos: pumpkin_util::math::position::BlockPos,
        item_id: String,
        item_amount: u32,
    ) -> Self {
        Self {
            source_pos,
            target_pos,
            item_id,
            item_amount,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryMoveItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryMoveItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryMoveItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryMoveItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryPickupItemEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub item_entity_id: i32,
    pub item_id: String,
    pub cancelled: bool,
}
impl InventoryPickupItemEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        item_entity_id: i32,
        item_id: String,
    ) -> Self {
        Self {
            block_pos,
            item_entity_id,
            item_id,
            cancelled: false,
        }
    }
}
impl super::Payload for InventoryPickupItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryPickupItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryPickupItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for InventoryPickupItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PrepareAnvilEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub rename_text: String,
    pub repair_cost: u32,
}
impl PrepareAnvilEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        rename_text: String,
        repair_cost: u32,
    ) -> Self {
        Self {
            player,
            rename_text,
            repair_cost,
        }
    }
}
impl super::Payload for PrepareAnvilEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareAnvilEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareAnvilEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PrepareGrindstoneEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub result_item: Option<String>,
}
impl PrepareGrindstoneEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        result_item: Option<String>,
    ) -> Self {
        Self {
            player,
            result_item,
        }
    }
}
impl super::Payload for PrepareGrindstoneEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareGrindstoneEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareGrindstoneEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PrepareInventoryResultEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub result_item: Option<String>,
}
impl PrepareInventoryResultEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        result_item: Option<String>,
    ) -> Self {
        Self {
            player,
            result_item,
        }
    }
}
impl super::Payload for PrepareInventoryResultEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareInventoryResultEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareInventoryResultEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PrepareItemCraftEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub recipe_id: String,
    pub cancelled: bool,
}
impl PrepareItemCraftEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        recipe_id: String,
    ) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PrepareItemCraftEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareItemCraftEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareItemCraftEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PrepareItemCraftEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PrepareSmithingEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub result_item: Option<String>,
}
impl PrepareSmithingEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        result_item: Option<String>,
    ) -> Self {
        Self {
            player,
            result_item,
        }
    }
}
impl super::Payload for PrepareSmithingEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareSmithingEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareSmithingEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct SmithItemEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub recipe_id: String,
    pub cancelled: bool,
}
impl SmithItemEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        recipe_id: String,
    ) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
impl super::Payload for SmithItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(SmithItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SmithItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SmithItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct TradeSelectEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub slot_index: u8,
    pub cancelled: bool,
}
impl TradeSelectEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        slot_index: u8,
    ) -> Self {
        Self {
            player,
            slot_index,
            cancelled: false,
        }
    }
}
impl super::Payload for TradeSelectEvent {
    fn get_name_static() -> &'static str {
        stringify!(TradeSelectEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(TradeSelectEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for TradeSelectEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleBlockCollisionEvent {
    pub vehicle_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl VehicleBlockCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            vehicle_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleBlockCollisionEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleBlockCollisionEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleBlockCollisionEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleBlockCollisionEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleCollisionEvent {
    pub vehicle_id: i32,
    pub cancelled: bool,
}
impl VehicleCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleCollisionEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleCollisionEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleCollisionEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleCollisionEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleCreateEvent {
    pub vehicle_id: i32,
    pub cancelled: bool,
}
impl VehicleCreateEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleCreateEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleCreateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleCreateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleCreateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleDamageEvent {
    pub vehicle_id: i32,
    pub damage: f32,
    pub attacker_id: Option<i32>,
    pub cancelled: bool,
}
impl VehicleDamageEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, damage: f32, attacker_id: Option<i32>) -> Self {
        Self {
            vehicle_id,
            damage,
            attacker_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleDamageEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleDamageEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleDamageEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleDamageEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleDestroyEvent {
    pub vehicle_id: i32,
    pub attacker_id: Option<i32>,
    pub cancelled: bool,
}
impl VehicleDestroyEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, attacker_id: Option<i32>) -> Self {
        Self {
            vehicle_id,
            attacker_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleDestroyEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleDestroyEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleDestroyEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleDestroyEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleEnterEvent {
    pub vehicle_id: i32,
    pub entered_id: i32,
    pub cancelled: bool,
}
impl VehicleEnterEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, entered_id: i32) -> Self {
        Self {
            vehicle_id,
            entered_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleEnterEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleEnterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleEnterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleEnterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleEntityCollisionEvent {
    pub vehicle_id: i32,
    pub collided_entity_id: i32,
    pub cancelled: bool,
}
impl VehicleEntityCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, collided_entity_id: i32) -> Self {
        Self {
            vehicle_id,
            collided_entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleEntityCollisionEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleEntityCollisionEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleEntityCollisionEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleEntityCollisionEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleExitEvent {
    pub vehicle_id: i32,
    pub exited_id: i32,
    pub cancelled: bool,
}
impl VehicleExitEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, exited_id: i32) -> Self {
        Self {
            vehicle_id,
            exited_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleExitEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleExitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleExitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleExitEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleMoveEvent {
    pub vehicle_id: i32,
    pub from_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub to_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl VehicleMoveEvent {
    #[must_use]
    pub const fn new(
        vehicle_id: i32,
        from_position: pumpkin_util::math::vector3::Vector3<f64>,
        to_position: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            vehicle_id,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleMoveEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleMoveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleMoveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleMoveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VehicleUpdateEvent {
    pub vehicle_id: i32,
    pub cancelled: bool,
}
impl VehicleUpdateEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
impl super::Payload for VehicleUpdateEvent {
    fn get_name_static() -> &'static str {
        stringify!(VehicleUpdateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VehicleUpdateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VehicleUpdateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PrepareItemEnchantEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub offers: Vec<EnchantmentOffer>,
    pub bookshelf_count: i32,
    pub cancelled: bool,
}
impl PrepareItemEnchantEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item: pumpkin_data::item_stack::ItemStack,
        offers: Vec<EnchantmentOffer>,
        bookshelf_count: i32,
    ) -> Self {
        Self {
            player,
            item,
            offers,
            bookshelf_count,
            cancelled: false,
        }
    }
}
impl super::Payload for PrepareItemEnchantEvent {
    fn get_name_static() -> &'static str {
        stringify!(PrepareItemEnchantEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PrepareItemEnchantEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PrepareItemEnchantEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EnchantItemEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub option: i32,
    pub cost: i32,
    pub enchantments_to_add: Vec<EnchantmentValue>,
    pub cancelled: bool,
}
impl EnchantItemEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item: pumpkin_data::item_stack::ItemStack,
        option: i32,
        cost: i32,
        enchantments_to_add: Vec<EnchantmentValue>,
    ) -> Self {
        Self {
            player,
            item,
            option,
            cost,
            enchantments_to_add,
            cancelled: false,
        }
    }
}
impl super::Payload for EnchantItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(EnchantItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EnchantItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EnchantItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct MapInitializeEvent {
    pub map_id: i32,
}
impl MapInitializeEvent {
    #[must_use]
    pub const fn new(map_id: i32) -> Self {
        Self { map_id }
    }
}
impl super::Payload for MapInitializeEvent {
    fn get_name_static() -> &'static str {
        stringify!(MapInitializeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(MapInitializeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct HangingBreakEvent {
    pub entity: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub remover: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    pub cancelled: bool,
}
impl HangingBreakEvent {
    #[must_use]
    pub const fn new(
        entity: std::sync::Arc<dyn crate::entity::EntityBase>,
        remover: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    ) -> Self {
        Self {
            entity,
            remover,
            cancelled: false,
        }
    }
}
impl super::Payload for HangingBreakEvent {
    fn get_name_static() -> &'static str {
        stringify!(HangingBreakEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(HangingBreakEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for HangingBreakEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct HangingBreakByEntityEvent {
    pub entity: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub remover: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub cancelled: bool,
}
impl HangingBreakByEntityEvent {
    #[must_use]
    pub const fn new(
        entity: std::sync::Arc<dyn crate::entity::EntityBase>,
        remover: std::sync::Arc<dyn crate::entity::EntityBase>,
    ) -> Self {
        Self {
            entity,
            remover,
            cancelled: false,
        }
    }
}
impl super::Payload for HangingBreakByEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(HangingBreakByEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(HangingBreakByEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for HangingBreakByEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct HangingPlaceEvent {
    pub entity: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub block_face: pumpkin_data::BlockDirection,
    pub cancelled: bool,
}
impl HangingPlaceEvent {
    #[must_use]
    pub const fn new(
        entity: std::sync::Arc<dyn crate::entity::EntityBase>,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
        block_pos: pumpkin_util::math::position::BlockPos,
        block_face: pumpkin_data::BlockDirection,
    ) -> Self {
        Self {
            entity,
            player,
            block_pos,
            block_face,
            cancelled: false,
        }
    }
}
impl super::Payload for HangingPlaceEvent {
    fn get_name_static() -> &'static str {
        stringify!(HangingPlaceEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(HangingPlaceEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for HangingPlaceEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BellResonateEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub cancelled: bool,
}
impl BellResonateEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            cancelled: false,
        }
    }
}
impl super::Payload for BellResonateEvent {
    fn get_name_static() -> &'static str {
        stringify!(BellResonateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BellResonateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BellResonateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BellRingEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    pub direction: Option<pumpkin_data::BlockDirection>,
    pub cancelled: bool,
}
impl BellRingEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
        direction: Option<pumpkin_data::BlockDirection>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            entity,
            direction,
            cancelled: false,
        }
    }
}
impl super::Payload for BellRingEvent {
    fn get_name_static() -> &'static str {
        stringify!(BellRingEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BellRingEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BellRingEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockBrushEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl BlockBrushEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        player: std::sync::Arc<crate::entity::player::Player>,
        item: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            player,
            item,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockBrushEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockBrushEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockBrushEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockBrushEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockCookEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub source: pumpkin_data::item_stack::ItemStack,
    pub result: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl BlockCookEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        source: pumpkin_data::item_stack::ItemStack,
        result: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            source,
            result,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockCookEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockCookEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockCookEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockCookEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockDamageAbortEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub item_in_hand: pumpkin_data::item_stack::ItemStack,
}
impl BlockDamageAbortEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        item_in_hand: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            player,
            block_pos,
            target_world,
            item_in_hand,
        }
    }
}
impl super::Payload for BlockDamageAbortEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDamageAbortEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDamageAbortEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct BlockDispenseArmorEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub target: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl BlockDispenseArmorEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        target: std::sync::Arc<dyn crate::entity::EntityBase>,
        item: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            target,
            item,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockDispenseArmorEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDispenseArmorEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDispenseArmorEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockDispenseArmorEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockDispenseLootEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub items: Vec<pumpkin_data::item_stack::ItemStack>,
    pub cancelled: bool,
}
impl BlockDispenseLootEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        items: Vec<pumpkin_data::item_stack::ItemStack>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            items,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockDispenseLootEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDispenseLootEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDispenseLootEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockDispenseLootEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockDropItemEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub items: Vec<pumpkin_data::item_stack::ItemStack>,
    pub cancelled: bool,
}
impl BlockDropItemEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
        items: Vec<pumpkin_data::item_stack::ItemStack>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            player,
            items,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockDropItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockDropItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockDropItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockDropItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockExpEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub exp: i32,
}
impl BlockExpEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        exp: i32,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            exp,
        }
    }
}
impl super::Payload for BlockExpEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockExpEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockExpEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct BlockFertilizeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub changed_blocks: Vec<(
        pumpkin_util::math::position::BlockPos,
        pumpkin_data::BlockStateId,
    )>,
    pub cancelled: bool,
}
impl BlockFertilizeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
        changed_blocks: Vec<(
            pumpkin_util::math::position::BlockPos,
            pumpkin_data::BlockStateId,
        )>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            player,
            changed_blocks,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockFertilizeEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockFertilizeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockFertilizeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockFertilizeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockMultiPlaceEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub placed_blocks: Vec<(
        pumpkin_util::math::position::BlockPos,
        pumpkin_data::BlockStateId,
    )>,
    pub cancelled: bool,
}
impl BlockMultiPlaceEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        target_world: std::sync::Arc<crate::world::World>,
        placed_blocks: Vec<(
            pumpkin_util::math::position::BlockPos,
            pumpkin_data::BlockStateId,
        )>,
    ) -> Self {
        Self {
            player,
            target_world,
            placed_blocks,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockMultiPlaceEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockMultiPlaceEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockMultiPlaceEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockMultiPlaceEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockReceiveGameEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub game_event: String,
    pub source_entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    pub cancelled: bool,
}
impl BlockReceiveGameEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        game_event: String,
        source_entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            game_event,
            source_entity,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockReceiveGameEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockReceiveGameEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockReceiveGameEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockReceiveGameEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockShearEntityEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub target: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl BlockShearEntityEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        target: std::sync::Arc<dyn crate::entity::EntityBase>,
        item: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            target,
            item,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockShearEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockShearEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockShearEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockShearEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BlockSpreadEvent {
    pub source_pos: pumpkin_util::math::position::BlockPos,
    pub target_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub new_state_id: pumpkin_data::BlockStateId,
    pub cancelled: bool,
}
impl BlockSpreadEvent {
    #[must_use]
    pub const fn new(
        source_pos: pumpkin_util::math::position::BlockPos,
        target_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        new_state_id: pumpkin_data::BlockStateId,
    ) -> Self {
        Self {
            source_pos,
            target_pos,
            target_world,
            new_state_id,
            cancelled: false,
        }
    }
}
impl super::Payload for BlockSpreadEvent {
    fn get_name_static() -> &'static str {
        stringify!(BlockSpreadEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BlockSpreadEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BlockSpreadEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BrewingStartEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub brewing_time: i32,
    pub cancelled: bool,
}
impl BrewingStartEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        brewing_time: i32,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            brewing_time,
            cancelled: false,
        }
    }
}
impl super::Payload for BrewingStartEvent {
    fn get_name_static() -> &'static str {
        stringify!(BrewingStartEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BrewingStartEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BrewingStartEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CampfireStartEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub slot: u8,
    pub cooking_time: i32,
    pub cancelled: bool,
}
impl CampfireStartEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        item: pumpkin_data::item_stack::ItemStack,
        slot: u8,
        cooking_time: i32,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            item,
            slot,
            cooking_time,
            cancelled: false,
        }
    }
}
impl super::Payload for CampfireStartEvent {
    fn get_name_static() -> &'static str {
        stringify!(CampfireStartEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CampfireStartEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CampfireStartEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CauldronLevelChangeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub old_level: i32,
    pub new_level: i32,
    pub reason: CauldronChangeReason,
    pub entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    pub cancelled: bool,
}
impl CauldronLevelChangeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        old_level: i32,
        new_level: i32,
        reason: CauldronChangeReason,
        entity: Option<std::sync::Arc<dyn crate::entity::EntityBase>>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            old_level,
            new_level,
            reason,
            entity,
            cancelled: false,
        }
    }
}
impl super::Payload for CauldronLevelChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(CauldronLevelChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CauldronLevelChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CauldronLevelChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CrafterCraftEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub result: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl CrafterCraftEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        result: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            result,
            cancelled: false,
        }
    }
}
impl super::Payload for CrafterCraftEvent {
    fn get_name_static() -> &'static str {
        stringify!(CrafterCraftEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CrafterCraftEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CrafterCraftEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityBlockFormEvent {
    pub entity: std::sync::Arc<dyn crate::entity::EntityBase>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub new_state_id: pumpkin_data::BlockStateId,
    pub cancelled: bool,
}
impl EntityBlockFormEvent {
    #[must_use]
    pub const fn new(
        entity: std::sync::Arc<dyn crate::entity::EntityBase>,
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        new_state_id: pumpkin_data::BlockStateId,
    ) -> Self {
        Self {
            entity,
            block_pos,
            target_world,
            new_state_id,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityBlockFormEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityBlockFormEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityBlockFormEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityBlockFormEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FluidLevelChangeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub new_state_id: pumpkin_data::BlockStateId,
    pub cancelled: bool,
}
impl FluidLevelChangeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        new_state_id: pumpkin_data::BlockStateId,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            new_state_id,
            cancelled: false,
        }
    }
}
impl super::Payload for FluidLevelChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(FluidLevelChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FluidLevelChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FluidLevelChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct InventoryBlockStartEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
}
impl InventoryBlockStartEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
        }
    }
}
impl super::Payload for InventoryBlockStartEvent {
    fn get_name_static() -> &'static str {
        stringify!(InventoryBlockStartEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(InventoryBlockStartEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct LeavesDecayEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub cancelled: bool,
}
impl LeavesDecayEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            cancelled: false,
        }
    }
}
impl super::Payload for LeavesDecayEvent {
    fn get_name_static() -> &'static str {
        stringify!(LeavesDecayEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(LeavesDecayEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for LeavesDecayEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct MoistureChangeEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub new_moisture: i32,
    pub cancelled: bool,
}
impl MoistureChangeEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        new_moisture: i32,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            new_moisture,
            cancelled: false,
        }
    }
}
impl super::Payload for MoistureChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(MoistureChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(MoistureChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for MoistureChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SculkBloomEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub charge: i32,
    pub cancelled: bool,
}
impl SculkBloomEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        charge: i32,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            charge,
            cancelled: false,
        }
    }
}
impl super::Payload for SculkBloomEvent {
    fn get_name_static() -> &'static str {
        stringify!(SculkBloomEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SculkBloomEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SculkBloomEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VaultDisplayItemEvent {
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub item: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl VaultDisplayItemEvent {
    #[must_use]
    pub const fn new(
        block_pos: pumpkin_util::math::position::BlockPos,
        target_world: std::sync::Arc<crate::world::World>,
        item: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            block_pos,
            target_world,
            item,
            cancelled: false,
        }
    }
}
impl super::Payload for VaultDisplayItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(VaultDisplayItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VaultDisplayItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VaultDisplayItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CreatureSpawnEvent {
    pub entity_id: i32,
    pub entity_type: String,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub target_world: std::sync::Arc<crate::world::World>,
    pub spawn_reason: CreatureSpawnReason,
    pub player: Option<std::sync::Arc<crate::entity::player::Player>>,
    pub cancelled: bool,
}
impl CreatureSpawnEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: String,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        target_world: std::sync::Arc<crate::world::World>,
        spawn_reason: CreatureSpawnReason,
        player: Option<std::sync::Arc<crate::entity::player::Player>>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            position,
            target_world,
            spawn_reason,
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for CreatureSpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(CreatureSpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CreatureSpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CreatureSpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EnderDragonChangePhaseEvent {
    pub entity_id: i32,
    pub current_phase: String,
    pub new_phase: String,
    pub cancelled: bool,
}
impl EnderDragonChangePhaseEvent {
    #[must_use]
    pub const fn new(entity_id: i32, current_phase: String, new_phase: String) -> Self {
        Self {
            entity_id,
            current_phase,
            new_phase,
            cancelled: false,
        }
    }
}
impl super::Payload for EnderDragonChangePhaseEvent {
    fn get_name_static() -> &'static str {
        stringify!(EnderDragonChangePhaseEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EnderDragonChangePhaseEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EnderDragonChangePhaseEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityBreakDoorEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityBreakDoorEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityBreakDoorEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityBreakDoorEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityBreakDoorEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityBreakDoorEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityChangeBlockEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub new_block: String,
    pub cancelled: bool,
}
impl EntityChangeBlockEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        block_pos: pumpkin_util::math::position::BlockPos,
        new_block: String,
    ) -> Self {
        Self {
            entity_id,
            block_pos,
            new_block,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityChangeBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityChangeBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityChangeBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityChangeBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDamageByBlockEvent {
    pub entity_id: i32,
    pub damager_pos: Option<pumpkin_util::math::position::BlockPos>,
    pub damage: f32,
    pub cause: String,
    pub cancelled: bool,
}
impl EntityDamageByBlockEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        damager_pos: Option<pumpkin_util::math::position::BlockPos>,
        damage: f32,
        cause: String,
    ) -> Self {
        Self {
            entity_id,
            damager_pos,
            damage,
            cause,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDamageByBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDamageByBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDamageByBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDamageByBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDamageByEntityEvent {
    pub entity_id: i32,
    pub damager_id: i32,
    pub damage: f32,
    pub cause: String,
    pub cancelled: bool,
}
impl EntityDamageByEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, damager_id: i32, damage: f32, cause: String) -> Self {
        Self {
            entity_id,
            damager_id,
            damage,
            cause,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDamageByEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDamageByEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDamageByEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDamageByEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityDropItemEvent {
    pub entity_id: i32,
    pub item_name: String,
    pub count: u8,
    pub cancelled: bool,
}
impl EntityDropItemEvent {
    #[must_use]
    pub const fn new(entity_id: i32, item_name: String, count: u8) -> Self {
        Self {
            entity_id,
            item_name,
            count,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityDropItemEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityDropItemEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityDropItemEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityDropItemEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityEnterBlockEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityEnterBlockEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityEnterBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityEnterBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityEnterBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityEnterBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityExhaustionEvent {
    pub entity_id: i32,
    pub exhaustion: f32,
    pub cancelled: bool,
}
impl EntityExhaustionEvent {
    #[must_use]
    pub const fn new(entity_id: i32, exhaustion: f32) -> Self {
        Self {
            entity_id,
            exhaustion,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityExhaustionEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityExhaustionEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityExhaustionEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityExhaustionEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityInteractEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityInteractEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityInteractEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityInteractEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityInteractEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityInteractEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityKnockbackEvent {
    pub entity_id: i32,
    pub hit_by_id: Option<i32>,
    pub knockback: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl EntityKnockbackEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        hit_by_id: Option<i32>,
        knockback: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            hit_by_id,
            knockback,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityKnockbackEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityKnockbackEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityKnockbackEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityKnockbackEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPlaceEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub block_name: String,
    pub cancelled: bool,
}
impl EntityPlaceEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        block_pos: pumpkin_util::math::position::BlockPos,
        block_name: String,
    ) -> Self {
        Self {
            entity_id,
            block_pos,
            block_name,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPlaceEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPlaceEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPlaceEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPlaceEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPoseChangeEvent {
    pub entity_id: i32,
    pub pose: String,
    pub cancelled: bool,
}
impl EntityPoseChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, pose: String) -> Self {
        Self {
            entity_id,
            pose,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPoseChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPoseChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPoseChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPoseChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPotionEffectEvent {
    pub entity_id: i32,
    pub effect_name: String,
    pub duration: i32,
    pub amplifier: u8,
    pub cancelled: bool,
}
impl EntityPotionEffectEvent {
    #[must_use]
    pub const fn new(entity_id: i32, effect_name: String, duration: i32, amplifier: u8) -> Self {
        Self {
            entity_id,
            effect_name,
            duration,
            amplifier,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPotionEffectEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPotionEffectEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPotionEffectEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPotionEffectEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntitySpellCastEvent {
    pub entity_id: i32,
    pub spell: String,
    pub cancelled: bool,
}
impl EntitySpellCastEvent {
    #[must_use]
    pub const fn new(entity_id: i32, spell: String) -> Self {
        Self {
            entity_id,
            spell,
            cancelled: false,
        }
    }
}
impl super::Payload for EntitySpellCastEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntitySpellCastEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntitySpellCastEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntitySpellCastEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTargetLivingEntityEvent {
    pub entity_id: i32,
    pub target_id: Option<i32>,
    pub reason: String,
    pub cancelled: bool,
}
impl EntityTargetLivingEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: Option<i32>, reason: String) -> Self {
        Self {
            entity_id,
            target_id,
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTargetLivingEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTargetLivingEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTargetLivingEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTargetLivingEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityToggleSitEvent {
    pub entity_id: i32,
    pub sitting: bool,
    pub cancelled: bool,
}
impl EntityToggleSitEvent {
    #[must_use]
    pub const fn new(entity_id: i32, sitting: bool) -> Self {
        Self {
            entity_id,
            sitting,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityToggleSitEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityToggleSitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityToggleSitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityToggleSitEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityToggleSwimEvent {
    pub entity_id: i32,
    pub is_swimming: bool,
    pub cancelled: bool,
}
impl EntityToggleSwimEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_swimming: bool) -> Self {
        Self {
            entity_id,
            is_swimming,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityToggleSwimEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityToggleSwimEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityToggleSwimEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityToggleSwimEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ExplosionPrimeEvent {
    pub entity_id: i32,
    pub radius: f32,
    pub fire: bool,
    pub cancelled: bool,
}
impl ExplosionPrimeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, radius: f32, fire: bool) -> Self {
        Self {
            entity_id,
            radius,
            fire,
            cancelled: false,
        }
    }
}
impl super::Payload for ExplosionPrimeEvent {
    fn get_name_static() -> &'static str {
        stringify!(ExplosionPrimeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ExplosionPrimeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ExplosionPrimeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FireworkExplodeEvent {
    pub entity_id: i32,
    pub cancelled: bool,
}
impl FireworkExplodeEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for FireworkExplodeEvent {
    fn get_name_static() -> &'static str {
        stringify!(FireworkExplodeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FireworkExplodeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FireworkExplodeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct FoodLevelChangeEvent {
    pub entity_id: i32,
    pub food_level: u8,
    pub cancelled: bool,
}
impl FoodLevelChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, food_level: u8) -> Self {
        Self {
            entity_id,
            food_level,
            cancelled: false,
        }
    }
}
impl super::Payload for FoodLevelChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(FoodLevelChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(FoodLevelChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for FoodLevelChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ItemDespawnEvent {
    pub entity_id: i32,
    pub cancelled: bool,
}
impl ItemDespawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for ItemDespawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(ItemDespawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ItemDespawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ItemDespawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ItemMergeEvent {
    pub entity_id: i32,
    pub target_id: i32,
    pub cancelled: bool,
}
impl ItemMergeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: i32) -> Self {
        Self {
            entity_id,
            target_id,
            cancelled: false,
        }
    }
}
impl super::Payload for ItemMergeEvent {
    fn get_name_static() -> &'static str {
        stringify!(ItemMergeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ItemMergeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ItemMergeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ItemSpawnEvent {
    pub entity_id: i32,
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub item_name: String,
    pub cancelled: bool,
}
impl ItemSpawnEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        position: pumpkin_util::math::vector3::Vector3<f64>,
        item_name: String,
    ) -> Self {
        Self {
            entity_id,
            position,
            item_name,
            cancelled: false,
        }
    }
}
impl super::Payload for ItemSpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(ItemSpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ItemSpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ItemSpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PiglinBarterEvent {
    pub entity_id: i32,
    pub input_item: pumpkin_data::item_stack::ItemStack,
    pub outcome: Vec<pumpkin_data::item_stack::ItemStack>,
    pub cancelled: bool,
}
impl PiglinBarterEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        input_item: pumpkin_data::item_stack::ItemStack,
        outcome: Vec<pumpkin_data::item_stack::ItemStack>,
    ) -> Self {
        Self {
            entity_id,
            input_item,
            outcome,
            cancelled: false,
        }
    }
}
impl super::Payload for PiglinBarterEvent {
    fn get_name_static() -> &'static str {
        stringify!(PiglinBarterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PiglinBarterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PiglinBarterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ProjectileHitEvent {
    pub entity_id: i32,
    pub hit_position: pumpkin_util::math::vector3::Vector3<f64>,
    pub hit_entity_id: Option<i32>,
    pub cancelled: bool,
}
impl ProjectileHitEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        hit_position: pumpkin_util::math::vector3::Vector3<f64>,
        hit_entity_id: Option<i32>,
    ) -> Self {
        Self {
            entity_id,
            hit_position,
            hit_entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for ProjectileHitEvent {
    fn get_name_static() -> &'static str {
        stringify!(ProjectileHitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ProjectileHitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ProjectileHitEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ProjectileLaunchEvent {
    pub entity_id: i32,
    pub shooter_id: Option<i32>,
    pub cancelled: bool,
}
impl ProjectileLaunchEvent {
    #[must_use]
    pub const fn new(entity_id: i32, shooter_id: Option<i32>) -> Self {
        Self {
            entity_id,
            shooter_id,
            cancelled: false,
        }
    }
}
impl super::Payload for ProjectileLaunchEvent {
    fn get_name_static() -> &'static str {
        stringify!(ProjectileLaunchEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ProjectileLaunchEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ProjectileLaunchEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SheepDyeWoolEvent {
    pub entity_id: i32,
    pub dye_color: u8,
    pub player_id: Option<i32>,
    pub cancelled: bool,
}
impl SheepDyeWoolEvent {
    #[must_use]
    pub const fn new(entity_id: i32, dye_color: u8, player_id: Option<i32>) -> Self {
        Self {
            entity_id,
            dye_color,
            player_id,
            cancelled: false,
        }
    }
}
impl super::Payload for SheepDyeWoolEvent {
    fn get_name_static() -> &'static str {
        stringify!(SheepDyeWoolEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SheepDyeWoolEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SheepDyeWoolEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SheepRegrowWoolEvent {
    pub entity_id: i32,
    pub cancelled: bool,
}
impl SheepRegrowWoolEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for SheepRegrowWoolEvent {
    fn get_name_static() -> &'static str {
        stringify!(SheepRegrowWoolEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SheepRegrowWoolEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SheepRegrowWoolEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SlimeSplitEvent {
    pub entity_id: i32,
    pub count: i32,
    pub cancelled: bool,
}
impl SlimeSplitEvent {
    #[must_use]
    pub const fn new(entity_id: i32, count: i32) -> Self {
        Self {
            entity_id,
            count,
            cancelled: false,
        }
    }
}
impl super::Payload for SlimeSplitEvent {
    fn get_name_static() -> &'static str {
        stringify!(SlimeSplitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SlimeSplitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SlimeSplitEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct StriderTemperatureChangeEvent {
    pub entity_id: i32,
    pub is_shivering: bool,
    pub cancelled: bool,
}
impl StriderTemperatureChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_shivering: bool) -> Self {
        Self {
            entity_id,
            is_shivering,
            cancelled: false,
        }
    }
}
impl super::Payload for StriderTemperatureChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(StriderTemperatureChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(StriderTemperatureChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for StriderTemperatureChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VillagerAcquireTradeEvent {
    pub entity_id: i32,
    pub recipe_index: i32,
    pub cancelled: bool,
}
impl VillagerAcquireTradeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, recipe_index: i32) -> Self {
        Self {
            entity_id,
            recipe_index,
            cancelled: false,
        }
    }
}
impl super::Payload for VillagerAcquireTradeEvent {
    fn get_name_static() -> &'static str {
        stringify!(VillagerAcquireTradeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VillagerAcquireTradeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VillagerAcquireTradeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VillagerCareerChangeEvent {
    pub entity_id: i32,
    pub profession: String,
    pub reason: String,
    pub cancelled: bool,
}
impl VillagerCareerChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, profession: String, reason: String) -> Self {
        Self {
            entity_id,
            profession,
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for VillagerCareerChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(VillagerCareerChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VillagerCareerChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VillagerCareerChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VillagerReplenishTradeEvent {
    pub entity_id: i32,
    pub restock_quantity: i32,
    pub cancelled: bool,
}
impl VillagerReplenishTradeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, restock_quantity: i32) -> Self {
        Self {
            entity_id,
            restock_quantity,
            cancelled: false,
        }
    }
}
impl super::Payload for VillagerReplenishTradeEvent {
    fn get_name_static() -> &'static str {
        stringify!(VillagerReplenishTradeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VillagerReplenishTradeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VillagerReplenishTradeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct WardenAngerChangeEvent {
    pub entity_id: i32,
    pub target_id: i32,
    pub old_anger: i32,
    pub new_anger: i32,
    pub cancelled: bool,
}
impl WardenAngerChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: i32, old_anger: i32, new_anger: i32) -> Self {
        Self {
            entity_id,
            target_id,
            old_anger,
            new_anger,
            cancelled: false,
        }
    }
}
impl super::Payload for WardenAngerChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(WardenAngerChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(WardenAngerChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for WardenAngerChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct AreaEffectCloudApplyEvent {
    pub entity_id: i32,
    pub affected_entities: Vec<i32>,
    pub cancelled: bool,
}
impl AreaEffectCloudApplyEvent {
    #[must_use]
    pub const fn new(entity_id: i32, affected_entities: Vec<i32>) -> Self {
        Self {
            entity_id,
            affected_entities,
            cancelled: false,
        }
    }
}
impl super::Payload for AreaEffectCloudApplyEvent {
    fn get_name_static() -> &'static str {
        stringify!(AreaEffectCloudApplyEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(AreaEffectCloudApplyEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for AreaEffectCloudApplyEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ArrowBodyCountChangeEvent {
    pub entity_id: i32,
    pub old_amount: u32,
    pub new_amount: u32,
    pub cancelled: bool,
}
impl ArrowBodyCountChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, old_amount: u32, new_amount: u32) -> Self {
        Self {
            entity_id,
            old_amount,
            new_amount,
            cancelled: false,
        }
    }
}
impl super::Payload for ArrowBodyCountChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(ArrowBodyCountChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ArrowBodyCountChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ArrowBodyCountChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct BatToggleSleepEvent {
    pub entity_id: i32,
    pub is_awake: bool,
    pub cancelled: bool,
}
impl BatToggleSleepEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_awake: bool) -> Self {
        Self {
            entity_id,
            is_awake,
            cancelled: false,
        }
    }
}
impl super::Payload for BatToggleSleepEvent {
    fn get_name_static() -> &'static str {
        stringify!(BatToggleSleepEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(BatToggleSleepEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for BatToggleSleepEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct CreeperPowerEvent {
    pub entity_id: i32,
    pub lightning_id: Option<i32>,
    pub cause: String,
    pub cancelled: bool,
}
impl CreeperPowerEvent {
    #[must_use]
    pub const fn new(entity_id: i32, lightning_id: Option<i32>, cause: String) -> Self {
        Self {
            entity_id,
            lightning_id,
            cause,
            cancelled: false,
        }
    }
}
impl super::Payload for CreeperPowerEvent {
    fn get_name_static() -> &'static str {
        stringify!(CreeperPowerEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(CreeperPowerEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for CreeperPowerEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityCombustByBlockEvent {
    pub entity_id: i32,
    pub combuster: pumpkin_util::math::position::BlockPos,
    pub duration: f32,
    pub cancelled: bool,
}
impl EntityCombustByBlockEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        combuster: pumpkin_util::math::position::BlockPos,
        duration: f32,
    ) -> Self {
        Self {
            entity_id,
            combuster,
            duration,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityCombustByBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityCombustByBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityCombustByBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityCombustByBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityCombustByEntityEvent {
    pub entity_id: i32,
    pub combuster_id: i32,
    pub duration: f32,
    pub cancelled: bool,
}
impl EntityCombustByEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, combuster_id: i32, duration: f32) -> Self {
        Self {
            entity_id,
            combuster_id,
            duration,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityCombustByEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityCombustByEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityCombustByEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityCombustByEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityKnockbackByEntityEvent {
    pub entity_id: i32,
    pub hit_by_id: i32,
    pub force: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub cancelled: bool,
}
impl EntityKnockbackByEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, hit_by_id: i32, force: f64, x: f64, y: f64, z: f64) -> Self {
        Self {
            entity_id,
            hit_by_id,
            force,
            x,
            y,
            z,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityKnockbackByEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityKnockbackByEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityKnockbackByEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityKnockbackByEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPortalEnterEvent {
    pub entity_id: i32,
    pub location: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityPortalEnterEvent {
    #[must_use]
    pub const fn new(entity_id: i32, location: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            location,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPortalEnterEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPortalEnterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPortalEnterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPortalEnterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityPortalExitEvent {
    pub entity_id: i32,
    pub from_pos: pumpkin_util::math::position::BlockPos,
    pub to_pos: Option<pumpkin_util::math::position::BlockPos>,
    pub cancelled: bool,
}
impl EntityPortalExitEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        from_pos: pumpkin_util::math::position::BlockPos,
        to_pos: Option<pumpkin_util::math::position::BlockPos>,
    ) -> Self {
        Self {
            entity_id,
            from_pos,
            to_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityPortalExitEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityPortalExitEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityPortalExitEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityPortalExitEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityRemoveEvent {
    pub entity_id: i32,
    pub cause: String,
    pub cancelled: bool,
}
impl EntityRemoveEvent {
    #[must_use]
    pub const fn new(entity_id: i32, cause: String) -> Self {
        Self {
            entity_id,
            cause,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityRemoveEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityRemoveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityRemoveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityRemoveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityTargetBlockEvent {
    pub entity_id: i32,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl EntityTargetBlockEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityTargetBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityTargetBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityTargetBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityTargetBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct EntityUnleashEvent {
    pub entity_id: i32,
    pub reason: String,
    pub cancelled: bool,
}
impl EntityUnleashEvent {
    #[must_use]
    pub const fn new(entity_id: i32, reason: String) -> Self {
        Self {
            entity_id,
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for EntityUnleashEvent {
    fn get_name_static() -> &'static str {
        stringify!(EntityUnleashEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(EntityUnleashEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for EntityUnleashEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ExpBottleEvent {
    pub entity_id: i32,
    pub experience: i32,
    pub location: pumpkin_util::math::position::BlockPos,
    pub show_effect: bool,
    pub cancelled: bool,
}
impl ExpBottleEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        experience: i32,
        location: pumpkin_util::math::position::BlockPos,
        show_effect: bool,
    ) -> Self {
        Self {
            entity_id,
            experience,
            location,
            show_effect,
            cancelled: false,
        }
    }
}
impl super::Payload for ExpBottleEvent {
    fn get_name_static() -> &'static str {
        stringify!(ExpBottleEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ExpBottleEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ExpBottleEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct HorseJumpEvent {
    pub entity_id: i32,
    pub power: f32,
    pub cancelled: bool,
}
impl HorseJumpEvent {
    #[must_use]
    pub const fn new(entity_id: i32, power: f32) -> Self {
        Self {
            entity_id,
            power,
            cancelled: false,
        }
    }
}
impl super::Payload for HorseJumpEvent {
    fn get_name_static() -> &'static str {
        stringify!(HorseJumpEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(HorseJumpEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for HorseJumpEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct LingeringPotionSplashEvent {
    pub entity_id: i32,
    pub location: pumpkin_util::math::position::BlockPos,
    pub potion_item: String,
    pub cancelled: bool,
}
impl LingeringPotionSplashEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        location: pumpkin_util::math::position::BlockPos,
        potion_item: String,
    ) -> Self {
        Self {
            entity_id,
            location,
            potion_item,
            cancelled: false,
        }
    }
}
impl super::Payload for LingeringPotionSplashEvent {
    fn get_name_static() -> &'static str {
        stringify!(LingeringPotionSplashEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(LingeringPotionSplashEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for LingeringPotionSplashEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PigZapEvent {
    pub entity_id: i32,
    pub lightning_id: i32,
    pub pig_zombie_id: i32,
    pub cancelled: bool,
}
impl PigZapEvent {
    #[must_use]
    pub const fn new(entity_id: i32, lightning_id: i32, pig_zombie_id: i32) -> Self {
        Self {
            entity_id,
            lightning_id,
            pig_zombie_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PigZapEvent {
    fn get_name_static() -> &'static str {
        stringify!(PigZapEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PigZapEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PigZapEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PigZombieAngerEvent {
    pub entity_id: i32,
    pub target_id: Option<i32>,
    pub new_anger: i32,
    pub cancelled: bool,
}
impl PigZombieAngerEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: Option<i32>, new_anger: i32) -> Self {
        Self {
            entity_id,
            target_id,
            new_anger,
            cancelled: false,
        }
    }
}
impl super::Payload for PigZombieAngerEvent {
    fn get_name_static() -> &'static str {
        stringify!(PigZombieAngerEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PigZombieAngerEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PigZombieAngerEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PotionSplashEvent {
    pub entity_id: i32,
    pub location: pumpkin_util::math::position::BlockPos,
    pub potion_item: String,
    pub affected_entities: Vec<i32>,
    pub cancelled: bool,
}
impl PotionSplashEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        location: pumpkin_util::math::position::BlockPos,
        potion_item: String,
        affected_entities: Vec<i32>,
    ) -> Self {
        Self {
            entity_id,
            location,
            potion_item,
            affected_entities,
            cancelled: false,
        }
    }
}
impl super::Payload for PotionSplashEvent {
    fn get_name_static() -> &'static str {
        stringify!(PotionSplashEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PotionSplashEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PotionSplashEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct SpawnerSpawnEvent {
    pub entity_id: i32,
    pub spawner_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl SpawnerSpawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32, spawner_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            spawner_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for SpawnerSpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(SpawnerSpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(SpawnerSpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for SpawnerSpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct TrialSpawnerSpawnEvent {
    pub entity_id: i32,
    pub spawner_pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl TrialSpawnerSpawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32, spawner_pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            entity_id,
            spawner_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for TrialSpawnerSpawnEvent {
    fn get_name_static() -> &'static str {
        stringify!(TrialSpawnerSpawnEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(TrialSpawnerSpawnEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for TrialSpawnerSpawnEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct VillagerReputationChangeEvent {
    pub entity_id: i32,
    pub target_id: i32,
    pub reputation_change: i32,
    pub cancelled: bool,
}
impl VillagerReputationChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: i32, reputation_change: i32) -> Self {
        Self {
            entity_id,
            target_id,
            reputation_change,
            cancelled: false,
        }
    }
}
impl super::Payload for VillagerReputationChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(VillagerReputationChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(VillagerReputationChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for VillagerReputationChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct AsyncPlayerChatEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub message: String,
    pub format: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl AsyncPlayerChatEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        message: String,
        format: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player,
            message,
            format,
            cancelled: false,
        }
    }
}
impl super::Payload for AsyncPlayerChatEvent {
    fn get_name_static() -> &'static str {
        stringify!(AsyncPlayerChatEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(AsyncPlayerChatEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for AsyncPlayerChatEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct AsyncPlayerPreLoginEvent {
    pub player_name: String,
    pub player_uuid: uuid::Uuid,
    pub ip_address: std::net::SocketAddr,
    pub kick_message: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl AsyncPlayerPreLoginEvent {
    #[must_use]
    pub const fn new(
        player_name: String,
        player_uuid: uuid::Uuid,
        ip_address: std::net::SocketAddr,
        kick_message: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player_name,
            player_uuid,
            ip_address,
            kick_message,
            cancelled: false,
        }
    }
}
impl super::Payload for AsyncPlayerPreLoginEvent {
    fn get_name_static() -> &'static str {
        stringify!(AsyncPlayerPreLoginEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(AsyncPlayerPreLoginEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for AsyncPlayerPreLoginEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerAdvancementDoneEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub advancement_id: String,
    pub cancelled: bool,
}
impl PlayerAdvancementDoneEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        advancement_id: String,
    ) -> Self {
        Self {
            player,
            advancement_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerAdvancementDoneEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerAdvancementDoneEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerAdvancementDoneEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerAdvancementDoneEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerAnimationEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub animation_type: PlayerAnimationType,
    pub cancelled: bool,
}
impl PlayerAnimationEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        animation_type: PlayerAnimationType,
    ) -> Self {
        Self {
            player,
            animation_type,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerAnimationEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerAnimationEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerAnimationEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerAnimationEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerArmorStandManipulateEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub armor_stand_id: i32,
    pub slot: u8,
    pub cancelled: bool,
}
impl PlayerArmorStandManipulateEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        armor_stand_id: i32,
        slot: u8,
    ) -> Self {
        Self {
            player,
            armor_stand_id,
            slot,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerArmorStandManipulateEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerArmorStandManipulateEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerArmorStandManipulateEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerArmorStandManipulateEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerBucketEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub bucket_item: String,
    pub cancelled: bool,
}
impl PlayerBucketEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        bucket_item: String,
    ) -> Self {
        Self {
            player,
            entity_id,
            bucket_item,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerBucketEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerBucketEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerBucketEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerBucketEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerChangedWorldEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub from_world: std::sync::Arc<crate::world::World>,
    pub to_world: std::sync::Arc<crate::world::World>,
    pub cancelled: bool,
}
impl PlayerChangedWorldEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        from_world: std::sync::Arc<crate::world::World>,
        to_world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            player,
            from_world,
            to_world,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerChangedWorldEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerChangedWorldEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerChangedWorldEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerChangedWorldEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerChannelEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub channel: String,
    pub cancelled: bool,
}
impl PlayerChannelEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        channel: String,
    ) -> Self {
        Self {
            player,
            channel,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerChannelEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerChannelEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerChannelEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerChannelEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerCommandPreprocessEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub command: String,
    pub cancelled: bool,
}
impl PlayerCommandPreprocessEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        command: String,
    ) -> Self {
        Self {
            player,
            command,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerCommandPreprocessEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerCommandPreprocessEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerCommandPreprocessEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerCommandPreprocessEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerEditBookEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub slot: u32,
    pub pages: Vec<String>,
    pub title: Option<String>,
    pub signing: bool,
    pub cancelled: bool,
}
impl PlayerEditBookEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        slot: u32,
        pages: Vec<String>,
        title: Option<String>,
        signing: bool,
    ) -> Self {
        Self {
            player,
            slot,
            pages,
            title,
            signing,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerEditBookEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerEditBookEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerEditBookEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerEditBookEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerElytraBoostEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub firework_id: i32,
    pub cancelled: bool,
}
impl PlayerElytraBoostEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        firework_id: i32,
    ) -> Self {
        Self {
            player,
            firework_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerElytraBoostEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerElytraBoostEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerElytraBoostEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerElytraBoostEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerExpCooldownChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub new_cooldown: i32,
    pub cancelled: bool,
}
impl PlayerExpCooldownChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        new_cooldown: i32,
    ) -> Self {
        Self {
            player,
            new_cooldown,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerExpCooldownChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerExpCooldownChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerExpCooldownChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerExpCooldownChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerHarvestBlockEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub harvested_items: Vec<pumpkin_data::item_stack::ItemStack>,
    pub cancelled: bool,
}
impl PlayerHarvestBlockEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        harvested_items: Vec<pumpkin_data::item_stack::ItemStack>,
    ) -> Self {
        Self {
            player,
            block_pos,
            harvested_items,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerHarvestBlockEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerHarvestBlockEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerHarvestBlockEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerHarvestBlockEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerHideEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub cancelled: bool,
}
impl PlayerHideEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
    ) -> Self {
        Self {
            player,
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerHideEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerHideEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerHideEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerHideEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerItemBreakEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
}
impl PlayerItemBreakEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
    ) -> Self {
        Self { player, item_name }
    }
}
impl super::Payload for PlayerItemBreakEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerItemBreakEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerItemBreakEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerItemMendEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
    pub repair_amount: i32,
    pub exp_consumed: i32,
    pub cancelled: bool,
}
impl PlayerItemMendEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
        repair_amount: i32,
        exp_consumed: i32,
    ) -> Self {
        Self {
            player,
            item_name,
            repair_amount,
            exp_consumed,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerItemMendEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerItemMendEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerItemMendEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerItemMendEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerKickEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub reason: String,
    pub cancelled: bool,
}
impl PlayerKickEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        reason: String,
    ) -> Self {
        Self {
            player,
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerKickEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerKickEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerKickEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerKickEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerLeashEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub holder_id: i32,
    pub cancelled: bool,
}
impl PlayerLeashEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        holder_id: i32,
    ) -> Self {
        Self {
            player,
            entity_id,
            holder_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerLeashEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLeashEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLeashEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerLeashEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerLevelChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub old_level: i32,
    pub new_level: i32,
}
impl PlayerLevelChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        old_level: i32,
        new_level: i32,
    ) -> Self {
        Self {
            player,
            old_level,
            new_level,
        }
    }
}
impl super::Payload for PlayerLevelChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLevelChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLevelChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
pub struct PlayerLocaleChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub new_locale: String,
    pub cancelled: bool,
}
impl PlayerLocaleChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        new_locale: String,
    ) -> Self {
        Self {
            player,
            new_locale,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerLocaleChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLocaleChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLocaleChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerLocaleChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerNameEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub name: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl PlayerNameEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        name: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player,
            entity_id,
            name,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerNameEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerNameEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerNameEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerNameEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerOpenSignEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub is_front: bool,
    pub cancelled: bool,
}
impl PlayerOpenSignEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        is_front: bool,
    ) -> Self {
        Self {
            player,
            block_pos,
            is_front,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerOpenSignEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerOpenSignEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerOpenSignEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerOpenSignEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerPortalEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub from_pos: pumpkin_util::math::position::BlockPos,
    pub to_pos: Option<pumpkin_util::math::position::BlockPos>,
    pub cancelled: bool,
}
impl PlayerPortalEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        from_pos: pumpkin_util::math::position::BlockPos,
        to_pos: Option<pumpkin_util::math::position::BlockPos>,
    ) -> Self {
        Self {
            player,
            from_pos,
            to_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerPortalEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerPortalEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerPortalEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerPortalEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerPreLoginEvent {
    pub player_name: String,
    pub player_uuid: uuid::Uuid,
    pub ip_address: std::net::SocketAddr,
    pub kick_message: pumpkin_util::text::TextComponent,
    pub cancelled: bool,
}
impl PlayerPreLoginEvent {
    #[must_use]
    pub const fn new(
        player_name: String,
        player_uuid: uuid::Uuid,
        ip_address: std::net::SocketAddr,
        kick_message: pumpkin_util::text::TextComponent,
    ) -> Self {
        Self {
            player_name,
            player_uuid,
            ip_address,
            kick_message,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerPreLoginEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerPreLoginEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerPreLoginEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerPreLoginEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRiptideEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub item_name: String,
    pub cancelled: bool,
}
impl PlayerRiptideEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        item_name: String,
    ) -> Self {
        Self {
            player,
            item_name,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerRiptideEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRiptideEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRiptideEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerRiptideEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerShearEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub hand: u8,
    pub cancelled: bool,
}
impl PlayerShearEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        hand: u8,
    ) -> Self {
        Self {
            player,
            entity_id,
            hand,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerShearEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerShearEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerShearEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerShearEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerShowEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub cancelled: bool,
}
impl PlayerShowEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
    ) -> Self {
        Self {
            player,
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerShowEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerShowEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerShowEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerShowEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerSpawnChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub new_spawn: Option<pumpkin_util::math::position::BlockPos>,
    pub forced: bool,
    pub cancelled: bool,
}
impl PlayerSpawnChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        new_spawn: Option<pumpkin_util::math::position::BlockPos>,
        forced: bool,
    ) -> Self {
        Self {
            player,
            new_spawn,
            forced,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerSpawnChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerSpawnChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerSpawnChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerSpawnChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerStatisticIncrementEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub statistic_id: String,
    pub amount: i32,
    pub cancelled: bool,
}
impl PlayerStatisticIncrementEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        statistic_id: String,
        amount: i32,
    ) -> Self {
        Self {
            player,
            statistic_id,
            amount,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerStatisticIncrementEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerStatisticIncrementEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerStatisticIncrementEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerStatisticIncrementEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerSwapHandsEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub cancelled: bool,
}
impl PlayerSwapHandsEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerSwapHandsEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerSwapHandsEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerSwapHandsEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerSwapHandsEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerTakeLecternBookEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub block_pos: pumpkin_util::math::position::BlockPos,
    pub book: pumpkin_data::item_stack::ItemStack,
    pub cancelled: bool,
}
impl PlayerTakeLecternBookEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        block_pos: pumpkin_util::math::position::BlockPos,
        book: pumpkin_data::item_stack::ItemStack,
    ) -> Self {
        Self {
            player,
            block_pos,
            book,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerTakeLecternBookEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerTakeLecternBookEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerTakeLecternBookEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerTakeLecternBookEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerUnleashEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub cancelled: bool,
}
impl PlayerUnleashEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
    ) -> Self {
        Self {
            player,
            entity_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerUnleashEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerUnleashEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerUnleashEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerUnleashEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerVelocityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub velocity: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl PlayerVelocityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        velocity: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            player,
            velocity,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerVelocityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerVelocityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerVelocityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerVelocityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerInputEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub input: String,
    pub cancelled: bool,
}
impl PlayerInputEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>, input: String) -> Self {
        Self {
            player,
            input,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerInputEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerInputEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerInputEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerInputEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerInteractAtEntityEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub entity_id: i32,
    pub clicked_x: f64,
    pub clicked_y: f64,
    pub clicked_z: f64,
    pub hand: u8,
    pub cancelled: bool,
}
impl PlayerInteractAtEntityEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        entity_id: i32,
        clicked_x: f64,
        clicked_y: f64,
        clicked_z: f64,
        hand: u8,
    ) -> Self {
        Self {
            player,
            entity_id,
            clicked_x,
            clicked_y,
            clicked_z,
            hand,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerInteractAtEntityEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerInteractAtEntityEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerInteractAtEntityEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerInteractAtEntityEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerLinksSendEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub links: Vec<String>,
    pub cancelled: bool,
}
impl PlayerLinksSendEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        links: Vec<String>,
    ) -> Self {
        Self {
            player,
            links,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerLinksSendEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerLinksSendEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerLinksSendEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerLinksSendEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerPickupArrowEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub arrow_id: i32,
    pub cancelled: bool,
}
impl PlayerPickupArrowEvent {
    #[must_use]
    pub const fn new(player: std::sync::Arc<crate::entity::player::Player>, arrow_id: i32) -> Self {
        Self {
            player,
            arrow_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerPickupArrowEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerPickupArrowEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerPickupArrowEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerPickupArrowEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRecipeBookClickEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub recipe_id: String,
    pub make_all: bool,
    pub cancelled: bool,
}
impl PlayerRecipeBookClickEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        recipe_id: String,
        make_all: bool,
    ) -> Self {
        Self {
            player,
            recipe_id,
            make_all,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerRecipeBookClickEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRecipeBookClickEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRecipeBookClickEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerRecipeBookClickEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRecipeBookSettingsChangeEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub book_type: String,
    pub is_open: bool,
    pub is_filtering: bool,
    pub cancelled: bool,
}
impl PlayerRecipeBookSettingsChangeEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        book_type: String,
        is_open: bool,
        is_filtering: bool,
    ) -> Self {
        Self {
            player,
            book_type,
            is_open,
            is_filtering,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerRecipeBookSettingsChangeEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRecipeBookSettingsChangeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRecipeBookSettingsChangeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerRecipeBookSettingsChangeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRecipeDiscoverEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub recipe_id: String,
    pub cancelled: bool,
}
impl PlayerRecipeDiscoverEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        recipe_id: String,
    ) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerRecipeDiscoverEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRecipeDiscoverEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRecipeDiscoverEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerRecipeDiscoverEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerRegisterChannelEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub channel: String,
    pub cancelled: bool,
}
impl PlayerRegisterChannelEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        channel: String,
    ) -> Self {
        Self {
            player,
            channel,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerRegisterChannelEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerRegisterChannelEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerRegisterChannelEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerRegisterChannelEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerResourcePackStatusEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub pack_id: String,
    pub status: String,
    pub cancelled: bool,
}
impl PlayerResourcePackStatusEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        pack_id: String,
        status: String,
    ) -> Self {
        Self {
            player,
            pack_id,
            status,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerResourcePackStatusEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerResourcePackStatusEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerResourcePackStatusEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerResourcePackStatusEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerSpawnLocationEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub spawn_pos: pumpkin_util::math::vector3::Vector3<f64>,
    pub cancelled: bool,
}
impl PlayerSpawnLocationEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        spawn_pos: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self {
            player,
            spawn_pos,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerSpawnLocationEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerSpawnLocationEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerSpawnLocationEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerSpawnLocationEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PlayerUnregisterChannelEvent {
    pub player: std::sync::Arc<crate::entity::player::Player>,
    pub channel: String,
    pub cancelled: bool,
}
impl PlayerUnregisterChannelEvent {
    #[must_use]
    pub const fn new(
        player: std::sync::Arc<crate::entity::player::Player>,
        channel: String,
    ) -> Self {
        Self {
            player,
            channel,
            cancelled: false,
        }
    }
}
impl super::Payload for PlayerUnregisterChannelEvent {
    fn get_name_static() -> &'static str {
        stringify!(PlayerUnregisterChannelEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PlayerUnregisterChannelEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PlayerUnregisterChannelEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct RaidFinishEvent {
    pub victory: bool,
    pub cancelled: bool,
}
impl RaidFinishEvent {
    #[must_use]
    pub const fn new(victory: bool) -> Self {
        Self {
            victory,
            cancelled: false,
        }
    }
}
impl super::Payload for RaidFinishEvent {
    fn get_name_static() -> &'static str {
        stringify!(RaidFinishEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(RaidFinishEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for RaidFinishEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct RaidSpawnWaveEvent {
    pub wave: u32,
    pub pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl RaidSpawnWaveEvent {
    #[must_use]
    pub const fn new(wave: u32, pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            wave,
            pos,
            cancelled: false,
        }
    }
}
impl super::Payload for RaidSpawnWaveEvent {
    fn get_name_static() -> &'static str {
        stringify!(RaidSpawnWaveEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(RaidSpawnWaveEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for RaidSpawnWaveEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct RaidStopEvent {
    pub reason: String,
    pub cancelled: bool,
}
impl RaidStopEvent {
    #[must_use]
    pub const fn new(reason: String) -> Self {
        Self {
            reason,
            cancelled: false,
        }
    }
}
impl super::Payload for RaidStopEvent {
    fn get_name_static() -> &'static str {
        stringify!(RaidStopEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(RaidStopEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for RaidStopEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct RaidTriggerEvent {
    pub pos: pumpkin_util::math::position::BlockPos,
    pub cancelled: bool,
}
impl RaidTriggerEvent {
    #[must_use]
    pub const fn new(pos: pumpkin_util::math::position::BlockPos) -> Self {
        Self {
            pos,
            cancelled: false,
        }
    }
}
impl super::Payload for RaidTriggerEvent {
    fn get_name_static() -> &'static str {
        stringify!(RaidTriggerEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(RaidTriggerEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for RaidTriggerEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct LightningStrikeEvent {
    pub position: pumpkin_util::math::vector3::Vector3<f64>,
    pub is_effect: bool,
    pub cancelled: bool,
}
impl LightningStrikeEvent {
    #[must_use]
    pub const fn new(position: pumpkin_util::math::vector3::Vector3<f64>, is_effect: bool) -> Self {
        Self {
            position,
            is_effect,
            cancelled: false,
        }
    }
}
impl super::Payload for LightningStrikeEvent {
    fn get_name_static() -> &'static str {
        stringify!(LightningStrikeEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(LightningStrikeEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for LightningStrikeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PluginEnableEvent {
    pub plugin_name: String,
    pub cancelled: bool,
}
impl PluginEnableEvent {
    #[must_use]
    pub const fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            cancelled: false,
        }
    }
}
impl super::Payload for PluginEnableEvent {
    fn get_name_static() -> &'static str {
        stringify!(PluginEnableEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PluginEnableEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PluginEnableEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct PluginDisableEvent {
    pub plugin_name: String,
    pub cancelled: bool,
}
impl PluginDisableEvent {
    #[must_use]
    pub const fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            cancelled: false,
        }
    }
}
impl super::Payload for PluginDisableEvent {
    fn get_name_static() -> &'static str {
        stringify!(PluginDisableEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(PluginDisableEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for PluginDisableEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ServiceRegisterEvent {
    pub service_name: String,
    pub cancelled: bool,
}
impl ServiceRegisterEvent {
    #[must_use]
    pub const fn new(service_name: String) -> Self {
        Self {
            service_name,
            cancelled: false,
        }
    }
}
impl super::Payload for ServiceRegisterEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServiceRegisterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServiceRegisterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ServiceRegisterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct ServiceUnregisterEvent {
    pub service_name: String,
    pub cancelled: bool,
}
impl ServiceUnregisterEvent {
    #[must_use]
    pub const fn new(service_name: String) -> Self {
        Self {
            service_name,
            cancelled: false,
        }
    }
}
impl super::Payload for ServiceUnregisterEvent {
    fn get_name_static() -> &'static str {
        stringify!(ServiceUnregisterEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(ServiceUnregisterEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for ServiceUnregisterEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
#[derive(Clone)]
pub struct TabCompleteEvent {
    pub buffer: String,
    pub completions: Vec<String>,
    pub cancelled: bool,
}
impl TabCompleteEvent {
    #[must_use]
    pub const fn new(buffer: String, completions: Vec<String>) -> Self {
        Self {
            buffer,
            completions,
            cancelled: false,
        }
    }
}
impl super::Payload for TabCompleteEvent {
    fn get_name_static() -> &'static str {
        stringify!(TabCompleteEvent)
    }
    fn get_name(&self) -> &'static str {
        stringify!(TabCompleteEvent)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl super::Cancellable for TabCompleteEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}
