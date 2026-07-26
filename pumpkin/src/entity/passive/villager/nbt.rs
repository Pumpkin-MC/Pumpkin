use std::sync::Arc;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::entity::NBTStorage;

use super::{GossipType, VillagerEntity};

impl NBTStorage for VillagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            let data = self.villager_data.lock().await;
            let mut villager_data_nbt = NbtCompound::new();
            villager_data_nbt.put_int("Type", data.r#type.0);
            villager_data_nbt.put_int("Profession", data.profession.0);
            villager_data_nbt.put_int("Level", data.level.0);
            nbt.put_compound("VillagerData", villager_data_nbt);

            nbt.put_int("FoodLevel", self.food_level.load(Ordering::Relaxed));
            nbt.put_int("Xp", self.xp.load(Ordering::Relaxed));
            nbt.put_long(
                "LastRestock",
                self.last_restock_time.load(Ordering::Relaxed),
            );
            nbt.put_int("RestocksToday", self.restocks_today.load(Ordering::Relaxed));

            let job_site_pos = *self.job_site.lock().unwrap();
            if let Some(pos) = job_site_pos {
                nbt.put_int("JobSiteX", pos.0.x);
                nbt.put_int("JobSiteY", pos.0.y);
                nbt.put_int("JobSiteZ", pos.0.z);
            }

            let home_pos = *self.home_pos.lock().unwrap();
            if let Some(pos) = home_pos {
                nbt.put_int("HomeX", pos.0.x);
                nbt.put_int("HomeY", pos.0.y);
                nbt.put_int("HomeZ", pos.0.z);
            }

            // Save Offers
            {
                let offers = self.offers.lock().await;
                let mut recipes = Vec::new();
                for offer in offers.iter() {
                    let mut recipe = NbtCompound::new();

                    let mut buy = NbtCompound::new();
                    offer.base_cost_a.0.write_item_stack(&mut buy);
                    recipe.put_compound("buy", buy);

                    if let Some(cost_b) = &offer.cost_b
                        && !cost_b.0.is_empty()
                    {
                        let mut buy_b = NbtCompound::new();
                        cost_b.0.write_item_stack(&mut buy_b);
                        recipe.put_compound("buyB", buy_b);
                    }

                    let mut sell_item = NbtCompound::new();
                    offer.output.0.write_item_stack(&mut sell_item);
                    recipe.put_compound("sell", sell_item);

                    recipe.put_int("uses", offer.uses);
                    recipe.put_int("maxUses", offer.max_uses);
                    recipe.put_bool("rewardExp", !offer.is_disabled);
                    recipe.put_int("xp", offer.xp);
                    recipe.put_float("priceMultiplier", offer.price_multiplier);
                    recipe.put_int("specialPrice", offer.special_price);
                    recipe.put_int("demand", offer.demand);

                    recipes.push(pumpkin_nbt::tag::NbtTag::Compound(recipe));
                }
                let mut offers_compound = NbtCompound::new();
                offers_compound.put("Recipes", pumpkin_nbt::tag::NbtTag::List(recipes));
                nbt.put_compound("Offers", offers_compound);
            };

            // Inventory
            let inventory = self.inventory.lock().await;
            let mut inventory_list = Vec::new();
            for stack_mutex in inventory.iter() {
                let stack = stack_mutex.lock().await;
                if !stack.is_empty() {
                    let mut item_nbt = NbtCompound::new();
                    stack.write_item_stack(&mut item_nbt);
                    inventory_list.push(pumpkin_nbt::tag::NbtTag::Compound(item_nbt));
                }
            }
            nbt.put("Inventory", pumpkin_nbt::tag::NbtTag::List(inventory_list));

            // Gossips
            let gossips = self.gossips.lock().await;
            let mut gossip_list = Vec::new();
            for (uuid, types) in gossips.iter() {
                for (gtype, value) in types {
                    let mut gossip_nbt = NbtCompound::new();
                    let uuid_val = uuid.as_u128();
                    gossip_nbt.put(
                        "Target",
                        pumpkin_nbt::tag::NbtTag::IntArray(vec![
                            (uuid_val >> 96) as i32,
                            ((uuid_val >> 64) & 0xFFFF_FFFF) as i32,
                            ((uuid_val >> 32) & 0xFFFF_FFFF) as i32,
                            (uuid_val & 0xFFFF_FFFF) as i32,
                        ]),
                    );
                    gossip_nbt.put_int("Type", *gtype as i32);
                    gossip_nbt.put_int("Value", *value);
                    gossip_list.push(pumpkin_nbt::tag::NbtTag::Compound(gossip_nbt));
                }
            }
            nbt.put("Gossips", pumpkin_nbt::tag::NbtTag::List(gossip_list));
        })
    }

    #[allow(clippy::too_many_lines)]
    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            if let Some(villager_data_nbt) = nbt.get_compound("VillagerData") {
                let mut data = self.villager_data.lock().await;
                if let Some(t) = villager_data_nbt.get_int("Type") {
                    data.r#type = VarInt(t);
                }
                if let Some(p) = villager_data_nbt.get_int("Profession") {
                    data.profession = VarInt(p);
                }
                if let Some(l) = villager_data_nbt.get_int("Level") {
                    data.level = VarInt(l);
                }
            }

            if let Some(food) = nbt.get_int("FoodLevel") {
                self.food_level.store(food, Ordering::Relaxed);
            }
            if let Some(xp) = nbt.get_int("Xp") {
                self.xp.store(xp, Ordering::Relaxed);
            }
            if let Some(restock) = nbt.get_long("LastRestock") {
                self.last_restock_time.store(restock, Ordering::Relaxed);
            }
            if let Some(today) = nbt.get_int("RestocksToday") {
                self.restocks_today.store(today, Ordering::Relaxed);
            }

            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("JobSiteX"),
                nbt.get_int("JobSiteY"),
                nbt.get_int("JobSiteZ"),
            ) {
                *self.job_site.lock().unwrap() = Some(BlockPos::new(x, y, z));
            } else {
                *self.job_site.lock().unwrap() = None;
            }

            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("HomeX").or_else(|| nbt.get_int("BedX")),
                nbt.get_int("HomeY").or_else(|| nbt.get_int("BedY")),
                nbt.get_int("HomeZ").or_else(|| nbt.get_int("BedZ")),
            ) {
                *self.home_pos.lock().unwrap() = Some(BlockPos::new(x, y, z));
            } else {
                *self.home_pos.lock().unwrap() = None;
            }

            if let Some(offers_compound) = nbt.get_compound("Offers")
                && let Some(recipes) = offers_compound.get_list("Recipes")
            {
                let mut offers = self.offers.lock().await;
                offers.clear();
                for tag in recipes {
                    if let Some(recipe) = tag.extract_compound() {
                        let buy = recipe
                            .get_compound("buy")
                            .and_then(ItemStack::read_item_stack);
                        let buy_b = recipe
                            .get_compound("buyB")
                            .and_then(ItemStack::read_item_stack);
                        let sell_item = recipe
                            .get_compound("sell")
                            .and_then(ItemStack::read_item_stack);

                        if let (Some(buy), Some(sell_item)) = (buy, sell_item) {
                            let uses = recipe.get_int("uses").unwrap_or(0);
                            let max_uses = recipe.get_int("maxUses").unwrap_or(12);
                            let reward_exp = recipe.get_bool("rewardExp").unwrap_or(true);
                            let xp = recipe.get_int("xp").unwrap_or(2);
                            let price_multiplier =
                                recipe.get_float("priceMultiplier").unwrap_or(0.05);
                            let special_price = recipe.get_int("specialPrice").unwrap_or(0);
                            let demand = recipe.get_int("demand").unwrap_or(0);

                            offers.push(pumpkin_protocol::java::client::play::MerchantOffer {
                                base_cost_a: buy.into(),
                                output: sell_item.into(),
                                cost_b: buy_b.map(Into::into),
                                is_disabled: !reward_exp,
                                uses,
                                max_uses,
                                xp,
                                special_price,
                                price_multiplier,
                                demand,
                            });
                        }
                    }
                }
            }

            // Inventory
            if let Some(inventory_list) = nbt.get_list("Inventory") {
                let mut inventory = self.inventory.lock().await;
                inventory.clear();
                for tag in inventory_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(stack) = ItemStack::read_item_stack(item_compound)
                    {
                        inventory.push(Arc::new(Mutex::new(stack)));
                    }
                }
            }

            // Gossips
            if let Some(gossip_list) = nbt.get_list("Gossips") {
                let mut gossips = self.gossips.lock().await;
                gossips.clear();
                for tag in gossip_list {
                    if let Some(gossip_nbt) = tag.extract_compound() {
                        let uuid = gossip_nbt.get_int_array("Target").map(|uuid_array| {
                            Uuid::from_u128(
                                (uuid_array[0] as u128) << 96
                                    | (uuid_array[1] as u128) << 64
                                    | (uuid_array[2] as u128) << 32
                                    | (uuid_array[3] as u128),
                            )
                        });
                        if let (Some(uuid), Some(gtype), Some(val)) = (
                            uuid,
                            gossip_nbt.get_int("Type"),
                            gossip_nbt.get_int("Value"),
                        ) {
                            let gossip_type = match gtype {
                                0 => GossipType::MajorNegative,
                                1 => GossipType::MinorNegative,
                                2 => GossipType::MajorPositive,
                                3 => GossipType::MinorPositive,
                                4 => GossipType::Trading,
                                _ => continue,
                            };
                            gossips.entry(uuid).or_default().insert(gossip_type, val);
                        }
                    }
                }
            }
        })
    }
}
