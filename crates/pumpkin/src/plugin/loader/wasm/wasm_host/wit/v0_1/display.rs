use std::sync::Arc;
use wasmtime::component::Resource;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::decoration::display::{
    BlockDisplayEntity as InternalBlockDisplayEntity, DisplayEntity as InternalDisplayEntity,
    ItemDisplayEntity as InternalItemDisplayEntity, TextDisplayEntity as InternalTextDisplayEntity,
};
use crate::entity::interaction::InteractionEntity as InternalInteractionEntity;
use crate::plugin::loader::wasm::wasm_host::{
    state::{
        BlockDisplayEntityResource, DisplayEntityResource, EntityResource,
        InteractionEntityResource, ItemDisplayEntityResource, PluginHostState,
        TextDisplayEntityResource,
    },
    wit::v0_1::pumpkin::plugin::{
        display::{
            BillboardMode, BlockDisplayEntity, DisplayEntity, DisplayTransformation, Host,
            HostBlockDisplayEntity, HostDisplayEntity, HostInteractionEntity,
            HostItemDisplayEntity, HostTextDisplayEntity, InteractionEntity, ItemDisplayEntity,
            ItemDisplayMode, Quaternionf, TextAlignment, TextDisplayEntity, Vector3f,
        },
        item_stack::ItemStack as WitHostItemStack,
        text::TextComponent,
        uuid::Uuid,
        world::Entity,
    },
    wit::v0_1::uuid::UuidExt,
};

impl Host for PluginHostState {}

const fn map_billboard_mode(mode: u8) -> BillboardMode {
    match mode {
        1 => BillboardMode::Vertical,
        2 => BillboardMode::Horizontal,
        3 => BillboardMode::Center,
        _ => BillboardMode::Fixed,
    }
}

const fn map_billboard_mode_rev(mode: BillboardMode) -> u8 {
    match mode {
        BillboardMode::Fixed => 0,
        BillboardMode::Vertical => 1,
        BillboardMode::Horizontal => 2,
        BillboardMode::Center => 3,
    }
}

const fn map_item_display_mode(mode: u8) -> ItemDisplayMode {
    match mode {
        1 => ItemDisplayMode::ThirdpersonLefthand,
        2 => ItemDisplayMode::ThirdpersonRighthand,
        3 => ItemDisplayMode::FirstpersonLefthand,
        4 => ItemDisplayMode::FirstpersonRighthand,
        5 => ItemDisplayMode::Head,
        6 => ItemDisplayMode::Gui,
        7 => ItemDisplayMode::Ground,
        8 => ItemDisplayMode::Fixed,
        _ => ItemDisplayMode::None,
    }
}

const fn map_item_display_mode_rev(mode: ItemDisplayMode) -> u8 {
    match mode {
        ItemDisplayMode::None => 0,
        ItemDisplayMode::ThirdpersonLefthand => 1,
        ItemDisplayMode::ThirdpersonRighthand => 2,
        ItemDisplayMode::FirstpersonLefthand => 3,
        ItemDisplayMode::FirstpersonRighthand => 4,
        ItemDisplayMode::Head => 5,
        ItemDisplayMode::Gui => 6,
        ItemDisplayMode::Ground => 7,
        ItemDisplayMode::Fixed => 8,
    }
}

const fn map_text_alignment(align: u8) -> TextAlignment {
    match align {
        1 => TextAlignment::Left,
        2 => TextAlignment::Right,
        _ => TextAlignment::Center,
    }
}

const fn map_text_alignment_rev(align: TextAlignment) -> u8 {
    match align {
        TextAlignment::Center => 0,
        TextAlignment::Left => 1,
        TextAlignment::Right => 2,
    }
}

fn get_display_entity<'a>(
    base: &'a (dyn crate::entity::EntityBase + 'static),
) -> Option<&'a InternalDisplayEntity> {
    if let Some(b) = base.cast_any().downcast_ref::<InternalBlockDisplayEntity>() {
        return Some(&b.display);
    }
    if let Some(i) = base.cast_any().downcast_ref::<InternalItemDisplayEntity>() {
        return Some(&i.display);
    }
    if let Some(t) = base.cast_any().downcast_ref::<InternalTextDisplayEntity>() {
        return Some(&t.display);
    }
    None
}

impl HostDisplayEntity for PluginHostState {
    async fn from_entity(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<DisplayEntity>>> {
        let entity_res = self
            .resource_table
            .get::<EntityResource>(&Resource::new_borrow(entity.rep()))?;
        if get_display_entity(entity_res.provider.as_ref()).is_some() {
            let res = self.add_display_entity(entity_res.provider.clone())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_entity(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let display_res = self.get_display_entity_res(&display)?;
        self.add_entity(display_res.provider.clone())
    }

    async fn get_transformation(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<DisplayTransformation> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            let translation = d.get_translation().await;
            let scale = d.get_scale().await;
            let left_rot = d.get_left_rotation().await;
            let right_rot = d.get_right_rotation().await;

            Ok(DisplayTransformation {
                translation: Vector3f {
                    x: translation.x,
                    y: translation.y,
                    z: translation.z,
                },
                scale: Vector3f {
                    x: scale.x,
                    y: scale.y,
                    z: scale.z,
                },
                left_rotation: Quaternionf {
                    x: left_rot[0],
                    y: left_rot[1],
                    z: left_rot[2],
                    w: left_rot[3],
                },
                right_rotation: Quaternionf {
                    x: right_rot[0],
                    y: right_rot[1],
                    z: right_rot[2],
                    w: right_rot[3],
                },
            })
        } else {
            Ok(DisplayTransformation {
                translation: Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: Vector3f {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                left_rotation: Quaternionf {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                right_rotation: Quaternionf {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            })
        }
    }

    async fn set_transformation(
        &mut self,
        display: Resource<DisplayEntity>,
        transformation: DisplayTransformation,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_translation(Vector3::new(
                transformation.translation.x,
                transformation.translation.y,
                transformation.translation.z,
            ))
            .await;
            d.set_scale(Vector3::new(
                transformation.scale.x,
                transformation.scale.y,
                transformation.scale.z,
            ))
            .await;
            d.set_left_rotation([
                transformation.left_rotation.x,
                transformation.left_rotation.y,
                transformation.left_rotation.z,
                transformation.left_rotation.w,
            ])
            .await;
            d.set_right_rotation([
                transformation.right_rotation.x,
                transformation.right_rotation.y,
                transformation.right_rotation.z,
                transformation.right_rotation.w,
            ])
            .await;
        }
        Ok(())
    }

    async fn get_interpolation_duration(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(get_display_entity(display_res.provider.as_ref()).map_or(
            0,
            crate::entity::decoration::display::DisplayEntity::get_interpolation_duration,
        ))
    }

    async fn set_interpolation_duration(
        &mut self,
        display: Resource<DisplayEntity>,
        duration: i32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_interpolation_duration(duration);
        }
        Ok(())
    }

    async fn get_interpolation_start(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(get_display_entity(display_res.provider.as_ref()).map_or(
            0,
            crate::entity::decoration::display::DisplayEntity::get_interpolation_start_delta_ticks,
        ))
    }

    async fn set_interpolation_start(
        &mut self,
        display: Resource<DisplayEntity>,
        delta_ticks: i32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_interpolation_start_delta_ticks(delta_ticks);
        }
        Ok(())
    }

    async fn get_teleport_duration(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(get_display_entity(display_res.provider.as_ref()).map_or(
            0,
            crate::entity::decoration::display::DisplayEntity::get_teleport_duration,
        ))
    }

    async fn set_teleport_duration(
        &mut self,
        display: Resource<DisplayEntity>,
        duration: i32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_teleport_duration(duration);
        }
        Ok(())
    }

    async fn get_billboard(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<BillboardMode> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(
            get_display_entity(display_res.provider.as_ref()).map_or(BillboardMode::Fixed, |d| {
                map_billboard_mode(d.get_billboard())
            }),
        )
    }

    async fn set_billboard(
        &mut self,
        display: Resource<DisplayEntity>,
        mode: BillboardMode,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_billboard(map_billboard_mode_rev(mode));
        }
        Ok(())
    }

    async fn get_view_range(&mut self, display: Resource<DisplayEntity>) -> wasmtime::Result<f32> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            Ok(d.get_view_range().await)
        } else {
            Ok(1.0)
        }
    }

    async fn set_view_range(
        &mut self,
        display: Resource<DisplayEntity>,
        range: f32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_view_range(range).await;
        }
        Ok(())
    }

    async fn get_shadow_radius(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<f32> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            Ok(d.get_shadow_radius().await)
        } else {
            Ok(0.0)
        }
    }

    async fn set_shadow_radius(
        &mut self,
        display: Resource<DisplayEntity>,
        radius: f32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_shadow_radius(radius).await;
        }
        Ok(())
    }

    async fn get_shadow_strength(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<f32> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            Ok(d.get_shadow_strength().await)
        } else {
            Ok(1.0)
        }
    }

    async fn set_shadow_strength(
        &mut self,
        display: Resource<DisplayEntity>,
        strength: f32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_shadow_strength(strength).await;
        }
        Ok(())
    }

    async fn get_display_width(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<f32> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            Ok(d.get_display_width().await)
        } else {
            Ok(0.0)
        }
    }

    async fn set_display_width(
        &mut self,
        display: Resource<DisplayEntity>,
        width: f32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_display_width(width).await;
        }
        Ok(())
    }

    async fn get_display_height(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<f32> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            Ok(d.get_display_height().await)
        } else {
            Ok(0.0)
        }
    }

    async fn set_display_height(
        &mut self,
        display: Resource<DisplayEntity>,
        height: f32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_display_height(height).await;
        }
        Ok(())
    }

    async fn get_glow_color_override(
        &mut self,
        display: Resource<DisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(get_display_entity(display_res.provider.as_ref()).map_or(
            -1,
            crate::entity::decoration::display::DisplayEntity::get_glow_color_override,
        ))
    }

    async fn set_glow_color_override(
        &mut self,
        display: Resource<DisplayEntity>,
        color: i32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_glow_color_override(color);
        }
        Ok(())
    }

    async fn get_brightness(&mut self, display: Resource<DisplayEntity>) -> wasmtime::Result<i32> {
        let display_res = self.get_display_entity_res(&display)?;
        Ok(get_display_entity(display_res.provider.as_ref()).map_or(
            -1,
            crate::entity::decoration::display::DisplayEntity::get_brightness,
        ))
    }

    async fn set_brightness(
        &mut self,
        display: Resource<DisplayEntity>,
        brightness: i32,
    ) -> wasmtime::Result<()> {
        let display_res = self.get_display_entity_res(&display)?;
        if let Some(d) = get_display_entity(display_res.provider.as_ref()) {
            d.set_brightness(brightness);
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<DisplayEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<DisplayEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostBlockDisplayEntity for PluginHostState {
    async fn from_entity(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<BlockDisplayEntity>>> {
        let entity_res = self
            .resource_table
            .get::<EntityResource>(&Resource::new_borrow(entity.rep()))?;
        if entity_res
            .provider
            .cast_any()
            .is::<InternalBlockDisplayEntity>()
        {
            let res = self.add_block_display_entity(entity_res.provider.clone())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_display(
        &mut self,
        block_display: Resource<BlockDisplayEntity>,
    ) -> wasmtime::Result<Resource<DisplayEntity>> {
        let block_res = self.get_block_display_entity_res(&block_display)?;
        self.add_display_entity(block_res.provider.clone())
    }

    async fn get_entity(
        &mut self,
        block_display: Resource<BlockDisplayEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let block_res = self.get_block_display_entity_res(&block_display)?;
        self.add_entity(block_res.provider.clone())
    }

    async fn get_block_state_id(
        &mut self,
        block_display: Resource<BlockDisplayEntity>,
    ) -> wasmtime::Result<u16> {
        let block_res = self.get_block_display_entity_res(&block_display)?;
        Ok(block_res
            .provider
            .cast_any()
            .downcast_ref::<InternalBlockDisplayEntity>()
            .map_or(0, |b| b.get_block_state() as u16))
    }

    async fn set_block_state_id(
        &mut self,
        block_display: Resource<BlockDisplayEntity>,
        state_id: u16,
    ) -> wasmtime::Result<()> {
        let block_res = self.get_block_display_entity_res(&block_display)?;
        if let Some(b) = block_res
            .provider
            .cast_any()
            .downcast_ref::<InternalBlockDisplayEntity>()
        {
            b.set_block_state(state_id as i32);
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<BlockDisplayEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<BlockDisplayEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostItemDisplayEntity for PluginHostState {
    async fn from_entity(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<ItemDisplayEntity>>> {
        let entity_res = self
            .resource_table
            .get::<EntityResource>(&Resource::new_borrow(entity.rep()))?;
        if entity_res
            .provider
            .cast_any()
            .is::<InternalItemDisplayEntity>()
        {
            let res = self.add_item_display_entity(entity_res.provider.clone())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_display(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
    ) -> wasmtime::Result<Resource<DisplayEntity>> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        self.add_display_entity(item_res.provider.clone())
    }

    async fn get_entity(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        self.add_entity(item_res.provider.clone())
    }

    async fn get_item(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        if let Some(i) = item_res
            .provider
            .cast_any()
            .downcast_ref::<InternalItemDisplayEntity>()
        {
            let item = i.get_item().await;
            if *item.item == pumpkin_data::item::Item::AIR || item.item_count == 0 {
                Ok(None)
            } else {
                let res = self.add_item_stack(Arc::new(tokio::sync::Mutex::new(item)))?;
                Ok(Some(res))
            }
        } else {
            Ok(None)
        }
    }

    async fn set_item(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        if let Some(i) = item_res
            .provider
            .cast_any()
            .downcast_ref::<InternalItemDisplayEntity>()
        {
            let stack = if let Some(item_res_val) = item {
                self.get_item_stack(&item_res_val)?.lock().await.clone()
            } else {
                pumpkin_data::item_stack::ItemStack::new(0, &pumpkin_data::item::Item::AIR)
            };
            i.set_item(stack).await;
        }
        Ok(())
    }

    async fn get_item_display_mode(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
    ) -> wasmtime::Result<ItemDisplayMode> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        Ok(item_res
            .provider
            .cast_any()
            .downcast_ref::<InternalItemDisplayEntity>()
            .map_or(ItemDisplayMode::None, |i| {
                map_item_display_mode(i.get_item_display_mode())
            }))
    }

    async fn set_item_display_mode(
        &mut self,
        item_display: Resource<ItemDisplayEntity>,
        mode: ItemDisplayMode,
    ) -> wasmtime::Result<()> {
        let item_res = self.get_item_display_entity_res(&item_display)?;
        if let Some(i) = item_res
            .provider
            .cast_any()
            .downcast_ref::<InternalItemDisplayEntity>()
        {
            i.set_item_display_mode(map_item_display_mode_rev(mode));
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<ItemDisplayEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ItemDisplayEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostTextDisplayEntity for PluginHostState {
    async fn from_entity(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<TextDisplayEntity>>> {
        let entity_res = self
            .resource_table
            .get::<EntityResource>(&Resource::new_borrow(entity.rep()))?;
        if entity_res
            .provider
            .cast_any()
            .is::<InternalTextDisplayEntity>()
        {
            let res = self.add_text_display_entity(entity_res.provider.clone())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_display(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<Resource<DisplayEntity>> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        self.add_display_entity(text_res.provider.clone())
    }

    async fn get_entity(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        self.add_entity(text_res.provider.clone())
    }

    async fn get_text(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<Resource<TextComponent>> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            let text = t.get_text().await;
            self.add_text_component(text)
        } else {
            self.add_text_component(pumpkin_util::text::TextComponent::text(""))
        }
    }

    async fn set_text(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let text_val = self.get_text_provider(&text)?;
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_text(text_val).await;
        }
        Ok(())
    }

    async fn get_line_width(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .map_or(200, InternalTextDisplayEntity::get_line_width))
    }

    async fn set_line_width(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        width: i32,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_line_width(width);
        }
        Ok(())
    }

    async fn get_background(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<i32> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .map_or(
                1_073_741_824,
                InternalTextDisplayEntity::get_background_color,
            ))
    }

    async fn set_background(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        color: i32,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_background_color(color);
        }
        Ok(())
    }

    async fn get_text_opacity(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<i8> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .map_or(-1, InternalTextDisplayEntity::get_text_opacity))
    }

    async fn set_text_opacity(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        opacity: i8,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_text_opacity(opacity);
        }
        Ok(())
    }

    async fn get_shadow(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<bool> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .is_some_and(InternalTextDisplayEntity::get_shadow))
    }

    async fn set_shadow(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        shadow: bool,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_shadow(shadow);
        }
        Ok(())
    }

    async fn get_see_through(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<bool> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .is_some_and(InternalTextDisplayEntity::get_see_through))
    }

    async fn set_see_through(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        see_through: bool,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_see_through(see_through);
        }
        Ok(())
    }

    async fn get_default_background(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<bool> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .is_some_and(InternalTextDisplayEntity::get_use_default_background))
    }

    async fn set_default_background(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        default_background: bool,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_use_default_background(default_background);
        }
        Ok(())
    }

    async fn get_alignment(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
    ) -> wasmtime::Result<TextAlignment> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        Ok(text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
            .map_or(TextAlignment::Center, |t| {
                map_text_alignment(t.get_alignment())
            }))
    }

    async fn set_alignment(
        &mut self,
        text_display: Resource<TextDisplayEntity>,
        alignment: TextAlignment,
    ) -> wasmtime::Result<()> {
        let text_res = self.get_text_display_entity_res(&text_display)?;
        if let Some(t) = text_res
            .provider
            .cast_any()
            .downcast_ref::<InternalTextDisplayEntity>()
        {
            t.set_alignment(map_text_alignment_rev(alignment));
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<TextDisplayEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<TextDisplayEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostInteractionEntity for PluginHostState {
    async fn from_entity(
        &mut self,
        entity: Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<InteractionEntity>>> {
        let entity_res = self
            .resource_table
            .get::<EntityResource>(&Resource::new_borrow(entity.rep()))?;
        if entity_res
            .provider
            .cast_any()
            .is::<InternalInteractionEntity>()
        {
            let res = self.add_interaction_entity(entity_res.provider.clone())?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_entity(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        self.add_entity(int_res.provider.clone())
    }

    async fn get_width(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<f32> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            Ok(i.get_width().await)
        } else {
            Ok(1.0)
        }
    }

    async fn set_width(
        &mut self,
        interaction: Resource<InteractionEntity>,
        width: f32,
    ) -> wasmtime::Result<()> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            i.set_width(width).await;
        }
        Ok(())
    }

    async fn get_height(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<f32> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            Ok(i.get_height().await)
        } else {
            Ok(1.0)
        }
    }

    async fn set_height(
        &mut self,
        interaction: Resource<InteractionEntity>,
        height: f32,
    ) -> wasmtime::Result<()> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            i.set_height(height).await;
        }
        Ok(())
    }

    async fn get_response(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<bool> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        Ok(int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
            .is_some_and(InternalInteractionEntity::get_response))
    }

    async fn set_response(
        &mut self,
        interaction: Resource<InteractionEntity>,
        response: bool,
    ) -> wasmtime::Result<()> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            i.set_response(response);
        }
        Ok(())
    }

    async fn get_last_attacker(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<Option<Uuid>> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            let action = i.get_last_attacker().await;
            Ok(action.map(|a| Uuid::to_wit(&a.player)))
        } else {
            Ok(None)
        }
    }

    async fn get_last_interaction(
        &mut self,
        interaction: Resource<InteractionEntity>,
    ) -> wasmtime::Result<Option<Uuid>> {
        let int_res = self.get_interaction_entity_res(&interaction)?;
        if let Some(i) = int_res
            .provider
            .cast_any()
            .downcast_ref::<InternalInteractionEntity>()
        {
            let action = i.get_target().await;
            Ok(action.map(|a| Uuid::to_wit(&a.player)))
        } else {
            Ok(None)
        }
    }

    async fn drop(&mut self, rep: Resource<InteractionEntity>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<InteractionEntityResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
