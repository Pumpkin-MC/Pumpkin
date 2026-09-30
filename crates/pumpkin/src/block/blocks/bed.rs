use std::sync::Arc;

use crate::block::entities::bed::BedBlockEntity;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::OnLandedUponArgs;
use crate::block::UpdateEntityMovementAfterFallOnArgs;
use crate::block::bounce_entity_after_fall;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, NormalUseArgs, OnPlaceArgs, OnStateReplacedArgs,
    PathComputationType, PlacedArgs,
};
use crate::entity::{Entity, EntityBase, passive::villager::VillagerEntity, player::Player};
use crate::world::World;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

const NO_SLEEP_IDS: &[u16] = &[
    EntityType::BLAZE.id,
    EntityType::BOGGED.id,
    EntityType::SKELETON.id,
    EntityType::STRAY.id,
    EntityType::WITHER_SKELETON.id,
    EntityType::BREEZE.id,
    EntityType::CREAKING.id,
    EntityType::CREEPER.id,
    EntityType::DROWNED.id,
    EntityType::ENDERMITE.id,
    EntityType::EVOKER.id,
    EntityType::GIANT.id,
    EntityType::GUARDIAN.id,
    EntityType::ELDER_GUARDIAN.id,
    EntityType::ILLUSIONER.id,
    EntityType::OCELOT.id,
    EntityType::PIGLIN.id,
    EntityType::PIGLIN_BRUTE.id,
    EntityType::PILLAGER.id,
    EntityType::PHANTOM.id,
    EntityType::RAVAGER.id,
    EntityType::SILVERFISH.id,
    EntityType::SPIDER.id,
    EntityType::CAVE_SPIDER.id,
    EntityType::VEX.id,
    EntityType::VINDICATOR.id,
    EntityType::WARDEN.id,
    EntityType::WITCH.id,
    EntityType::WITHER.id,
    EntityType::ZOGLIN.id,
    EntityType::ZOMBIE.id,
    EntityType::ZOMBIE_VILLAGER.id,
    EntityType::HUSK.id,
    EntityType::ENDERMAN.id,
    EntityType::ZOMBIFIED_PIGLIN.id,
];

#[pumpkin_block_from_tag("minecraft:beds")]
pub struct BedBlock;

impl BlockBehaviour for BedBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player {
            let facing = player.get_entity().get_horizontal_facing();
            return args
                .block_accessor
                .get_block_state(args.position)
                .replaceable()
                && args
                    .block_accessor
                    .get_block_state(&args.position.offset(facing.to_offset()))
                    .replaceable();
        }
        false
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance * 0.5, 1.0);
        }
    }

    fn update_entity_movement_after_fall_on(&self, args: UpdateEntityMovementAfterFallOnArgs<'_>) {
        bounce_entity_after_fall(args.entity, 0.66);
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bed_props = BedProperties::default(args.block);

        bed_props.facing = args.player.get_entity().get_horizontal_facing();
        bed_props.part = BedPart::Foot;

        bed_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let bed_entity = BedBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(bed_entity));

            let mut bed_head_props = BedProperties::default(args.block);
            bed_head_props.facing = BedProperties::from_state_id(args.state_id).facing;
            bed_head_props.part = BedPart::Head;

            let bed_head_pos = args.position.offset(bed_head_props.facing.to_offset());
            args.world.set_block_state(
                &bed_head_pos,
                bed_head_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            );

            let bed_head_entity = BedBlockEntity::new(bed_head_pos);
            args.world.add_block_entity(Arc::new(bed_head_entity));
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let bed_props = BedProperties::from_state_id(args.state.id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };
        let neighbor_state_id = args.world.get_block_state_id(&other_half_pos);
        if neighbor_state_id.to_block_id() != args.block.id {
            args.world.update_neighbors(&other_half_pos, None);
            return;
        }

        let is_creative = args.player.gamemode.load() == GameMode::Creative;
        let flags = if bed_props.part == BedPart::Foot && !is_creative {
            // Breaking foot in survival -> allow head to drop
            BlockFlags::NOTIFY_ALL
        } else {
            // Breaking head OR creative mode -> skip drops
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
        };

        args.world
            .break_block(&other_half_pos, Some(args.player), flags);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved {
            return;
        }

        let bed_props = BedProperties::from_state_id(args.old_state_id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };

        let (other_block, other_state) = args.world.get_block_and_state(&other_half_pos);
        if other_block == args.block {
            let other_props = BedProperties::from_state_id(other_state.id);
            if other_props.part != bed_props.part {
                args.world.break_block(
                    &other_half_pos,
                    None,
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::use_bed(args.world, args.player, args.block, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl BedBlock {
    #[expect(clippy::too_many_lines)]
    fn use_bed(
        world: &Arc<World>,
        player: &Arc<Player>,
        block: &Block,
        position: &BlockPos,
    ) -> BlockActionResult {
        let state_id = world.get_block_state_id(position);
        let bed_props = BedProperties::from_state_id(state_id);

        let (bed_head_pos, bed_foot_pos) = if bed_props.part == BedPart::Head {
            (
                *position,
                position.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (position.offset(bed_props.facing.to_offset()), *position)
        };

        // Explode if bed rule explodes (EnvironmentAttributes.BED_RULE)
        if world.dimension.bed_rule.explodes {
            world.break_block(&bed_head_pos, None, BlockFlags::SKIP_DROPS);
            world.break_block(&bed_foot_pos, None, BlockFlags::SKIP_DROPS);

            world.explode(
                bed_head_pos.to_centered_f64(),
                5.0,
                crate::world::ExplosionInteraction::Block,
            );

            return BlockActionResult::SuccessServer;
        }

        // Vanilla handles an occupied bed before distance, spawn-point, safety, and
        // enter-bed checks. A sleeping villager is woken and the click ends.
        if bed_props.occupied {
            let villager_woken = world.entities.load().iter().any(|entity| {
                entity
                    .cast_any()
                    .downcast_ref::<VillagerEntity>()
                    .is_some_and(|villager| villager.wake_up_if_sleeping_at(bed_head_pos))
            });

            if villager_woken {
                Self::set_occupied(
                    false,
                    world,
                    block,
                    &bed_head_pos,
                    world.get_block_state_id(&bed_head_pos),
                );
            } else {
                player.send_system_message_raw(
                    &pumpkin_macros::translate_cross!(
                        translation::java::BLOCK_MINECRAFT_BED_OCCUPIED,
                        translation::bedrock::TILE_BED_OCCUPIED
                    ),
                    true,
                );
            }
            return BlockActionResult::SuccessServer;
        }

        let is_dark = world.is_dark_outside();
        let can_sleep = world.dimension.bed_rule.can_sleep(is_dark);
        let can_set_spawn = world.dimension.bed_rule.can_set_spawn(is_dark);

        if !can_set_spawn && !can_sleep {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure the bed is not obstructed.
        if world.get_block_state(&bed_head_pos.up()).is_solid()
            || world.get_block_state(&bed_foot_pos.up()).is_solid()
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_OBSTRUCTED,
                    translation::bedrock::TILE_BED_OBSTRUCTED
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure player is close enough
        if !player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 3.0, 3.0)
            && !player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 3.0, 3.0)
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_TOO_FAR_AWAY,
                    translation::bedrock::TILE_BED_TOOFAR
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Set respawn point
        if can_set_spawn
            && player.set_respawn_point(
                world.dimension.clone(),
                bed_head_pos,
                player.get_entity().yaw.load(),
                player.get_entity().pitch.load(),
                false,
            )
        {
            player.send_system_message(&pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_SET_SPAWN,
                translation::bedrock::TILE_BED_RESPAWNSET
            ));
        }

        // Make sure the time and weather allows sleep
        if !can_sleep {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Make sure there are no monsters nearby
        for entity in world.entities.load().iter() {
            if !entity_prevents_sleep(entity.get_entity()) {
                continue;
            }

            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0)
                || pos.is_within_bounds(bed_foot_pos.to_f64(), 8.0, 5.0, 8.0)
            {
                player.send_system_message_raw(
                    &pumpkin_macros::translate_cross!(
                        translation::java::BLOCK_MINECRAFT_BED_NOT_SAFE,
                        translation::bedrock::TILE_BED_NOTSAFE
                    ),
                    true,
                );
                return BlockActionResult::SuccessServer;
            }
        }

        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::player::player_bed::PlayerBedEnterEvent::new(
                    player.clone(),
                    bed_head_pos,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return BlockActionResult::SuccessServer;
            }
        }

        player.sleep(bed_head_pos);
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::SleptInBed,
        );
        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::SleepInBed as i32,
            1,
        );
        Self::set_occupied(true, world, block, position, state_id);

        BlockActionResult::SuccessServer
    }
}

impl BedBlock {
    pub fn set_occupied(
        occupied: bool,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        state_id: BlockStateId,
    ) {
        let mut bed_props = BedProperties::from_state_id(state_id);
        bed_props.occupied = occupied;
        world.set_block_state(
            block_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        );

        let other_half_pos = if bed_props.part == BedPart::Head {
            block_pos.offset(bed_props.facing.opposite().to_offset())
        } else {
            block_pos.offset(bed_props.facing.to_offset())
        };
        bed_props.part = if bed_props.part == BedPart::Head {
            BedPart::Foot
        } else {
            BedPart::Head
        };
        world.set_block_state(
            &other_half_pos,
            bed_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

fn entity_prevents_sleep(entity: &Entity) -> bool {
    NO_SLEEP_IDS.contains(&entity.entity_type.id)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::{
        path::Path,
        sync::{Arc, RwLock},
    };

    use arc_swap::ArcSwap;
    use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
    use pumpkin_data::block_properties::BedPart;
    use pumpkin_data::entity::{EntityPose, EntityType};
    use pumpkin_data::{Block, BlockDirection, dimension::Dimension};
    use pumpkin_util::GameMode;
    use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
    use pumpkin_world::world::BlockFlags;
    use tokio::net::{TcpListener, TcpStream};
    use uuid::Uuid;

    use crate::block::{BlockBehaviour, BlockHitResult, NormalUseArgs};
    use crate::data::{
        VanillaData, banned_ip::BannedIpList, banned_player::BannedPlayerList, op::OperatorConfig,
        usercache::UserCache, whitelist::WhitelistConfig,
    };
    use crate::entity::{Entity, EntityBase, passive::villager::VillagerEntity, player::Player};
    use crate::net::java::JavaClient;
    use crate::net::java::pending::PendingConnection;
    use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
    use crate::server::Server;

    use super::{BedBlock, BedProperties};

    fn empty_vanilla_data() -> VanillaData {
        VanillaData {
            banned_ip_list: RwLock::new(BannedIpList::default()),
            banned_player_list: RwLock::new(BannedPlayerList::default()),
            operator_config: RwLock::new(OperatorConfig::default()),
            user_cache: RwLock::new(UserCache::default()),
            whitelist_config: RwLock::new(WhitelistConfig::default()),
        }
    }

    async fn test_server(world_path: &Path) -> std::sync::Arc<Server> {
        let basic_config = BasicConfiguration {
            default_level_name: world_path.to_string_lossy().into_owned(),
            allow_nether: false,
            allow_end: false,
            ..BasicConfiguration::default()
        };

        let mut advanced_config = AdvancedConfiguration::default();
        advanced_config.networking.bedrock.online_mode = false;

        let telemetry_config = TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        };

        Server::new(
            basic_config,
            advanced_config,
            telemetry_config,
            empty_vanilla_data(),
        )
        .await
    }

    async fn test_player(
        world: &std::sync::Arc<crate::world::World>,
    ) -> (std::sync::Arc<Player>, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (client_socket, accepted_socket) =
            tokio::join!(TcpStream::connect(address), listener.accept(),);
        let client_socket = client_socket.unwrap();
        let server_socket = accepted_socket.unwrap().0;
        let pending = PendingConnection::new(
            server_socket,
            address,
            1,
            PacketRateLimiter::new(false, 0.0, 0.0),
        );
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: "bed-click-test".to_string(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let client = Arc::new(ClientPlatform::Java(JavaClient::from_pending(
            pending,
            profile.clone(),
            PlayerConfig::default(),
        )));
        let player = Arc::new(Player::new(
            client,
            profile,
            PlayerConfig::default(),
            world,
            GameMode::Survival,
        ));

        (player, client_socket)
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(clippy::too_many_lines)]
    async fn clicking_an_obstructed_occupied_bed_wakes_the_villager_and_returns() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let server = test_server(&temporary_directory.path().join("world")).await;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        world
            .level
            .get_or_fetch_chunk(Vector2::new(0, 0), |_| ())
            .await;

        let (player, _client_socket) = test_player(&world).await;
        player
            .get_entity()
            .set_pos(Vector3::new(100.0, 100.0, 100.0));

        let foot_pos = BlockPos::new(8, 64, 8);
        let mut foot_properties = BedProperties::default(&Block::RED_BED);
        foot_properties.part = BedPart::Foot;
        let facing = foot_properties.facing;
        let mut head_properties = BedProperties::default(&Block::RED_BED);
        head_properties.facing = facing;
        head_properties.part = BedPart::Head;
        let head_pos = foot_pos.offset(facing.to_offset());
        let flags = BlockFlags::SKIP_DROPS | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK;

        world.set_block_state(
            &foot_pos,
            foot_properties.to_state_id(&Block::RED_BED),
            flags,
        );
        world.set_block_state(
            &head_pos,
            head_properties.to_state_id(&Block::RED_BED),
            flags,
        );
        BedBlock::set_occupied(
            true,
            &world,
            &Block::RED_BED,
            &foot_pos,
            world.get_block_state_id(&foot_pos),
        );
        world.set_block_state(&head_pos.up(), Block::STONE.default_state.id, flags);

        let villager = VillagerEntity::new(Entity::from_uuid(
            Uuid::new_v4(),
            world.clone(),
            Vector3::new(8.5, 64.0, 8.5),
            &EntityType::VILLAGER,
        ));
        villager.get_entity().set_pose(EntityPose::Sleeping);
        villager
            .mob_entity
            .living_entity
            .sleeping_pos
            .store(Some(head_pos));
        world.spawn_entity_non_save(villager.clone());

        let face = BlockDirection::Up;
        let cursor_pos = Vector3::new(0.5, 1.0, 0.5);
        let hit = BlockHitResult {
            face: &face,
            cursor_pos: &cursor_pos,
        };
        let head_was_occupied =
            BedProperties::from_state_id(world.get_block_state_id(&head_pos)).occupied;
        let foot_was_occupied =
            BedProperties::from_state_id(world.get_block_state_id(&foot_pos)).occupied;
        let head_was_head =
            BedProperties::from_state_id(world.get_block_state_id(&head_pos)).part == BedPart::Head;
        let foot_was_foot =
            BedProperties::from_state_id(world.get_block_state_id(&foot_pos)).part == BedPart::Foot;
        let result = BedBlock.normal_use(NormalUseArgs {
            server: &server,
            world: &world,
            block: &Block::RED_BED,
            position: &foot_pos,
            player: &player,
            hit: &hit,
        });

        let villager_pose = villager.get_entity().pose.load();
        let villager_sleeping_pos = villager.mob_entity.living_entity.sleeping_pos.load();
        let head_occupied =
            BedProperties::from_state_id(world.get_block_state_id(&head_pos)).occupied;
        let foot_occupied =
            BedProperties::from_state_id(world.get_block_state_id(&foot_pos)).occupied;
        let player_pose = player.get_entity().pose.load();
        let player_sleeping_since = player.sleeping_since.load();

        server.shutdown().await;

        assert!(matches!(
            result,
            crate::block::registry::BlockActionResult::SuccessServer
        ));
        assert!(
            head_was_occupied && foot_was_occupied,
            "test bed was not occupied before click"
        );
        assert!(
            head_was_head && foot_was_foot,
            "test bed halves had incorrect parts before click"
        );
        assert!(villager_pose == EntityPose::Standing);
        assert_eq!(villager_sleeping_pos, None);
        assert!(
            !head_occupied && !foot_occupied,
            "bed stayed occupied after click (head={head_occupied}, foot={foot_occupied})"
        );
        assert!(player_pose != EntityPose::Sleeping);
        assert_eq!(player_sleeping_since, None);
    }
}
