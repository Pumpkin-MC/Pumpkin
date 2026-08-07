use std::sync::Arc;
use wasmtime::component::Resource;

use crate::block::entities::BlockEntity as InternalBlockEntity;
use crate::block::entities::chest::ChestBlockEntity as InternalChestBlockEntity;
use crate::block::entities::command_block::CommandBlockEntity as InternalCommandBlockEntity;
use crate::block::entities::jukebox::JukeboxBlockEntity as InternalJukeboxBlockEntity;
use crate::block::entities::mob_spawner::MobSpawnerBlockEntity as InternalMobSpawnerBlockEntity;
use crate::block::entities::sign::{
    DyeColor as InternalDyeColor, SignBlockEntity as InternalSignBlockEntity, Text as InternalText,
};
use crate::plugin::loader::wasm::wasm_host::{
    state::{BlockEntityResource, PluginHostState},
    wit::v0_1::pumpkin::{
        self,
        plugin::{
            block_entity::{
                BlockEntity, ChestBlockEntity, CommandBlockEntity, DyeColor, HostBlockEntity,
                HostChestBlockEntity, HostCommandBlockEntity, HostJukeboxBlockEntity,
                HostMobSpawnerBlockEntity, HostSignBlockEntity, JukeboxBlockEntity,
                MobSpawnerBlockEntity, SignBlockEntity, SignText,
            },
            common::BlockPos as WitBlockPos,
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
            .unwrap()
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
        nbt.put_string(
            "color",
            InternalDyeColor::from(from_wasm_dye_color(text.color) as i8).into(),
        );
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

    async fn drop(&mut self, rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockEntityResource>(Resource::new_own(rep.rep()));
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
        if let Some(cmd) = entity.as_any().downcast_ref::<InternalCommandBlockEntity>() {
            Ok(cmd.last_output.lock().await.clone())
        } else {
            Err(wasmtime::Error::msg("Not a command block entity"))
        }
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
        if let Some(cmd) = entity.as_any().downcast_ref::<InternalCommandBlockEntity>() {
            Ok(cmd.command.lock().await.clone())
        } else {
            Err(wasmtime::Error::msg("Not a command block entity"))
        }
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
                    (*sign.front_text.messages.lock().unwrap())
                        .clone_from(&new_text.messages.lock().unwrap());
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
                    (*sign.back_text.messages.lock().unwrap())
                        .clone_from(&new_text.messages.lock().unwrap());
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
