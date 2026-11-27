use crate::entity::projectile::owner::ProjectileOwner;
use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::block::blocks::carved_pumpkin::can_spawn_golem;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::blocks::tnt::TNTBlock;
use crate::block::blocks::wither_skull::find_wither_pattern;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, GetScreenHandlerFactoryArgs, NormalUseArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::item::ItemEntity;
use crate::entity::mob::Mob;
use crate::entity::mob::sulfur_cube::{SulfurCubeEntity, is_swallowable_item};
use crate::entity::passive::abstract_horse::try_equip_dispenser_chest;
use crate::entity::passive::armadillo::{ArmadilloEntity, BRUSH_DURABILITY_COST};
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::projectile::egg::EggEntity;
use crate::entity::projectile::experience_bottle::ExperienceBottleEntity;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::projectile::lingering_potion::LingeringPotionEntity;
use crate::entity::projectile::small_fireball::SmallFireballEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use crate::entity::tnt::TNTEntity;
use crate::entity::r#type::from_type;
use crate::entity::vehicle::boat::BoatEntity;
use crate::entity::vehicle::minecart::MinecartEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::ItemMetadata;
use crate::item::items::boat::BoatItem;
use crate::item::items::bone_meal::BoneMealItem;
use crate::item::items::bucket::{
    FilledBucketItem, dispense_filled_bucket_at, try_pickup_fluid_at,
};
use crate::item::items::honeycomb::try_wax_block;
use crate::item::items::ignite::ignition::Ignition;
use crate::item::items::minecart::MinecartItem;
use crate::item::items::spawn_egg::prepare_egg_mob;
use crate::plugin::api::events::block::block_shear_entity::BlockShearEntityEvent;
use crate::plugin::api::events::entity::creature_spawn::CreatureSpawnReason;
use crate::world::World;

use crate::block::entities::dispenser::DispenserBlockEntity;
use pumpkin_data::block_properties::{
    BeeNestLikeProperties, BlockProperties, EndRodLikeProperties, Facing,
    PoweredRailLikeProperties, RailLikeProperties, RespawnAnchorLikeProperties,
    SkeletonSkullLikeProperties,
};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{EquippableImpl, IDSet, PotionContentsImpl};
use pumpkin_data::entity::{EntityType, entity_from_egg};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockStateId, FacingExt};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct DispenserScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for DispenserScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_generic_3x3(sync_id, player_inventory, self.0.clone(), player);
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
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

const fn to_rotation16(facing: Facing) -> u8 {
    match facing {
        Facing::South | Facing::Up | Facing::Down => 0,
        Facing::West => 4,
        Facing::North => 8,
        Facing::East => 12,
    }
}

fn is_allowed_entity(
    allowed: Option<&IDSet<EntityType>>,
    entity_type: &'static EntityType,
) -> bool {
    match allowed {
        None => true,
        Some(IDSet::Tag(tag)) => entity_type.is_tagged_with(tag).unwrap_or(false),
        Some(IDSet::IDs(types)) => types.contains(&entity_type),
    }
}

fn water_bottle() -> ItemStack {
    ItemStack::new_with_component(
        1,
        &Item::POTION,
        vec![(
            DataComponent::PotionContents,
            Some(Box::new(PotionContentsImpl {
                potion_id: Some(i32::from(Potion::WATER.id)),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            }) as Box<_>),
        )],
    )
}

fn is_water_bottle(stack: &ItemStack) -> bool {
    stack.item.id == Item::POTION.id
        && stack
            .get_data_component::<PotionContentsImpl>()
            .and_then(|contents| contents.potion_id)
            == Some(i32::from(Potion::WATER.id))
}

impl BlockBehaviour for DispenserBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::InspectDispenser as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }
        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.get_inventory()?;
        Some(Box::new(DispenserScreenFactory(inventory)))
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = DispenserLikeProperties::default(args.block);
        props.facing = args.player.get_entity().get_facing().opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let dispenser_block_entity = DispenserBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(dispenser_block_entity));
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let powered = block_receives_redstone_power(args.world, args.position)
            || block_receives_redstone_power(args.world, &args.position.up());

        let mut props =
            DispenserLikeProperties::from_state_id(args.world.get_block_state(args.position).id);

        if powered && !props.triggered {
            args.world
                .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal);
            props.triggered = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        } else if !powered && props.triggered {
            props.triggered = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (_block, state) = args.world.get_block_and_state(args.position);
        if let Some(block_entity) = args.world.get_block_entity(args.position) {
            let Some(dispenser) = block_entity.as_any().downcast_ref::<DispenserBlockEntity>()
            else {
                return;
            };

            if let Some((slot_index, mut item)) = dispenser.get_random_slot() {
                let props = DispenserLikeProperties::from_state_id(state.id);
                let ctx = DispenseContext {
                    world: args.world,
                    position: args.position,
                    facing: props.facing,
                };
                Self::dispense(&ctx, dispenser, &mut item);
                dispenser.set_stack(slot_index, item);
            } else {
                args.world
                    .sync_world_event(WorldEvent::SoundDispenserFail, *args.position, 0);
                args.world.emit_game_event(
                    GameEvent::BlockActivate.name(),
                    args.position.to_centered_f64(),
                );
            }
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        crate::block::container_comparator_output(&args)
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

    fn dispense(ctx: &DispenseContext<'_>, dispenser: &DispenserBlockEntity, item: &mut ItemStack) {
        let mut event = crate::plugin::api::events::block::block_dispense::BlockDispenseEvent::new(
            *ctx.position,
            item.item.registry_key.to_string(),
        );
        if let Some(server) = ctx.world.server.upgrade() {
            event = server.plugin_manager.fire_blocking(&server, event);
        }
        if event.cancelled {
            ctx.world
                .sync_world_event(WorldEvent::SoundDispenserFail, *ctx.position, 0);
            return;
        }

        // Still missing some specific dispenser behavior that you can find here:
        // https://minecraft.wiki/w/Dispenser#Usage
        if Self::try_dispense_projectile(ctx, item) {
            return;
        }

        let boats = BoatItem::ids();
        if boats.contains(&item.item.id) {
            // Boats
            if !Self::dispense_boat(ctx, item) {
                Self::drop_item(ctx, item);
            }
        } else if MinecartItem::ids().contains(&item.item.id) {
            // Minecarts
            if !Self::dispense_minecart(ctx, item) {
                Self::drop_item(ctx, item);
            }
        } else if item.item.id == Item::ARMOR_STAND.id {
            // Armor stands
            if !Self::dispense_armor_stand(ctx, item) {
                Self::drop_item(ctx, item);
            }
        } else if item.item.id == Item::TNT.id {
            // TNT
            Self::dispense_tnt(ctx, item);
        } else if item.item.id == Item::BUCKET.id {
            // Empty buckets pick up the fluid in front of the dispenser
            Self::dispense_empty_bucket(ctx, dispenser, item);
        } else if FilledBucketItem::ids().contains(&item.item.id) {
            // Filled buckets place their fluid and any stored mob in front of the dispenser
            Self::dispense_filled_bucket(ctx, dispenser, item);
        } else if item.item.id == Item::FLINT_AND_STEEL.id {
            // Flint and steel prime explosive sulfur cubes, light fires, and prime TNT
            Self::dispense_flint_and_steel(ctx, item);
        } else if item.item.id == Item::BONE_MEAL.id {
            Self::dispense_bone_meal(ctx, item);
        } else if item.item.id == Item::BRUSH.id {
            Self::dispense_brush(ctx, item);
        } else if item.item.id == Item::CHEST.id {
            Self::dispense_chest(ctx, item);
        } else if item.item.id == Item::HONEYCOMB.id {
            // Honeycombs wax copper blocks or drop when facing non-waxable blocks
            Self::dispense_honeycomb(ctx, item);
        } else if crate::item::items::spawn_egg::entity_type_from_spawn_egg_stack(item).is_some()
            && entity_from_egg(item.item.id).is_some()
        {
            // Spawn eggs
            Self::dispense_spawn_egg(ctx, item);
        } else if item.item.id == Item::SHEARS.id {
            // Shears harvest full beehives, snip leashes, and shear shearable mobs
            Self::dispense_shears(ctx, item);
        } else if item.item.id == Item::GLASS_BOTTLE.id {
            // Glass bottles fill from water and full beehives
            if !Self::dispense_glass_bottle(ctx, dispenser, item) {
                Self::drop_item_with_sound(ctx, item, WorldEvent::SoundDispenserFail);
            }
        } else if is_water_bottle(item) {
            // Water bottles convert dirt-likes into mud
            if !Self::dispense_water_bottle(ctx, dispenser, item) {
                Self::drop_item(ctx, item);
            }
        } else if item.item.id == Item::GLOWSTONE.id {
            match Self::dispense_glowstone(ctx, item) {
                Some(true) => Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense),
                Some(false) => Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail),
                None => Self::drop_item(ctx, item),
            }
        } else if item.item.id == Item::WITHER_SKELETON_SKULL.id {
            // Placed only when it completes a wither, otherwise worn as a helmet
            Self::dispense_mob_head(ctx, item, &Block::WITHER_SKELETON_SKULL);
        } else if item.item.id == Item::CARVED_PUMPKIN.id {
            // Placed only when it completes a golem, otherwise worn as a helmet
            Self::dispense_mob_head(ctx, item, &Block::CARVED_PUMPKIN);
        } else if Block::from_item_id(item.item.id)
            .is_some_and(|block| block.has_tag(&tag::Block::MINECRAFT_SHULKER_BOXES))
        {
            // Shulker boxes place themselves
            if !Self::dispense_shulker_box(ctx, item) {
                Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
            }
        } else if Self::dispense_equipment(ctx, item) {
            // Armor, elytra, heads, saddles, horse/wolf armor and llama carpets
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else if is_swallowable_item(item) {
            if Self::dispense_sulfur_cube_block(ctx, item) {
                Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
            } else {
                Self::drop_item(ctx, item);
            }
        } else {
            // Default / Drop
            Self::drop_item(ctx, item);
        }
    }

    fn try_dispense_projectile(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let id = item.item.id;
        if id == Item::ARROW.id || id == Item::TIPPED_ARROW.id || id == Item::SPECTRAL_ARROW.id {
            Self::fire_arrow(ctx, item);
        } else if id == Item::SNOWBALL.id {
            Self::dispense_snowball(ctx, item);
        } else if id == Item::EGG.id || id == Item::BLUE_EGG.id || id == Item::BROWN_EGG.id {
            Self::dispense_egg(ctx, item);
        } else if id == Item::SPLASH_POTION.id {
            Self::dispense_splash_potion(ctx, item);
        } else if id == Item::LINGERING_POTION.id {
            Self::dispense_lingering_potion(ctx, item);
        } else if id == Item::FIRE_CHARGE.id {
            Self::dispense_fire_charge(ctx, item);
        } else if id == Item::WIND_CHARGE.id {
            Self::dispense_wind_charge(ctx, item);
        } else if id == Item::FIREWORK_ROCKET.id {
            Self::dispense_firework_rocket(ctx, item);
        } else if id == Item::EXPERIENCE_BOTTLE.id {
            Self::dispense_experience_bottle(ctx, item);
        } else {
            return false;
        }
        true
    }

    // Vanilla default `DispenseConfig` position: 0.7 blocks out, 0.1 up.
    fn projectile_spawn_position(ctx: &DispenseContext<'_>) -> Vector3<f64> {
        ctx.position
            .to_centered_f64()
            .add(&(to_normal(ctx.facing) * 0.7))
            .add(&Vector3::new(0.0, 0.1, 0.0))
    }

    // Fire charge and wind charge `DispenseConfig` position: 1 block out.
    fn charge_spawn_position(ctx: &DispenseContext<'_>) -> Vector3<f64> {
        ctx.position.to_centered_f64().add(&to_normal(ctx.facing))
    }

    fn launch_thrown(
        ctx: &DispenseContext<'_>,
        thrown: &ThrownItemEntity,
        power: f64,
        uncertainty: f64,
    ) {
        let facing = to_normal(ctx.facing);
        thrown.set_velocity(facing.x, facing.y, facing.z, power, uncertainty);
    }

    fn finish_projectile_launch(
        ctx: &DispenseContext<'_>,
        projectile: Arc<dyn EntityBase>,
        launch_event: WorldEvent,
    ) {
        ctx.world.spawn_entity(projectile);
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

    fn fire_arrow(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);

        let facing = to_normal(ctx.facing);
        let arrow_entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            ArrowEntity::entity_type_for_item(projectile.item),
        );
        let arrow =
            ArrowEntity::new_with_item(arrow_entity, None, &projectile, ArrowPickup::Allowed);
        arrow.apply_on_projectile_spawned(&projectile);

        arrow.set_velocity(
            facing.x,
            facing.y,
            facing.z,
            Self::DEFAULT_PROJECTILE_POWER,
            Self::DEFAULT_PROJECTILE_UNCERTAINTY,
        );

        Self::finish_projectile_launch(
            ctx,
            Arc::new(arrow),
            WorldEvent::SoundDispenserProjectileLaunch,
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

    fn dispense_boat(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let target = Self::target_position(ctx);
        let is_water = |id: u16| Fluid::same_fluid_type(id, Fluid::WATER.id);

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

        let dispensed = item.split(1);
        let facing = to_normal(ctx.facing);
        let entity = Entity::new(ctx.world.clone(), spawn_pos, entity_type);
        entity.set_rotation(facing.x.atan2(facing.z) as f32 * 57.295_776, 0.0);
        let boat = Arc::new(BoatEntity::new(entity));
        crate::item::items::spawn_egg::apply_entity_variant(&dispensed, boat.as_ref(), None);
        ctx.world.spawn_entity(boat);

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        true
    }

    fn dispense_armor_stand(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
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

        let dispensed = item.split(1);
        let facing = to_normal(ctx.facing);
        let entity = Entity::new(ctx.world.clone(), spawn_pos, &EntityType::ARMOR_STAND);
        entity.set_rotation(facing.x.atan2(facing.z) as f32 * 57.295_776, 0.0);

        ctx.world.play_sound(
            Sound::EntityArmorStandPlace,
            SoundCategory::Blocks,
            &spawn_pos,
        );
        let armor_stand = Arc::new(ArmorStandEntity::new(entity));
        crate::item::items::spawn_egg::apply_entity_variant(&dispensed, armor_stand.as_ref(), None);
        ctx.world.spawn_entity(armor_stand);

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        true
    }

    fn dispense_sulfur_cube_block(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let target_box = BoundingBox::from_block(&Self::target_position(ctx));
        for entity in ctx.world.get_entities_at_box(&target_box) {
            let Some(sulfur_cube) = entity.cast_any().downcast_ref::<SulfurCubeEntity>() else {
                continue;
            };
            if sulfur_cube.equip_item(item) {
                item.decrement(1);
                return true;
            }
        }
        false
    }

    fn dispense_tnt(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        // Vanilla keeps the item and plays the fail click when TNT is disabled.
        if !ctx.world.level_info.load().game_rules.tnt_explodes {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
            return;
        }

        if Self::dispense_sulfur_cube_block(ctx, item) {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
            return;
        }

        let _ = item.split(1);
        let target = Self::target_position(ctx);

        let tnt = TNTEntity::primed(ctx.world, &target, TNTEntity::DEFAULT_FUSE, None);
        let spawn_pos = tnt.get_entity().pos.load();
        ctx.world.spawn_entity(tnt);
        ctx.world
            .play_sound(Sound::EntityTntPrimed, SoundCategory::Blocks, &spawn_pos);
        ctx.world
            .emit_game_event(GameEvent::EntityPlace.name(), target.to_centered_f64());

        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
    }

    fn dispense_spawn_egg(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let Some(entity_type) =
            crate::item::items::spawn_egg::entity_type_from_spawn_egg_stack(item)
        else {
            return;
        };

        let spawn_pos = Self::target_position(ctx).to_f64();

        let mob = from_type(entity_type, spawn_pos, ctx.world, Uuid::new_v4());
        let yaw = wrap_degrees(rng().random::<f32>() * 360.0) % 360.0;
        mob.get_entity().set_rotation(yaw, 0.0);
        // A dispenser has no acting player, matching vanilla's null `user` for this source.
        prepare_egg_mob(
            item,
            &mob,
            ctx.world,
            crate::entity::mob::spawn::EntitySpawnReason::Dispenser,
            None,
        );

        // Vanilla SpawnEggItemBehavior keeps the egg when nothing spawned.
        if ctx
            .world
            .spawn_creature(mob, CreatureSpawnReason::DispenseEgg, None)
        {
            item.decrement(1);
        }

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
    }

    fn dispense_snowball(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
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
        );
    }

    fn dispense_egg(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::EGG,
        );
        let egg = EggEntity::new(entity);
        egg.set_item_stack(projectile);
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
        );
    }

    fn dispense_splash_potion(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::SPLASH_POTION,
        );
        let potion = SplashPotionEntity::new(entity);
        potion.set_item_stack(projectile);
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
        );
    }

    fn dispense_lingering_potion(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::LINGERING_POTION,
        );
        let potion = LingeringPotionEntity::new(entity);
        potion.set_item_stack(projectile);
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
        );
    }

    fn dispense_fire_charge(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::charge_spawn_position(ctx),
            &EntityType::SMALL_FIREBALL,
        );
        let fireball = SmallFireballEntity::new(entity);
        let facing = to_normal(ctx.facing);
        let dir = Vector3::new(
            triangle(&mut rng(), facing.x, 0.114_850_000_000_000_01),
            triangle(&mut rng(), facing.y, 0.114_850_000_000_000_01),
            triangle(&mut rng(), facing.z, 0.114_850_000_000_000_01),
        )
        .normalize();
        fireball.thrown.assign_directional_movement(dir);
        Self::finish_projectile_launch(ctx, Arc::new(fireball), WorldEvent::SoundBlazeFireball);
    }

    fn dispense_wind_charge(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let _ = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::charge_spawn_position(ctx),
            &EntityType::WIND_CHARGE,
        );
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
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
        );
    }

    fn dispense_firework_rocket(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let launched_stack = item.split(1);
        let facing = to_normal(ctx.facing);
        // Vanilla `getEntityJustOutsideOfBlockPos`.
        let position = ctx
            .position
            .to_centered_f64()
            .add(&(facing * 0.500_009_999_999_747_4));
        let entity = Entity::new(ctx.world.clone(), position, &EntityType::FIREWORK_ROCKET);
        let rocket = FireworkRocketEntity::new_with_stack(entity, launched_stack);
        rocket.set_shot_at_angle(true);

        // `FireworkRocketEntity` does not expose its inner projectile, so replicate
        // `ThrownItemEntity::set_velocity` here.
        let deviation = 0.017_227_5 * Self::FIREWORK_PROJECTILE_UNCERTAINTY;
        let velocity = Vector3::new(facing.x, facing.y, facing.z)
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

        Self::finish_projectile_launch(ctx, Arc::new(rocket), WorldEvent::SoundFireworkShoot);
    }

    fn dispense_empty_bucket(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) {
        let front = Self::target_position(ctx);
        let Some(filled) = try_pickup_fluid_at(ctx.world, front) else {
            Self::drop_item(ctx, item);
            return;
        };

        Self::consume_with_remainder(ctx, dispenser, item, ItemStack::new(1, filled));
        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
    }

    /// Places `stack` into the first empty slot, returning it back if every slot is occupied.
    /// The slot currently being dispensed from still holds its pre-dispense stack, so it is
    /// never considered free.
    fn add_to_first_free_slot(
        dispenser: &DispenserBlockEntity,
        stack: ItemStack,
    ) -> Option<ItemStack> {
        let mut items = dispenser
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in items.iter_mut() {
            if slot.is_empty() {
                *slot = stack;
                dispenser.mark_dirty();
                return None;
            }
        }
        Some(stack)
    }

    fn dispense_filled_bucket(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) {
        if dispense_filled_bucket_at(
            ctx.world,
            *ctx.position,
            ctx.facing.to_block_direction(),
            item,
        ) {
            Self::consume_with_remainder(ctx, dispenser, item, ItemStack::new(1, &Item::BUCKET));
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::drop_item(ctx, item);
        }
    }

    fn try_ignite_explosive_entities(world: &Arc<World>, pos: &BlockPos) -> bool {
        let target_box = BoundingBox::from_block(pos);
        for entity in world.get_entities_at_box(&target_box) {
            let Some(sulfur_cube) = entity.cast_any().downcast_ref::<SulfurCubeEntity>() else {
                continue;
            };
            if sulfur_cube.can_explode() {
                sulfur_cube.prime_time(false);
                return true;
            }
        }
        false
    }

    fn dispense_flint_and_steel(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let front = Self::target_position(ctx);
        let front_block = ctx.world.get_block(&front);

        let ignited = if Self::try_ignite_explosive_entities(ctx.world, &front) {
            true
        } else if front_block == &Block::TNT {
            TNTBlock::prime(ctx.world, &front)
        } else {
            Ignition::ignite_block(
                |world: Arc<World>, pos: BlockPos, new_state_id: BlockStateId| {
                    world.set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL);
                },
                ctx.world,
                front,
                front,
                front_block,
            )
        };

        if ignited {
            // `damage_item` already consumes the tool from the stack when it breaks.
            let _ = item.damage_item(1);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    fn dispense_bone_meal(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let target = Self::target_position(ctx);
        if BoneMealItem::grow_crop(item, ctx.world, &target)
            || BoneMealItem::grow_water_plant(item, ctx.world, &target, None)
        {
            ctx.world
                .sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, target, 15);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    fn dispense_brush(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let target_box = BoundingBox::from_block(&Self::target_position(ctx));
        for entity in ctx.world.get_entities_at_box(&target_box) {
            if entity.is_spectator() {
                continue;
            }
            let Some(armadillo) = entity.cast_any().downcast_ref::<ArmadilloEntity>() else {
                continue;
            };
            if armadillo.brush_off_scute(None, item) {
                let _ = item.damage_item(BRUSH_DURABILITY_COST);
                Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
                return;
            }
        }
        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
    }

    fn dispense_chest(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let target_box = BoundingBox::from_block(&Self::target_position(ctx));
        for entity in ctx.world.get_entities_at_box(&target_box) {
            if try_equip_dispenser_chest(entity.as_ref()) {
                item.decrement(1);
                Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
                return;
            }
        }
        Self::drop_item(ctx, item);
    }

    fn dispense_honeycomb(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let front = Self::target_position(ctx);
        let front_block = ctx.world.get_block(&front);

        if try_wax_block(ctx.world, front, front_block) {
            item.decrement(1);
            ctx.world
                .sync_world_event(WorldEvent::ParticlesWaxOn, front, 0);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::drop_item(ctx, item);
        }
    }

    fn dispense_minecart(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        fn rail_is_ascending(world: &Arc<World>, pos: &BlockPos) -> Option<bool> {
            let (block, state_id) = world.get_block_and_state_id(pos);
            if !block.has_tag(&tag::Block::MINECRAFT_RAILS) {
                return None;
            }
            Some(if PoweredRailLikeProperties::handles_block_id(block.id) {
                PoweredRailLikeProperties::from_state_id(state_id)
                    .shape
                    .is_ascending()
            } else {
                RailLikeProperties::from_state_id(state_id)
                    .shape
                    .is_ascending()
            })
        }

        let target = Self::target_position(ctx);
        let height = if let Some(ascending) = rail_is_ascending(ctx.world, &target) {
            if ascending { 0.6 } else { 0.1 }
        } else if ctx.world.get_block_state(&target).is_air()
            && let Some(ascending) = rail_is_ascending(ctx.world, &target.down())
        {
            if ascending && ctx.facing != Facing::Down {
                -0.4
            } else {
                -0.9
            }
        } else {
            return false;
        };

        let entity_type = MinecartItem::item_to_entity(item.item);
        let dispensed = item.split(1);

        let normal = to_normal(ctx.facing);
        let center = ctx.position.to_centered_f64();
        let spawn_pos = Vector3::new(
            normal.x.mul_add(1.125, center.x),
            f64::from(ctx.position.0.y) + normal.y + height,
            normal.z.mul_add(1.125, center.z),
        );

        let entity = Entity::new(ctx.world.clone(), spawn_pos, entity_type);
        let minecart = Arc::new(MinecartEntity::new(entity));
        crate::item::items::spawn_egg::apply_entity_variant(&dispensed, minecart.as_ref(), None);
        ctx.world.spawn_entity(minecart);

        ctx.world
            .sync_world_event(WorldEvent::SoundDispenserDispense, *ctx.position, 0);
        true
    }

    fn dispense_shears(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        if Self::shear_beehive(ctx, item) || Self::shear_entity_in_front(ctx, item) {
            // `damage_item` already consumes the tool from the stack when it breaks.
            let _ = item.damage_item(1);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    fn shear_beehive(ctx: &DispenseContext<'_>, tool: &ItemStack) -> bool {
        const FULL_HONEY_LEVEL: u8 = 5;

        let target = Self::target_position(ctx);
        let (block, state_id) = ctx.world.get_block_and_state_id(&target);
        if !block.has_tag(&tag::Block::MINECRAFT_BEEHIVES) {
            return false;
        }

        let mut props = BeeNestLikeProperties::from_state_id(state_id);
        if props.honey_level < FULL_HONEY_LEVEL {
            return false;
        }

        ctx.world
            .play_block_sound(Sound::BlockBeehiveShear, SoundCategory::Blocks, target);
        if let Some(table) = ctx.world.get_loot_table("minecraft:harvest/beehive") {
            let params = crate::world::loot::LootContextParameters {
                world: Some(ctx.world),
                position: Some(target.to_f64()),
                block_state: Some(pumpkin_data::BlockState::from_id(state_id)),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            for stack in
                crate::world::loot::generate_loot_from_handle(&table, rand::random(), &params)
            {
                Self::drop_at(ctx.world, target.to_centered_f64(), stack);
            }
        }

        props.honey_level = 0;
        ctx.world
            .set_block_state(&target, props.to_state_id(block), BlockFlags::NOTIFY_ALL);

        true
    }

    fn shear_entity_in_front(ctx: &DispenseContext<'_>, tool: &ItemStack) -> bool {
        let target = Self::target_position(ctx);
        let target_box = BoundingBox::from_block(&target);

        for entity in ctx.world.get_entities_at_box(&target_box) {
            if entity.is_spectator() {
                continue;
            }

            match entity
                .get_entity()
                .shear_off_all_leash_connections(entity.as_ref(), None)
            {
                Some(true) => return true,
                Some(false) => continue,
                None => {}
            }

            // Vanilla `LivingEntity.isAlive` also needs health left, so mobs in their death animation are skipped.
            if !entity.get_entity().is_alive()
                || entity
                    .get_living_entity()
                    .is_some_and(|living| living.health.load() <= 0.0)
            {
                continue;
            }
            let Some(shearable) = entity.get_mob().and_then(Mob::as_shearable) else {
                continue;
            };
            if !shearable.ready_for_shearing() {
                continue;
            }

            if let Some(server) = ctx.world.server.upgrade() {
                let event = BlockShearEntityEvent::new(
                    *ctx.position,
                    ctx.world.clone(),
                    entity.clone(),
                    tool.clone(),
                );
                let event = server.plugin_manager.fire_blocking(&server, event);
                if event.cancelled {
                    continue;
                }
            }

            if !shearable.shear(SoundCategory::Blocks, tool) {
                continue;
            }
            ctx.world
                .emit_game_event(GameEvent::Shear.name(), target.to_centered_f64());
            return true;
        }

        false
    }

    fn dispense_glass_bottle(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) -> bool {
        const FULL_HONEY_LEVEL: u8 = 5;

        let target = Self::target_position(ctx);
        let (block, state_id) = ctx.world.get_block_and_state_id(&target);

        if block.has_tag(&tag::Block::MINECRAFT_BEEHIVES) {
            let mut props = BeeNestLikeProperties::from_state_id(state_id);
            if props.honey_level < FULL_HONEY_LEVEL {
                return false;
            }

            props.honey_level = 0;
            ctx.world
                .set_block_state(&target, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
            Self::consume_with_remainder(
                ctx,
                dispenser,
                item,
                ItemStack::new(1, &Item::HONEY_BOTTLE),
            );
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
            return true;
        }

        let fluid = ctx.world.get_fluid(&target);
        if fluid.matches_type(&Fluid::WATER) {
            Self::consume_with_remainder(ctx, dispenser, item, water_bottle());
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
            return true;
        }

        false
    }

    fn dispense_water_bottle(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
    ) -> bool {
        let target = Self::target_position(ctx);
        if !ctx
            .world
            .get_block(&target)
            .has_tag(&tag::Block::MINECRAFT_CONVERTIBLE_TO_MUD)
        {
            return false;
        }

        ctx.world.spawn_particle(
            target.to_centered_f64(),
            Vector3::new(0.5, 0.5, 0.5),
            1.0,
            5,
            Particle::Splash,
        );
        ctx.world
            .play_block_sound(Sound::ItemBottleEmpty, SoundCategory::Blocks, *ctx.position);
        ctx.world
            .emit_game_event(GameEvent::FluidPlace.name(), ctx.position.to_centered_f64());
        ctx.world
            .set_block_state(&target, Block::MUD.default_state.id, BlockFlags::NOTIFY_ALL);

        Self::consume_with_remainder(ctx, dispenser, item, ItemStack::new(1, &Item::GLASS_BOTTLE));
        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        true
    }

    fn dispense_glowstone(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> Option<bool> {
        const MAX_CHARGES: u8 = 4;

        let target = Self::target_position(ctx);
        let (block, state_id) = ctx.world.get_block_and_state_id(&target);
        if block != &Block::RESPAWN_ANCHOR {
            return None;
        }

        let mut props = RespawnAnchorLikeProperties::from_state_id(state_id);
        if props.charges >= MAX_CHARGES {
            return Some(false);
        }

        props.charges += 1;
        let _ = item.split(1);
        ctx.world
            .set_block_state(&target, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        ctx.world.play_block_sound(
            Sound::BlockRespawnAnchorCharge,
            SoundCategory::Blocks,
            target,
        );

        Some(true)
    }

    fn dispense_experience_bottle(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        let projectile = item.split(1);
        let entity = Entity::new(
            ctx.world.clone(),
            Self::projectile_spawn_position(ctx),
            &EntityType::EXPERIENCE_BOTTLE,
        );
        let bottle = ExperienceBottleEntity::new(entity);
        bottle.set_item_stack(projectile);
        Self::launch_thrown(
            ctx,
            &bottle.thrown,
            Self::POTION_PROJECTILE_POWER,
            Self::POTION_PROJECTILE_UNCERTAINTY,
        );
        Self::finish_projectile_launch(
            ctx,
            Arc::new(bottle),
            WorldEvent::SoundDispenserProjectileLaunch,
        );
    }

    fn dispense_mob_head(ctx: &DispenseContext<'_>, item: &mut ItemStack, block: &'static Block) {
        let target = Self::target_position(ctx);

        let summons_mob = ctx.world.get_block_state(&target).is_air()
            && if block == &Block::WITHER_SKELETON_SKULL {
                find_wither_pattern(ctx.world, &target).is_some()
            } else {
                can_spawn_golem(ctx.world, &target)
            };

        if summons_mob {
            let state_id = if block == &Block::WITHER_SKELETON_SKULL {
                let mut props = SkeletonSkullLikeProperties::default(block);
                props.rotation = to_rotation16(ctx.facing);
                props.to_state_id(block)
            } else {
                block.default_state.id
            };

            let _ = item.split(1);
            ctx.world
                .set_block_state(&target, state_id, BlockFlags::NOTIFY_ALL);
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else if Self::dispense_equipment(ctx, item) {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);
        } else {
            Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserFail);
        }
    }

    fn dispense_shulker_box(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let Some(block) = Block::from_item_id(item.item.id) else {
            return false;
        };

        let target = Self::target_position(ctx);
        if !ctx.world.get_block_state(&target).replaceable() {
            return false;
        }

        let mut props = EndRodLikeProperties::default(block);
        props.facing = if ctx.world.get_block_state(&target.down()).is_air() {
            ctx.facing
        } else {
            Facing::Up
        };

        let placed = item.split(1);
        ctx.world
            .set_block_state(&target, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        if let Some(block_entity) = ctx.world.get_block_entity(&target) {
            block_entity.apply_item_components(&placed);
        }
        Self::play_dispense_effects(ctx, WorldEvent::SoundDispenserDispense);

        true
    }

    fn dispense_equipment(ctx: &DispenseContext<'_>, item: &mut ItemStack) -> bool {
        let (slot, allowed_entities, equip_sound) = {
            let Some(equippable) = item.get_data_component::<EquippableImpl>() else {
                return false;
            };
            if !equippable.dispensable {
                return false;
            }
            (
                equippable.slot,
                equippable.allowed_entities.clone(),
                equippable.equip_sound.clone(),
            )
        };

        let target_box = BoundingBox::from_block(&Self::target_position(ctx));
        let players = ctx
            .world
            .get_players_at_box(&target_box)
            .into_iter()
            .map(|player| player as Arc<dyn EntityBase>);

        for entity in ctx
            .world
            .get_entities_at_box(&target_box)
            .into_iter()
            .chain(players)
        {
            let Some(living) = entity.get_living_entity() else {
                continue;
            };
            if !living.is_alive()
                || entity.is_spectator()
                || !is_allowed_entity(allowed_entities.as_ref(), entity.get_entity().entity_type)
                || entity.get_mob().is_some_and(|mob| {
                    !mob.can_use_slot(slot) || !mob.can_dispenser_equip_into_slot(slot)
                })
            {
                continue;
            }

            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !equipment.get(slot).is_empty() {
                continue;
            }

            let stack = item.split(1);
            equipment.put(slot, stack.clone());
            drop(equipment);

            living.send_equipment_changes(&[(slot.clone(), stack)]);
            let pos = entity.get_entity().pos.load();
            ctx.world.emit_game_event(GameEvent::Equip.name(), pos);
            if let Some(mob) = entity.get_mob() {
                let mob_entity = mob.get_mob_entity();
                mob_entity.set_guaranteed_drop(slot);
                mob_entity
                    .persistence_required
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                match mob.get_equip_sound(slot) {
                    Some(sound) => ctx.world.play_sound(sound, SoundCategory::Neutral, &pos),
                    None => ctx
                        .world
                        .play_sound_event(&equip_sound, SoundCategory::Blocks, &pos),
                }
            } else {
                ctx.world
                    .play_sound_event(&equip_sound, SoundCategory::Blocks, &pos);
            }

            return true;
        }

        false
    }

    fn consume_with_remainder(
        ctx: &DispenseContext<'_>,
        dispenser: &DispenserBlockEntity,
        item: &mut ItemStack,
        remainder: ItemStack,
    ) {
        item.decrement(1);
        if item.is_empty() {
            *item = remainder;
        } else if let Some(rest) = Self::add_to_first_free_slot(dispenser, remainder) {
            Self::eject_item(ctx, rest);
        }
    }

    fn drop_at(world: &Arc<World>, position: Vector3<f64>, stack: ItemStack) {
        let entity = Entity::new(world.clone(), position, &EntityType::ITEM);
        world.spawn_entity(Arc::new(ItemEntity::new(entity, stack)));
    }

    fn drop_item(ctx: &DispenseContext<'_>, item: &mut ItemStack) {
        Self::drop_item_with_sound(ctx, item, WorldEvent::SoundDispenserDispense);
    }

    fn drop_item_with_sound(ctx: &DispenseContext<'_>, item: &mut ItemStack, sound: WorldEvent) {
        let drop_item = item.split(1);
        Self::eject_item(ctx, drop_item);
        Self::play_dispense_effects(ctx, sound);
    }

    fn eject_item(ctx: &DispenseContext<'_>, stack: ItemStack) {
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
        ctx.world.spawn_entity(item_entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ageable::AgeableMob;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::skeleton::bogged::BoggedSkeletonEntity;
    use crate::entity::passive::abstract_horse::AbstractHorse;
    use crate::entity::passive::copper_golem::CopperGolemEntity;
    use crate::entity::passive::copper_golem::EQUIPMENT_SLOT_ANTENNA;
    use crate::entity::passive::donkey::DonkeyEntity;
    use crate::entity::passive::snow_golem::SnowGolemEntity;
    use crate::entity::shearable::Shearable;
    use pumpkin_data::data_component_impl::SulfurCubeContentImpl;

    #[tokio::test]
    async fn dispenser_cuts_leash_connections_before_shearing_the_pumpkin()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let golem = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(1.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        let holder = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        let attached = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(10.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        let outside = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(17.9, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(golem.clone());
        world.spawn_entity(holder.clone());
        world.spawn_entity(attached.clone());
        world.spawn_entity(outside.clone());
        golem
            .get_entity()
            .attach_leash(&(holder as Arc<dyn EntityBase>));
        let golem_base: Arc<dyn EntityBase> = golem.clone();
        attached.get_entity().attach_leash(&golem_base);
        outside.get_entity().attach_leash(&golem_base);
        let position = BlockPos::new(0, 64, 0);
        let ctx = DispenseContext {
            world: &world,
            position: &position,
            facing: Facing::East,
        };
        let mut shears = ItemStack::new(1, &Item::SHEARS);
        DispenserBlock::dispense_shears(&ctx, &mut shears);
        assert!(golem.has_pumpkin());
        assert!(golem.get_entity().get_leash_holder().is_none());
        assert!(attached.get_entity().get_leash_holder().is_none());
        assert!(outside.get_entity().get_leash_holder().is_some());
        assert_eq!(shears.get_damage(), 1);
        let leads: usize = world
            .entities
            .load()
            .iter()
            .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
            .map(|item| {
                let stack = item.get_item_stack().lock().unwrap();
                if stack.item == &Item::LEAD {
                    usize::from(stack.item_count)
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(leads, 2);
        DispenserBlock::dispense_shears(&ctx, &mut shears);
        assert!(!golem.has_pumpkin());
        assert_eq!(shears.get_damage(), 2);
        world.level.shutdown().await;
        Ok(())
    }

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn fire_dispenser(
        world: &Arc<World>,
        pos: BlockPos,
        facing: Facing,
        stack: ItemStack,
    ) -> ItemStack {
        let dispenser = DispenserBlockEntity::new(pos);
        dispenser.set_stack(0, stack);
        let mut dispensed = dispenser.get_stack(0);
        let ctx = DispenseContext {
            world,
            position: &pos,
            facing,
        };
        DispenserBlock::dispense(&ctx, &dispenser, &mut dispensed);
        dispenser.set_stack(0, dispensed);
        dispenser.get_stack(0)
    }

    #[tokio::test]
    async fn dispenser_sulfur_cube_swallows_tnt_primes_with_flint_and_shears() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dispenser_pos = BlockPos::new(0, 64, 0);
        let target_pos = BlockPos::new(0, 64, 1);

        let cube = SulfurCubeEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::SULFUR_CUBE,
        ));
        cube.set_size(2, true);
        world.spawn_entity(cube.clone());

        // Dispensing TNT into an adult SulfurCube feeds TNT to the cube instead of spawning PrimedTnt
        let remaining_tnt = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(2, &Item::TNT),
        );
        assert_eq!(remaining_tnt.item_count, 1);
        assert!(cube.has_body_item());
        assert!(cube.can_explode());
        let has_primed_tnt = world
            .get_entities_at_box(&BoundingBox::from_block(&target_pos).expand_all(2.0))
            .iter()
            .any(|e| e.get_entity().entity_type == &EntityType::TNT);
        assert!(!has_primed_tnt);

        // Dispensing Shears ejects the body item from the SulfurCube and damages shears by 1
        let remaining_shears = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(1, &Item::SHEARS),
        );
        assert!(!cube.has_body_item());
        assert_eq!(remaining_shears.get_damage(), 1);

        // Dispensing TNT again and then Flint & Steel primes the SulfurCube
        let _ = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(1, &Item::TNT),
        );
        assert!(cube.can_explode());
        let remaining_flint = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(1, &Item::FLINT_AND_STEEL),
        );
        assert!(cube.is_primed());
        assert_eq!(remaining_flint.get_damage(), 1);

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dispenser_shears_bogged_copper_golem_snow_golem_and_leash() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dispenser_pos = BlockPos::new(10, 64, 0);
        let target_pos = BlockPos::new(10, 64, 1);

        let bogged = BoggedSkeletonEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::BOGGED,
        ));
        world.spawn_entity(bogged.clone());
        assert!(!bogged.is_sheared());
        let shears = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(1, &Item::SHEARS),
        );
        assert!(bogged.is_sheared());
        assert_eq!(shears.get_damage(), 1);
        bogged.get_entity().remove();

        let golem = CopperGolemEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::COPPER_GOLEM,
        ));
        golem
            .mob_entity
            .set_item_slot(&EQUIPMENT_SLOT_ANTENNA, ItemStack::new(1, &Item::POPPY));
        world.spawn_entity(golem.clone());
        assert!(golem.ready_for_shearing());
        let shears = fire_dispenser(&world, dispenser_pos, Facing::South, shears);
        assert!(!golem.ready_for_shearing());
        assert_eq!(shears.get_damage(), 2);

        // Leash golem to snow golem and verify shears snip the leash first before shearing pumpkin
        let snow_golem = SnowGolemEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::SNOW_GOLEM,
        ));
        world.spawn_entity(snow_golem.clone());
        snow_golem.get_entity().leash_to(golem.clone());
        assert!(snow_golem.get_entity().is_leashed());

        let shears = fire_dispenser(&world, dispenser_pos, Facing::South, shears);
        assert!(!snow_golem.get_entity().is_leashed());
        assert!(snow_golem.has_pumpkin());
        assert_eq!(shears.get_damage(), 3);

        let shears = fire_dispenser(&world, dispenser_pos, Facing::South, shears);
        assert!(!snow_golem.has_pumpkin());
        assert_eq!(shears.get_damage(), 4);

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dispenser_chest_brush_colored_eggs_and_sulfur_cube_bucket() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dispenser_pos = BlockPos::new(20, 64, 0);
        let target_pos = BlockPos::new(20, 64, 1);

        // Tamed donkey receives chest from dispenser
        let donkey = DonkeyEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::DONKEY,
        ));
        donkey.set_tamed(true);
        world.spawn_entity(donkey.clone());
        let remaining_chest = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(2, &Item::CHEST),
        );
        assert!(donkey.has_chest());
        assert_eq!(remaining_chest.item_count, 1);
        donkey.get_entity().remove();

        // Baby armadillo is skipped; adult armadillo is brushed and damages brush by 16
        let baby_armadillo = ArmadilloEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::ARMADILLO,
        ));
        baby_armadillo.set_baby(true);
        world.spawn_entity(baby_armadillo.clone());
        let untouched_brush = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(1, &Item::BRUSH),
        );
        assert_eq!(untouched_brush.get_damage(), 0);

        let adult_armadillo = ArmadilloEntity::new(Entity::new(
            world.clone(),
            target_pos.to_centered_f64(),
            &EntityType::ARMADILLO,
        ));
        world.spawn_entity(adult_armadillo.clone());
        let used_brush = fire_dispenser(&world, dispenser_pos, Facing::South, untouched_brush);
        assert_eq!(used_brush.get_damage(), 16);
        baby_armadillo.get_entity().remove();
        adult_armadillo.get_entity().remove();

        // Blue egg launches EggEntity carrying Item::BLUE_EGG
        let remaining_eggs = fire_dispenser(
            &world,
            dispenser_pos,
            Facing::South,
            ItemStack::new(2, &Item::BLUE_EGG),
        );
        assert_eq!(remaining_eggs.item_count, 1);
        let launched_egg = world
            .get_entities_at_box(&BoundingBox::from_block(&target_pos).expand_all(2.0))
            .into_iter()
            .find(|e| e.get_entity().entity_type == &EntityType::EGG);
        assert!(launched_egg.is_some());

        // Sulfur cube bucket spawns SulfurCube with stored body item and returns empty bucket
        let mut bucket = ItemStack::new(1, &Item::SULFUR_CUBE_BUCKET);
        bucket.set_data_component(SulfurCubeContentImpl {
            item: ItemStack::new(1, &Item::TNT),
        });
        let result_bucket = fire_dispenser(&world, dispenser_pos, Facing::South, bucket);
        assert_eq!(result_bucket.item.id, Item::BUCKET.id);
        let spawned_cube = world
            .get_entities_at_box(&BoundingBox::from_block(&target_pos).expand_all(2.0))
            .into_iter()
            .find_map(|e| {
                e.cast_any()
                    .downcast_ref::<SulfurCubeEntity>()
                    .map(|c| (c.has_body_item(), c.can_explode()))
            });
        assert_eq!(spawned_cube, Some((true, true)));

        world.level.shutdown().await;
        Ok(())
    }
}
