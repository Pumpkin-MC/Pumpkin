use std::sync::Arc;
use std::sync::atomic::Ordering;
use wasmtime::component::Resource;

use crate::block::entities::BlockEntity as InternalBlockEntity;
use crate::block::entities::furnace_like_block_entity::CookingBlockEntityBase;
use crate::block::entities::sign::{DyeColor as InternalDyeColor, Text as InternalText};
use crate::plugin::loader::wasm::wasm_host::state::{self, FromResource};
use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
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
        has_glowing_text: text.has_glowing_text.load(Ordering::Relaxed),
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
        let entity = self.get(&res)?;
        Ok(entity.resource_location().to_string())
    }

    async fn get_position(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<WitBlockPos> {
        let entity = self.get(&res)?;
        let pos = entity.get_position();
        Ok(WitBlockPos {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
        })
    }

    async fn get_id(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<u32> {
        let entity = self.get(&res)?;
        Ok(entity.get_id())
    }

    async fn is_dirty(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<bool> {
        let entity = self.get(&res)?;
        Ok(entity.is_dirty())
    }

    async fn clear_dirty(&mut self, res: Resource<BlockEntity>) -> wasmtime::Result<()> {
        let entity = self.get(&res)?;
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
        let entity = self.get(&res)?;
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
        let entity = self.get(&res)?;
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
        let entity = self.get(&res)?;
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
        let entity = self.get(&res)?;
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
        self.drop(rep)
    }
}

fn get_container_from_be(
    state: &mut PluginHostState,
    res: &Resource<impl FromResource<Internal = Arc<impl InternalBlockEntity>>>,
) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
    let entity = state.get(res)?.clone();
    let provider = entity.clone();
    entity.get_inventory().map_or_else(
        || Err(wasmtime::Error::msg("Block entity inventory not available")),
        |inventory| {
            state.add(state::ContainerBlockEntity {
                provider,
                inventory,
            })
        },
    )
}

impl HostContainerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.provider.clone())
    }

    async fn get_inventory(
        &mut self,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::inventory::Inventory,
        >,
    >{
        let inventory = self.get(&res)?.inventory.clone();
        self.add(
            crate::plugin::loader::wasm::wasm_host::state::InventoryProvider::Generic(inventory),
        )
    }

    async fn get_size(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<u32> {
        Ok(self.get(&res)?.inventory.size() as u32)
    }

    async fn is_empty(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.inventory.is_empty())
    }

    async fn get_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let inventory = self.get(&res)?.inventory.clone();
        let stack = inventory.get_stack(slot as usize);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(stack)))?))
        }
    }

    async fn set_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
        stack_res: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = match stack_res {
            Some(res) => self.take(res)?.lock().await.clone(),
            None => pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
        };
        let inventory = self.get(&res)?.inventory.clone();

        inventory.set_stack(slot as usize, stack);
        Ok(())
    }

    async fn remove_stack(
        &mut self,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let inventory = self.get(&res)?.inventory.clone();
        let removed = inventory.remove_stack(slot as usize);
        if removed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(removed)))?))
        }
    }

    async fn clear(&mut self, res: Resource<ContainerBlockEntity>) -> wasmtime::Result<()> {
        let inventory = self.get(&res)?.inventory.clone();
        inventory.clear();
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<ContainerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCommandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn last_output(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .last_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }

    async fn track_output(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.track_output.load(Ordering::Relaxed))
    }

    async fn success_count(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<u32> {
        Ok(self.get(&res)?.success_count.load(Ordering::Relaxed))
    }

    async fn command(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }

    async fn auto(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.auto.load(Ordering::Relaxed))
    }

    async fn condition_met(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.condition_met.load(Ordering::Relaxed))
    }

    async fn powered(&mut self, res: Resource<CommandBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.powered.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSignBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_front_text(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        Ok(to_wasm_sign_text(&self.get(&res)?.front_text))
    }

    async fn set_front_text(
        &mut self,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let sign = self.get(&res)?;
        let new_text = from_wasm_sign_text(text);
        sign.front_text.has_glowing_text.store(
            new_text.has_glowing_text.load(Ordering::Relaxed),
            Ordering::Relaxed,
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
    }

    async fn get_back_text(
        &mut self,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        Ok(to_wasm_sign_text(&self.get(&res)?.back_text))
    }

    async fn set_back_text(
        &mut self,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let sign = self.get(&res)?;
        let new_text = from_wasm_sign_text(text);
        sign.back_text.has_glowing_text.store(
            new_text.has_glowing_text.load(Ordering::Relaxed),
            Ordering::Relaxed,
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
    }

    async fn is_waxed(&mut self, res: Resource<SignBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.is_waxed.load(Ordering::Relaxed))
    }

    async fn set_waxed(
        &mut self,
        res: Resource<SignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.is_waxed.store(waxed, Ordering::Relaxed);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<SignBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostJukeboxBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_container(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn is_playing(&mut self, res: Resource<JukeboxBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.is_playing())
    }

    async fn stop_playing(&mut self, res: Resource<JukeboxBlockEntity>) -> wasmtime::Result<()> {
        self.get(&res)?.stop_playing();
        Ok(())
    }

    async fn start_playing(
        &mut self,
        res: Resource<JukeboxBlockEntity>,
        length_in_ticks: u64,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.start_playing(length_in_ticks);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<JukeboxBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_container(
        &mut self,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn viewer_count(&mut self, res: Resource<ChestBlockEntity>) -> wasmtime::Result<u32> {
        Ok(self.get(&res)?.get_viewer_count() as u32)
    }

    async fn drop(&mut self, rep: Resource<ChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostMobSpawnerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_spawn_count(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.spawn_count)
    }

    async fn get_spawn_range(
        &mut self,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.spawn_range)
    }

    async fn get_delay(&mut self, res: Resource<MobSpawnerBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.delay.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<MobSpawnerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostMapBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_map_id(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.get_map_id())
    }

    async fn set_map_id(
        &mut self,
        res: Resource<MapBlockEntity>,
        map_id: i32,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.set_map_id(map_id);
        Ok(())
    }

    async fn get_colors(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<Vec<u8>> {
        Ok(self.get(&res)?.get_colors())
    }

    async fn set_colors(
        &mut self,
        res: Resource<MapBlockEntity>,
        colors: Vec<u8>,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.set_colors(&colors);
        Ok(())
    }

    async fn set_pixel(
        &mut self,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
        color: u8,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.set_pixel(x as usize, y as usize, color);
        Ok(())
    }

    async fn get_pixel(
        &mut self,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
    ) -> wasmtime::Result<u8> {
        Ok(self.get(&res)?.get_pixel(x as usize, y as usize))
    }

    async fn update(&mut self, res: Resource<MapBlockEntity>) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server context not set"))?;
        self.get(&res)?.broadcast_map_data(server);
        Ok(())
    }

    async fn stream_frame(
        &mut self,
        res: Resource<MapBlockEntity>,
        frame_data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let server_opt = self.server.as_deref();
        self.get(&res)?.stream_frame(&frame_data, server_opt);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<MapBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostHangingSignBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_front_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        Ok(to_wasm_sign_text(&self.get(&res)?.front_text))
    }

    async fn set_front_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let sign = self.get(&res)?;
        let new_text = from_wasm_sign_text(text);
        sign.front_text.has_glowing_text.store(
            new_text.has_glowing_text.load(Ordering::Relaxed),
            Ordering::Relaxed,
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
    }

    async fn get_back_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        Ok(to_wasm_sign_text(&self.get(&res)?.back_text))
    }

    async fn set_back_text(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        let sign = self.get(&res)?;
        let new_text = from_wasm_sign_text(text);
        sign.back_text.has_glowing_text.store(
            new_text.has_glowing_text.load(Ordering::Relaxed),
            Ordering::Relaxed,
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
    }

    async fn is_waxed(&mut self, res: Resource<HangingSignBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.is_waxed.load(Ordering::Relaxed))
    }

    async fn set_waxed(
        &mut self,
        res: Resource<HangingSignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        self.get(&res)?.is_waxed.store(waxed, Ordering::Relaxed);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<HangingSignBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostTrappedChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        Ok(self.get(&res)?.get_viewer_count() as u32)
    }

    async fn drop(&mut self, rep: Resource<TrappedChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

macro_rules! impl_basic_block_entity {
    ($trait_name:ident, $resource_name:ident, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn get_block_entity(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                self.add(self.get(&res)?.clone() as _)
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
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
                self.add(self.get(&res)?.clone() as _)
            }

            async fn get_container(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
                get_container_from_be(self, &res)
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
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
                self.add(self.get(&res)?.clone() as _)
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
                Ok(self.get(&res)?.get_cooking_time_spent())
            }

            async fn get_cooking_total_time(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                Ok(self.get(&res)?.get_cooking_total_time())
            }

            async fn get_lit_time_remaining(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                Ok(self.get(&res)?.get_lit_time_remaining())
            }

            async fn get_lit_total_time(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                Ok(self.get(&res)?.get_lit_total_time())
            }

            async fn is_burning(
                &mut self,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<bool> {
                Ok(self.get(&res)?.is_burning())
            }

            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
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
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_custom_name(
        &mut self,
        res: Resource<BannerBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        Ok(self
            .get(&res)?
            .custom_name
            .try_lock()
            .ok()
            .and_then(|g| g.clone()))
    }

    async fn drop(&mut self, rep: Resource<BannerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBarrelBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        self.drop(rep)
    }
}

impl HostBeaconBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        Ok(self.get(&res)?.primary_effect.load(Ordering::Relaxed))
    }

    async fn get_secondary_effect(
        &mut self,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.secondary_effect.load(Ordering::Relaxed))
    }

    async fn get_levels(&mut self, res: Resource<BeaconBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.levels.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<BeaconBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBeehiveBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BeehiveBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_bee_count(&mut self, res: Resource<BeehiveBlockEntity>) -> wasmtime::Result<u32> {
        Ok(self
            .get(&res)?
            .bees
            .try_lock()
            .ok()
            .and_then(|g| g.as_ref().map(|v| v.len() as u32))
            .unwrap_or(0))
    }

    async fn drop(&mut self, rep: Resource<BeehiveBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBellBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BellBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn is_ringing(&mut self, res: Resource<BellBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.ringing.load())
    }

    async fn get_ring_ticks(&mut self, res: Resource<BellBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.ring_ticks.load())
    }

    async fn drop(&mut self, rep: Resource<BellBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBrewingStandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        Ok(self.get(&res)?.brew_time.load(Ordering::Relaxed))
    }

    async fn get_fuel(&mut self, res: Resource<BrewingStandBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.fuel.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<BrewingStandBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostChiseledBookshelfBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        Ok(self.get(&res)?.last_interacted_slot.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<ChiseledBookshelfBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostComparatorBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_output_signal(
        &mut self,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<u8> {
        Ok(self.get(&res)?.output_signal.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<ComparatorBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCrafterBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        Ok(self
            .get(&res)?
            .crafting_ticks_remaining
            .load(Ordering::Relaxed))
    }

    async fn is_triggered(&mut self, res: Resource<CrafterBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.triggered.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<CrafterBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCreakingHeartBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_creaking_uuid(
        &mut self,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        Ok(self.get(&res)?.creaking_uuid.load().map(|u| u.to_string()))
    }

    async fn drop(&mut self, rep: Resource<CreakingHeartBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostEndGatewayBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_age(&mut self, res: Resource<EndGatewayBlockEntity>) -> wasmtime::Result<i64> {
        Ok(self.get(&res)?.age.try_lock().ok().map_or(0, |g| *g))
    }

    async fn is_exact_teleport(
        &mut self,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.exact_teleport.try_lock().is_ok_and(|g| *g))
    }

    async fn drop(&mut self, rep: Resource<EndGatewayBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostEnderChestBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn viewer_count(
        &mut self,
        _res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }

    async fn drop(&mut self, rep: Resource<EnderChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostShulkerBoxBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
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
        self.drop(rep)
    }
}

impl HostHopperBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_container(
        &mut self,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_cooldown(&mut self, res: Resource<HopperBlockEntity>) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.cooldown_time.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<HopperBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostJigsawBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_name(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .name
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_target(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .target
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_pool(&mut self, res: Resource<JigsawBlockEntity>) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .pool
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_final_state(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .final_state
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_selection_priority(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.selection_priority.load(Ordering::Relaxed))
    }

    async fn get_placement_priority(
        &mut self,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.placement_priority.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<JigsawBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostLecternBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_container(
        &mut self,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        get_container_from_be(self, &res)
    }

    async fn get_page(&mut self, res: Resource<LecternBlockEntity>) -> wasmtime::Result<u32> {
        Ok(self.get(&res)?.page.load(Ordering::Relaxed) as u32)
    }

    async fn drop(&mut self, rep: Resource<LecternBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostPistonBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_progress(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<f32> {
        Ok(self.get(&res)?.current_progress.load())
    }

    async fn is_extending(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.extending)
    }

    async fn is_source(&mut self, res: Resource<PistonBlockEntity>) -> wasmtime::Result<bool> {
        Ok(self.get(&res)?.source)
    }

    async fn drop(&mut self, rep: Resource<PistonBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSculkShriekerBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_warning_level(
        &mut self,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        Ok(self.get(&res)?.warning_level.try_lock().map_or(0, |g| *g))
    }

    async fn drop(&mut self, rep: Resource<SculkShriekerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSkullBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_note_block_sound(
        &mut self,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        Ok(self
            .get(&res)?
            .note_block_sound
            .try_lock()
            .ok()
            .and_then(|g| g.clone()))
    }

    async fn drop(&mut self, rep: Resource<SkullBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostStructureBlockBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        self.add(self.get(&res)?.clone() as _)
    }

    async fn get_name(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .name
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_author(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .author
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_mode(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        Ok(self
            .get(&res)?
            .mode
            .try_lock()
            .ok()
            .map_or_else(String::new, |g| g.clone()))
    }

    async fn get_integrity(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<f32> {
        Ok(self.get(&res)?.integrity.try_lock().map_or(1.0, |g| *g))
    }

    async fn get_seed(
        &mut self,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<i64> {
        Ok(self.get(&res)?.seed.try_lock().map_or(0, |g| *g))
    }

    async fn drop(&mut self, rep: Resource<StructureBlockBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
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
