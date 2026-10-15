use pumpkin_data::enchantment::Enchantment;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::block::BlockBehaviour;
use crate::block::BrokenArgs;
use crate::entity::r#type::from_type;

#[pumpkin_block_from_tag("c:cobblestones/infested")]
pub struct InfestedBlock;

impl BlockBehaviour for InfestedBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        if args.player.gamemode.load() == GameMode::Creative {
            return;
        }

        let held_item = args.player.inventory().held_item();
        if held_item.get_enchantment_level(&Enchantment::SILK_TOUCH) > 0 {
            return;
        }

        let spawn_pos = Vector3::new(
            f64::from(args.position.0.x) + 0.5,
            f64::from(args.position.0.y),
            f64::from(args.position.0.z) + 0.5,
        );

        let silverfish = from_type(&EntityType::SILVERFISH, spawn_pos, args.world, Uuid::new_v4());
        args.world.spawn_entity(silverfish);
    }
}
