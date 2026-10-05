use super::AccessorExt;
use crate::pumpkin::{
    self,
    plugin::{
        block_entity::{
            BannerBlockEntity, BarrelBlockEntity, BeaconBlockEntity, BedBlockEntity,
            BeehiveBlockEntity, BellBlockEntity, BlastingFurnaceBlockEntity, BlockEntity,
            BrewingStandBlockEntity, BrushableBlockBlockEntity, CalibratedSculkSensorBlockEntity,
            CampfireBlockEntity, ChestBlockEntity, ChiseledBookshelfBlockEntity,
            CommandBlockEntity, ComparatorBlockEntity, ConduitBlockEntity, ContainerBlockEntity,
            CopperGolemStatueBlockEntity, CrafterBlockEntity, CreakingHeartBlockEntity,
            DaylightDetectorBlockEntity, DecoratedPotBlockEntity, DispenserBlockEntity,
            DropperBlockEntity, DyeColor, EnchantingTableBlockEntity, EndGatewayBlockEntity,
            EndPortalBlockEntity, EnderChestBlockEntity, FurnaceBlockEntity,
            HangingSignBlockEntity, HopperBlockEntity, HostBannerBlockEntity,
            HostBannerBlockEntityWithStore, HostBarrelBlockEntity, HostBarrelBlockEntityWithStore,
            HostBeaconBlockEntity, HostBeaconBlockEntityWithStore, HostBedBlockEntity,
            HostBedBlockEntityWithStore, HostBeehiveBlockEntity, HostBeehiveBlockEntityWithStore,
            HostBellBlockEntity, HostBellBlockEntityWithStore, HostBlastingFurnaceBlockEntity,
            HostBlastingFurnaceBlockEntityWithStore, HostBlockEntity, HostBlockEntityWithStore,
            HostBrewingStandBlockEntity, HostBrewingStandBlockEntityWithStore,
            HostBrushableBlockBlockEntity, HostBrushableBlockBlockEntityWithStore,
            HostCalibratedSculkSensorBlockEntity, HostCalibratedSculkSensorBlockEntityWithStore,
            HostCampfireBlockEntity, HostCampfireBlockEntityWithStore, HostChestBlockEntity,
            HostChestBlockEntityWithStore, HostChiseledBookshelfBlockEntity,
            HostChiseledBookshelfBlockEntityWithStore, HostCommandBlockEntity,
            HostCommandBlockEntityWithStore, HostComparatorBlockEntity,
            HostComparatorBlockEntityWithStore, HostConduitBlockEntity,
            HostConduitBlockEntityWithStore, HostContainerBlockEntity,
            HostContainerBlockEntityWithStore, HostCopperGolemStatueBlockEntity,
            HostCopperGolemStatueBlockEntityWithStore, HostCrafterBlockEntity,
            HostCrafterBlockEntityWithStore, HostCreakingHeartBlockEntity,
            HostCreakingHeartBlockEntityWithStore, HostDaylightDetectorBlockEntity,
            HostDaylightDetectorBlockEntityWithStore, HostDecoratedPotBlockEntity,
            HostDecoratedPotBlockEntityWithStore, HostDispenserBlockEntity,
            HostDispenserBlockEntityWithStore, HostDropperBlockEntity,
            HostDropperBlockEntityWithStore, HostEnchantingTableBlockEntity,
            HostEnchantingTableBlockEntityWithStore, HostEndGatewayBlockEntity,
            HostEndGatewayBlockEntityWithStore, HostEndPortalBlockEntity,
            HostEndPortalBlockEntityWithStore, HostEnderChestBlockEntity,
            HostEnderChestBlockEntityWithStore, HostFurnaceBlockEntity,
            HostFurnaceBlockEntityWithStore, HostHangingSignBlockEntity,
            HostHangingSignBlockEntityWithStore, HostHopperBlockEntity,
            HostHopperBlockEntityWithStore, HostJigsawBlockEntity, HostJigsawBlockEntityWithStore,
            HostJukeboxBlockEntity, HostJukeboxBlockEntityWithStore, HostLecternBlockEntity,
            HostLecternBlockEntityWithStore, HostMapBlockEntity, HostMapBlockEntityWithStore,
            HostMobSpawnerBlockEntity, HostMobSpawnerBlockEntityWithStore, HostPistonBlockEntity,
            HostPistonBlockEntityWithStore, HostPotentSulfurBlockEntity,
            HostPotentSulfurBlockEntityWithStore, HostSculkCatalystBlockEntity,
            HostSculkCatalystBlockEntityWithStore, HostSculkSensorBlockEntity,
            HostSculkSensorBlockEntityWithStore, HostSculkShriekerBlockEntity,
            HostSculkShriekerBlockEntityWithStore, HostShelfBlockEntity,
            HostShelfBlockEntityWithStore, HostShulkerBoxBlockEntity,
            HostShulkerBoxBlockEntityWithStore, HostSignBlockEntity, HostSignBlockEntityWithStore,
            HostSkullBlockEntity, HostSkullBlockEntityWithStore, HostSmokerBlockEntity,
            HostSmokerBlockEntityWithStore, HostStructureBlockBlockEntity,
            HostStructureBlockBlockEntityWithStore, HostTestBlockBlockEntity,
            HostTestBlockBlockEntityWithStore, HostTestInstanceBlockBlockEntity,
            HostTestInstanceBlockBlockEntityWithStore, HostTrappedChestBlockEntity,
            HostTrappedChestBlockEntityWithStore, HostTrialSpawnerBlockEntity,
            HostTrialSpawnerBlockEntityWithStore, HostVaultBlockEntity,
            HostVaultBlockEntityWithStore, JigsawBlockEntity, JukeboxBlockEntity,
            LecternBlockEntity, MapBlockEntity, MobSpawnerBlockEntity, PistonBlockEntity,
            PotentSulfurBlockEntity, SculkCatalystBlockEntity, SculkSensorBlockEntity,
            SculkShriekerBlockEntity, ShelfBlockEntity, ShulkerBoxBlockEntity, SignBlockEntity,
            SignText, SkullBlockEntity, SmokerBlockEntity, StructureBlockBlockEntity,
            TestBlockBlockEntity, TestInstanceBlockBlockEntity, TrappedChestBlockEntity,
            TrialSpawnerBlockEntity, VaultBlockEntity,
        },
        common::BlockPos as WitBlockPos,
        item_stack::ItemStack as WitHostItemStack,
    },
};
use pumpkin_core::block::entities::{
    BlockEntity as InternalBlockEntity,
    furnace_like_block_entity::CookingBlockEntityBase,
    sign::{DyeColor as InternalDyeColor, Text as InternalText},
};
use pumpkin_wasm_host_common::state::{self, FromResource, PluginHostState};
use std::sync::{Arc, atomic::Ordering};
use wasmtime::component::{Accessor, HasSelf, Resource};

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
    async fn drop(&mut self, rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn resource_location(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            Ok(entity.resource_location().to_string())
        })
    }

    async fn get_position(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
    ) -> wasmtime::Result<WitBlockPos> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            let pos = entity.get_position();
            Ok(WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            })
        })
    }

    async fn get_id(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            Ok(entity.get_id())
        })
    }

    async fn is_dirty(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            Ok(entity.is_dirty())
        })
    }

    async fn clear_dirty(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            entity.clear_dirty();
            Ok(())
        })
    }

    async fn set_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
        value: super::common::WitNbtTree,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            let pos = entity.get_position();
            let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
            if let Some(server) = &state.server {
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
        })
    }

    async fn get_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<super::common::WitNbtTree>> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            let pos = entity.get_position();
            if let Some(server) = &state.server {
                for world in server.worlds.load().iter() {
                    if let Some(tag) = world.get_block_entity_custom_data(&pos, &namespace, &key) {
                        return Ok(Some(super::common::to_wit_nbt_tree(tag)));
                    }
                }
            }
            Ok(None)
        })
    }

    async fn remove_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            let pos = entity.get_position();
            if let Some(server) = &state.server {
                for world in server.worlds.load().iter() {
                    world.remove_block_entity_custom_data(&pos, &namespace, &key);
                }
            }
            Ok(())
        })
    }

    async fn has_custom_data(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BlockEntity>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let entity = state.get(&res)?;
            let pos = entity.get_position();
            if let Some(server) = &state.server {
                for world in server.worlds.load().iter() {
                    if world.has_block_entity_custom_data(&pos, &namespace, &key) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        })
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
    async fn drop(&mut self, rep: Resource<ContainerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostContainerBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.provider.clone())
        })
    }

    async fn get_inventory(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<Resource<crate::pumpkin::plugin::inventory::Inventory>> {
        accessor.with(|mut host| {
            let st = host.get();
            let inventory = st.get(&res)?.inventory.clone();
            st.add(pumpkin_wasm_host_common::state::InventoryProvider::Generic(
                inventory,
            ))
        })
    }

    async fn get_size(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.inventory.size() as u32)
        })
    }

    async fn is_empty(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.inventory.is_empty())
        })
    }

    async fn get_stack(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let inventory = state.get(&res)?.inventory.clone();
            let stack = inventory.get_stack(slot as usize);
            if stack.is_empty() {
                Ok(None)
            } else {
                Ok(Some(state.add(Arc::new(tokio::sync::Mutex::new(stack)))?))
            }
        })
    }

    async fn set_stack(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
        stack_res: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = match stack_res {
            Some(res) => accessor.take_res(res)?.lock().await.clone(),
            None => pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
        };
        let inventory = accessor.get_res(&res)?.inventory.clone();

        inventory.set_stack(slot as usize, stack);
        Ok(())
    }

    async fn remove_stack(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        accessor.with(|mut host| {
            let state = host.get();
            let inventory = state.get(&res)?.inventory.clone();
            let removed = inventory.remove_stack(slot as usize);
            if removed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(state.add(Arc::new(tokio::sync::Mutex::new(removed)))?))
            }
        })
    }

    async fn clear(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ContainerBlockEntity>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let inventory = state.get(&res)?.inventory.clone();
            inventory.clear();
            Ok(())
        })
    }
}

impl HostCommandBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCommandBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn last_output(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .last_output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone())
        })
    }

    async fn track_output(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.track_output.load(Ordering::Relaxed))
        })
    }

    async fn success_count(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.success_count.load(Ordering::Relaxed))
        })
    }

    async fn command(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .command
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone())
        })
    }

    async fn auto(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.auto.load(Ordering::Relaxed))
        })
    }

    async fn condition_met(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.condition_met.load(Ordering::Relaxed))
        })
    }

    async fn powered(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CommandBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.powered.load(Ordering::Relaxed))
        })
    }
}

impl HostSignBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<SignBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSignBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_front_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(to_wasm_sign_text(&state.get(&res)?.front_text))
        })
    }

    async fn set_front_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let sign = state.get(&res)?;
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
        })
    }

    async fn get_back_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(to_wasm_sign_text(&state.get(&res)?.back_text))
        })
    }

    async fn set_back_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let sign = state.get(&res)?;
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
        })
    }

    async fn is_waxed(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.is_waxed.load(Ordering::Relaxed))
        })
    }

    async fn set_waxed(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.is_waxed.store(waxed, Ordering::Relaxed);
            Ok(())
        })
    }
}

impl HostJukeboxBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<JukeboxBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostJukeboxBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn is_playing(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.is_playing())
        })
    }

    async fn stop_playing(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JukeboxBlockEntity>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.stop_playing();
            Ok(())
        })
    }

    async fn start_playing(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JukeboxBlockEntity>,
        length_in_ticks: u64,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.start_playing(length_in_ticks);
            Ok(())
        })
    }
}

impl HostChestBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<ChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostChestBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn viewer_count(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.get_viewer_count() as u32)
        })
    }
}

impl HostMobSpawnerBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<MobSpawnerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostMobSpawnerBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_spawn_count(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.spawn_count)
        })
    }

    async fn get_spawn_range(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.spawn_range)
        })
    }

    async fn get_delay(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MobSpawnerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.delay.load(Ordering::Relaxed))
        })
    }
}

impl HostMapBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<MapBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostMapBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_map_id(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.get_map_id())
        })
    }

    async fn set_map_id(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
        map_id: i32,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.set_map_id(map_id);
            Ok(())
        })
    }

    async fn get_colors(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<Vec<u8>> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.get_colors())
        })
    }

    async fn set_colors(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
        colors: Vec<u8>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.set_colors(&colors);
            Ok(())
        })
    }

    async fn set_pixel(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
        color: u8,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.set_pixel(x as usize, y as usize, color);
            Ok(())
        })
    }

    async fn get_pixel(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
        x: u32,
        y: u32,
    ) -> wasmtime::Result<u8> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.get_pixel(x as usize, y as usize))
        })
    }

    async fn update(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server context not set"))?;
            state.get(&res)?.broadcast_map_data(server);
            Ok(())
        })
    }

    async fn stream_frame(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<MapBlockEntity>,
        frame_data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let server_opt = state.server.as_deref();
            state.get(&res)?.stream_frame(&frame_data, server_opt);
            Ok(())
        })
    }
}

impl HostHangingSignBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<HangingSignBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostHangingSignBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_front_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(to_wasm_sign_text(&state.get(&res)?.front_text))
        })
    }

    async fn set_front_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let sign = state.get(&res)?;
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
        })
    }

    async fn get_back_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<SignText> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(to_wasm_sign_text(&state.get(&res)?.back_text))
        })
    }

    async fn set_back_text(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
        text: SignText,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            let sign = state.get(&res)?;
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
        })
    }

    async fn is_waxed(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.is_waxed.load(Ordering::Relaxed))
        })
    }

    async fn set_waxed(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HangingSignBlockEntity>,
        waxed: bool,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut host| {
            let state = host.get();
            state.get(&res)?.is_waxed.store(waxed, Ordering::Relaxed);
            Ok(())
        })
    }
}

impl HostTrappedChestBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<TrappedChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostTrappedChestBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn viewer_count(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<TrappedChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.get_viewer_count() as u32)
        })
    }
}

macro_rules! impl_basic_block_entity {
    ($trait_name:ident, $with_store:ident, $resource_name:ident, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
            }
        }

        impl $with_store<PluginHostState> for HasSelf<PluginHostState> {
            async fn get_block_entity(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                accessor.with(|mut host| {
                    let state = host.get();
                    state.add(state.get(&res)?.clone() as _)
                })
            }
        }
    };
}

macro_rules! impl_container_basic_block_entity {
    ($trait_name:ident, $with_store:ident, $resource_name:ident, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
            }
        }

        impl $with_store<PluginHostState> for HasSelf<PluginHostState> {
            async fn get_block_entity(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                accessor.with(|mut host| {
                    let state = host.get();
                    state.add(state.get(&res)?.clone() as _)
                })
            }

            async fn get_container(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
                accessor.with(|mut host| get_container_from_be(host.get(), &res))
            }
        }
    };
}

macro_rules! impl_cooking_host_block_entity {
    ($trait_name:ident, $with_store:ident, $resource_name:ident, $internal_type:ty, $name_str:expr) => {
        impl $trait_name for PluginHostState {
            async fn drop(&mut self, rep: Resource<$resource_name>) -> wasmtime::Result<()> {
                self.drop(rep)
            }
        }

        impl $with_store<PluginHostState> for HasSelf<PluginHostState> {
            async fn get_block_entity(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<BlockEntity>> {
                accessor.with(|mut host| {
                    let state = host.get();
                    state.add(state.get(&res)?.clone() as _)
                })
            }

            async fn get_container(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
                accessor.with(|mut host| get_container_from_be(host.get(), &res))
            }

            async fn get_cooking_time_spent(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                accessor.with(|mut host| Ok(host.get().get(&res)?.get_cooking_time_spent()))
            }

            async fn get_cooking_total_time(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                accessor.with(|mut host| Ok(host.get().get(&res)?.get_cooking_total_time()))
            }

            async fn get_lit_time_remaining(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                accessor.with(|mut host| Ok(host.get().get(&res)?.get_lit_time_remaining()))
            }

            async fn get_lit_total_time(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<u16> {
                accessor.with(|mut host| Ok(host.get().get(&res)?.get_lit_total_time()))
            }

            async fn is_burning(
                accessor: &Accessor<PluginHostState, Self>,
                res: Resource<$resource_name>,
            ) -> wasmtime::Result<bool> {
                accessor.with(|mut host| Ok(host.get().get(&res)?.is_burning()))
            }
        }
    };
}

impl_cooking_host_block_entity!(
    HostBlastingFurnaceBlockEntity,
    HostBlastingFurnaceBlockEntityWithStore,
    BlastingFurnaceBlockEntity,
    InternalBlastingFurnaceBlockEntity,
    "blasting furnace block entity"
);
impl_cooking_host_block_entity!(
    HostFurnaceBlockEntity,
    HostFurnaceBlockEntityWithStore,
    FurnaceBlockEntity,
    InternalFurnaceBlockEntity,
    "furnace block entity"
);
impl_cooking_host_block_entity!(
    HostSmokerBlockEntity,
    HostSmokerBlockEntityWithStore,
    SmokerBlockEntity,
    InternalSmokerBlockEntity,
    "smoker block entity"
);

impl HostBannerBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BannerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBannerBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BannerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_custom_name(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BannerBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .custom_name
                .try_lock()
                .ok()
                .and_then(|g| g.clone()))
        })
    }
}

impl HostBarrelBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BarrelBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBarrelBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn viewer_count(
        _accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<BarrelBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }
}

impl HostBeaconBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BeaconBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBeaconBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_primary_effect(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.primary_effect.load(Ordering::Relaxed))
        })
    }

    async fn get_secondary_effect(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.secondary_effect.load(Ordering::Relaxed))
        })
    }

    async fn get_levels(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeaconBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.levels.load(Ordering::Relaxed))
        })
    }
}

impl HostBeehiveBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BeehiveBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBeehiveBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeehiveBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_bee_count(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BeehiveBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .bees
                .try_lock()
                .ok()
                .and_then(|g| g.as_ref().map(|v| v.len() as u32))
                .unwrap_or(0))
        })
    }
}

impl HostBellBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BellBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBellBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BellBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn is_ringing(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BellBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.ringing.load())
        })
    }

    async fn get_ring_ticks(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BellBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.ring_ticks.load())
        })
    }
}

impl HostBrewingStandBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<BrewingStandBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostBrewingStandBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_brew_time(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.brew_time.load(Ordering::Relaxed))
        })
    }

    async fn get_fuel(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<BrewingStandBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.fuel.load(Ordering::Relaxed))
        })
    }
}

impl HostChiseledBookshelfBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<ChiseledBookshelfBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostChiseledBookshelfBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_last_interacted_slot(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ChiseledBookshelfBlockEntity>,
    ) -> wasmtime::Result<i8> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .last_interacted_slot
                .load(Ordering::Relaxed))
        })
    }
}

impl HostComparatorBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<ComparatorBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostComparatorBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_output_signal(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ComparatorBlockEntity>,
    ) -> wasmtime::Result<u8> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.output_signal.load(Ordering::Relaxed))
        })
    }
}

impl HostCrafterBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<CrafterBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCrafterBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_crafting_ticks_remaining(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .crafting_ticks_remaining
                .load(Ordering::Relaxed))
        })
    }

    async fn is_triggered(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CrafterBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.triggered.load(Ordering::Relaxed))
        })
    }
}

impl HostCreakingHeartBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<CreakingHeartBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostCreakingHeartBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_creaking_uuid(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<CreakingHeartBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.creaking_uuid.load().map(|u| u.to_string()))
        })
    }
}

impl HostEndGatewayBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<EndGatewayBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostEndGatewayBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_age(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<i64> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.age.try_lock().ok().map_or(0, |g| *g))
        })
    }

    async fn is_exact_teleport(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<EndGatewayBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.exact_teleport.try_lock().is_ok_and(|g| *g))
        })
    }
}

impl HostEnderChestBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<EnderChestBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostEnderChestBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn viewer_count(
        _accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<EnderChestBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }
}

impl HostShulkerBoxBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<ShulkerBoxBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostShulkerBoxBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn viewer_count(
        _accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<ShulkerBoxBlockEntity>,
    ) -> wasmtime::Result<u32> {
        Ok(0)
    }
}

impl HostHopperBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<HopperBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostHopperBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_cooldown(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<HopperBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.cooldown_time.load(Ordering::Relaxed))
        })
    }
}

impl HostJigsawBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<JigsawBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostJigsawBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_name(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .name
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_target(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .target
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_pool(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .pool
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_final_state(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .final_state
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_selection_priority(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.selection_priority.load(Ordering::Relaxed))
        })
    }

    async fn get_placement_priority(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<JigsawBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.placement_priority.load(Ordering::Relaxed))
        })
    }
}

impl HostLecternBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<LecternBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostLecternBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_container(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<Resource<ContainerBlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            get_container_from_be(state, &res)
        })
    }

    async fn get_page(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<LecternBlockEntity>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.page.load(Ordering::Relaxed) as u32)
        })
    }
}

impl HostPistonBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<PistonBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostPistonBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_progress(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.current_progress.load())
        })
    }

    async fn is_extending(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.extending)
        })
    }

    async fn is_source(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<PistonBlockEntity>,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.source)
        })
    }
}

impl HostSculkShriekerBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<SculkShriekerBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSculkShriekerBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_warning_level(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SculkShriekerBlockEntity>,
    ) -> wasmtime::Result<i32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.warning_level.try_lock().map_or(0, |g| *g))
        })
    }
}

impl HostSkullBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<SkullBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostSkullBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_note_block_sound(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<SkullBlockEntity>,
    ) -> wasmtime::Result<Option<String>> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .note_block_sound
                .try_lock()
                .ok()
                .and_then(|g| g.clone()))
        })
    }
}

impl HostStructureBlockBlockEntity for PluginHostState {
    async fn drop(&mut self, rep: Resource<StructureBlockBlockEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostStructureBlockBlockEntityWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn get_block_entity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<Resource<BlockEntity>> {
        accessor.with(|mut host| {
            let state = host.get();
            state.add(state.get(&res)?.clone() as _)
        })
    }

    async fn get_name(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .name
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_author(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .author
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_mode(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<String> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state
                .get(&res)?
                .mode
                .try_lock()
                .ok()
                .map_or_else(String::new, |g| g.clone()))
        })
    }

    async fn get_integrity(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<f32> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.integrity.try_lock().map_or(1.0, |g| *g))
        })
    }

    async fn get_seed(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<StructureBlockBlockEntity>,
    ) -> wasmtime::Result<i64> {
        accessor.with(|mut host| {
            let state = host.get();
            Ok(state.get(&res)?.seed.try_lock().map_or(0, |g| *g))
        })
    }
}

impl_basic_block_entity!(
    HostBedBlockEntity,
    HostBedBlockEntityWithStore,
    BedBlockEntity,
    "bed block entity"
);
impl_basic_block_entity!(
    HostBrushableBlockBlockEntity,
    HostBrushableBlockBlockEntityWithStore,
    BrushableBlockBlockEntity,
    "brushable block block entity"
);
impl_basic_block_entity!(
    HostCalibratedSculkSensorBlockEntity,
    HostCalibratedSculkSensorBlockEntityWithStore,
    CalibratedSculkSensorBlockEntity,
    "calibrated sculk sensor block entity"
);
impl_container_basic_block_entity!(
    HostCampfireBlockEntity,
    HostCampfireBlockEntityWithStore,
    CampfireBlockEntity,
    "campfire block entity"
);
impl_basic_block_entity!(
    HostConduitBlockEntity,
    HostConduitBlockEntityWithStore,
    ConduitBlockEntity,
    "conduit block entity"
);
impl_basic_block_entity!(
    HostCopperGolemStatueBlockEntity,
    HostCopperGolemStatueBlockEntityWithStore,
    CopperGolemStatueBlockEntity,
    "copper golem statue block entity"
);
impl_basic_block_entity!(
    HostDaylightDetectorBlockEntity,
    HostDaylightDetectorBlockEntityWithStore,
    DaylightDetectorBlockEntity,
    "daylight detector block entity"
);
impl_basic_block_entity!(
    HostDecoratedPotBlockEntity,
    HostDecoratedPotBlockEntityWithStore,
    DecoratedPotBlockEntity,
    "decorated pot block entity"
);
impl_container_basic_block_entity!(
    HostDispenserBlockEntity,
    HostDispenserBlockEntityWithStore,
    DispenserBlockEntity,
    "dispenser block entity"
);
impl_container_basic_block_entity!(
    HostDropperBlockEntity,
    HostDropperBlockEntityWithStore,
    DropperBlockEntity,
    "dropper block entity"
);
impl_basic_block_entity!(
    HostEnchantingTableBlockEntity,
    HostEnchantingTableBlockEntityWithStore,
    EnchantingTableBlockEntity,
    "enchanting table block entity"
);
impl_basic_block_entity!(
    HostEndPortalBlockEntity,
    HostEndPortalBlockEntityWithStore,
    EndPortalBlockEntity,
    "end portal block entity"
);
impl_basic_block_entity!(
    HostPotentSulfurBlockEntity,
    HostPotentSulfurBlockEntityWithStore,
    PotentSulfurBlockEntity,
    "potent sulfur block entity"
);
impl_basic_block_entity!(
    HostSculkCatalystBlockEntity,
    HostSculkCatalystBlockEntityWithStore,
    SculkCatalystBlockEntity,
    "sculk catalyst block entity"
);
impl_basic_block_entity!(
    HostSculkSensorBlockEntity,
    HostSculkSensorBlockEntityWithStore,
    SculkSensorBlockEntity,
    "sculk sensor block entity"
);
impl_container_basic_block_entity!(
    HostShelfBlockEntity,
    HostShelfBlockEntityWithStore,
    ShelfBlockEntity,
    "shelf block entity"
);
impl_basic_block_entity!(
    HostTestBlockBlockEntity,
    HostTestBlockBlockEntityWithStore,
    TestBlockBlockEntity,
    "test block block entity"
);
impl_basic_block_entity!(
    HostTestInstanceBlockBlockEntity,
    HostTestInstanceBlockBlockEntityWithStore,
    TestInstanceBlockBlockEntity,
    "test instance block block entity"
);
impl_basic_block_entity!(
    HostTrialSpawnerBlockEntity,
    HostTrialSpawnerBlockEntityWithStore,
    TrialSpawnerBlockEntity,
    "trial spawner block entity"
);
impl_basic_block_entity!(
    HostVaultBlockEntity,
    HostVaultBlockEntityWithStore,
    VaultBlockEntity,
    "vault block entity"
);
