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

/// Groups identical stacks, so each placed stack comes from a single item with the same components.
fn group_stacks(stacks: &[ItemStack]) -> Vec<(&ItemStack, u32)> {
    let mut groups: Vec<(&ItemStack, u32)> = Vec::new();
    for stack in stacks.iter().filter(|stack| !stack.is_empty()) {
        if let Some(group) = groups
            .iter_mut()
            .find(|(other, _)| other.are_items_and_components_equal(stack))
        {
            group.1 += u32::from(stack.item_count);
        } else {
            groups.push((stack, u32::from(stack.item_count)));
        }
    }
    groups
}

/// Picks a group for every ingredient that can supply `amount` items without going over its max stack size.
fn pick_groups(
    groups: &[(&ItemStack, u32)],
    ingredients: &[GenericIngredient<'_>],
    amount: u32,
) -> Option<Vec<usize>> {
    let mut budget: Vec<u32> = groups.iter().map(|(_, count)| *count).collect();
    ingredients
        .iter()
        .map(|ing| {
            let idx = groups.iter().zip(&budget).position(|((stack, _), count)| {
                *count >= amount
                    && amount <= u32::from(stack.get_max_stack_size())
                    && ing.match_item(stack.item)
            })?;
            budget[idx] -= amount;
            Some(idx)
        })
        .collect()
}

/// Takes `amount` items for every ingredient, or nothing if the inventory can't supply all of them.
pub fn take_ingredients(
    inventory: &PlayerInventory,
    ingredients: &[GenericIngredient<'_>],
    amount: u8,
) -> Option<Vec<ItemStack>> {
    let mut main_inventory = inventory
        .main_inventory
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let templates: Vec<ItemStack> = {
        let groups = group_stacks(main_inventory.as_slice());
        pick_groups(&groups, ingredients, u32::from(amount))?
            .into_iter()
            .map(|idx| groups[idx].0.clone())
            .collect()
    };

    let mut taken_stacks = Vec::with_capacity(templates.len());
    for template in &templates {
        let mut result = ItemStack::EMPTY.clone();
        for stack in main_inventory.iter_mut() {
            if result.item_count >= amount {
                break;
            }
            if !stack.is_empty() && stack.are_items_and_components_equal(template) {
                let sub_stack = stack.split(amount - result.item_count);
                if result.is_empty() {
                    result = sub_stack;
                } else {
                    result.item_count += sub_stack.item_count;
                }
            }
        }
        taken_stacks.push(result);
    }
    Some(taken_stacks)
}

pub fn compute_biggest_craftable(
    ingredients: &[GenericIngredient<'_>],
    inventory: &PlayerInventory,
) -> u8 {
    let main_inventory = inventory
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let groups = group_stacks(main_inventory.as_slice());
    (1u8..=64)
        .rev()
        .find(|amount| pick_groups(&groups, ingredients, u32::from(*amount)).is_some())
        .unwrap_or(0)
}
