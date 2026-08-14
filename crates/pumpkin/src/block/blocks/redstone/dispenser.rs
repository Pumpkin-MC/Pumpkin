use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::blocks::tnt::TNTBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::projectile::egg::EggEntity;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::projectile::lingering_potion::LingeringPotionEntity;
use crate::entity::projectile::small_fireball::SmallFireballEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use crate::entity::tnt::TNTEntity;
use crate::entity::r#type::from_type;
use crate::entity::vehicle::boat::BoatEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::ItemMetadata;
use crate::item::items::boat::BoatItem;
use crate::item::items::bucket::{
    FilledBucketItem, play_bucket_evaporation, should_evaporate_in_nether, try_pickup_fluid_at,
    try_place_filled_bucket,
};
use crate::item::items::honeycomb::try_wax_block;
use crate::item::items::ignite::ignition::Ignition;
use crate::item::items::spawn_egg::apply_entity_variant;
use crate::world::World;

use crate::block::entities::dispenser::DispenserBlockEntity;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::{EntityType, entity_from_egg};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockStateId, FacingExt};
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct DispenserScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for DispenserScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_3x3(sync_id, player_inventory, self.0.clone()).await;
            let screen_handler_arc = Arc::new(Mutex::new(handler));

            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_DISPENSER,
            translation::bedrock::CONTAINER_DISPENSER
        )
    }
}

#[pumpkin_block("minecraft:dispenser")]
pub struct DispenserBlock;

type DispenserLikeProperties = pumpkin_data::block_properties::DispenserLikeProperties;

struct DispenseContext<'a> {
    world: &'a Arc<World>,
    position: &'a BlockPos,
    facing: Facing,
}

impl<'a> DispenseContext<'a> {
    const fn new(args: &OnScheduledTickArgs<'a>, facing: Facing) -> Self {
        Self {
            world: args.world,
            position: args.position,
            facing,
        }
    }
}

fn triangle<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    (rng.random::<f64>() - rng.random::<f64>()).mul_add(max, min)
}

const fn to_normal(facing: Facing) -> Vector3<f64> {
    match facing {
        Facing::North => Vector3::new(0., 0., -1.),
        Facing::East => Vector3::new(1., 0., 0.),
        Facing::South => Vector3::new(0., 0., 1.),
        Facing::West => Vector3::new(-1., 0., 0.),
        Facing::Up => Vector3::new(0., 1., 0.),
        Facing::Down => Vector3::new(0., -1., 0.),
    }
}

const fn to_data3d(facing: Facing) -> i32 {
    match facing {
        Facing::North => 2,
        Facing::East => 5,
        Facing::South => 3,
        Facing::West => 4,
        Facing::Up => 1,
        Facing::Down => 0,
    }
}

impl BlockBehaviour for DispenserBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&DispenserScreenFactory(inventory), Some(*args.position))
                    .await;
            }
            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = DispenserLikeProperties::default(args.block);
            props.facing = args.player.get_entity().get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let dispenser_block_entity = DispenserBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(dispenser_block_entity));
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let powered = block_receives_redstone_power(args.world, args.position).await
                || block_receives_redstone_power(args.world, &args.position.up()).await;

            let mut props = DispenserLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );

            if powered && !props.triggered {
                args.world
                    .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal);
                props.triggered = true;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            } else if !powered && props.triggered {
                props.triggered = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position) {
                let Some(dispenser) = block_entity.as_any().downcast_ref::<DispenserBlockEntity>()
                else {
                    return;
                };

                if let Some((slot_index, mut item)) = dispenser.get_random_slot().await {
                    let props = DispenserLikeProperties::from_state_id(
                        args.world.get_block_state(args.position).id,
                        args.block,
                    );
                    let ctx = DispenseContext::new(&args, props.facing);
                    Self::dispense(&ctx, dispenser, &mut item).await;
                    dispenser.set_stack(slot_index, item).await;
                } else {
                    args.world
                        .sync_world_event(WorldEvent::SoundDispenserFail, *args.position, 0);
                }
            }
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(inventory) = block_entity.get_inventory()
            {
                Some(crate::block::calculate_comparator_output(inventory.as_ref()).await)
            } else {
                None
            }
        })
    }
}

impl DispenserBlock {
    // Velocity values match the vanilla dispenser projectile settings.
    const DEFAULT_PROJECTILE_POWER: f64 = 1.1;
    const DEFAULT_PROJECTILE_UNCERTAINTY: f64 = 6.0;
    const POTION_PROJECTILE_POWER: f64 = 1.375;
    const POTION_PROJECTILE_UNCERTAINTY: f64 = 3.0;
    // Fire charges and wind charges share these values.
    const FIREBALL_PROJECTILE_POWER: f64 = 1.0;
    const FIREBALL_PROJECTILE_UNCERTAINTY: f64 = 6.666_666_5;
    const FIREWORK_PROJECTILE_POWER: f64 = 0.5;
    const FIREWORK_PROJECTILE_UNCERTAINTY: f64 = 1.0;

    async fn dispense(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) {
        let mut event = crate::plugin::api::events::block::block_dispense::BlockDispenseEvent::new(
            *ctx.position,
            item.item.registry_key.to_string(),
        );
        if let Some(server) = ctx.world.server.upgrade() {
            server.plugin_manager.fire(&server, &mut event).await;
        }
        if event.cancelled {
            ctx.world
                .sync_world_event(WorldEvent::SoundDispenserFail, *ctx.position, 0);
            return;
        }

        // Still missing some specific dispenser behavior that you can find here:
        // https://minecraft.wiki/w/Dispenser#Usage
        let arrows = [
            Item::ARROW.id,
            Item::TIPPED_ARROW.id,
            Item::SPECTRAL_ARROW.id,
        ];
        let boats = BoatItem::ids();

        if arrows.contains(&item.item.id) {
            // Arrows
            Self::fire_arrow(ctx, item).await;
        } else if boats.contains(&item.item.id) {
            // Boats
            if !Self::dispense_boat(ctx, item).await {
                Self::drop_item(ctx, item).await;
            }
        } else if item.item.id == Item::ARMOR_STAND.id {
            // Armor stands
            if !Self::dispense_armor_stand(ctx, item).await {
                Self::drop_item(ctx, item).await;
            }
        } else if item.item.id == Item::TNT.id {
            // TNT
            Self::dispense_tnt(ctx, item).await;
        } else if item.item.id == Item::SNOWBALL.id {
            Self::dispense_snowball(ctx, item).await;
        } else if item.item.id == Item::EGG.id {
            Self::dispense_egg(ctx, item).await;
        } else if item.item.id == Item::SPLASH_POTION.id {
            Self::dispense_splash_potion(ctx, item).await;
        } else if item.item.id == Item::LINGERING_POTION.id {
            Self::dispense_lingering_potion(ctx, item).await;
        } else if item.item.id == Item::FIRE_CHARGE.id {
            Self::dispense_fire_charge(ctx, item).await;
        } else if item.item.id == Item::WIND_CHARGE.id {
            Self::dispense_wind_charge(ctx, item).await;
        } else if item.item.id == Item::FIREWORK_ROCKET.id {
            Self::dispense_firework_rocket(ctx, item).await;
        } else if item.item.id == Item::BUCKET.id {
            // Empty buckets pick up the fluid in front of the dispenser
            Self::dispense_empty_bucket(ctx, dispenser, item).await;
        } else if FilledBucketItem::ids().contains(&item.item.id) {
            // Filled buckets place their fluid in front of the dispenser
            Self::dispense_filled_bucket(ctx, item).await;
        } else if item.item.id == Item::FLINT_AND_STEEL.id {
            // Flint and steel light fires and prime TNT
            Self::dispense_flint_and_steel(ctx, item).await;
        } else if item.item.id == Item::HONEYCOMB.id {
            // Honeycombs wax copper blocks
            Self::dispense_honeycomb(ctx, item).await;
        } else if entity_from_egg(item.item.id).is_some() {
            // Spawn eggs
            Self::dispense_spawn_egg(ctx, item).await;
        } else {
            // Default / Drop
            Self::drop_item(ctx, item).await;
        }
    }

    fn projectile_spawn_position(ctx: &DispenseContext<'_>) -> Vector3<f64> {
        ctx.position
            .to_centered_f64()
            .add(&(to_normal(ctx.facing) * 0.7))
    }

    fn launch_thrown(
        ctx: &DispenseContext<'_>,
        thrown: &ThrownItemEntity,
        power: f64,
        uncertainty: f64,
    ) {
        let facing = to_normal(ctx.facing);
        thrown.set_velocity(facing.x, facing.y + 0.1, facing.z, power, uncertainty);
    }

    async fn finish_projectile_launch(
        ctx: &DispenseContext<'_>,
        projectile: Arc<dyn EntityBase>,
        launch_event: WorldEvent,
    ) {
        ctx.world.spawn_entity(projectile).await;
        Self::play_dispense_effects(ctx, launch_event);
    }

    fn play_dispense_effects(ctx: &DispenseContext<'_>, sound_event: WorldEvent) {
        ctx.world.sync_world_event(sound_event, *ctx.position, 0);
        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
    }

    async fn fire_arrow(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);

        let facing = to_normal(ctx.facing);
        let arrow_entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            ArrowEntity::entity_type_for_item(projectile.item),
        );
        let arrow =
            ArrowEntity::new_with_item(arrow_entity, None, &projectile, ArrowPickup::Allowed);

        arrow.set_velocity(
            facing.x,
            facing.y + 0.1,
            facing.z,
            Self::DEFAULT_PROJECTILE_POWER,
            Self::DEFAULT_PROJECTILE_UNCERTAINTY,
        );

        Self::finish_projectile_launch(
            ctx,
            Arc::new(arrow),
            WorldEvent::SoundDispenserProjectileLaunch,
        )
        .await;
    }

    fn target_position(ctx: &DispenseContext<'_>) -> BlockPos {
        let facing = to_normal(ctx.facing);
        ctx.position.offset(Vector3::new(
            facing.x as i32,
            facing.y as i32,
            facing.z as i32,
        ))
    }

    fn has_room_for(
        ctx: &DispenseContext<'_>,
        spawn_pos: Vector3<f64>,
        size: &EntityDimensions,
    ) -> bool {
        let bounding_box = BoundingBox::new_from_pos(spawn_pos.x, spawn_pos.y, spawn_pos.z, size);
        ctx.world.is_space_empty(bounding_box)
            && ctx.world.get_entities_at_box(&bounding_box).is_empty()
    }

    async fn dispense_boat(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let target = Self::target_position(ctx);
        let is_water = |id: u16| id == Fluid::WATER.id || id == Fluid::FLOWING_WATER.id;

        let spawn_pos = if is_water(ctx.world.get_fluid(&target).id) {
            target.to_f64()
        } else if ctx.world.get_block_state(&target).is_air()
            && is_water(ctx.world.get_fluid(&target.down()).id)
        {
            target.down().to_f64()
        } else {
            return false;
        };

        let entity_type = BoatItem::item_to_entity(item.item);
        let dimensions = EntityDimensions::new(
            entity_type.dimension[0],
            entity_type.dimension[1],
            entity_type.eye_height,
        );
        if !Self::has_room_for(ctx, spawn_pos, &dimensions) {
            return false;
        }

        let _ = item.split(1);
        let facing = to_normal(ctx.facing);
        let entity = Entity::new(ctx.world.clone(), spawn_pos, entity_type);
        entity.set_rotation(facing.x.atan2(facing.z) as f32 * 57.295_776, 0.0);
        ctx.world
            .spawn_entity(Arc::new(BoatEntity::new(entity)))
            .await;

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        true
    }

    async fn dispense_armor_stand(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let target = Self::target_position(ctx);
        let spawn_pos = target.to_f64();
        let dimensions = EntityDimensions::new(
            EntityType::ARMOR_STAND.dimension[0],
            EntityType::ARMOR_STAND.dimension[1],
            EntityType::ARMOR_STAND.eye_height,
        );
        if !Self::has_room_for(ctx, spawn_pos, &dimensions) {
            return false;
        }

        let _ = item.split(1);
        let facing = to_normal(ctx.facing);
        let entity = Entity::new(ctx.world.clone(), spawn_pos, &EntityType::ARMOR_STAND);
        entity.set_rotation(facing.x.atan2(facing.z) as f32 * 57.295_776, 0.0);

        ctx.world.play_sound(
            Sound::EntityArmorStandPlace,
            SoundCategory::Blocks,
            &spawn_pos,
        );
        ctx.world
            .spawn_entity(Arc::new(ArmorStandEntity::new(entity)))
            .await;

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        true
    }

    async fn dispense_tnt(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        const TNT_POWER: f32 = 4.0;
        const TNT_FUSE: u32 = 80;

        let _ = item.split(1);
        let spawn_pos = Self::target_position(ctx).to_f64();

        let entity = Entity::new(ctx.world.clone(), spawn_pos, &EntityType::TNT);
        let tnt = Arc::new(TNTEntity::new(entity, TNT_POWER, TNT_FUSE));
        ctx.world.spawn_entity(tnt).await;
        ctx.world
            .play_sound(Sound::EntityTntPrimed, SoundCategory::Blocks, &spawn_pos);

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
    }

    async fn dispense_spawn_egg(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let Some(entity_type) = entity_from_egg(item.item.id) else {
            return;
        };

        let _ = item.split(1);
        let spawn_pos = Self::target_position(ctx).to_f64();

        let mob = from_type(entity_type, spawn_pos, ctx.world, Uuid::new_v4());
        let yaw = wrap_degrees(rng().random::<f32>() * 360.0) % 360.0;
        mob.get_entity().set_rotation(yaw, 0.0);
        apply_entity_variant(item, mob.as_ref());

        ctx.world.spawn_entity(mob).await;

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
    }

    async fn dispense_snowball(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::SNOWBALL,
        );
        let snowball = SnowballEntity::new(entity);
        Self::launch_thrown(
            ctx,
            &snowball.thrown,
            Self::DEFAULT_PROJECTILE_POWER,
            Self::DEFAULT_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(snowball),
            WorldEvent::SoundDispenserProjectileLaunch,
        )
        .await;
    }

    async fn dispense_egg(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::EGG,
        );
        let egg = EggEntity::new(entity);
        egg.set_item_stack(projectile).await;
        Self::launch_thrown(
            ctx,
            &egg.thrown,
            Self::DEFAULT_PROJECTILE_POWER,
            Self::DEFAULT_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(egg),
            WorldEvent::SoundDispenserProjectileLaunch,
        )
        .await;
    }

    async fn dispense_splash_potion(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::SPLASH_POTION,
        );
        let potion = SplashPotionEntity::new(entity);
        potion.set_item_stack(projectile).await;
        Self::launch_thrown(
            ctx,
            &potion.thrown,
            Self::POTION_PROJECTILE_POWER,
            Self::POTION_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(potion),
            WorldEvent::SoundDispenserProjectileLaunch,
        )
        .await;
    }

    async fn dispense_lingering_potion(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::LINGERING_POTION,
        );
        let potion = LingeringPotionEntity::new(entity);
        potion.set_item_stack(projectile).await;
        Self::launch_thrown(
            ctx,
            &potion.thrown,
            Self::POTION_PROJECTILE_POWER,
            Self::POTION_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(potion),
            WorldEvent::SoundDispenserProjectileLaunch,
        )
        .await;
    }

    async fn dispense_fire_charge(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::SMALL_FIREBALL,
        );
        let fireball = SmallFireballEntity::new(entity);
        // Vanilla aims fire charges straight along the facing axis, without the +0.1 Y bias
        // other projectiles get.
        let facing = to_normal(ctx.facing);
        fireball.thrown.set_velocity(
            facing.x,
            facing.y,
            facing.z,
            Self::FIREBALL_PROJECTILE_POWER,
            Self::FIREBALL_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(ctx, Arc::new(fireball), WorldEvent::SoundBlazeFireball)
            .await;
    }

    async fn dispense_wind_charge(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::WIND_CHARGE,
        );
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: WIND_CHARGE_GRAVITY,
        };
        Self::launch_thrown(
            ctx,
            &thrown,
            Self::FIREBALL_PROJECTILE_POWER,
            Self::FIREBALL_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(WindChargeEntity::new_normal(thrown)),
            WorldEvent::SoundWindChargeShoot,
        )
        .await;
    }

    async fn dispense_firework_rocket(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let facing = to_normal(ctx.facing);
        // Vanilla spawns fireworks closer to the dispenser face and slightly above center.
        let position = ctx
            .position
            .to_centered_f64()
            .add(&(facing * (0.7 * 0.5125)))
            .add(&Vector3::new(0.0, 0.08, 0.0));
        let entity = Entity::new(ctx.world.clone(), position, &EntityType::FIREWORK_ROCKET);
        let rocket = FireworkRocketEntity::new(entity);

        // `FireworkRocketEntity` does not expose its inner projectile, so replicate
        // `ThrownItemEntity::set_velocity` here.
        let deviation = 0.017_227_5 * Self::FIREWORK_PROJECTILE_UNCERTAINTY;
        let velocity = Vector3::new(facing.x, facing.y + 0.1, facing.z)
            .normalize()
            .add_raw(
                triangle(&mut rng(), 0.0, deviation),
                triangle(&mut rng(), 0.0, deviation),
                triangle(&mut rng(), 0.0, deviation),
            )
            .multiply(
                Self::FIREWORK_PROJECTILE_POWER,
                Self::FIREWORK_PROJECTILE_POWER,
                Self::FIREWORK_PROJECTILE_POWER,
            );
        let rocket_entity = rocket.get_entity();
        rocket_entity.set_velocity(velocity);
        rocket_entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(velocity.horizontal_length()) as f32 * 57.295_776,
        );

        Self::finish_projectile_launch(ctx, Arc::new(rocket), WorldEvent::SoundFireworkShoot).await;
    }

    async fn dispense_empty_bucket(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) {
        let front = Self::target_position(ctx);
        let Some(filled) = try_pickup_fluid_at(ctx.world, front).await else {
            Self::drop_item(ctx, item).await;
            return;
        };

        item.decrement(1);
        let filled_stack = ItemStack::new(1, filled);
        if item.is_empty() {
            *item = filled_stack;
        } else if let Some(rest) = Self::add_to_first_free_slot(dispenser, filled_stack).await {
            Self::eject_item(ctx, rest).await;
        }

        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
    }

    /// Places `stack` into the first empty slot, returning it back if every slot is occupied.
    /// The slot currently being dispensed from still holds its pre-dispense stack, so it is
    /// never considered free.
    async fn add_to_first_free_slot(
        dispenser: &DispenserBlockEntity,
        stack: ItemStack,
    ) -> Option<ItemStack> {
        let mut items = dispenser.items.write().await;
        for slot in items.iter_mut() {
            if slot.is_empty() {
                *slot = stack;
                dispenser.mark_dirty();
                return None;
            }
        }
        Some(stack)
    }

    async fn dispense_filled_bucket(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let front = Self::target_position(ctx);

        // TODO: Spawn the stored entity for axolotl/fish/tadpole buckets, like the player path.
        let emptied = if should_evaporate_in_nether(item.item, ctx.world) {
            play_bucket_evaporation(ctx.world, &front.to_f64());
            true
        } else {
            try_place_filled_bucket(
                ctx.world,
                item.item,
                *ctx.position,
                ctx.facing.to_block_direction(),
            )
            .await
        };

        if emptied {
            *item = ItemStack::new(1, &Item::BUCKET);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::drop_item(ctx, item).await;
        }
    }

    async fn dispense_flint_and_steel(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let front = Self::target_position(ctx);
        let front_block = ctx.world.get_block(&front);

        let ignited = if front_block == &Block::TNT {
            TNTBlock::prime(ctx.world, &front).await;
            true
        } else {
            Ignition::ignite_block(
                |world: Arc<World>, pos: BlockPos, new_state_id: BlockStateId| async move {
                    world
                        .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                        .await;
                },
                ctx.world,
                front,
                front,
                front_block,
            )
            .await
        };

        if ignited {
            // `damage_item` already consumes the tool from the stack when it breaks.
            let _ = item.damage_item(1);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    async fn dispense_honeycomb(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let front = Self::target_position(ctx);
        let front_block = ctx.world.get_block(&front);

        if try_wax_block(ctx.world, front, front_block).await {
            item.decrement(1);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    async fn drop_item(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let drop_item = item.split(1);
        Self::eject_item(ctx, drop_item).await;
        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
    }

    async fn eject_item(ctx: &DispenseContext<'_>, stack: ItemStack) {
        let facing = to_normal(ctx.facing);
        let mut position = ctx.position.to_centered_f64().add(&(facing * 0.7));

        position.y -= match ctx.facing {
            Facing::Up | Facing::Down => 0.125,
            _ => 0.15625,
        };

        let entity = Entity::new(ctx.world.clone(), position, &EntityType::ITEM);
        let rd = rng().random::<f64>().mul_add(0.1, 0.2);

        let velocity = Vector3::new(
            triangle(&mut rng(), facing.x * rd, 0.017_227_5 * 6.),
            triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
            triangle(&mut rng(), facing.z * rd, 0.017_227_5 * 6.),
        );

        let item_entity = Arc::new(ItemEntity::new_with_velocity(entity, stack, velocity, 40));
        ctx.world.spawn_entity(item_entity).await;
    }
}
