use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct EnchantingTableBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<String>>,
}

impl BlockEntity for EnchantingTableBlockEntity {
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
        Self {
            position,
            custom_name: Mutex::new(custom_name),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            nbt.put_string("CustomName", name.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(name) = self.custom_name.try_lock()
            && let Some(ref name) = *name
        {
            nbt.put_string("CustomName", name.clone());
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let custom_name = self.custom_name.try_lock().ok().and_then(|g| g.clone());
        Some(
            super::bedrock::BedrockBlockActorBuilder::new("EnchantTable", self.position)
                .movable(false)
                .custom_name(custom_name.as_deref())
                .put_float("rott", 0.0)
                .finish(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                name: pumpkin_util::text::TextComponent::text(name.clone()),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
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

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.custom_name
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .map(pumpkin_util::text::TextComponent::text)
    }
}

impl EnchantingTableBlockEntity {
    pub const ID: &'static str = "minecraft:enchanting_table";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::new(None),
        }
    }
}
