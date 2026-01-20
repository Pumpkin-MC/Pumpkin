use pumpkin_data::sound::Sound;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, OnLandedUponArgs};

#[pumpkin_block("minecraft:powder_snow")]
pub struct PowderSnowBlock;

impl BlockBehaviour for PowderSnowBlock {
    fn on_landed_upon<'a>(&'a self, args: OnLandedUponArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = args.entity.get_living_entity()
                && !(args.fall_distance < 4.0)
            {
                let sound = if args.fall_distance < 7.0 {
                    Sound::EntityGenericSmallFall
                } else {
                    Sound::EntityGenericBigFall
                };

                living.entity.play_sound(sound).await;
            }
        })
    }
}
