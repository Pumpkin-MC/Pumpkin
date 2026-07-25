use rand::{Rng, RngExt, rng};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::item::ItemEntity;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::projectile::egg::EggEntity;
use crate::entity::projectile::lingering_potion::LingeringPotionEntity;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::projectile::splash_potion::SplashPotionEntity;
use crate::entity::tnt::TNTEntity;
use crate::entity::{Entity, EntityBase};

use crate::block::entities::dispenser::DispenserBlockEntity;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_inventory::generic_container_screen_handler::create_generic_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct ProjectileSettings {
    entity: &'static EntityType,
    power: f64,
    uncertainty: f64,
    facing_offset: f64,
    up_nudge: f64,
}

/// Dispenser projectile settings, keyed by item id. Mirrors each item's
/// vanilla `ProjectileItem.Settings` (position offset, power, uncertainty).
///
/// NOT included, and why:
/// - Arrows (arrow/tipped/spectral) use the `ArrowEntity` path (owner/pickup) — handled separately.
/// - Firework rocket: custom position function + dispense sound 1004 — does not fit this table.
/// - Experience bottle (vanilla power 1.375, uncertainty 3.0): no projectile entity exists in Pumpkin yet.
static PROJECTILES: &[(u16, ProjectileSettings)] = &[
    // Default settings: power 1.1, uncertainty 6.0, offset 0.7, +0.1 up.
    (
        Item::SNOWBALL.id,
        ProjectileSettings {
            entity: &EntityType::SNOWBALL,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    (
        Item::EGG.id,
        ProjectileSettings {
            entity: &EntityType::EGG,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    (
        Item::BLUE_EGG.id,
        ProjectileSettings {
            entity: &EntityType::EGG,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    (
        Item::BROWN_EGG.id,
        ProjectileSettings {
            entity: &EntityType::EGG,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    (
        Item::SPLASH_POTION.id,
        ProjectileSettings {
            entity: &EntityType::SPLASH_POTION,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    (
        Item::LINGERING_POTION.id,
        ProjectileSettings {
            entity: &EntityType::LINGERING_POTION,
            power: 1.1,
            uncertainty: 6.0,
            facing_offset: 0.7,
            up_nudge: 0.1,
        },
    ),
    // Charges: full-block offset, no up-nudge, power 1.0, uncertainty 6.6666665.
    // NOTE: SMALL_FIREBALL and WIND_CHARGE entities currently only expose `new_shot(shooter)`;
    // they need an ownerless constructor before these rows can actually spawn from a dispenser.
    (
        Item::FIRE_CHARGE.id,
        ProjectileSettings {
            entity: &EntityType::SMALL_FIREBALL,
            power: 1.0,
            uncertainty: 6.666_666_5,
            facing_offset: 1.0,
            up_nudge: 0.0,
        },
    ),
    (
        Item::WIND_CHARGE.id,
        ProjectileSettings {
            entity: &EntityType::WIND_CHARGE,
            power: 1.0,
            uncertainty: 6.666_666_5,
            facing_offset: 1.0,
            up_nudge: 0.0,
        },
    ),
];

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

                    // Dispatch on the item type; the default arm ejects it as an item entity.
                    let drop_item = item.split(1);
                    let facing = to_normal(props.facing);

                    match drop_item.get_item().id {
                        id if id == Item::TNT.id => {
                            // Vanilla: block in front of the dispenser, centered on X/Z,
                            // at the block's bottom on Y (no +0.5 on Y).
                            let position = Vector3::new(
                                args.position.0.x as f64 + facing.x + 0.5,
                                args.position.0.y as f64 + facing.y,
                                args.position.0.z as f64 + facing.z + 0.5,
                            );

                            let entity =
                                Entity::new(args.world.clone(), position, &EntityType::TNT);

                            let tnt_entity = Arc::new(TNTEntity::new(entity, 4.0, 80));
                            args.world.spawn_entity(tnt_entity).await;
                        }
                        id if id == Item::ARROW.id => {
                            // Vanilla: block center + 0.7 in the facing direction, nudged up 0.1.
                            let position = args.position.to_centered_f64()
                                + facing * 0.7
                                + Vector3::new(0.0, 0.1, 0.0);
                            let arrow_entity =
                                Entity::new(args.world.clone(), position, &EntityType::ARROW);
                            let mut arrow = ArrowEntity::new(arrow_entity, None);
                            arrow.pickup = ArrowPickup::Allowed;

                            arrow.set_velocity(facing.x, facing.y, facing.z, 1.1, 6.0);

                            args.world
                                .spawn_entity(Arc::new(arrow) as Arc<dyn EntityBase>)
                                .await;
                        }
                        id => {
                            if let Some((_, settings)) = PROJECTILES.iter().find(|p| p.0 == id) {
                                let position = args.position.to_centered_f64()
                                    + facing * settings.facing_offset
                                    + Vector3::new(0.0, settings.up_nudge, 0.0);

                                let entity =
                                    Entity::new(args.world.clone(), position, settings.entity);

                                // Distinct projectile structs can't be built generically from
                                // `&EntityType`, so match to construct the right one, then apply
                                // the shared velocity (power/uncertainty from the table).
                                let projectile: Option<Arc<dyn EntityBase>> = match settings.entity.id
                                {
                                    id if id == EntityType::SNOWBALL.id => {
                                        let e = SnowballEntity::new(entity);
                                        e.thrown.set_velocity(
                                            facing.x,
                                            facing.y,
                                            facing.z,
                                            settings.power,
                                            settings.uncertainty,
                                        );
                                        Some(Arc::new(e) as Arc<dyn EntityBase>)
                                    }
                                    id if id == EntityType::EGG.id => {
                                        let e = EggEntity::new(entity);
                                        e.thrown.set_velocity(
                                            facing.x,
                                            facing.y,
                                            facing.z,
                                            settings.power,
                                            settings.uncertainty,
                                        );
                                        Some(Arc::new(e) as Arc<dyn EntityBase>)
                                    }
                                    id if id == EntityType::SPLASH_POTION.id => {
                                        let e = SplashPotionEntity::new(entity);
                                        e.thrown.set_velocity(
                                            facing.x,
                                            facing.y,
                                            facing.z,
                                            settings.power,
                                            settings.uncertainty,
                                        );
                                        Some(Arc::new(e) as Arc<dyn EntityBase>)
                                    }
                                    id if id == EntityType::LINGERING_POTION.id => {
                                        let e = LingeringPotionEntity::new(entity);
                                        e.thrown.set_velocity(
                                            facing.x,
                                            facing.y,
                                            facing.z,
                                            settings.power,
                                            settings.uncertainty,
                                        );
                                        Some(Arc::new(e) as Arc<dyn EntityBase>)
                                    }
                                    // FIRE_CHARGE / WIND_CHARGE: entities only expose
                                    // `new_shot(shooter)` — need an ownerless constructor first.
                                    _ => None,
                                };

                                if let Some(projectile) = projectile {
                                    args.world.spawn_entity(projectile).await;
                                }
                            } else {
                                let mut position =
                                    args.position.to_centered_f64().add(&(facing * 0.7));

                                position.y -= match props.facing {
                                    Facing::Up | Facing::Down => 0.125,
                                    _ => 0.15625,
                                };

                                let entity =
                                    Entity::new(args.world.clone(), position, &EntityType::ITEM);
                                let rd = rng().random::<f64>().mul_add(0.1, 0.2);

                                let velocity = Vector3::new(
                                    triangle(&mut rng(), facing.x * rd, 0.017_227_5 * 6.),
                                    triangle(&mut rng(), 0.2, 0.017_227_5 * 6.),
                                    triangle(&mut rng(), facing.z * rd, 0.017_227_5 * 6.),
                                );

                                let item_entity = Arc::new(ItemEntity::new_with_velocity(
                                    entity, drop_item, velocity, 40,
                                ));
                                args.world.spawn_entity(item_entity).await;
                            }
                        }
                    }

                    args.world.sync_world_event(
                        WorldEvent::SoundDispenserDispense,
                        *args.position,
                        0,
                    );

                    args.world.sync_world_event(
                        WorldEvent::ParticlesShootSmoke,
                        *args.position,
                        to_data3d(props.facing),
                    );
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
