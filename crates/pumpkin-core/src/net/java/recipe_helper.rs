use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::RecipeIngredientTypes;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_protocol::codec::recipe::OwnedRecipeIngredient;

#[derive(Clone, Copy)]
pub enum GenericIngredient<'a> {
    Vanilla(&'a RecipeIngredientTypes),
    Dynamic(&'a OwnedRecipeIngredient),
}

impl GenericIngredient<'_> {
    #[must_use]
    pub fn match_item(&self, item: &Item) -> bool {
        match self {
            Self::Vanilla(v) => v.match_item(item),
            Self::Dynamic(d) => d.match_item(item),
        }
    }
}

pub fn take_n_ingredient(
    inventory: &PlayerInventory,
    ingredient: &GenericIngredient<'_>,
    count: u8,
) -> ItemStack {
    let mut taken = 0u8;
    let mut result: Option<ItemStack> = None;

    let mut main_inventory = inventory
        .main_inventory
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for stack in main_inventory.iter_mut() {
        if !stack.is_empty() && ingredient.match_item(stack.item) {
            if let Some(existing) = &result
                && !existing.are_items_and_components_equal(stack)
            {
                continue;
            }
            let max_stack = result
                .as_ref()
                .map_or_else(|| stack.get_max_stack_size(), ItemStack::get_max_stack_size);
            let target_count = count.min(max_stack);
            if taken >= target_count {
                break;
            }
            let to_take = (target_count - taken).min(stack.item_count);
            let sub_stack = stack.split(to_take);
            taken += sub_stack.item_count;

            match &mut result {
                None => result = Some(sub_stack),
                Some(r) => r.item_count += sub_stack.item_count,
            }

            if taken >= target_count {
                break;
            }
        }
    }
    result.unwrap_or_else(|| ItemStack::EMPTY.clone())
}

pub fn compute_biggest_craftable(
    ingredients: &[GenericIngredient<'_>],
    inventory: &PlayerInventory,
) -> u8 {
    let mut available: Vec<(&'static Item, u8, u32)> = Vec::new();
    let main_inventory = inventory
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for stack in main_inventory.iter() {
        if !stack.is_empty() {
            let max_stack = stack.get_max_stack_size();
            if let Some(e) = available
                .iter_mut()
                .find(|(i, ms, _)| i.id == stack.item.id && *ms == max_stack)
            {
                e.2 += u32::from(stack.item_count);
            } else {
                available.push((stack.item, max_stack, u32::from(stack.item_count)));
            }
        }
    }

    'outer: for amount in (1u32..=64).rev() {
        let mut budget = available.clone();
        for ing in ingredients {
            let Some(idx) = budget.iter().position(|(item, max_stack, count)| {
                *count >= amount && u32::from(*max_stack) >= amount && ing.match_item(item)
            }) else {
                continue 'outer;
            };
            budget[idx].2 -= amount;
        }
        return amount as u8;
    }
    0
}
