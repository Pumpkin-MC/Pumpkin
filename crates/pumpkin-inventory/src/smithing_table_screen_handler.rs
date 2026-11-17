use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use crate::crafting::recipe_provider::RecipeProvider;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty,
};
use crate::slot::Slot;
use crate::window_property::PropertyDelegate;

use crate::inventory::Inventory;
use crate::inventory::SimpleInventory;
use pumpkin_data::data_component_impl::{ProvidesTrimMaterialImpl, TrimImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{
    RECIPES_SMITHING_TRANSFORM, RECIPES_SMITHING_TRIM, get_smithing_transform_recipe,
    get_smithing_trim_recipe,
};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_protocol::java::server::play::SlotActionType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SmithingInputSlotKind {
    Template,
    Base,
    Addition,
}

#[must_use]
pub fn is_smithing_template_item(item: &Item, provider: Option<&dyn RecipeProvider>) -> bool {
    RECIPES_SMITHING_TRANSFORM
        .iter()
        .any(|r| r.template.match_item(item))
        || RECIPES_SMITHING_TRIM
            .iter()
            .any(|r| r.template.match_item(item))
        || provider.is_some_and(|p| {
            p.get_dynamic_recipes().iter().any(|d| match d {
                DynamicRecipe::SmithingTransform(r) => r.template.match_item(item),
                DynamicRecipe::SmithingTrim(r) => r.template.match_item(item),
                _ => false,
            })
        })
}

#[must_use]
pub fn is_smithing_base_item(item: &Item, provider: Option<&dyn RecipeProvider>) -> bool {
    RECIPES_SMITHING_TRANSFORM
        .iter()
        .any(|r| r.base.match_item(item))
        || RECIPES_SMITHING_TRIM
            .iter()
            .any(|r| r.base.match_item(item))
        || provider.is_some_and(|p| {
            p.get_dynamic_recipes().iter().any(|d| match d {
                DynamicRecipe::SmithingTransform(r) => r.base.match_item(item),
                DynamicRecipe::SmithingTrim(r) => r.base.match_item(item),
                _ => false,
            })
        })
}

#[must_use]
pub fn is_smithing_addition_item(item: &Item, provider: Option<&dyn RecipeProvider>) -> bool {
    RECIPES_SMITHING_TRANSFORM
        .iter()
        .any(|r| r.addition.match_item(item))
        || RECIPES_SMITHING_TRIM
            .iter()
            .any(|r| r.addition.match_item(item))
        || provider.is_some_and(|p| {
            p.get_dynamic_recipes().iter().any(|d| match d {
                DynamicRecipe::SmithingTransform(r) => r.addition.match_item(item),
                DynamicRecipe::SmithingTrim(r) => r.addition.match_item(item),
                _ => false,
            })
        })
}

pub struct SmithingErrorDelegate(Arc<AtomicI32>);

impl PropertyDelegate for SmithingErrorDelegate {
    fn get_property(&self, index: i32) -> i32 {
        if index == 0 {
            self.0.load(Ordering::Relaxed)
        } else {
            0
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        if index == 0 {
            self.0.store(value, Ordering::Relaxed);
        }
    }

    fn get_properties_size(&self) -> i32 {
        1
    }
}

pub struct SmithingInputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub kind: SmithingInputSlotKind,
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
}

impl SmithingInputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        index: usize,
        kind: SmithingInputSlotKind,
        recipe_provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            kind,
            recipe_provider,
        }
    }
}

impl Slot for SmithingInputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> bool {
        let provider = self.recipe_provider.as_deref();
        match self.kind {
            SmithingInputSlotKind::Template => is_smithing_template_item(stack.item, provider),
            SmithingInputSlotKind::Base => is_smithing_base_item(stack.item, provider),
            SmithingInputSlotKind::Addition => is_smithing_addition_item(stack.item, provider),
        }
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

pub struct SmithingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub has_recipe_error: Arc<AtomicI32>,
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
}

impl SmithingTableScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        recipe_provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Smithing));
        let input_inventory = Arc::new(SimpleInventory::new(3));
        let output_inventory = Arc::new(SimpleInventory::new(1));
        let has_recipe_error = Arc::new(AtomicI32::new(0));

        let mut handler = Self {
            behaviour,
            input_inventory,
            output_inventory,
            has_recipe_error: has_recipe_error.clone(),
            recipe_provider: recipe_provider.clone(),
        };

        handler.add_slot(Arc::new(SmithingInputSlot::new(
            handler.input_inventory.clone(),
            0,
            SmithingInputSlotKind::Template,
            recipe_provider.clone(),
        )));
        handler.add_slot(Arc::new(SmithingInputSlot::new(
            handler.input_inventory.clone(),
            1,
            SmithingInputSlotKind::Base,
            recipe_provider.clone(),
        )));
        handler.add_slot(Arc::new(SmithingInputSlot::new(
            handler.input_inventory.clone(),
            2,
            SmithingInputSlotKind::Addition,
            recipe_provider,
        )));
        handler.add_slot(Arc::new(SmithingOutputSlot::new(
            handler.output_inventory.clone(),
            handler.input_inventory.clone(),
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);
        handler.add_property(ScreenProperty::new(
            Arc::new(SmithingErrorDelegate(has_recipe_error)),
            0,
        ));

        handler
    }

    fn find_transform_result(
        &self,
        template: &ItemStack,
        base: &ItemStack,
        addition: &ItemStack,
    ) -> Option<ItemStack> {
        if let Some(recipe) = get_smithing_transform_recipe(template.item, base.item, addition.item)
        {
            let res_key = recipe
                .result
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(recipe.result.id);
            if let Some(target_item) = Item::from_registry_key(res_key) {
                let mut result = base.transmute_copy(target_item, recipe.result.count);
                for (component, value) in recipe.result.components {
                    let boxed = Some(value.clone_dyn());
                    if let Some((_, c)) = result.patch.iter_mut().find(|(id, _)| id == component) {
                        *c = boxed;
                    } else {
                        result.patch.push((*component, boxed));
                    }
                }
                return Some(result);
            }
        }

        if let Some(provider) = &self.recipe_provider {
            for dynamic in provider.get_dynamic_recipes() {
                if let DynamicRecipe::SmithingTransform(recipe) = dynamic
                    && recipe.template.match_item(template.item)
                    && recipe.base.match_item(base.item)
                    && recipe.addition.match_item(addition.item)
                {
                    let res_key = recipe
                        .result
                        .item_id
                        .strip_prefix("minecraft:")
                        .unwrap_or(&recipe.result.item_id);
                    if let Some(target_item) = Item::from_registry_key(res_key) {
                        return Some(base.transmute_copy(target_item, recipe.result.count));
                    }
                }
            }
        }
        None
    }

    fn find_trim_pattern(
        &self,
        template: &ItemStack,
        base: &ItemStack,
        addition: &ItemStack,
    ) -> Option<String> {
        if let Some(recipe) = get_smithing_trim_recipe(template.item, base.item, addition.item) {
            return Some(recipe.pattern.to_string());
        }
        let provider = self.recipe_provider.as_ref()?;
        provider.get_dynamic_recipes().iter().find_map(|d| {
            if let DynamicRecipe::SmithingTrim(recipe) = d
                && recipe.template.match_item(template.item)
                && recipe.base.match_item(base.item)
                && recipe.addition.match_item(addition.item)
            {
                let pattern = template
                    .item
                    .registry_key
                    .strip_suffix("_armor_trim_smithing_template")
                    .unwrap_or(template.item.registry_key);
                Some(pattern.to_string())
            } else {
                None
            }
        })
    }

    fn can_move_into_input_slots(&self, stack: &ItemStack) -> bool {
        let provider = self.recipe_provider.as_deref();
        (is_smithing_template_item(stack.item, provider)
            && self.input_inventory.get_stack(0).is_empty())
            || (is_smithing_base_item(stack.item, provider)
                && self.input_inventory.get_stack(1).is_empty())
            || (is_smithing_addition_item(stack.item, provider)
                && self.input_inventory.get_stack(2).is_empty())
    }

    pub fn update_output(&self) {
        let template_stack = self.input_inventory.get_stack(0);
        let base_stack = self.input_inventory.get_stack(1);
        let addition_stack = self.input_inventory.get_stack(2);

        let all_inputs_present =
            !template_stack.is_empty() && !base_stack.is_empty() && !addition_stack.is_empty();
        if !all_inputs_present {
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
            self.has_recipe_error.store(0, Ordering::Relaxed);
            return;
        }

        if let Some(result) =
            self.find_transform_result(&template_stack, &base_stack, &addition_stack)
        {
            let err = i32::from(result.is_empty());
            self.output_inventory.set_stack(0, result);
            self.has_recipe_error.store(err, Ordering::Relaxed);
            return;
        }

        if let Some(pattern) = self.find_trim_pattern(&template_stack, &base_stack, &addition_stack)
            && let Some(provides) = addition_stack.get_data_component::<ProvidesTrimMaterialImpl>()
        {
            let material = provides.material.as_ref();
            let norm_mat = material.strip_prefix("minecraft:").unwrap_or(material);
            let norm_pat = pattern.strip_prefix("minecraft:").unwrap_or(&pattern);
            if let Some(trim) = base_stack.get_data_component::<TrimImpl>() {
                let curr_mat = match &trim.material {
                    NbtTag::String(s) => s.strip_prefix("minecraft:").unwrap_or(s.as_ref()),
                    _ => "",
                };
                let curr_pat = match &trim.pattern {
                    NbtTag::String(s) => s.strip_prefix("minecraft:").unwrap_or(s.as_ref()),
                    _ => "",
                };
                if curr_mat == norm_mat && curr_pat == norm_pat {
                    self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
                    self.has_recipe_error.store(1, Ordering::Relaxed);
                    return;
                }
            }

            let mut result = base_stack.copy_with_count(1);
            result.set_data_component(TrimImpl {
                material: NbtTag::String(material.to_string().into()),
                pattern: NbtTag::String(pattern.into()),
            });

            self.output_inventory.set_stack(0, result);
            self.has_recipe_error.store(0, Ordering::Relaxed);
            return;
        }

        self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
        self.has_recipe_error.store(1, Ordering::Relaxed);
    }
}

impl ScreenHandler for SmithingTableScreenHandler {
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        self.update_output();
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let Some(slot) = self.get_behaviour().slots.get(slot_index as usize).cloned() else {
            return ItemStack::EMPTY.clone();
        };
        if !slot.has_stack() {
            return ItemStack::EMPTY.clone();
        }

        let mut stack = slot.get_cloned_stack();
        let clicked = stack.clone();
        match slot_index.cmp(&3) {
            std::cmp::Ordering::Equal => {
                if !self.insert_item(&mut stack, 4, 40, true) {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_quick_move_crafted(stack.clone(), clicked.clone());
            }
            std::cmp::Ordering::Less => {
                if !self.insert_item(&mut stack, 4, 40, false) {
                    return ItemStack::EMPTY.clone();
                }
            }
            std::cmp::Ordering::Greater => {
                if self.can_move_into_input_slots(&stack) && (4..40).contains(&slot_index) {
                    if !self.insert_item(&mut stack, 0, 3, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (4..31).contains(&slot_index) {
                    if !self.insert_item(&mut stack, 31, 40, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (31..40).contains(&slot_index)
                    && !self.insert_item(&mut stack, 4, 31, false)
                {
                    return ItemStack::EMPTY.clone();
                }
            }
        }

        slot.set_stack(if stack.is_empty() {
            ItemStack::EMPTY.clone()
        } else {
            stack.clone()
        });
        slot.mark_dirty();
        self.update_output();

        if stack.item_count == clicked.item_count {
            return ItemStack::EMPTY.clone();
        }

        slot.on_take_item(player, &stack);
        self.update_output();

        clicked
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.drop_inventory(player, self.input_inventory.clone());
    }
}

pub struct SmithingOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub input_inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl SmithingOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        input_inventory: Arc<dyn Inventory>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            input_inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for SmithingOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        player.on_recipe_crafted(stack);
        self.input_inventory.remove_stack_specific(0, 1);
        self.input_inventory.remove_stack_specific(1, 1);
        self.input_inventory.remove_stack_specific(2, 1);
        player.sync_container_world_event(WorldEvent::SoundSmithingTableUsed);
        self.mark_dirty();
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn get_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn get_cloned_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn has_stack(&self) -> bool {
        !self.inventory.get_stack(self.index).is_empty()
    }

    fn set_stack(&self, stack: ItemStack) {
        self.inventory.set_stack(self.index, stack);
    }

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {}

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}
