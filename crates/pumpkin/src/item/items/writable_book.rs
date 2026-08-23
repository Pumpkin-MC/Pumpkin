use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::COpenBook;

pub struct WritableBookItem;

impl ItemMetadata for WritableBookItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::WRITABLE_BOOK.id, Item::WRITTEN_BOOK.id])
    }
}

impl ItemBehaviour for WritableBookItem {
    fn normal_use<'a>(
        &'a self,
        item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if item.id == Item::WRITTEN_BOOK.id {
                player
                    .send_client_packet(&COpenBook::new(VarInt(0))) // 0 = main hand
                    .await;
                player.world().play_sound(
                    Sound::ItemBookPageTurn,
                    SoundCategory::Players,
                    &player.position(),
                );
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
