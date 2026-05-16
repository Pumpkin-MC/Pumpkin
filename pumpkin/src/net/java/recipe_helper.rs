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

pub async fn take_n_ingredient(
    inventory: &PlayerInventory,
    ingredient: &GenericIngredient<'_>,
    count: u8,
) -> ItemStack {
    let mut taken = 0u8;
    let mut result: Option<ItemStack> = None;

    for slot in &inventory.main_inventory {
        let mut stack = slot.lock().await;
        if !stack.is_empty() && ingredient.match_item(stack.item) {
            let to_take = (count - taken).min(stack.item_count);
            let sub_stack = stack.split(to_take);
            taken += sub_stack.item_count;

            match &mut result {
                None => result = Some(sub_stack),
                Some(r) => r.item_count += sub_stack.item_count,
            }

            if taken >= count {
                break;
            }
        }
    }
    result.unwrap_or_else(|| ItemStack::EMPTY.clone())
}

pub async fn compute_biggest_craftable(
    ingredients: &[GenericIngredient<'_>],
    inventory: &PlayerInventory,
) -> u8 {
    let mut available: Vec<(&'static Item, u32)> = Vec::new();
    for slot in &inventory.main_inventory {
        let stack = slot.lock().await;
        if !stack.is_empty() {
            if let Some(e) = available.iter_mut().find(|(i, _)| i.id == stack.item.id) {
                e.1 += u32::from(stack.item_count);
            } else {
                available.push((stack.item, u32::from(stack.item_count)));
            }
        }
    }

    'outer: for amount in (1u32..=64).rev() {
        let mut budget = available.clone();
        for ing in ingredients {
            let Some(idx) = budget
                .iter()
                .position(|(item, count)| *count >= amount && ing.match_item(item))
            else {
                continue 'outer;
            };
            budget[idx].1 -= amount;
        }
        return amount as u8;
    }
    0
}
