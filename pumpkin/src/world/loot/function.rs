use super::LootContextParameters;
use super::condition::LootConditionExt;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, item::Item};
use pumpkin_util::loot_table::{
    LootFunction, LootFunctionBonusParameter, LootFunctionNumberProvider, LootFunctionTypes,
};
use rand::RngExt;

pub(super) trait LootFunctionExt {
    fn apply(&self, stacks: &mut Vec<ItemStack>, params: &LootContextParameters);
}

fn apply_bonus(
    stacks: &mut [ItemStack],
    enchantment_name: &str,
    formula: &str,
    parameters: Option<&LootFunctionBonusParameter>,
    params: &LootContextParameters,
) {
    let enchantment_level = params.tool.as_ref().map_or(0, |tool| {
        pumpkin_data::Enchantment::from_name(enchantment_name)
            .map_or(0, |enchantment| tool.get_enchantment_level(enchantment))
    });
    if enchantment_level > 0 {
        for stack in stacks {
            match formula {
                "minecraft:binomial_with_bonus_count" => {
                    if let Some(LootFunctionBonusParameter::Probability { extra, probability }) =
                        parameters
                    {
                        let n = enchantment_level + *extra;
                        let mut extra_items = 0;
                        for _ in 0..n {
                            if rand::rng().random::<f32>() < *probability {
                                extra_items += 1;
                            }
                        }
                        stack.item_count = stack.item_count.saturating_add(extra_items as u8);
                    }
                }
                "minecraft:uniform_bonus_count" => {
                    if let Some(LootFunctionBonusParameter::Multiplier { bonus_multiplier }) =
                        parameters
                    {
                        let extra =
                            rand::rng().random_range(0..=(enchantment_level * *bonus_multiplier));
                        stack.item_count = stack.item_count.saturating_add(extra as u8);
                    }
                }
                "minecraft:ore_drops" if enchantment_level > 0 => {
                    let multiplier = rand::rng().random_range(0..=(enchantment_level + 1));
                    if multiplier > 0 {
                        stack.item_count = stack.item_count.saturating_mul(multiplier as u8);
                    }
                }
                _ => {}
            }
        }
    }
}

impl LootFunctionExt for LootFunction {
    #[allow(clippy::too_many_lines)]
    fn apply(&self, stacks: &mut Vec<ItemStack>, params: &LootContextParameters) {
        if let Some(conditions) = self.conditions
            && !conditions.iter().all(|cond| cond.is_fulfilled(params))
        {
            return;
        }

        match &self.content {
            LootFunctionTypes::SetCount { count, add } => {
                for stack in stacks {
                    if *add {
                        stack.item_count += count.generate().round() as u8;
                    } else {
                        stack.item_count = count.generate().round() as u8;
                    }
                }
            }
            LootFunctionTypes::LimitCount { min, max } => {
                if let Some(min) = min.map(|min| min.round() as u8) {
                    for stack in stacks.iter_mut() {
                        if stack.item_count < min {
                            stack.item_count = min;
                        }
                    }
                }

                if let Some(max) = max.map(|max| max.round() as u8) {
                    for stack in stacks.iter_mut() {
                        if stack.item_count > max {
                            stack.item_count = max;
                        }
                    }
                }
            }
            LootFunctionTypes::ExplosionDecay => {
                if let Some(radius) = params.explosion_radius {
                    let survival_chance = 1.0 / radius;
                    for stack in stacks.iter_mut() {
                        let mut survived = 0;
                        for _ in 0..stack.item_count {
                            if rand::rng().random::<f32>() <= survival_chance {
                                survived += 1;
                            }
                        }
                        stack.item_count = survived;
                    }
                    // Remove empty stacks
                    stacks.retain(|stack| stack.item_count > 0);
                }
            }
            LootFunctionTypes::ApplyBonus {
                enchantment,
                formula,
                parameters,
            } => {
                apply_bonus(stacks, enchantment, formula, parameters.as_ref(), params);
            }
            LootFunctionTypes::EnchantedCountIncrease {
                enchantment,
                count,
                limit,
            } => {
                let level = params.tool.as_ref().map_or(0.0, |tool| {
                    pumpkin_data::Enchantment::from_name(enchantment)
                        .map_or(0.0, |enc| tool.get_enchantment_level(enc) as f32)
                });
                let mut additional = (count.generate() * level).round() as u32;
                if let Some(lim) = limit {
                    let lim_u32 = lim.round() as u32;
                    if additional > lim_u32 {
                        additional = lim_u32;
                    }
                }
                for stack in stacks {
                    stack.item_count = stack.item_count.saturating_add(additional as u8);
                }
            }
            LootFunctionTypes::CopyComponents { source, include } => {
                tracing::warn!(
                    "CopyComponents not supported from source: {} for {:?}",
                    source,
                    include
                );
            }
            LootFunctionTypes::CopyState {
                block: _,
                properties,
            } => {
                if let Some(state) = params.block_state
                    && let Some(props_data) =
                        Block::properties(Block::from_state_id(state.id), state.id)
                {
                    let actual_props = props_data.to_props();
                    let mut properties_to_copy = std::collections::HashMap::new();
                    for &prop_name in *properties {
                        if let Some((_, value)) = actual_props.iter().find(|(k, _)| k == &prop_name)
                        {
                            properties_to_copy.insert(prop_name.to_string(), value.to_string());
                        }
                    }
                    if !properties_to_copy.is_empty() {
                        for stack in stacks.iter_mut() {
                            if let Some(block_state_comp) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::BlockStateImpl>() {
                                    let mut props = block_state_comp.properties.to_mut().clone();
                                    for (k, v) in &properties_to_copy {
                                        if let Some(pos) = props.iter().position(|(pk, _)| pk.as_ref() == k) {
                                            props[pos].1 = std::borrow::Cow::Owned(v.clone());
                                        } else {
                                            props.push((std::borrow::Cow::Owned(k.clone()), std::borrow::Cow::Owned(v.clone())));
                                        }
                                    }
                                    block_state_comp.properties = std::borrow::Cow::Owned(props);
                                } else {
                                    let properties: Vec<(std::borrow::Cow<'static, str>, std::borrow::Cow<'static, str>)> = properties_to_copy
                                        .iter()
                                        .map(|(k, v)| (std::borrow::Cow::Owned(k.clone()), std::borrow::Cow::Owned(v.clone())))
                                        .collect();
                                    stack.patch.push((
                                        pumpkin_data::data_component::DataComponent::BlockState,
                                        Some(Box::new(pumpkin_data::data_component_impl::BlockStateImpl {
                                            properties: std::borrow::Cow::Owned(properties),
                                        })),
                                    ));
                                }
                        }
                    }
                }
            }
            LootFunctionTypes::SetOminousBottleAmplifier => {
                let amplifier = rand::random_range(0..5); // Random 0 to 4
                for stack in stacks.iter_mut() {
                    if let Some(amplifier_comp) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::OminousBottleAmplifierImpl>() {
                        amplifier_comp.amplifier = amplifier;
                    } else {
                        stack.patch.push((
                            pumpkin_data::data_component::DataComponent::OminousBottleAmplifier,
                            Some(Box::new(pumpkin_data::data_component_impl::OminousBottleAmplifierImpl {
                                amplifier,
                            })),
                        ));
                    }
                }
            }
            LootFunctionTypes::SetPotion { id } => {
                let name = id.strip_prefix("minecraft:").unwrap_or(id);
                if let Some(potion) = pumpkin_data::potion::Potion::from_name(name) {
                    let potion_id = Some(potion.id as i32);
                    for stack in stacks.iter_mut() {
                        if let Some(potion_contents) = stack.get_data_component_mut::<pumpkin_data::data_component_impl::PotionContentsImpl>() {
                            potion_contents.potion_id = potion_id;
                        } else {
                            stack.patch.push((
                                pumpkin_data::data_component::DataComponent::PotionContents,
                                Some(Box::new(pumpkin_data::data_component_impl::PotionContentsImpl {
                                    potion_id,
                                    custom_color: None,
                                    custom_effects: Vec::new(),
                                    custom_name: None,
                                })),
                            ));
                        }
                    }
                }
            }
            LootFunctionTypes::FurnaceSmelt => {
                for stack in stacks.iter_mut() {
                    for recipe_type in pumpkin_data::recipes::RECIPES_COOKING {
                        if let pumpkin_data::recipes::CookingRecipeType::Smelting(recipe) =
                            recipe_type
                            && recipe.ingredient.match_item(stack.item)
                        {
                            let result_key = recipe
                                .result
                                .id
                                .strip_prefix("minecraft:")
                                .unwrap_or(recipe.result.id);
                            if let Some(smelted_item) = Item::from_registry_key(result_key) {
                                stack.item = smelted_item;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub(super) trait LootFunctionNumberProviderExt {
    fn generate(&self) -> f32;
}

impl LootFunctionNumberProviderExt for LootFunctionNumberProvider {
    fn generate(&self) -> f32 {
        match self {
            Self::Constant { value } => *value,
            Self::Uniform { min, max } => rand::random::<f32>() * (max - min) + min,
            Self::Binomial { n, p } => (0..n.floor() as u32).fold(0.0, |c, _| {
                if rand::rng().random_bool(f64::from(*p)) {
                    c + 1.0
                } else {
                    c
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_provider_generates_its_value() {
        let provider = LootFunctionNumberProvider::Constant { value: 3.0 };
        assert!((provider.generate() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_provider_stays_in_range() {
        let provider = LootFunctionNumberProvider::Uniform { min: 1.0, max: 4.0 };
        for _ in 0..64 {
            let value = provider.generate();
            assert!((1.0..=4.0).contains(&value));
        }
    }

    #[test]
    fn set_count_overwrites_stack_size() {
        let function = LootFunction {
            content: LootFunctionTypes::SetCount {
                count: LootFunctionNumberProvider::Constant { value: 5.0 },
                add: false,
            },
            conditions: None,
        };
        let mut stacks = vec![ItemStack::new(1, &Item::DIAMOND)];
        function.apply(&mut stacks, &LootContextParameters::default());
        assert_eq!(stacks[0].item_count, 5);
    }

    #[test]
    fn set_count_add_increments_stack_size() {
        let function = LootFunction {
            content: LootFunctionTypes::SetCount {
                count: LootFunctionNumberProvider::Constant { value: 2.0 },
                add: true,
            },
            conditions: None,
        };
        let mut stacks = vec![ItemStack::new(3, &Item::DIAMOND)];
        function.apply(&mut stacks, &LootContextParameters::default());
        assert_eq!(stacks[0].item_count, 5);
    }

    #[test]
    fn limit_count_clamps_between_min_and_max() {
        let function = LootFunction {
            content: LootFunctionTypes::LimitCount {
                min: Some(2.0),
                max: Some(4.0),
            },
            conditions: None,
        };
        let mut stacks = vec![
            ItemStack::new(1, &Item::DIAMOND),
            ItemStack::new(9, &Item::DIAMOND),
        ];
        function.apply(&mut stacks, &LootContextParameters::default());
        assert_eq!(stacks[0].item_count, 2);
        assert_eq!(stacks[1].item_count, 4);
    }
}
