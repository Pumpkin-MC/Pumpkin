use std::sync::Arc;
use wasmtime::component::Resource;

use crate::block::entities::BlockEntity as InternalBlockEntity;
use crate::block::entities::banner::BannerBlockEntity as InternalBannerBlockEntity;
use crate::block::entities::beacon::BeaconBlockEntity as InternalBeaconBlockEntity;
use crate::block::entities::beehive::BeehiveBlockEntity as InternalBeehiveBlockEntity;
use crate::block::entities::bell::BellBlockEntity as InternalBellBlockEntity;
use crate::block::entities::blasting_furnace::BlastingFurnaceBlockEntity as InternalBlastingFurnaceBlockEntity;
use crate::block::entities::brewing_stand::BrewingStandBlockEntity as InternalBrewingStandBlockEntity;
use crate::block::entities::chest::ChestBlockEntity as InternalChestBlockEntity;
use crate::block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity as InternalChiseledBookshelfBlockEntity;
use crate::block::entities::command_block::CommandBlockEntity as InternalCommandBlockEntity;
use crate::block::entities::comparator::ComparatorBlockEntity as InternalComparatorBlockEntity;
use crate::block::entities::crafter::CrafterBlockEntity as InternalCrafterBlockEntity;
use crate::block::entities::creaking_heart::CreakingHeartBlockEntity as InternalCreakingHeartBlockEntity;
use crate::block::entities::end_gateway::EndGatewayBlockEntity as InternalEndGatewayBlockEntity;
use crate::block::entities::furnace::FurnaceBlockEntity as InternalFurnaceBlockEntity;
use crate::block::entities::furnace_like_block_entity::CookingBlockEntityBase;
use crate::block::entities::hanging_sign::HangingSignBlockEntity as InternalHangingSignBlockEntity;
use crate::block::entities::hopper::HopperBlockEntity as InternalHopperBlockEntity;
use crate::block::entities::jigsaw_block::JigsawBlockEntity as InternalJigsawBlockEntity;
use crate::block::entities::jukebox::JukeboxBlockEntity as InternalJukeboxBlockEntity;
use crate::block::entities::lectern::LecternBlockEntity as InternalLecternBlockEntity;
use crate::block::entities::map::MapBlockEntity as InternalMapBlockEntity;
use crate::block::entities::mob_spawner::MobSpawnerBlockEntity as InternalMobSpawnerBlockEntity;
use crate::block::entities::piston::PistonBlockEntity as InternalPistonBlockEntity;
use crate::block::entities::sculk_shrieker::SculkShriekerBlockEntity as InternalSculkShriekerBlockEntity;
use crate::block::entities::sign::{
    DyeColor as InternalDyeColor, SignBlockEntity as InternalSignBlockEntity, Text as InternalText,
};
use crate::block::entities::skull::SkullBlockEntity as InternalSkullBlockEntity;
use crate::block::entities::smoker::SmokerBlockEntity as InternalSmokerBlockEntity;
use crate::block::entities::structure_block::StructureBlockBlockEntity as InternalStructureBlockBlockEntity;
use crate::plugin::loader::wasm::wasm_host::{
    state::{BlockEntityResource, ContainerBlockEntityResource, PluginHostState},
    wit::v0_1::pumpkin::{
        self,
        plugin::{
            block_entity::{
                BannerBlockEntity, BarrelBlockEntity, BeaconBlockEntity, BedBlockEntity,
                BeehiveBlockEntity, BellBlockEntity, BlastingFurnaceBlockEntity, BlockEntity,
                BrewingStandBlockEntity, BrushableBlockBlockEntity,
                CalibratedSculkSensorBlockEntity, CampfireBlockEntity, ChestBlockEntity,
                ChiseledBookshelfBlockEntity, CommandBlockEntity, ComparatorBlockEntity,
                ConduitBlockEntity, ContainerBlockEntity, CopperGolemStatueBlockEntity,
                CrafterBlockEntity, CreakingHeartBlockEntity, DaylightDetectorBlockEntity,
                DecoratedPotBlockEntity, DispenserBlockEntity, DropperBlockEntity, DyeColor,
                EnchantingTableBlockEntity, EndGatewayBlockEntity, EndPortalBlockEntity,
                EnderChestBlockEntity, FurnaceBlockEntity, HangingSignBlockEntity,
                HopperBlockEntity, HostBannerBlockEntity, HostBarrelBlockEntity,
                HostBeaconBlockEntity, HostBedBlockEntity, HostBeehiveBlockEntity,
                HostBellBlockEntity, HostBlastingFurnaceBlockEntity, HostBlockEntity,
                HostBrewingStandBlockEntity, HostBrushableBlockBlockEntity,
                HostCalibratedSculkSensorBlockEntity, HostCampfireBlockEntity,
                HostChestBlockEntity, HostChiseledBookshelfBlockEntity, HostCommandBlockEntity,
                HostComparatorBlockEntity, HostConduitBlockEntity, HostContainerBlockEntity,
                HostCopperGolemStatueBlockEntity, HostCrafterBlockEntity,
                HostCreakingHeartBlockEntity, HostDaylightDetectorBlockEntity,
                HostDecoratedPotBlockEntity, HostDispenserBlockEntity, HostDropperBlockEntity,
                HostEnchantingTableBlockEntity, HostEndGatewayBlockEntity,
                HostEndPortalBlockEntity, HostEnderChestBlockEntity, HostFurnaceBlockEntity,
                HostHangingSignBlockEntity, HostHopperBlockEntity, HostJigsawBlockEntity,
                HostJukeboxBlockEntity, HostLecternBlockEntity, HostMapBlockEntity,
                HostMobSpawnerBlockEntity, HostPistonBlockEntity, HostPotentSulfurBlockEntity,
                HostSculkCatalystBlockEntity, HostSculkSensorBlockEntity,
                HostSculkShriekerBlockEntity, HostShelfBlockEntity, HostShulkerBoxBlockEntity,
                HostSignBlockEntity, HostSkullBlockEntity, HostSmokerBlockEntity,
                HostStructureBlockBlockEntity, HostTestBlockBlockEntity,
                HostTestInstanceBlockBlockEntity, HostTrappedChestBlockEntity,
                HostTrialSpawnerBlockEntity, HostVaultBlockEntity, JigsawBlockEntity,
                JukeboxBlockEntity, LecternBlockEntity, MapBlockEntity, MobSpawnerBlockEntity,
                PistonBlockEntity, PotentSulfurBlockEntity, SculkCatalystBlockEntity,
                SculkSensorBlockEntity, SculkShriekerBlockEntity, ShelfBlockEntity,
                ShulkerBoxBlockEntity, SignBlockEntity, SignText, SkullBlockEntity,
                SmokerBlockEntity, StructureBlockBlockEntity, TestBlockBlockEntity,
                TestInstanceBlockBlockEntity, TrappedChestBlockEntity, TrialSpawnerBlockEntity,
                VaultBlockEntity,
            },
            common::BlockPos as WitBlockPos,
            item_stack::ItemStack as WitHostItemStack,
        },
    },
};

impl pumpkin::plugin::block_entity::Host for PluginHostState {}

fn block_entity_from_resource(
    state: &PluginHostState,
    entity: &Resource<BlockEntity>,
) -> wasmtime::Result<Arc<dyn InternalBlockEntity>> {
    state
        .resource_table
        .get::<BlockEntityResource>(&Resource::new_own(entity.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid block entity resource handle"))
        .map(|resource| resource.provider.clone())
}

const fn from_wasm_dye_color(color: DyeColor) -> InternalDyeColor {
    match color {
        DyeColor::White => InternalDyeColor::White,
        DyeColor::Orange => InternalDyeColor::Orange,
        DyeColor::Magenta => InternalDyeColor::Magenta,
        DyeColor::LightBlue => InternalDyeColor::LightBlue,
        DyeColor::Yellow => InternalDyeColor::Yellow,
        DyeColor::Lime => InternalDyeColor::Lime,
        DyeColor::Pink => InternalDyeColor::Pink,
        DyeColor::Gray => InternalDyeColor::Gray,
        DyeColor::LightGray => InternalDyeColor::LightGray,
        DyeColor::Cyan => InternalDyeColor::Cyan,
        DyeColor::Purple => InternalDyeColor::Purple,
        DyeColor::Blue => InternalDyeColor::Blue,
        DyeColor::Brown => InternalDyeColor::Brown,
        DyeColor::Green => InternalDyeColor::Green,
        DyeColor::Red => InternalDyeColor::Red,
        DyeColor::Black => InternalDyeColor::Black,
    }
}

const fn to_wasm_dye_color(color: InternalDyeColor) -> DyeColor {
    match color {
        InternalDyeColor::White => DyeColor::White,
        InternalDyeColor::Orange => DyeColor::Orange,
        InternalDyeColor::Magenta => DyeColor::Magenta,
        InternalDyeColor::LightBlue => DyeColor::LightBlue,
        InternalDyeColor::Yellow => DyeColor::Yellow,
        InternalDyeColor::Lime => DyeColor::Lime,
        InternalDyeColor::Pink => DyeColor::Pink,
        InternalDyeColor::Gray => DyeColor::Gray,
        InternalDyeColor::LightGray => DyeColor::LightGray,
        InternalDyeColor::Cyan => DyeColor::Cyan,
        InternalDyeColor::Purple => DyeColor::Purple,
        InternalDyeColor::Blue => DyeColor::Blue,
        InternalDyeColor::Brown => DyeColor::Brown,
        InternalDyeColor::Green => DyeColor::Green,
        InternalDyeColor::Red => DyeColor::Red,
        InternalDyeColor::Black => DyeColor::Black,
    }
}

fn to_wasm_sign_text(text: &InternalText) -> SignText {
    SignText {
        messages: text
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .map(str::into_string)
            .to_vec(),
        color: to_wasm_dye_color(text.get_color()),
        has_glowing_text: text
            .has_glowing_text
            .load(std::sync::atomic::Ordering::Relaxed),
    }
}

fn from_wasm_sign_text(text: SignText) -> InternalText {
    let mut messages = [String::new(), String::new(), String::new(), String::new()];
    for (i, msg) in text.messages.into_iter().take(4).enumerate() {
        messages[i] = msg;
    }
    InternalText::from(pumpkin_nbt::tag::NbtTag::Compound({
        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        nbt.put_bool("has_glowing_text", text.has_glowing_text);
        nbt.put_string("color", from_wasm_dye_color(text.color).name().to_string());
        nbt.put_list(
            "messages",
            messages
                .iter()
                .map(|s| pumpkin_nbt::tag::NbtTag::String(s.clone().into()))
                .collect(),
        );
        nbt
    }))
}

impl HostBlockEntity for PluginHostState {
    async fn resource_location(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &res)?;
        Ok(entity.resource_location().to_string())
    }

    async fn get_position(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<WitBlockPos> {
        let entity = block_entity_from_resource(self, &res)?;
        let pos = entity.get_position();
        Ok(WitBlockPos {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
        })
    }

    async fn get_id(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &res)?;
        Ok(entity.get_id())
    }

    async fn is_dirty(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &res)?;
        Ok(entity.is_dirty())
    }

    async fn clear_dirty(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &res)?;
        entity.clear_dirty();
        Ok(())
    }

    async fn set_custom_data(
        &mut self,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
        value: super::common::WitNbtTree,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &res)?;
        let pos = entity.get_position();
        let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
        if let Some(server) = &self.server {
            for world in server.worlds.load().iter() {
                if world
                    .block_entities
                    .get(&pos.chunk_position())
                    .is_some_and(|m| m.contains_key(&pos))
                {
                    world.set_block_entity_custom_data(&pos, &namespace, &key, tag);
                    return Ok(());
                }
            }
            if let Some(world) = server.worlds.load().first() {
                world.set_block_entity_custom_data(&pos, &namespace, &key, tag);
            }
        }
        Ok(())
    }

    async fn get_custom_data(
        &mut self,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<super::common::WitNbtTree>> {
        let entity = block_entity_from_resource(self, &res)?;
        let pos = entity.get_position();
        if let Some(server) = &self.server {
            for world in server.worlds.load().iter() {
                if let Some(tag) = world.get_block_entity_custom_data(&pos, &namespace, &key) {
                    return Ok(Some(super::common::to_wit_nbt_tree(tag)));
                }
            }
        }
        Ok(None)
    }

    async fn remove_custom_data(
        &mut self,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &res)?;
        let pos = entity.get_position();
        if let Some(server) = &self.server {
            for world in server.worlds.load().iter() {
                world.remove_block_entity_custom_data(&pos, &namespace, &key);
            }
        }
        Ok(())
    }

    async fn has_custom_data(
        &mut self,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &res)?;
        let pos = entity.get_position();
        if let Some(server) = &self.server {
            for world in server.worlds.load().iter() {
                if world.has_block_entity_custom_data(&pos, &namespace, &key) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    async fn drop(&mut self, rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

fn get_container_from_be<T>(
    state: &mut PluginHostState,
    res: &Resource<T>,
) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
    let entity = block_entity_from_resource(state, &Resource::new_own(res.rep()))?;
    let provider = entity.clone();
    entity.get_inventory().map_or_else(
        || Err(wasmtime::Error::msg("Block entity inventory not available")),
        |inventory| state.add_container_block_entity(provider, inventory),
    )
}

impl HostContainerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let provider = container.provider.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_inventory(
        &mut self,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::inventory::Inventory,
        >,
    >{
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();
        self.add_inventory(
            crate::plugin::loader::wasm::wasm_host::state::InventoryProvider::Generic(inventory),
        )
    }

    async fn get_size(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<u32> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        Ok(container.provider.inventory.size() as u32)
    }

    async fn is_empty(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<bool> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();
        Ok(inventory.is_empty())
    }

    async fn get_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();
        let stack = inventory.get_stack(slot as usize);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(
                tokio::sync::Mutex::new(stack),
            ))?))
        }
    }

    async fn set_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
        stack_res: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();

        let stack = match stack_res {
            Some(res) => {
                let lock = self.get_item_stack(&res)?;
                lock.lock().await.clone()
            }
            None => pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
        };

        inventory.set_stack(slot as usize, stack);
        Ok(())
    }

    async fn remove_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();
        let removed = inventory.remove_stack(slot as usize);
        if removed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(
                tokio::sync::Mutex::new(removed),
            ))?))
        }
    }

    async fn clear(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<()> {
        let container = self
            .resource_table
            .get::<ContainerBlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid container block entity resource handle"))?;
        let inventory = container.provider.inventory.clone();
        inventory.clear();
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<ContainerBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContainerBlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostCommandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid command block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn last_output(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| {
                    Ok(cmd
                        .last_output
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone())
                },
            )
    }

    async fn track_output(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| Ok(cmd.track_output.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn success_count(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| Ok(cmd.success_count.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn command(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| {
                    Ok(cmd
                        .command
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone())
                },
            )
    }

    async fn auto(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| Ok(cmd.auto.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn condition_met(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| Ok(cmd.condition_met.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn powered(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalCommandBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a command block entity")),
                |cmd| Ok(cmd.powered.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn drop(&mut self, rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostSignBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid sign block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_front_text(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| Ok(to_wasm_sign_text(&sign.front_text)),
            )
    }

    async fn set_front_text(
        &mut self,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| {
                    let new_text = from_wasm_sign_text(text);
                    sign.front_text.has_glowing_text.store(
                        new_text
                            .has_glowing_text
                            .load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    sign.front_text.set_color(new_text.get_color());
                    (*sign
                        .front_text
                        .messages
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner))
                    .clone_from(
                        &new_text
                            .messages
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                    Ok(())
                },
            )
    }

    async fn get_back_text(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| Ok(to_wasm_sign_text(&sign.back_text)),
            )
    }

    async fn set_back_text(
        &mut self,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| {
                    let new_text = from_wasm_sign_text(text);
                    sign.back_text.has_glowing_text.store(
                        new_text
                            .has_glowing_text
                            .load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    sign.back_text.set_color(new_text.get_color());
                    (*sign
                        .back_text
                        .messages
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner))
                    .clone_from(
                        &new_text
                            .messages
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                    Ok(())
                },
            )
    }

    async fn is_waxed(&mut self, res: Resource<SignBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| Ok(sign.is_waxed.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn set_waxed(
        &mut self,
        res: Resource<SignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a sign block entity")),
                |sign| {
                    sign.is_waxed
                        .store(waxed, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                },
            )
    }

    async fn drop(&mut self, rep: Resource<SignBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostJukeboxBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid jukebox block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn is_playing(&mut self, res: Resource<JukeboxBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalJukeboxBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a jukebox block entity")),
                |jukebox| Ok(jukebox.is_playing()),
            )
    }

    async fn stop_playing(&mut self, res: Resource<JukeboxBlockEntity>) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalJukeboxBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a jukebox block entity")),
                |jukebox| {
                    jukebox.stop_playing();
                    Ok(())
                },
            )
    }

    async fn start_playing(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
        length_in_ticks: u64,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalJukeboxBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a jukebox block entity")),
                |jukebox| {
                    jukebox.start_playing(length_in_ticks);
                    Ok(())
                },
            )
    }

    async fn drop(&mut self, rep: Resource<JukeboxBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid chest block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn viewer_count(&mut self, res: Resource<ChestBlockEntity>) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalChestBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a chest block entity")),
                |chest| Ok(chest.get_viewer_count() as u32),
            )
    }

    async fn drop(&mut self, rep: Resource<ChestBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostMobSpawnerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid mob spawner block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_spawn_count(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalMobSpawnerBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a mob spawner block entity")),
                |spawner| Ok(spawner.spawn_count),
            )
    }

    async fn get_spawn_range(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalMobSpawnerBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a mob spawner block entity")),
                |spawner| Ok(spawner.spawn_range),
            )
    }

    async fn get_delay(&mut self, res: Resource<MobSpawnerBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalMobSpawnerBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a mob spawner block entity")),
                |spawner| Ok(spawner.delay.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn drop(&mut self, rep: Resource<MobSpawnerBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostMapBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid map block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_map_id(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a map block entity")),
                |map_be| Ok(map_be.get_map_id()),
            )
    }

    async fn set_map_id(
        &mut self,
        res: Resource<MapBlockEntity>,
        map_id: i32,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        map_be.set_map_id(map_id);
        Ok(())
    }

    async fn get_colors(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<Vec<u8>> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        Ok(map_be.get_colors())
    }

    async fn set_colors(
        &mut self,
        res: Resource<MapBlockEntity>,
        colors: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        map_be.set_colors(&colors);
        Ok(())
    }

    async fn set_pixel(
        &mut self,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
        color: u8,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        map_be.set_pixel(x as usize, y as usize, color);
        Ok(())
    }

    async fn get_pixel(
        &mut self,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
    ) -> wasmtime::Result<u8> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        Ok(map_be.get_pixel(x as usize, y as usize))
    }

    async fn update(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server context not set"))?;
        map_be.broadcast_map_data(server);
        Ok(())
    }

    async fn stream_frame(
        &mut self,
        res: Resource<MapBlockEntity>,
        frame_data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        let map_be = entity
            .as_any()
            .downcast_ref::<InternalMapBlockEntity>()
            .ok_or_else(|| wasmtime::Error::msg("Not a map block entity"))?;
        let server_opt = self.server.as_deref();
        map_be.stream_frame(&frame_data, server_opt);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<MapBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostHangingSignBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid hanging sign block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_front_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| Ok(to_wasm_sign_text(&sign.front_text)),
            )
    }

    async fn set_front_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| {
                    let new_text = from_wasm_sign_text(text);
                    sign.front_text.has_glowing_text.store(
                        new_text
                            .has_glowing_text
                            .load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    sign.front_text.set_color(new_text.get_color());
                    (*sign
                        .front_text
                        .messages
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner))
                    .clone_from(
                        &new_text
                            .messages
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                    Ok(())
                },
            )
    }

    async fn get_back_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| Ok(to_wasm_sign_text(&sign.back_text)),
            )
    }

    async fn set_back_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| {
                    let new_text = from_wasm_sign_text(text);
                    sign.back_text.has_glowing_text.store(
                        new_text
                            .has_glowing_text
                            .load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    sign.back_text.set_color(new_text.get_color());
                    (*sign
                        .back_text
                        .messages
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner))
                    .clone_from(
                        &new_text
                            .messages
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                    Ok(())
                },
            )
    }

    async fn is_waxed(&mut self, res: Resource<HangingSignBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| Ok(sign.is_waxed.load(std::sync::atomic::Ordering::Relaxed)),
            )
    }

    async fn set_waxed(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<InternalHangingSignBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a hanging sign block entity")),
                |sign| {
                    sign.is_waxed
                        .store(waxed, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                },
            )
    }

    async fn drop(&mut self, rep: Resource<HangingSignBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostTrappedChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid trapped chest block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn viewer_count(
        &mut self,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        entity
            .as_any()
            .downcast_ref::<crate::block::entities::trapped_chest::TrappedChestBlockEntity>()
            .map_or_else(
                || Err(wasmtime::Error::msg("Not a trapped chest block entity")),
                |chest| Ok(chest.get_viewer_count() as u32),
            )
    }

    async fn drop(&mut self, rep: Resource<TrappedChestBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

macro_rules! impl_basic_block_entity {
    ($trait_name:ident, $resource_name:ident, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn get_block_entity(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                let entity = self
                    .resource_table
                    .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
                    .map_err(|_| {
                        wasmtime::Error::msg(concat!("invalid ", $name_str, " resource handle"))
                    })?;
                let provider = entity.provider.clone();
                self.add_block_entity(provider)
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                let _ = self
                    .resource_table
                    .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
                Ok(())
            }
        }
    };
}

macro_rules! impl_container_basic_block_entity {
    ($trait_name:ident, $resource_name:ident, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn get_block_entity(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                let entity = self
                    .resource_table
                    .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
                    .map_err(|_| {
                        wasmtime::Error::msg(concat!("invalid ", $name_str, " resource handle"))
                    })?;
                let provider = entity.provider.clone();
                self.add_block_entity(provider)
            }

            async fn get_container(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
                get_container_from_be(self, &res)
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                let _ = self
                    .resource_table
                    .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
                Ok(())
            }
        }
    };
}

macro_rules! impl_cooking_host_block_entity {
    ($trait_name:ident, $resource_name:ident, $internal_type:ty, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn get_block_entity(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                let entity = self
                    .resource_table
                    .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
                    .map_err(|_| {
                        wasmtime::Error::msg(concat!("invalid ", $name_str, " resource handle"))
                    })?;
                let provider = entity.provider.clone();
                self.add_block_entity(provider)
            }

            async fn get_container(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
                get_container_from_be(self, &res)
            }

            async fn get_cooking_time_spent(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
                Ok(entity
                    .as_any()
                    .downcast_ref::<$internal_type>()
                    .map_or(0, |b| b.get_cooking_time_spent()))
            }

            async fn get_cooking_total_time(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
                Ok(entity
                    .as_any()
                    .downcast_ref::<$internal_type>()
                    .map_or(0, |b| b.get_cooking_total_time()))
            }

            async fn get_lit_time_remaining(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
                Ok(entity
                    .as_any()
                    .downcast_ref::<$internal_type>()
                    .map_or(0, |b| b.get_lit_time_remaining()))
            }

            async fn get_lit_total_time(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
                Ok(entity
                    .as_any()
                    .downcast_ref::<$internal_type>()
                    .map_or(0, |b| b.get_lit_total_time()))
            }

            async fn is_burning(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<bool> {
                let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
                Ok(entity
                    .as_any()
                    .downcast_ref::<$internal_type>()
                    .is_some_and(|b| b.is_burning()))
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                let _ = self
                    .resource_table
                    .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
                Ok(())
            }
        }
    };
}

impl_cooking_host_block_entity!(
    HostBlastingFurnaceBlockEntity,
    BlastingFurnaceBlockEntity,
    InternalBlastingFurnaceBlockEntity,
    "blasting furnace block entity"
);
impl_cooking_host_block_entity!(
    HostFurnaceBlockEntity,
    FurnaceBlockEntity,
    InternalFurnaceBlockEntity,
    "furnace block entity"
);
impl_cooking_host_block_entity!(
    HostSmokerBlockEntity,
    SmokerBlockEntity,
    InternalSmokerBlockEntity,
    "smoker block entity"
);

impl HostBannerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BannerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid banner block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_custom_name(
        &mut self,
        res: Resource<BannerBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBannerBlockEntity>()
            .and_then(|b| b.custom_name.try_lock().ok().and_then(|g| g.clone())))
    }

    async fn drop(&mut self, rep: Resource<BannerBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBarrelBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid barrel block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn viewer_count(&mut self, _res: Resource<BarrelBlockEntity>) -> wasmtime::Result<u32> {
        Ok(0)
    }

    async fn drop(&mut self, rep: Resource<BarrelBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBeaconBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid beacon block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_primary_effect(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBeaconBlockEntity>()
            .map_or(-1, |b| {
                b.primary_effect.load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn get_secondary_effect(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBeaconBlockEntity>()
            .map_or(-1, |b| {
                b.secondary_effect
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn get_levels(&mut self, res: Resource<BeaconBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBeaconBlockEntity>()
            .map_or(0, |b| b.levels.load(std::sync::atomic::Ordering::Relaxed)))
    }

    async fn drop(&mut self, rep: Resource<BeaconBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBeehiveBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BeehiveBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid beehive block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_bee_count(&mut self, res: Resource<BeehiveBlockEntity>) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBeehiveBlockEntity>()
            .map_or(0, |b| {
                b.bees
                    .try_lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|v| v.len() as u32))
                    .unwrap_or(0)
            }))
    }

    async fn drop(&mut self, rep: Resource<BeehiveBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBellBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BellBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid bell block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn is_ringing(&mut self, res: Resource<BellBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBellBlockEntity>()
            .is_some_and(|b| b.ringing.load()))
    }

    async fn get_ring_ticks(&mut self, res: Resource<BellBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBellBlockEntity>()
            .map_or(0, |b| b.ring_ticks.load()))
    }

    async fn drop(&mut self, rep: Resource<BellBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBrewingStandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid brewing stand block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_brew_time(
        &mut self,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBrewingStandBlockEntity>()
            .map_or(0, |b| {
                b.brew_time.load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn get_fuel(&mut self, res: Resource<BrewingStandBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalBrewingStandBlockEntity>()
            .map_or(0, |b| b.fuel.load(std::sync::atomic::Ordering::Relaxed)))
    }

    async fn drop(&mut self, rep: Resource<BrewingStandBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostChiseledBookshelfBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid chiseled bookshelf block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_last_interacted_slot(
        &mut self,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<i8> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalChiseledBookshelfBlockEntity>()
            .map_or(-1, |b| {
                b.last_interacted_slot
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn drop(&mut self, rep: Resource<ChiseledBookshelfBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostComparatorBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid comparator block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_output_signal(
        &mut self,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<u8> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalComparatorBlockEntity>()
            .map_or(0, |b| {
                b.output_signal.load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn drop(&mut self, rep: Resource<ComparatorBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostCrafterBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid crafter block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_crafting_ticks_remaining(
        &mut self,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalCrafterBlockEntity>()
            .map_or(0, |b| {
                b.crafting_ticks_remaining
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn is_triggered(&mut self, res: Resource<CrafterBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalCrafterBlockEntity>()
            .is_some_and(|b| b.triggered.load(std::sync::atomic::Ordering::Relaxed)))
    }

    async fn drop(&mut self, rep: Resource<CrafterBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostCreakingHeartBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid creaking heart block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_creaking_uuid(
        &mut self,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalCreakingHeartBlockEntity>()
            .and_then(|b| b.creaking_uuid.load().map(|u| u.to_string())))
    }

    async fn drop(&mut self, rep: Resource<CreakingHeartBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostEndGatewayBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid end gateway block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_age(&mut self, res: Resource<EndGatewayBlockEntity>) -> wasmtime::Result<i64> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalEndGatewayBlockEntity>()
            .map_or(0, |b| b.age.try_lock().ok().map_or(0, |g| *g)))
    }

    async fn is_exact_teleport(
        &mut self,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalEndGatewayBlockEntity>()
            .is_some_and(|b| b.exact_teleport.try_lock().is_ok_and(|g| *g)))
    }

    async fn drop(&mut self, rep: Resource<EndGatewayBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostEnderChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid ender chest block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn viewer_count(
        &mut self,
        _res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }

    async fn drop(&mut self, rep: Resource<EnderChestBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostShulkerBoxBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid shulker box block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn viewer_count(
        &mut self,
        _res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }

    async fn drop(&mut self, rep: Resource<ShulkerBoxBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostHopperBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid hopper block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_cooldown(&mut self, res: Resource<HopperBlockEntity>) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalHopperBlockEntity>()
            .map_or(0, |b| {
                b.cooldown_time.load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn drop(&mut self, rep: Resource<HopperBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostJigsawBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid jigsaw block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_name(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or_else(String::new, |b| {
                b.name
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_target(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or_else(String::new, |b| {
                b.target
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_pool(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or_else(String::new, |b| {
                b.pool
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_final_state(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or_else(String::new, |b| {
                b.final_state
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_selection_priority(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or(0, |b| {
                b.selection_priority
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn get_placement_priority(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalJigsawBlockEntity>()
            .map_or(0, |b| {
                b.placement_priority
                    .load(std::sync::atomic::Ordering::Relaxed)
            }))
    }

    async fn drop(&mut self, rep: Resource<JigsawBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostLecternBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid lectern block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_container(
        &mut self,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_page(&mut self, res: Resource<LecternBlockEntity>) -> wasmtime::Result<u32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalLecternBlockEntity>()
            .map_or(0, |b| {
                b.page.load(std::sync::atomic::Ordering::Relaxed) as u32
            }))
    }

    async fn drop(&mut self, rep: Resource<LecternBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostPistonBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid piston block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_progress(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<f32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalPistonBlockEntity>()
            .map_or(0.0, |b| b.current_progress.load()))
    }

    async fn is_extending(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalPistonBlockEntity>()
            .is_some_and(|b| b.extending))
    }

    async fn is_source(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<bool> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalPistonBlockEntity>()
            .is_some_and(|b| b.source))
    }

    async fn drop(&mut self, rep: Resource<PistonBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostSculkShriekerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid sculk shrieker block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_warning_level(
        &mut self,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalSculkShriekerBlockEntity>()
            .map_or(0, |b| b.warning_level.try_lock().ok().map_or(0, |g| *g)))
    }

    async fn drop(&mut self, rep: Resource<SculkShriekerBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostSkullBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid skull block entity resource handle"))?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_note_block_sound(
        &mut self,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalSkullBlockEntity>()
            .and_then(|b| b.note_block_sound.try_lock().ok().and_then(|g| g.clone())))
    }

    async fn drop(&mut self, rep: Resource<SkullBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostStructureBlockBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        let entity = self
            .resource_table
            .get::<BlockEntityResource>(&Resource::new_own(res.rep()))
            .map_err(|_| {
                wasmtime::Error::msg("invalid structure block block entity resource handle")
            })?;
        let provider = entity.provider.clone();
        self.add_block_entity(provider)
    }

    async fn get_name(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalStructureBlockBlockEntity>()
            .map_or_else(String::new, |b| {
                b.name
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_author(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalStructureBlockBlockEntity>()
            .map_or_else(String::new, |b| {
                b.author
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_mode(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalStructureBlockBlockEntity>()
            .map_or_else(String::new, |b| {
                b.mode
                    .try_lock()
                    .ok()
                    .map_or_else(String::new, |g| g.clone())
            }))
    }

    async fn get_integrity(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<f32> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalStructureBlockBlockEntity>()
            .map_or(1.0, |b| b.integrity.try_lock().ok().map_or(1.0, |g| *g)))
    }

    async fn get_seed(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<i64> {
        let entity = block_entity_from_resource(self, &Resource::new_own(res.rep()))?;
        Ok(entity
            .as_any()
            .downcast_ref::<InternalStructureBlockBlockEntity>()
            .map_or(0, |b| b.seed.try_lock().ok().map_or(0, |g| *g)))
    }

    async fn drop(&mut self, rep: Resource<StructureBlockBlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl_basic_block_entity!(HostBedBlockEntity, BedBlockEntity, "bed block entity");
impl_basic_block_entity!(
    HostBrushableBlockBlockEntity,
    BrushableBlockBlockEntity,
    "brushable block block entity"
);
impl_basic_block_entity!(
    HostCalibratedSculkSensorBlockEntity,
    CalibratedSculkSensorBlockEntity,
    "calibrated sculk sensor block entity"
);
impl_container_basic_block_entity!(
    HostCampfireBlockEntity,
    CampfireBlockEntity,
    "campfire block entity"
);
impl_basic_block_entity!(
    HostConduitBlockEntity,
    ConduitBlockEntity,
    "conduit block entity"
);
impl_basic_block_entity!(
    HostCopperGolemStatueBlockEntity,
    CopperGolemStatueBlockEntity,
    "copper golem statue block entity"
);
impl_basic_block_entity!(
    HostDaylightDetectorBlockEntity,
    DaylightDetectorBlockEntity,
    "daylight detector block entity"
);
impl_basic_block_entity!(
    HostDecoratedPotBlockEntity,
    DecoratedPotBlockEntity,
    "decorated pot block entity"
);
impl_container_basic_block_entity!(
    HostDispenserBlockEntity,
    DispenserBlockEntity,
    "dispenser block entity"
);
impl_container_basic_block_entity!(
    HostDropperBlockEntity,
    DropperBlockEntity,
    "dropper block entity"
);
impl_basic_block_entity!(
    HostEnchantingTableBlockEntity,
    EnchantingTableBlockEntity,
    "enchanting table block entity"
);
impl_basic_block_entity!(
    HostEndPortalBlockEntity,
    EndPortalBlockEntity,
    "end portal block entity"
);
impl_basic_block_entity!(
    HostPotentSulfurBlockEntity,
    PotentSulfurBlockEntity,
    "potent sulfur block entity"
);
impl_basic_block_entity!(
    HostSculkCatalystBlockEntity,
    SculkCatalystBlockEntity,
    "sculk catalyst block entity"
);
impl_basic_block_entity!(
    HostSculkSensorBlockEntity,
    SculkSensorBlockEntity,
    "sculk sensor block entity"
);
impl_container_basic_block_entity!(HostShelfBlockEntity, ShelfBlockEntity, "shelf block entity");
impl_basic_block_entity!(
    HostTestBlockBlockEntity,
    TestBlockBlockEntity,
    "test block block entity"
);
impl_basic_block_entity!(
    HostTestInstanceBlockBlockEntity,
    TestInstanceBlockBlockEntity,
    "test instance block block entity"
);
impl_basic_block_entity!(
    HostTrialSpawnerBlockEntity,
    TrialSpawnerBlockEntity,
    "trial spawner block entity"
);
impl_basic_block_entity!(HostVaultBlockEntity, VaultBlockEntity, "vault block entity");
