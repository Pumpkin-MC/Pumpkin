use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::item::ItemEntity;
use crate::entity::passive::sheep::SheepEntity;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::tnt::TNTEntity;
use crate::entity::r#type::from_type;
use crate::entity::vehicle::boat::BoatEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::ItemMetadata;
use crate::item::items::boat::BoatItem;
use crate::item::items::spawn_egg::apply_entity_variant;
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};

use crate::block::entities::dispenser::DispenserBlockEntity;
use crate::block::entities::hopper::HopperBlockEntity;
use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::{EntityType, entity_from_egg};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
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
        TextComponent::translate_cross(
            translation::java::CONTAINER_DISPENSER,
            translation::bedrock::CONTAINER_DISPENSER,
            &[],
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

                if let Some(mut item) = dispenser.get_random_slot().await {
                    let props = DispenserLikeProperties::from_state_id(
                        args.world.get_block_state(args.position).id,
                        args.block,
                    );
                    let ctx = DispenseContext::new(&args, props.facing);

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
                        Self::fire_arrow(&ctx, &mut item).await;
                    } else if boats.contains(&item.item.id) {
                        // Boats
                        if !Self::dispense_boat(&ctx, &mut item).await {
                            Self::drop_item(&ctx, &mut item).await;
                        }
                    } else if item.item.id == Item::ARMOR_STAND.id {
                        // Armor stands
                        if !Self::dispense_armor_stand(&ctx, &mut item).await {
                            Self::drop_item(&ctx, &mut item).await;
                        }
                    } else if item.item.id == Item::TNT.id {
                        // TNT
                        Self::dispense_tnt(&ctx, &mut item).await;
                    } else if item.item.id == Item::WIND_CHARGE.id {
                        // Wind charges
                        Self::dispense_wind_charge(&ctx, &mut item).await;
                    } else if entity_from_egg(item.item.id).is_some() {
                        // Spawn eggs
                        Self::dispense_spawn_egg(&ctx, &mut item).await;
                    } else if item.item.id == Item::WATER_BUCKET.id
                        || item.item.id == Item::LAVA_BUCKET.id
                        || item.item.id == Item::POWDER_SNOW_BUCKET.id
                    {
                        // Filled buckets: empty onto the block the dispenser faces
                        if !Self::dispense_bucket_empty(&ctx, &mut item, dispenser).await {
                            Self::drop_item(&ctx, &mut item).await;
                        }
                    } else if item.item.id == Item::BUCKET.id {
                        // Empty bucket: pick up a water/lava source in front
                        if !Self::dispense_bucket_fill(&ctx, &mut item, dispenser).await {
                            Self::drop_item(&ctx, &mut item).await;
                        }
                    } else if item.is_shears() {
                        // Shears: unlike the other behaviors above, vanilla's
                        // OptionalDispenseItemBehavior never falls back to dropping the
                        // item on failure - it just plays the fail sound and leaves the
                        // shears in the dispenser.
                        Self::dispense_shears(&ctx, &mut item).await;
                    } else {
                        // Default / Drop
                        Self::drop_item(&ctx, &mut item).await;
                    }

                    drop(item);
                    dispenser.mark_dirty();
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
    const ARROW_DISPENSE_POWER: f64 = 1.1;
    const ARROW_DISPENSE_UNCERTAINTY: f64 = 6.0;

    async fn fire_arrow(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);

        let facing = to_normal(ctx.facing);
        let position = ctx.position.to_centered_f64().add(&(facing * 0.7));
        let world = ctx.world;

        let arrow_entity = Entity::new(
            world.clone(),
            position,
            ArrowEntity::entity_type_for_item(projectile.item),
        );
        let arrow =
            ArrowEntity::new_with_item(arrow_entity, None, &projectile, ArrowPickup::Allowed);

        arrow.set_velocity(
            facing.x,
            facing.y + 0.1,
            facing.z,
            Self::ARROW_DISPENSE_POWER,
            Self::ARROW_DISPENSE_UNCERTAINTY,
        );

        let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
        world.spawn_entity(arrow_arc).await;

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserProjectileLaunch, *ctx.position, 0);

        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
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

    /// Vanilla `WindChargeItem`. `asProjectile` spreads the launch direction per-axis via
    /// `random.triangle(direction.component, 0.11485)` and sets it as the wind charge's
    /// delta movement directly (no normalization, no power scaling). `WindChargeItem.shoot`
    /// is overridden to a no-op, so `createDispenseConfig`'s `uncertainty = 6.6666665F` /
    /// `power = 1.0F` are declared but never actually applied by the generic dispenser
    /// machinery for this item - `asProjectile`'s own spread is the only randomization that
    /// happens. Do NOT route this through `ThrownItemEntity::set_velocity`'s uncertainty/power
    /// pipeline (`0.0172275`-scaled normalize-then-scale): that is vanilla's *generic*
    /// `Projectile.spawnProjectileUsingShoot` formula, a different one from this item's own,
    /// and using it here would both double the spread and incorrectly renormalize direction.
    async fn dispense_wind_charge(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let facing = to_normal(ctx.facing);
        // Vanilla: `DispenserBlock.getDispensePosition(source, 1.0, Vec3.ZERO)` - a full
        // block offset along facing, no extra vector.
        let spawn_pos = ctx.position.to_centered_f64().add(&(facing * 1.0));

        let dir = {
            let mut random = rng();
            Vector3::new(
                triangle(&mut random, facing.x, 0.11485),
                triangle(&mut random, facing.y, 0.11485),
                triangle(&mut random, facing.z, 0.11485),
            )
        };

        let entity = Entity::new(ctx.world.clone(), spawn_pos, &EntityType::WIND_CHARGE);
        entity.velocity.store(dir);
        let len = dir.horizontal_length();
        entity.set_rotation(
            dir.x.atan2(dir.z) as f32 * 57.295_776,
            dir.y.atan2(len) as f32 * 57.295_776,
        );

        let thrown = crate::entity::projectile::ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: std::sync::atomic::AtomicBool::new(false),
            gravity: crate::entity::projectile::wind_charge::WIND_CHARGE_GRAVITY,
        };
        let wind_charge =
            Arc::new(crate::entity::projectile::wind_charge::WindChargeEntity::new_normal(thrown));
        ctx.world.spawn_entity(wind_charge).await;

        // Vanilla `overrideDispenseEvent = 1051`, a level-event id distinct from the generic
        // dispense sound/smoke fired by the other branches above.
        ctx.world
            .sync_world_event(WorldEvent::SoundWindChargeShoot, *ctx.position, 0);
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

    async fn drop_item(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let drop_item = item.split(1);
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

        let item_entity = Arc::new(ItemEntity::new_with_velocity(
            entity, drop_item, velocity, 0,
        ));
        ctx.world.spawn_entity(item_entity).await;

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);

        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
    }

    /// Vanilla `DispenseItemBehavior.bootStrap`'s `filledBucketBehavior`: empties the
    /// bucket onto the faced block if it is air or otherwise replaceable, then returns
    /// the empty bucket to the slot. Waterlogging a partial block (e.g. a stair) and the
    /// mob-bucket entity-spawn cases are not handled here; those fall through to the
    /// default drop, same as an unrecognized item.
    async fn dispense_bucket_empty(
        ctx: &DispenseContext<'_>,
        item: &mut ItemStack,
        dispenser: &DispenserBlockEntity,
    ) -> bool {
        let fluid_block = if item.item.id == Item::LAVA_BUCKET.id {
            &Block::LAVA
        } else if item.item.id == Item::WATER_BUCKET.id {
            &Block::WATER
        } else if item.item.id == Item::POWDER_SNOW_BUCKET.id {
            &Block::POWDER_SNOW
        } else {
            return false;
        };

        let target = Self::target_position(ctx);
        let target_state = ctx.world.get_block_state(&target);
        if !(target_state.is_air() || target_state.replaceable()) {
            return false;
        }

        ctx.world
            .set_block_state(
                &target,
                fluid_block.default_state.id,
                BlockFlags::NOTIFY_NEIGHBORS,
            )
            .await;

        let _ = item.split(1);
        *item = ItemStack::new(1, &Item::BUCKET);
        dispenser.mark_dirty();

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
        true
    }

    /// Vanilla `DispenseItemBehavior.bootStrap`'s `Items.BUCKET` behavior: picks up a
    /// water or lava source block the dispenser faces. Non-source (flowing) fluid is
    /// left alone, matching `LiquidBlock.pickupBlock`'s `isSource` check.
    async fn dispense_bucket_fill(
        ctx: &DispenseContext<'_>,
        item: &mut ItemStack,
        dispenser: &DispenserBlockEntity,
    ) -> bool {
        let target = Self::target_position(ctx);
        let state = ctx.world.get_block_state(&target);
        let filled = if state.id == Block::WATER.default_state.id {
            &Item::WATER_BUCKET
        } else if state.id == Block::LAVA.default_state.id {
            &Item::LAVA_BUCKET
        } else {
            return false;
        };

        ctx.world
            .set_block_state(
                &target,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_NEIGHBORS,
            )
            .await;

        let _ = item.split(1);
        dispenser.mark_dirty();

        let all_slots: Vec<usize> = (0..DispenserBlockEntity::INVENTORY_SIZE).collect();
        let mut remainder = ItemStack::new(1, filled);
        if !HopperBlockEntity::add_one_item(dispenser, dispenser, remainder.clone(), &all_slots)
            .await
        {
            Self::drop_item(ctx, &mut remainder).await;
        }

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
        true
    }

    /// Vanilla `ShearsDispenseItemBehavior`: only the sheep branch of `tryShearEntity` is
    /// ported (beehive honeycomb harvesting and leash-cutting are separate block-entity /
    /// entity-graph features out of scope here). `OptionalDispenseItemBehavior.dispense`
    /// always plays the animation smoke and plays sound 1000 on success / 1001 on
    /// failure - never falls back to a plain item drop.
    async fn dispense_shears(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let target = Self::target_position(ctx);
        let min = target.to_f64();
        let aabb = BoundingBox::new(min, min.add_raw(1.0, 1.0, 1.0));

        let mut success = false;
        for entity in ctx.world.get_entities_at_box(&aabb) {
            if let Some(sheep) = entity.cast_any().downcast_ref::<SheepEntity>()
                && sheep.ready_for_shearing()
            {
                sheep.shear(ctx.world).await;
                emit_game_event(
                    ctx.world,
                    GameEvent::Shear,
                    target.to_centered_f64(),
                    GameEventContext::none(),
                )
                .await;
                let _ = item.damage_item(1);
                success = true;
                break;
            }
        }

        ctx.world.sync_world_event(
            if success {
                WorldEvent::SoundDispenserDispense
            } else {
                WorldEvent::SoundDispenserFail
            },
            *ctx.position,
            0,
        );
        ctx.world.sync_world_event(
            WorldEvent::ParticlesShootSmoke,
            *ctx.position,
            to_data3d(ctx.facing),
        );
    }
}
