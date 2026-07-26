use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CMerchantOffers;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;

use crate::entity::EntityBase;

use super::VillagerEntity;

impl ScreenHandlerFactory for VillagerEntity {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let offers = self.offers.lock().await;
            let self_weak = self.self_weak.lock().unwrap().clone().unwrap();
            let player_uuid = player
                .as_any()
                .downcast_ref::<crate::entity::player::Player>()
                .map_or_else(uuid::Uuid::nil, |p| p.get_entity().entity_uuid);
            let world = self.get_entity().world.load().clone();

            let mut handler = MerchantScreenHandler::new(
                sync_id,
                player_inventory,
                self.merchant_inventory.clone(),
                offers.clone(),
            )
            .await;

            handler.on_trade = Some(Box::new(move |offer_index| {
                if let Some(villager) = self_weak.upgrade() {
                    let world = world.clone();
                    tokio::spawn(async move {
                        if let Some(player) = world.get_player_by_uuid(player_uuid) {
                            let mut offers = villager.offers.lock().await;
                            if offer_index < offers.len() {
                                let offer = &mut offers[offer_index];
                                offer.uses += 1;

                                let xp_gain = offer.xp;
                                let current_xp =
                                    villager.xp.fetch_add(xp_gain, Ordering::Relaxed) + xp_gain;

                                let mut data = villager.villager_data.lock().await;
                                let current_level = data.level.0;
                                if current_level < 5 {
                                    let max_xp = match current_level {
                                        1 => 10,
                                        2 => 70,
                                        3 => 150,
                                        4 => 250,
                                        _ => 0,
                                    };
                                    if current_xp >= max_xp {
                                        data.level.0 += 1;
                                        let new_level = data.level.0;
                                        let prof = data.profession_enum();
                                        drop(data);

                                        // Level up! Add new trades for the new level
                                        villager.add_trades(prof, new_level).await;

                                        // Play sound & particles for level up!
                                        let entity = villager.get_entity();
                                        entity.world.load().send_entity_status(
                                            entity,
                                            pumpkin_data::entity::EntityStatus::VillagerHappy,
                                        );
                                        entity.play_sound(
                                            pumpkin_data::sound::Sound::EntityVillagerCelebrate,
                                        );
                                    } else {
                                        drop(data);
                                    }
                                } else {
                                    drop(data);
                                }

                                let current_level = villager.villager_data.lock().await.level;
                                player
                                    .client
                                    .enqueue_packet(&CMerchantOffers::new(
                                        VarInt(sync_id as i32),
                                        offers.clone(),
                                        current_level,
                                        VarInt(current_xp),
                                        true,
                                        true,
                                    ))
                                    .await;
                            }
                        }
                    });
                }
            }));

            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // TODO: Localized name based on profession
        TextComponent::text("Villager")
    }
}
