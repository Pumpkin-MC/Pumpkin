use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct BannerBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<String>>,
    pub patterns: Mutex<Option<Vec<NbtTag>>>,
}

impl BlockEntity for BannerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let custom_name = nbt.get_string("CustomName").map(ToString::to_string);
        let patterns = nbt.get_list("patterns").map(<[_]>::to_vec);
        Self {
            position,
            custom_name: Mutex::new(custom_name),
            patterns: Mutex::new(patterns),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Ok(pats) = self.patterns.lock()
            && let Some(pats) = pats.as_ref()
        {
            nbt.put_list("patterns", pats.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(name) = self.custom_name.try_lock()
            && let Some(ref name) = *name
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Ok(patterns) = self.patterns.try_lock()
            && let Some(ref pats) = *patterns
        {
            nbt.put_list("patterns", pats.clone());
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let (block, _) = pumpkin_data::BlockState::from_id_with_block(state_id);
        let base_color =
            crate::block::entities::bedrock::banner_base_color_from_block_name(block.name)
                .unwrap_or(pumpkin_data::dye_color::DyeColor::White);
        let patterns_guard = self.patterns.lock().ok();
        let patterns = patterns_guard.as_ref().and_then(|p| p.as_deref());
        let name_guard = self.custom_name.lock().ok();
        let name = name_guard.as_ref().and_then(|n| n.as_deref());

        Some(crate::block::entities::bedrock::build_banner_actor(
            self.position,
            base_color,
            patterns,
            name,
        ))
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(pats) = self.patterns.lock()
            && let Some(pats) = pats.as_ref()
            && !pats.is_empty()
            && let Some(patterns) = pumpkin_data::data_component_impl::BannerPatternsImpl::read_data(
                &NbtTag::List(pats.clone()),
            )
        {
            stack.set_data_component(patterns);
        }
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                name: pumpkin_util::text::TextComponent::text(name.clone()),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        if let Some(patterns) =
            stack.get_data_component::<pumpkin_data::data_component_impl::BannerPatternsImpl>()
            && !patterns.layers.is_empty()
        {
            use pumpkin_data::data_component_impl::DataComponentImpl;
            if let NbtTag::List(pats) = patterns.write_data() {
                *self
                    .patterns
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pats);
            }
        }
        if let Some(custom_name) =
            stack.get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
        {
            *self
                .custom_name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(custom_name.name.clone().get_text());
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BannerBlockEntity {
    pub const ID: &'static str = "minecraft:banner";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::new(None),
            patterns: Mutex::new(None),
        }
    }
}
