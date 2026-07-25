use crate::plugin::loader::wasm::wasm_host::state::{ItemStackResource, PluginHostState};
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::data_components::DataComponent as WitDataComponent;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::enchantments::Enchantment as WitEnchantment;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::item_stack::{
    DataComponentValue as WitDataComponentValue, EnchantmentValue as WitEnchantmentValue,
    Host as ItemStackInterfaceHost, HostItemStack, ItemStack as ItemStackHandle,
    NbtEntry as WitNbtEntry, NbtTag as WitNbtTag, NbtTree as WitNbtTree,
};
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent as WitTextComponent;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::wit::v0_1::player::text_component_from_resource;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{CustomNameImpl, EnchantmentsImpl};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::data_component::{deserialize, serialize};
use std::borrow::Cow;

pub(crate) fn to_wit_data_component(id: DataComponent) -> WitDataComponent {
    // Safety: WIT enum is generated in the same order as the internal enum
    unsafe { std::mem::transmute(id as u8) }
}

pub(crate) fn from_wit_data_component(id: WitDataComponent) -> DataComponent {
    // Safety: WIT enum is generated in the same order as the internal enum
    unsafe { std::mem::transmute(id as u8) }
}

pub(crate) fn to_wit_enchantment(id: &Enchantment) -> WitEnchantment {
    // Safety: WIT enum is generated in the same order as the internal enum
    unsafe { std::mem::transmute(id.id) }
}

pub(crate) fn from_wit_enchantment(id: WitEnchantment) -> &'static Enchantment {
    // Safety: WIT enum is generated in the same order as the internal enum
    Enchantment::from_id(id as u8).unwrap()
}

fn push_wit_nbt_tag(tag: NbtTag, tags: &mut Vec<WitNbtTag>) -> u32 {
    let index = tags.len() as u32;
    tags.push(WitNbtTag::Byte(0));
    let tag = match tag {
        NbtTag::End => WitNbtTag::Compound(Vec::new()),
        NbtTag::Byte(value) => WitNbtTag::Byte(value),
        NbtTag::Short(value) => WitNbtTag::Short(value),
        NbtTag::Int(value) => WitNbtTag::Int(value),
        NbtTag::Long(value) => WitNbtTag::Long(value),
        NbtTag::Float(value) => WitNbtTag::Float(value),
        NbtTag::Double(value) => WitNbtTag::Double(value),
        NbtTag::ByteArray(value) => WitNbtTag::ByteArray(value.into_vec()),
        NbtTag::String(value) => WitNbtTag::StringTag(value.into()),
        NbtTag::List(value) => WitNbtTag::ListTag(
            value
                .into_iter()
                .map(|value| push_wit_nbt_tag(value, tags))
                .collect(),
        ),
        NbtTag::Compound(value) => WitNbtTag::Compound(
            value
                .child_tags
                .into_iter()
                .map(|(key, value)| WitNbtEntry {
                    key: key.into(),
                    value: push_wit_nbt_tag(value, tags),
                })
                .collect(),
        ),
        NbtTag::IntArray(value) => WitNbtTag::IntArray(value),
        NbtTag::LongArray(value) => WitNbtTag::LongArray(value),
    };
    tags[index as usize] = tag;
    index
}

fn to_wit_nbt_tree(tag: NbtTag) -> WitNbtTree {
    let mut tags = Vec::new();
    let root = push_wit_nbt_tag(tag, &mut tags);
    WitNbtTree { root, tags }
}

fn from_wit_nbt_tree(tree: &WitNbtTree) -> Result<NbtTag, String> {
    fn read_tag(index: u32, tags: &[WitNbtTag], visiting: &mut Vec<u32>) -> Result<NbtTag, String> {
        let Some(tag) = tags.get(index as usize) else {
            return Err(format!("NBT tag index {index} is out of bounds"));
        };
        if visiting.contains(&index) {
            return Err(format!("NBT tag tree contains a cycle at index {index}"));
        }
        visiting.push(index);
        let tag = match tag {
            WitNbtTag::Byte(value) => NbtTag::Byte(*value),
            WitNbtTag::Short(value) => NbtTag::Short(*value),
            WitNbtTag::Int(value) => NbtTag::Int(*value),
            WitNbtTag::Long(value) => NbtTag::Long(*value),
            WitNbtTag::Float(value) => NbtTag::Float(*value),
            WitNbtTag::Double(value) => NbtTag::Double(*value),
            WitNbtTag::ByteArray(value) => NbtTag::ByteArray(value.clone().into()),
            WitNbtTag::StringTag(value) => NbtTag::String(value.clone().into()),
            WitNbtTag::ListTag(value) => NbtTag::List(
                value
                    .iter()
                    .map(|value| read_tag(*value, tags, visiting))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            WitNbtTag::Compound(value) => NbtTag::Compound(NbtCompound {
                child_tags: value
                    .iter()
                    .map(|entry| {
                        read_tag(entry.value, tags, visiting)
                            .map(|value| (entry.key.clone().into(), value))
                    })
                    .collect::<Result<_, _>>()?,
            }),
            WitNbtTag::IntArray(value) => NbtTag::IntArray(value.clone()),
            WitNbtTag::LongArray(value) => NbtTag::LongArray(value.clone()),
        };
        visiting.pop();
        Ok(tag)
    }

    read_tag(tree.root, &tree.tags, &mut Vec::new())
}

impl PluginHostState {
    pub fn get_item_stack(
        &self,
        res: &Resource<ItemStackHandle>,
    ) -> wasmtime::Result<Arc<Mutex<pumpkin_data::item_stack::ItemStack>>> {
        self.resource_table
            .get::<ItemStackResource>(&Resource::new_own(res.rep()))
            .map(|r| r.provider.clone())
            .map_err(wasmtime::Error::from)
    }
}

impl ItemStackInterfaceHost for PluginHostState {}

impl HostItemStack for PluginHostState {
    async fn new(
        &mut self,
        registry_key: String,
        count: u8,
    ) -> wasmtime::Result<Resource<ItemStackHandle>> {
        let item = pumpkin_data::item::Item::from_registry_key(
            registry_key
                .strip_prefix("minecraft:")
                .unwrap_or(&registry_key),
        )
        .unwrap_or(&pumpkin_data::item::Item::AIR);
        let stack = pumpkin_data::item_stack::ItemStack::new(count, item);
        self.add_item_stack(Arc::new(Mutex::new(stack)))
    }

    async fn get_registry_key(
        &mut self,
        res: Resource<ItemStackHandle>,
    ) -> wasmtime::Result<String> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        Ok(stack.item.registry_key.to_string())
    }

    async fn get_count(&mut self, res: Resource<ItemStackHandle>) -> wasmtime::Result<u8> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        Ok(stack.item_count)
    }

    async fn set_count(
        &mut self,
        res: Resource<ItemStackHandle>,
        count: u8,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        stack.item_count = count;
        Ok(())
    }

    async fn get_max_count(&mut self, res: Resource<ItemStackHandle>) -> wasmtime::Result<u8> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        // Search in components for MaxStackSize
        if let Some((_, data)) = stack
            .item
            .components
            .iter()
            .find(|(id, _)| *id == DataComponent::MaxStackSize)
            && let Some(max_size) = data
                .as_any()
                .downcast_ref::<pumpkin_data::data_component_impl::MaxStackSizeImpl>()
        {
            return Ok(max_size.size);
        }
        Ok(64) // Default
    }

    async fn get_enchantments(
        &mut self,
        res: Resource<ItemStackHandle>,
    ) -> wasmtime::Result<Vec<WitEnchantmentValue>> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        let mut enchantments = Vec::new();
        if let Some((_, Some(data))) = stack
            .patch
            .iter()
            .find(|(id, _)| *id == DataComponent::Enchantments)
            && let Some(enc_impl) = data.as_any().downcast_ref::<EnchantmentsImpl>()
        {
            for (enc, level) in enc_impl.enchantment.iter() {
                enchantments.push(WitEnchantmentValue {
                    enchantment: to_wit_enchantment(enc),
                    level: *level as u32,
                });
            }
        }
        Ok(enchantments)
    }

    async fn add_enchantment(
        &mut self,
        res: Resource<ItemStackHandle>,
        enchantment: WitEnchantment,
        level: u32,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        let enc = from_wit_enchantment(enchantment);

        let mut current_encs = if let Some((_, Some(data))) = stack
            .patch
            .iter()
            .find(|(id, _)| *id == DataComponent::Enchantments)
        {
            data.as_any()
                .downcast_ref::<EnchantmentsImpl>()
                .map(|e| e.enchantment.clone().into_owned())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        current_encs.retain(|(e, _)| e.id != enc.id);
        current_encs.push((enc, level as i32));

        if let Some((_, data)) = stack
            .patch
            .iter_mut()
            .find(|(id, _)| *id == DataComponent::Enchantments)
        {
            *data = Some(Box::new(EnchantmentsImpl {
                enchantment: Cow::from(current_encs),
            }));
        } else {
            stack.patch.push((
                DataComponent::Enchantments,
                Some(Box::new(EnchantmentsImpl {
                    enchantment: Cow::from(current_encs),
                })),
            ));
        }
        Ok(())
    }

    async fn remove_enchantment(
        &mut self,
        res: Resource<ItemStackHandle>,
        enchantment: WitEnchantment,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        let enc = from_wit_enchantment(enchantment);

        if let Some((_, Some(data))) = stack
            .patch
            .iter_mut()
            .find(|(id, _)| *id == DataComponent::Enchantments)
            && let Some(enc_impl) = data.as_mut_any().downcast_mut::<EnchantmentsImpl>()
        {
            let mut encs = enc_impl.enchantment.clone().into_owned();
            encs.retain(|(e, _)| e.id != enc.id);
            enc_impl.enchantment = Cow::from(encs);
        }
        Ok(())
    }

    async fn get_lore(
        &mut self,
        res: Resource<ItemStackHandle>,
    ) -> wasmtime::Result<Vec<Resource<WitTextComponent>>> {
        let _stack = self.get_item_stack(&res)?;
        // LoreImpl is currently not fully implemented with data.
        Ok(Vec::new())
    }

    async fn set_lore(
        &mut self,
        res: Resource<ItemStackHandle>,
        _lore: Vec<Resource<WitTextComponent>>,
    ) -> wasmtime::Result<()> {
        let _stack = self.get_item_stack(&res)?;
        Ok(())
    }

    async fn add_lore(
        &mut self,
        res: Resource<ItemStackHandle>,
        _line: Resource<WitTextComponent>,
    ) -> wasmtime::Result<()> {
        let _stack = self.get_item_stack(&res)?;
        Ok(())
    }

    async fn get_custom_name(
        &mut self,
        res: Resource<ItemStackHandle>,
    ) -> wasmtime::Result<Option<Resource<WitTextComponent>>> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        if let Some((_, Some(data))) = stack
            .patch
            .iter()
            .find(|(id, _)| *id == DataComponent::CustomName)
            && let Some(name_impl) = data.as_any().downcast_ref::<CustomNameImpl>()
        {
            return Ok(Some(self.add_text_component(name_impl.name.clone())?));
        }
        Ok(None)
    }

    async fn set_custom_name(
        &mut self,
        res: Resource<ItemStackHandle>,
        name: Option<Resource<WitTextComponent>>,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        if let Some(name_res) = name {
            let name = text_component_from_resource(self, &name_res);
            if let Some((_, data)) = stack
                .patch
                .iter_mut()
                .find(|(id, _)| *id == DataComponent::CustomName)
            {
                *data = Some(Box::new(CustomNameImpl { name }));
            } else {
                stack.patch.push((
                    DataComponent::CustomName,
                    Some(Box::new(CustomNameImpl { name })),
                ));
            }
        } else {
            stack
                .patch
                .retain(|(id, _)| *id != DataComponent::CustomName);
        }
        Ok(())
    }

    async fn set_custom_data(
        &mut self,
        res: Resource<ItemStackHandle>,
        namespace: String,
        key: String,
        value: WitNbtTree,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        let value = from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
        stack.set_custom_data(&namespace, &key, value);
        Ok(())
    }

    async fn get_custom_data(
        &mut self,
        res: Resource<ItemStackHandle>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<WitNbtTree>> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        Ok(stack.get_custom_data(&namespace, &key).map(to_wit_nbt_tree))
    }

    async fn remove_custom_data(
        &mut self,
        res: Resource<ItemStackHandle>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        stack.remove_custom_data(&namespace, &key);
        Ok(())
    }

    async fn has_custom_data(
        &mut self,
        res: Resource<ItemStackHandle>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        Ok(stack.has_custom_data(&namespace, &key))
    }

    async fn get_components(
        &mut self,
        res: Resource<ItemStackHandle>,
    ) -> wasmtime::Result<Vec<WitDataComponentValue>> {
        let stack = self.get_item_stack(&res)?;
        let stack = stack.lock().await;
        let mut components = Vec::new();
        for (id, data) in &stack.patch {
            if let Some(data) = data {
                let mut buf = Vec::new();
                serialize(*id, data.as_ref(), &mut buf).unwrap();

                components.push(WitDataComponentValue {
                    component: to_wit_data_component(*id),
                    value: buf,
                });
            }
        }
        Ok(components)
    }

    async fn set_component(
        &mut self,
        res: Resource<ItemStackHandle>,
        component: WitDataComponent,
        value: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        let id = from_wit_data_component(component);
        let mut cursor = std::io::Cursor::new(value);

        if let Ok(component_impl) = deserialize(id, &mut cursor) {
            if let Some((_, data)) = stack.patch.iter_mut().find(|(pid, _)| *pid == id) {
                *data = Some(component_impl);
            } else {
                stack.patch.push((id, Some(component_impl)));
            }
        }
        Ok(())
    }

    async fn remove_component(
        &mut self,
        res: Resource<ItemStackHandle>,
        component: WitDataComponent,
    ) -> wasmtime::Result<()> {
        let stack = self.get_item_stack(&res)?;
        let mut stack = stack.lock().await;
        let id = from_wit_data_component(component);
        stack.patch.retain(|(pid, _)| *pid != id);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<ItemStackHandle>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ItemStackResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
