impl Item {
    pub const LEATHER_BOOTS: Self = Self {
        id: 985,
        registry_key: "leather_boots",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.leather_boots",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 65 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.boots",
                            amount: 1f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Feet,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.boots",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Feet,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::FEET,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipLeather),
                    asset_id: Some(Cow::Borrowed("minecraft:leather")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:leather_boots"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LEATHER_CHESTPLATE: Self = Self {
        id: 983,
        registry_key: "leather_chestplate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.leather_chestplate",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 80 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.chestplate",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Chest,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.chestplate",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Chest,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::CHEST,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipLeather),
                    asset_id: Some(Cow::Borrowed("minecraft:leather")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:leather_chestplate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LEATHER_HELMET: Self = Self {
        id: 982,
        registry_key: "leather_helmet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.leather_helmet",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 55 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.helmet",
                            amount: 1f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.helmet",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipLeather),
                    asset_id: Some(Cow::Borrowed("minecraft:leather")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:leather_helmet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LEATHER_HORSE_ARMOR: Self = Self {
        id: 1290,
        registry_key: "leather_horse_armor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.leather_horse_armor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.body",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.body",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHorseArmor),
                    asset_id: Some(Cow::Borrowed("minecraft:leather")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_wear_horse_armor"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: false,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemHorseArmorUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:leather_horse_armor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LEATHER_LEGGINGS: Self = Self {
        id: 984,
        registry_key: "leather_leggings",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.leather_leggings",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 75 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.leggings",
                            amount: 2f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Legs,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.leggings",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Legs,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::LEGS,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipLeather),
                    asset_id: Some(Cow::Borrowed("minecraft:leather")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:leather_leggings"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LECTERN: Self = Self {
        id: 758,
        registry_key: "lectern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lectern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lectern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LEVER: Self = Self {
        id: 760,
        registry_key: "lever",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lever",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lever"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT: Self = Self {
        id: 531,
        registry_key: "light",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(Cow::Borrowed("level"), Cow::Borrowed("15"))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_BANNER: Self = Self {
        id: 1299,
        registry_key: "light_blue_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_BED: Self = Self {
        id: 1118,
        registry_key: "light_blue_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_BUNDLE: Self = Self {
        id: 1069,
        registry_key: "light_blue_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_blue_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_CANDLE: Self = Self {
        id: 1433,
        registry_key: "light_blue_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_CARPET: Self = Self {
        id: 536,
        registry_key: "light_blue_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:light_blue_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_CONCRETE: Self = Self {
        id: 645,
        registry_key: "light_blue_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_CONCRETE_POWDER: Self = Self {
        id: 661,
        registry_key: "light_blue_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_DYE: Self = Self {
        id: 1098,
        registry_key: "light_blue_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_blue_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_GLAZED_TERRACOTTA: Self = Self {
        id: 629,
        registry_key: "light_blue_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_HARNESS: Self = Self {
        id: 869,
        registry_key: "light_blue_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_blue_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:light_blue_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_SHULKER_BOX: Self = Self {
        id: 613,
        registry_key: "light_blue_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_STAINED_GLASS: Self = Self {
        id: 561,
        registry_key: "light_blue_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_STAINED_GLASS_PANE: Self = Self {
        id: 577,
        registry_key: "light_blue_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_TERRACOTTA: Self = Self {
        id: 517,
        registry_key: "light_blue_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_BLUE_WOOL: Self = Self {
        id: 243,
        registry_key: "light_blue_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_blue_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_blue_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_BANNER: Self = Self {
        id: 1304,
        registry_key: "light_gray_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_BED: Self = Self {
        id: 1123,
        registry_key: "light_gray_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_BUNDLE: Self = Self {
        id: 1074,
        registry_key: "light_gray_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_gray_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_CANDLE: Self = Self {
        id: 1438,
        registry_key: "light_gray_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_CARPET: Self = Self {
        id: 541,
        registry_key: "light_gray_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:light_gray_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_CONCRETE: Self = Self {
        id: 650,
        registry_key: "light_gray_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_CONCRETE_POWDER: Self = Self {
        id: 666,
        registry_key: "light_gray_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_DYE: Self = Self {
        id: 1103,
        registry_key: "light_gray_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_gray_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_GLAZED_TERRACOTTA: Self = Self {
        id: 634,
        registry_key: "light_gray_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_HARNESS: Self = Self {
        id: 874,
        registry_key: "light_gray_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.light_gray_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:light_gray_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_SHULKER_BOX: Self = Self {
        id: 618,
        registry_key: "light_gray_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_STAINED_GLASS: Self = Self {
        id: 566,
        registry_key: "light_gray_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_STAINED_GLASS_PANE: Self = Self {
        id: 582,
        registry_key: "light_gray_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_TERRACOTTA: Self = Self {
        id: 522,
        registry_key: "light_gray_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_GRAY_WOOL: Self = Self {
        id: 248,
        registry_key: "light_gray_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_gray_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_gray_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: Self = Self {
        id: 793,
        registry_key: "light_weighted_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.light_weighted_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:light_weighted_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIGHTNING_ROD: Self = Self {
        id: 761,
        registry_key: "lightning_rod",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lightning_rod",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lightning_rod"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LILAC: Self = Self {
        id: 553,
        registry_key: "lilac",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lilac",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lilac"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LILY_OF_THE_VALLEY: Self = Self {
        id: 270,
        registry_key: "lily_of_the_valley",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lily_of_the_valley",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lily_of_the_valley"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LILY_PAD: Self = Self {
        id: 451,
        registry_key: "lily_pad",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lily_pad",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lily_pad"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_BANNER: Self = Self {
        id: 1301,
        registry_key: "lime_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_BED: Self = Self {
        id: 1120,
        registry_key: "lime_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_BUNDLE: Self = Self {
        id: 1071,
        registry_key: "lime_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.lime_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_CANDLE: Self = Self {
        id: 1435,
        registry_key: "lime_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_CARPET: Self = Self {
        id: 538,
        registry_key: "lime_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:lime_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_CONCRETE: Self = Self {
        id: 647,
        registry_key: "lime_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_CONCRETE_POWDER: Self = Self {
        id: 663,
        registry_key: "lime_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_DYE: Self = Self {
        id: 1100,
        registry_key: "lime_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.lime_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_GLAZED_TERRACOTTA: Self = Self {
        id: 631,
        registry_key: "lime_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_HARNESS: Self = Self {
        id: 871,
        registry_key: "lime_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.lime_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:lime_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_SHULKER_BOX: Self = Self {
        id: 615,
        registry_key: "lime_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_STAINED_GLASS: Self = Self {
        id: 563,
        registry_key: "lime_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_STAINED_GLASS_PANE: Self = Self {
        id: 579,
        registry_key: "lime_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_TERRACOTTA: Self = Self {
        id: 519,
        registry_key: "lime_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LIME_WOOL: Self = Self {
        id: 245,
        registry_key: "lime_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lime_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lime_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LINGERING_POTION: Self = Self {
        id: 1324,
        registry_key: "lingering_potion",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.lingering_potion",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lingering_potion"),
                },
            ),
            (Lore, &LoreImpl),
            (
                PotionContents,
                &PotionContentsImpl {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: Vec::new(),
                    custom_name: None,
                },
            ),
            (PotionDurationScale, &PotionDurationScaleImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LLAMA_SPAWN_EGG: Self = Self {
        id: 1175,
        registry_key: "llama_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.llama_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:llama_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LODESTONE: Self = Self {
        id: 1414,
        registry_key: "lodestone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.lodestone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:lodestone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const LOOM: Self = Self {
        id: 1372,
        registry_key: "loom",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.loom",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:loom"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MACE: Self = Self {
        id: 1253,
        registry_key: "mace",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mace",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 500 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 5f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3.4000000953674316f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[]),
                    default_mining_speed: 1.0,
                    damage_per_block: 2,
                    can_destroy_blocks_in_creative: false,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mace"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_BANNER: Self = Self {
        id: 1298,
        registry_key: "magenta_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_BED: Self = Self {
        id: 1117,
        registry_key: "magenta_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_BUNDLE: Self = Self {
        id: 1068,
        registry_key: "magenta_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.magenta_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_CANDLE: Self = Self {
        id: 1432,
        registry_key: "magenta_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_CARPET: Self = Self {
        id: 535,
        registry_key: "magenta_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:magenta_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_CONCRETE: Self = Self {
        id: 644,
        registry_key: "magenta_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_CONCRETE_POWDER: Self = Self {
        id: 660,
        registry_key: "magenta_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_DYE: Self = Self {
        id: 1097,
        registry_key: "magenta_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.magenta_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_GLAZED_TERRACOTTA: Self = Self {
        id: 628,
        registry_key: "magenta_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_HARNESS: Self = Self {
        id: 868,
        registry_key: "magenta_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.magenta_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:magenta_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_SHULKER_BOX: Self = Self {
        id: 612,
        registry_key: "magenta_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_STAINED_GLASS: Self = Self {
        id: 560,
        registry_key: "magenta_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_STAINED_GLASS_PANE: Self = Self {
        id: 576,
        registry_key: "magenta_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_TERRACOTTA: Self = Self {
        id: 516,
        registry_key: "magenta_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGENTA_WOOL: Self = Self {
        id: 242,
        registry_key: "magenta_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magenta_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magenta_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGMA_BLOCK: Self = Self {
        id: 603,
        registry_key: "magma_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.magma_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magma_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGMA_CREAM: Self = Self {
        id: 1154,
        registry_key: "magma_cream",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.magma_cream",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magma_cream"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAGMA_CUBE_SPAWN_EGG: Self = Self {
        id: 1237,
        registry_key: "magma_cube_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.magma_cube_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:magma_cube_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_BOAT: Self = Self {
        id: 907,
        registry_key: "mangrove_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mangrove_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_BUTTON: Self = Self {
        id: 787,
        registry_key: "mangrove_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_CHEST_BOAT: Self = Self {
        id: 908,
        registry_key: "mangrove_chest_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mangrove_chest_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_chest_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_DOOR: Self = Self {
        id: 816,
        registry_key: "mangrove_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_FENCE: Self = Self {
        id: 380,
        registry_key: "mangrove_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_FENCE_GATE: Self = Self {
        id: 857,
        registry_key: "mangrove_fence_gate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_fence_gate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_fence_gate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_HANGING_SIGN: Self = Self {
        id: 1036,
        registry_key: "mangrove_hanging_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_hanging_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_hanging_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_LEAVES: Self = Self {
        id: 217,
        registry_key: "mangrove_leaves",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_leaves",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_leaves"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_LOG: Self = Self {
        id: 169,
        registry_key: "mangrove_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_PLANKS: Self = Self {
        id: 71,
        registry_key: "mangrove_planks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_planks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_planks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_PRESSURE_PLATE: Self = Self {
        id: 803,
        registry_key: "mangrove_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_PROPAGULE: Self = Self {
        id: 84,
        registry_key: "mangrove_propagule",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_propagule",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_propagule"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_ROOTS: Self = Self {
        id: 170,
        registry_key: "mangrove_roots",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_roots",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_roots"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_SHELF: Self = Self {
        id: 340,
        registry_key: "mangrove_shelf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_shelf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_shelf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_SIGN: Self = Self {
        id: 1024,
        registry_key: "mangrove_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_SLAB: Self = Self {
        id: 306,
        registry_key: "mangrove_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_STAIRS: Self = Self {
        id: 477,
        registry_key: "mangrove_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_TRAPDOOR: Self = Self {
        id: 837,
        registry_key: "mangrove_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MANGROVE_WOOD: Self = Self {
        id: 206,
        registry_key: "mangrove_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mangrove_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mangrove_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MAP: Self = Self {
        id: 1261,
        registry_key: "map",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.map",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:map"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MEDIUM_AMETHYST_BUD: Self = Self {
        id: 1447,
        registry_key: "medium_amethyst_bud",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.medium_amethyst_bud",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:medium_amethyst_bud"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MELON: Self = Self {
        id: 437,
        registry_key: "melon",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.melon",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:melon"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MELON_SEEDS: Self = Self {
        id: 1138,
        registry_key: "melon_seeds",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.melon_seeds",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:melon_seeds"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MELON_SLICE: Self = Self {
        id: 1135,
        registry_key: "melon_slice",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.melon_slice",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 1.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:melon_slice"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MILK_BUCKET: Self = Self {
        id: 1046,
        registry_key: "milk_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.milk_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Drink,
                    sound_event: IdOr::Id(Sound::EntityGenericDrink),
                    consume_particles: false,
                    effects: Cow::Borrowed(&[ConsumeEffect::ClearAllEffects]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:milk_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
            (UseRemainder, &UseRemainderImpl),
        ],
    };
    pub const MINECART: Self = Self {
        id: 882,
        registry_key: "minecart",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.minecart",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:minecart"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MINER_POTTERY_SHERD: Self = Self {
        id: 1491,
        registry_key: "miner_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.miner_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:miner_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOJANG_BANNER_PATTERN: Self = Self {
        id: 1376,
        registry_key: "mojang_banner_pattern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mojang_banner_pattern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mojang_banner_pattern"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesBannerPatterns, &ProvidesBannerPatternsImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOOSHROOM_SPAWN_EGG: Self = Self {
        id: 1193,
        registry_key: "mooshroom_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mooshroom_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mooshroom_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSS_BLOCK: Self = Self {
        id: 290,
        registry_key: "moss_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.moss_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:moss_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSS_CARPET: Self = Self {
        id: 289,
        registry_key: "moss_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.moss_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:moss_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_COBBLESTONE: Self = Self {
        id: 348,
        registry_key: "mossy_cobblestone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_cobblestone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_cobblestone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_COBBLESTONE_SLAB: Self = Self {
        id: 731,
        registry_key: "mossy_cobblestone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_cobblestone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_cobblestone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_COBBLESTONE_STAIRS: Self = Self {
        id: 713,
        registry_key: "mossy_cobblestone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_cobblestone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_cobblestone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_COBBLESTONE_WALL: Self = Self {
        id: 485,
        registry_key: "mossy_cobblestone_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_cobblestone_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_cobblestone_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_STONE_BRICK_SLAB: Self = Self {
        id: 729,
        registry_key: "mossy_stone_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_stone_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_stone_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_STONE_BRICK_STAIRS: Self = Self {
        id: 711,
        registry_key: "mossy_stone_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_stone_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_stone_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_STONE_BRICK_WALL: Self = Self {
        id: 489,
        registry_key: "mossy_stone_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_stone_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_stone_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOSSY_STONE_BRICKS: Self = Self {
        id: 404,
        registry_key: "mossy_stone_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mossy_stone_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mossy_stone_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MOURNER_POTTERY_SHERD: Self = Self {
        id: 1492,
        registry_key: "mourner_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mourner_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mourner_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUD: Self = Self {
        id: 59,
        registry_key: "mud",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mud",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mud"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUD_BRICK_SLAB: Self = Self {
        id: 319,
        registry_key: "mud_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mud_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mud_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUD_BRICK_STAIRS: Self = Self {
        id: 449,
        registry_key: "mud_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mud_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mud_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUD_BRICK_WALL: Self = Self {
        id: 492,
        registry_key: "mud_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mud_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mud_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUD_BRICKS: Self = Self {
        id: 408,
        registry_key: "mud_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mud_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mud_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUDDY_MANGROVE_ROOTS: Self = Self {
        id: 171,
        registry_key: "muddy_mangrove_roots",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.muddy_mangrove_roots",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:muddy_mangrove_roots"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MULE_SPAWN_EGG: Self = Self {
        id: 1166,
        registry_key: "mule_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mule_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mule_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSHROOM_STEM: Self = Self {
        id: 417,
        registry_key: "mushroom_stem",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mushroom_stem",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mushroom_stem"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSHROOM_STEW: Self = Self {
        id: 975,
        registry_key: "mushroom_stew",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mushroom_stew",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 6,
                    saturation: 7.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mushroom_stew"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
            (UseRemainder, &UseRemainderImpl),
        ],
    };
    pub const MUSIC_DISC_11: Self = Self {
        id: 1353,
        registry_key: "music_disc_11",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:11",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_11",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_11"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_13: Self = Self {
        id: 1339,
        registry_key: "music_disc_13",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:13",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_13",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_13"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_5: Self = Self {
        id: 1357,
        registry_key: "music_disc_5",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:5",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_5",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_5"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_BLOCKS: Self = Self {
        id: 1341,
        registry_key: "music_disc_blocks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:blocks",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_blocks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_blocks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_BOUNCE: Self = Self {
        id: 1342,
        registry_key: "music_disc_bounce",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:bounce",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_bounce",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_bounce"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_CAT: Self = Self {
        id: 1340,
        registry_key: "music_disc_cat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:cat",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_cat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_cat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_CHIRP: Self = Self {
        id: 1343,
        registry_key: "music_disc_chirp",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:chirp",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_chirp",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_chirp"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_CREATOR: Self = Self {
        id: 1344,
        registry_key: "music_disc_creator",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:creator",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_creator",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_creator"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_CREATOR_MUSIC_BOX: Self = Self {
        id: 1345,
        registry_key: "music_disc_creator_music_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:creator_music_box",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_creator_music_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_creator_music_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_FAR: Self = Self {
        id: 1346,
        registry_key: "music_disc_far",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:far",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_far",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_far"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_LAVA_CHICKEN: Self = Self {
        id: 1347,
        registry_key: "music_disc_lava_chicken",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:lava_chicken",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_lava_chicken",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_lava_chicken"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_MALL: Self = Self {
        id: 1348,
        registry_key: "music_disc_mall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:mall",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_mall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_mall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_MELLOHI: Self = Self {
        id: 1349,
        registry_key: "music_disc_mellohi",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:mellohi",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_mellohi",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_mellohi"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_OTHERSIDE: Self = Self {
        id: 1355,
        registry_key: "music_disc_otherside",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:otherside",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_otherside",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_otherside"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_PIGSTEP: Self = Self {
        id: 1358,
        registry_key: "music_disc_pigstep",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:pigstep",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_pigstep",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_pigstep"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_PRECIPICE: Self = Self {
        id: 1359,
        registry_key: "music_disc_precipice",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:precipice",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_precipice",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_precipice"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_RELIC: Self = Self {
        id: 1356,
        registry_key: "music_disc_relic",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:relic",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_relic",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_relic"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_STAL: Self = Self {
        id: 1350,
        registry_key: "music_disc_stal",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:stal",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_stal",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_stal"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_STRAD: Self = Self {
        id: 1351,
        registry_key: "music_disc_strad",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:strad",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_strad",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_strad"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_TEARS: Self = Self {
        id: 1360,
        registry_key: "music_disc_tears",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:tears",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_tears",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_tears"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_WAIT: Self = Self {
        id: 1354,
        registry_key: "music_disc_wait",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:wait",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_wait",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_wait"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUSIC_DISC_WARD: Self = Self {
        id: 1352,
        registry_key: "music_disc_ward",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                JukeboxPlayable,
                &JukeboxPlayableImpl {
                    song: "minecraft:ward",
                },
            ),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.music_disc_ward",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:music_disc_ward"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MUTTON: Self = Self {
        id: 1294,
        registry_key: "mutton",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.mutton",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 1.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mutton"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const MYCELIUM: Self = Self {
        id: 450,
        registry_key: "mycelium",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.mycelium",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:mycelium"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NAME_TAG: Self = Self {
        id: 1292,
        registry_key: "name_tag",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.name_tag",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:name_tag"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NAUTILUS_SHELL: Self = Self {
        id: 1363,
        registry_key: "nautilus_shell",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.nautilus_shell",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nautilus_shell"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NAUTILUS_SPAWN_EGG: Self = Self {
        id: 1185,
        registry_key: "nautilus_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.nautilus_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nautilus_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICK: Self = Self {
        id: 1275,
        registry_key: "nether_brick",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.nether_brick",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_brick"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICK_FENCE: Self = Self {
        id: 455,
        registry_key: "nether_brick_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_brick_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_brick_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICK_SLAB: Self = Self {
        id: 320,
        registry_key: "nether_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICK_STAIRS: Self = Self {
        id: 456,
        registry_key: "nether_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICK_WALL: Self = Self {
        id: 493,
        registry_key: "nether_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_BRICKS: Self = Self {
        id: 452,
        registry_key: "nether_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_GOLD_ORE: Self = Self {
        id: 107,
        registry_key: "nether_gold_ore",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_gold_ore",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_gold_ore"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_QUARTZ_ORE: Self = Self {
        id: 108,
        registry_key: "nether_quartz_ore",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_quartz_ore",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_quartz_ore"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_SPROUTS: Self = Self {
        id: 281,
        registry_key: "nether_sprouts",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_sprouts",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_sprouts"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_STAR: Self = Self {
        id: 1270,
        registry_key: "nether_star",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.nether_star",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Explosion,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (EnchantmentGlintOverride, &EnchantmentGlintOverrideImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_star"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_WART: Self = Self {
        id: 1148,
        registry_key: "nether_wart",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.nether_wart",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_wart"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHER_WART_BLOCK: Self = Self {
        id: 604,
        registry_key: "nether_wart_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.nether_wart_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:nether_wart_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_AXE: Self = Self {
        id: 972,
        registry_key: "netherite_axe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_axe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 9f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_netherite_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/axe")),
                            speed: Some(9f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_axe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_BLOCK: Self = Self {
        id: 128,
        registry_key: "netherite_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.netherite_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_BOOTS: Self = Self {
        id: 1009,
        registry_key: "netherite_boots",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_boots",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 481 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.boots",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Feet,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.boots",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Feet,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.boots",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Feet,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::FEET,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipNetherite),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_boots"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_CHESTPLATE: Self = Self {
        id: 1007,
        registry_key: "netherite_chestplate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_chestplate",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 592 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.chestplate",
                            amount: 8f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Chest,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.chestplate",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Chest,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.chestplate",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Chest,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::CHEST,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipNetherite),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_chestplate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_HELMET: Self = Self {
        id: 1006,
        registry_key: "netherite_helmet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_helmet",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 407 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.helmet",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.helmet",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.helmet",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipNetherite),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_helmet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_HOE: Self = Self {
        id: 973,
        registry_key: "netherite_hoe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_hoe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_netherite_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/hoe")),
                            speed: Some(9f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_hoe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_HORSE_ARMOR: Self = Self {
        id: 1289,
        registry_key: "netherite_horse_armor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_horse_armor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.body",
                            amount: 19f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.body",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.body",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHorseArmor),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_wear_horse_armor"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: false,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemHorseArmorUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_horse_armor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_INGOT: Self = Self {
        id: 937,
        registry_key: "netherite_ingot",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_ingot",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_ingot"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesTrimMaterial, &ProvidesTrimMaterialImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_LEGGINGS: Self = Self {
        id: 1008,
        registry_key: "netherite_leggings",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_leggings",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 555 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.leggings",
                            amount: 6f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Legs,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.leggings",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Legs,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.leggings",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Legs,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::LEGS,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipNetherite),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_leggings"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_NAUTILUS_ARMOR: Self = Self {
        id: 1367,
        registry_key: "netherite_nautilus_armor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_nautilus_armor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.body",
                            amount: 19f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.body",
                            amount: 3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                        Modifier {
                            r#type: &Attributes::KNOCKBACK_RESISTANCE,
                            id: "minecraft:armor.body",
                            amount: 0.10000000149011612f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Body,
                        },
                    ]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipNautilus),
                    asset_id: Some(Cow::Borrowed("minecraft:netherite")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_wear_nautilus_armor"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: false,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemArmorUnequipNautilus),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_nautilus_armor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_PICKAXE: Self = Self {
        id: 971,
        registry_key: "netherite_pickaxe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_pickaxe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 5f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.799999952316284f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_netherite_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/pickaxe")),
                            speed: Some(9f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_pickaxe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_SCRAP: Self = Self {
        id: 938,
        registry_key: "netherite_scrap",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_scrap",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_scrap"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_SHOVEL: Self = Self {
        id: 970,
        registry_key: "netherite_shovel",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_shovel",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 5.5f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_netherite_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/shovel")),
                            speed: Some(9f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_shovel"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_SPEAR: Self = Self {
        id: 1332,
        registry_key: "netherite_spear",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_spear",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 4f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3.13043475151062f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (AttackRange, &AttackRangeImpl),
            (BreakSound, &BreakSoundImpl),
            (DamageType, &DamageTypeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_spear"),
                },
            ),
            (KineticWeapon, &KineticWeaponImpl),
            (Lore, &LoreImpl),
            (MinimumAttackCharge, &MinimumAttackChargeImpl),
            (PiercingWeapon, &PiercingWeaponImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_SWORD: Self = Self {
        id: 969,
        registry_key: "netherite_sword",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_sword",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 2031 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 7f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.4000000953674316f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: IDs(Cow::Borrowed(&[&Block::COBWEB])),
                            speed: Some(15f32),
                            correct_for_drops: Some(true),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:sword_instantly_mines")),
                            speed: Some(340282350000000000000000000000000000000f32),
                            correct_for_drops: None,
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:sword_efficient")),
                            speed: Some(1.5f32),
                            correct_for_drops: None,
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 2,
                    can_destroy_blocks_in_creative: false,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (
                DamageResistant,
                &DamageResistantImpl {
                    res_type: DamageResistantType::Fire,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 15 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_sword"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERITE_UPGRADE_SMITHING_TEMPLATE: Self = Self {
        id: 1458,
        registry_key: "netherite_upgrade_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.netherite_upgrade_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherite_upgrade_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NETHERRACK: Self = Self {
        id: 387,
        registry_key: "netherrack",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.netherrack",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:netherrack"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const NOTE_BLOCK: Self = Self {
        id: 776,
        registry_key: "note_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.note_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:note_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_BOAT: Self = Self {
        id: 891,
        registry_key: "oak_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.oak_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_BUTTON: Self = Self {
        id: 779,
        registry_key: "oak_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_CHEST_BOAT: Self = Self {
        id: 892,
        registry_key: "oak_chest_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.oak_chest_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_chest_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_DOOR: Self = Self {
        id: 808,
        registry_key: "oak_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_FENCE: Self = Self {
        id: 372,
        registry_key: "oak_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_FENCE_GATE: Self = Self {
        id: 849,
        registry_key: "oak_fence_gate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_fence_gate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_fence_gate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_HANGING_SIGN: Self = Self {
        id: 1028,
        registry_key: "oak_hanging_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_hanging_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_hanging_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_LEAVES: Self = Self {
        id: 209,
        registry_key: "oak_leaves",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_leaves",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_leaves"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_LOG: Self = Self {
        id: 161,
        registry_key: "oak_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_PLANKS: Self = Self {
        id: 63,
        registry_key: "oak_planks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_planks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_planks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_PRESSURE_PLATE: Self = Self {
        id: 795,
        registry_key: "oak_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_SAPLING: Self = Self {
        id: 76,
        registry_key: "oak_sapling",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_sapling",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_sapling"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_SHELF: Self = Self {
        id: 341,
        registry_key: "oak_shelf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_shelf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_shelf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_SIGN: Self = Self {
        id: 1016,
        registry_key: "oak_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_SLAB: Self = Self {
        id: 298,
        registry_key: "oak_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_STAIRS: Self = Self {
        id: 469,
        registry_key: "oak_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_TRAPDOOR: Self = Self {
        id: 829,
        registry_key: "oak_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OAK_WOOD: Self = Self {
        id: 198,
        registry_key: "oak_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oak_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oak_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OBSERVER: Self = Self {
        id: 754,
        registry_key: "observer",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.observer",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:observer"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OBSIDIAN: Self = Self {
        id: 349,
        registry_key: "obsidian",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.obsidian",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:obsidian"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OCELOT_SPAWN_EGG: Self = Self {
        id: 1176,
        registry_key: "ocelot_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.ocelot_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ocelot_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OCHRE_FROGLIGHT: Self = Self {
        id: 1452,
        registry_key: "ochre_froglight",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.ochre_froglight",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ochre_froglight"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OMINOUS_BOTTLE: Self = Self {
        id: 1536,
        registry_key: "ominous_bottle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.ominous_bottle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Drink,
                    sound_event: IdOr::Id(Sound::EntityGenericDrink),
                    consume_particles: false,
                    effects: Cow::Borrowed(&[ConsumeEffect::PlaySound(IdOr::Id(
                        Sound::ItemOminousBottleDispose,
                    ))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ominous_bottle"),
                },
            ),
            (Lore, &LoreImpl),
            (
                OminousBottleAmplifier,
                &OminousBottleAmplifierImpl { amplifier: 0 },
            ),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OMINOUS_TRIAL_KEY: Self = Self {
        id: 1534,
        registry_key: "ominous_trial_key",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.ominous_trial_key",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ominous_trial_key"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OPEN_EYEBLOSSOM: Self = Self {
        id: 258,
        registry_key: "open_eyeblossom",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.open_eyeblossom",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:open_eyeblossom"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_BANNER: Self = Self {
        id: 1297,
        registry_key: "orange_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_BED: Self = Self {
        id: 1116,
        registry_key: "orange_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_BUNDLE: Self = Self {
        id: 1067,
        registry_key: "orange_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.orange_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_CANDLE: Self = Self {
        id: 1431,
        registry_key: "orange_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_CARPET: Self = Self {
        id: 534,
        registry_key: "orange_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:orange_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_CONCRETE: Self = Self {
        id: 643,
        registry_key: "orange_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_CONCRETE_POWDER: Self = Self {
        id: 659,
        registry_key: "orange_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_DYE: Self = Self {
        id: 1096,
        registry_key: "orange_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.orange_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_GLAZED_TERRACOTTA: Self = Self {
        id: 627,
        registry_key: "orange_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_HARNESS: Self = Self {
        id: 867,
        registry_key: "orange_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.orange_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:orange_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_SHULKER_BOX: Self = Self {
        id: 611,
        registry_key: "orange_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_STAINED_GLASS: Self = Self {
        id: 559,
        registry_key: "orange_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_STAINED_GLASS_PANE: Self = Self {
        id: 575,
        registry_key: "orange_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_TERRACOTTA: Self = Self {
        id: 515,
        registry_key: "orange_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_TULIP: Self = Self {
        id: 265,
        registry_key: "orange_tulip",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_tulip",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_tulip"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ORANGE_WOOL: Self = Self {
        id: 241,
        registry_key: "orange_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.orange_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:orange_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXEYE_DAISY: Self = Self {
        id: 268,
        registry_key: "oxeye_daisy",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxeye_daisy",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxeye_daisy"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_CHISELED_COPPER: Self = Self {
        id: 132,
        registry_key: "oxidized_chiseled_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_chiseled_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_chiseled_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER: Self = Self {
        id: 121,
        registry_key: "oxidized_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_BARS: Self = Self {
        id: 422,
        registry_key: "oxidized_copper_bars",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_bars",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_bars"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_BULB: Self = Self {
        id: 1511,
        registry_key: "oxidized_copper_bulb",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_bulb",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_bulb"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_CHAIN: Self = Self {
        id: 431,
        registry_key: "oxidized_copper_chain",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_chain",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_chain"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_CHEST: Self = Self {
        id: 1519,
        registry_key: "oxidized_copper_chest",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_chest",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_chest"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_DOOR: Self = Self {
        id: 823,
        registry_key: "oxidized_copper_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_GOLEM_STATUE: Self = Self {
        id: 1527,
        registry_key: "oxidized_copper_golem_statue",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_golem_statue",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(
                        Cow::Borrowed("copper_golem_pose"),
                        Cow::Borrowed("standing"),
                    )]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_golem_statue"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_GRATE: Self = Self {
        id: 1503,
        registry_key: "oxidized_copper_grate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_grate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_grate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_LANTERN: Self = Self {
        id: 1399,
        registry_key: "oxidized_copper_lantern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_lantern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_lantern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_COPPER_TRAPDOOR: Self = Self {
        id: 844,
        registry_key: "oxidized_copper_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_copper_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_copper_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_CUT_COPPER: Self = Self {
        id: 140,
        registry_key: "oxidized_cut_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_cut_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_cut_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_CUT_COPPER_SLAB: Self = Self {
        id: 156,
        registry_key: "oxidized_cut_copper_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_cut_copper_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_cut_copper_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_CUT_COPPER_STAIRS: Self = Self {
        id: 148,
        registry_key: "oxidized_cut_copper_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_cut_copper_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_cut_copper_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const OXIDIZED_LIGHTNING_ROD: Self = Self {
        id: 764,
        registry_key: "oxidized_lightning_rod",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.oxidized_lightning_rod",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:oxidized_lightning_rod"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PACKED_ICE: Self = Self {
        id: 550,
        registry_key: "packed_ice",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.packed_ice",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:packed_ice"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PACKED_MUD: Self = Self {
        id: 407,
        registry_key: "packed_mud",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.packed_mud",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:packed_mud"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PAINTING: Self = Self {
        id: 1013,
        registry_key: "painting",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.painting",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:painting"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_HANGING_MOSS: Self = Self {
        id: 292,
        registry_key: "pale_hanging_moss",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_hanging_moss",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_hanging_moss"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_MOSS_BLOCK: Self = Self {
        id: 293,
        registry_key: "pale_moss_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_moss_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_moss_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_MOSS_CARPET: Self = Self {
        id: 291,
        registry_key: "pale_moss_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_moss_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_moss_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_BOAT: Self = Self {
        id: 905,
        registry_key: "pale_oak_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pale_oak_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_BUTTON: Self = Self {
        id: 786,
        registry_key: "pale_oak_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_CHEST_BOAT: Self = Self {
        id: 906,
        registry_key: "pale_oak_chest_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pale_oak_chest_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_chest_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_DOOR: Self = Self {
        id: 815,
        registry_key: "pale_oak_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_FENCE: Self = Self {
        id: 379,
        registry_key: "pale_oak_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_FENCE_GATE: Self = Self {
        id: 856,
        registry_key: "pale_oak_fence_gate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_fence_gate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_fence_gate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_HANGING_SIGN: Self = Self {
        id: 1035,
        registry_key: "pale_oak_hanging_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_hanging_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_hanging_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_LEAVES: Self = Self {
        id: 216,
        registry_key: "pale_oak_leaves",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_leaves",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_leaves"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_LOG: Self = Self {
        id: 167,
        registry_key: "pale_oak_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_PLANKS: Self = Self {
        id: 70,
        registry_key: "pale_oak_planks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_planks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_planks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_PRESSURE_PLATE: Self = Self {
        id: 802,
        registry_key: "pale_oak_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_SAPLING: Self = Self {
        id: 83,
        registry_key: "pale_oak_sapling",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_sapling",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_sapling"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_SHELF: Self = Self {
        id: 342,
        registry_key: "pale_oak_shelf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_shelf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_shelf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_SIGN: Self = Self {
        id: 1023,
        registry_key: "pale_oak_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_SLAB: Self = Self {
        id: 305,
        registry_key: "pale_oak_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_STAIRS: Self = Self {
        id: 476,
        registry_key: "pale_oak_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_TRAPDOOR: Self = Self {
        id: 836,
        registry_key: "pale_oak_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PALE_OAK_WOOD: Self = Self {
        id: 204,
        registry_key: "pale_oak_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pale_oak_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pale_oak_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PANDA_SPAWN_EGG: Self = Self {
        id: 1177,
        registry_key: "panda_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.panda_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:panda_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PAPER: Self = Self {
        id: 1057,
        registry_key: "paper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.paper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:paper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PARCHED_SPAWN_EGG: Self = Self {
        id: 1206,
        registry_key: "parched_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.parched_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:parched_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PARROT_SPAWN_EGG: Self = Self {
        id: 1168,
        registry_key: "parrot_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.parrot_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:parrot_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PEARLESCENT_FROGLIGHT: Self = Self {
        id: 1454,
        registry_key: "pearlescent_froglight",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pearlescent_froglight",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pearlescent_froglight"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PEONY: Self = Self {
        id: 555,
        registry_key: "peony",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.peony",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:peony"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PETRIFIED_OAK_SLAB: Self = Self {
        id: 315,
        registry_key: "petrified_oak_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.petrified_oak_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:petrified_oak_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PHANTOM_MEMBRANE: Self = Self {
        id: 889,
        registry_key: "phantom_membrane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.phantom_membrane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:phantom_membrane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PHANTOM_SPAWN_EGG: Self = Self {
        id: 1223,
        registry_key: "phantom_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.phantom_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:phantom_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PIG_SPAWN_EGG: Self = Self {
        id: 1161,
        registry_key: "pig_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pig_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pig_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PIGLIN_BANNER_PATTERN: Self = Self {
        id: 1378,
        registry_key: "piglin_banner_pattern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.piglin_banner_pattern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:piglin_banner_pattern"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesBannerPatterns, &ProvidesBannerPatternsImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PIGLIN_BRUTE_SPAWN_EGG: Self = Self {
        id: 1239,
        registry_key: "piglin_brute_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.piglin_brute_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:piglin_brute_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PIGLIN_HEAD: Self = Self {
        id: 1269,
        registry_key: "piglin_head",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.piglin_head",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[Modifier {
                        r#type: &Attributes::WAYPOINT_TRANSMIT_RANGE,
                        id: "minecraft:waypoint_transmit_range_hide",
                        amount: -1f64,
                        operation: Operation::AddMultipliedTotal,
                        slot: AttributeModifierSlot::Head,
                    }]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipGeneric),
                    asset_id: None,
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: false,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:piglin_head"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PIGLIN_SPAWN_EGG: Self = Self {
        id: 1238,
        registry_key: "piglin_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.piglin_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:piglin_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PILLAGER_SPAWN_EGG: Self = Self {
        id: 1229,
        registry_key: "pillager_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pillager_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pillager_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_BANNER: Self = Self {
        id: 1302,
        registry_key: "pink_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_BED: Self = Self {
        id: 1121,
        registry_key: "pink_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_BUNDLE: Self = Self {
        id: 1072,
        registry_key: "pink_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pink_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_CANDLE: Self = Self {
        id: 1436,
        registry_key: "pink_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_CARPET: Self = Self {
        id: 539,
        registry_key: "pink_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:pink_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_CONCRETE: Self = Self {
        id: 648,
        registry_key: "pink_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_CONCRETE_POWDER: Self = Self {
        id: 664,
        registry_key: "pink_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_DYE: Self = Self {
        id: 1101,
        registry_key: "pink_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pink_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_GLAZED_TERRACOTTA: Self = Self {
        id: 632,
        registry_key: "pink_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_HARNESS: Self = Self {
        id: 872,
        registry_key: "pink_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pink_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:pink_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_PETALS: Self = Self {
        id: 286,
        registry_key: "pink_petals",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_petals",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_petals"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_SHULKER_BOX: Self = Self {
        id: 616,
        registry_key: "pink_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_STAINED_GLASS: Self = Self {
        id: 564,
        registry_key: "pink_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_STAINED_GLASS_PANE: Self = Self {
        id: 580,
        registry_key: "pink_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_TERRACOTTA: Self = Self {
        id: 520,
        registry_key: "pink_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_TULIP: Self = Self {
        id: 267,
        registry_key: "pink_tulip",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_tulip",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_tulip"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PINK_WOOL: Self = Self {
        id: 246,
        registry_key: "pink_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pink_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pink_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PISTON: Self = Self {
        id: 750,
        registry_key: "piston",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.piston",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:piston"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PITCHER_PLANT: Self = Self {
        id: 273,
        registry_key: "pitcher_plant",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pitcher_plant",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pitcher_plant"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PITCHER_POD: Self = Self {
        id: 1316,
        registry_key: "pitcher_pod",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pitcher_pod",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pitcher_pod"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PLAYER_HEAD: Self = Self {
        id: 1265,
        registry_key: "player_head",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.player_head",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[Modifier {
                        r#type: &Attributes::WAYPOINT_TRANSMIT_RANGE,
                        id: "minecraft:waypoint_transmit_range_hide",
                        amount: -1f64,
                        operation: Operation::AddMultipliedTotal,
                        slot: AttributeModifierSlot::Head,
                    }]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipGeneric),
                    asset_id: None,
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: false,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:player_head"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PLENTY_POTTERY_SHERD: Self = Self {
        id: 1493,
        registry_key: "plenty_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.plenty_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:plenty_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PODZOL: Self = Self {
        id: 57,
        registry_key: "podzol",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.podzol",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:podzol"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POINTED_DRIPSTONE: Self = Self {
        id: 1450,
        registry_key: "pointed_dripstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pointed_dripstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pointed_dripstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POISONOUS_POTATO: Self = Self {
        id: 1260,
        registry_key: "poisonous_potato",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.poisonous_potato",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 1.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[ConsumeEffect::ApplyEffects((
                        Cow::Borrowed(&[StatusEffectInstance {
                            effect_id: Cow::Borrowed("minecraft:poison"),
                            amplifier: 0i32,
                            duration: 100i32,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                        }]),
                        0.6f32,
                    ))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:poisonous_potato"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLAR_BEAR_SPAWN_EGG: Self = Self {
        id: 1178,
        registry_key: "polar_bear_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.polar_bear_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polar_bear_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_ANDESITE: Self = Self {
        id: 7,
        registry_key: "polished_andesite",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_andesite",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_andesite"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_ANDESITE_SLAB: Self = Self {
        id: 738,
        registry_key: "polished_andesite_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_andesite_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_andesite_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_ANDESITE_STAIRS: Self = Self {
        id: 721,
        registry_key: "polished_andesite_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_andesite_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_andesite_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BASALT: Self = Self {
        id: 391,
        registry_key: "polished_basalt",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_basalt",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_basalt"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE: Self = Self {
        id: 1420,
        registry_key: "polished_blackstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: Self = Self {
        id: 1425,
        registry_key: "polished_blackstone_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: Self = Self {
        id: 1426,
        registry_key: "polished_blackstone_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_BRICK_WALL: Self = Self {
        id: 501,
        registry_key: "polished_blackstone_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_BRICKS: Self = Self {
        id: 1424,
        registry_key: "polished_blackstone_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_BUTTON: Self = Self {
        id: 778,
        registry_key: "polished_blackstone_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: Self = Self {
        id: 792,
        registry_key: "polished_blackstone_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_SLAB: Self = Self {
        id: 1421,
        registry_key: "polished_blackstone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_STAIRS: Self = Self {
        id: 1422,
        registry_key: "polished_blackstone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_BLACKSTONE_WALL: Self = Self {
        id: 500,
        registry_key: "polished_blackstone_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_blackstone_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_blackstone_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_CINNABAR: Self = Self {
        id: 44,
        registry_key: "polished_cinnabar",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_cinnabar",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_cinnabar"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_CINNABAR_SLAB: Self = Self {
        id: 45,
        registry_key: "polished_cinnabar_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_cinnabar_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_cinnabar_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_CINNABAR_STAIRS: Self = Self {
        id: 46,
        registry_key: "polished_cinnabar_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_cinnabar_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_cinnabar_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_CINNABAR_WALL: Self = Self {
        id: 47,
        registry_key: "polished_cinnabar_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_cinnabar_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_cinnabar_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DEEPSLATE: Self = Self {
        id: 10,
        registry_key: "polished_deepslate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_deepslate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_deepslate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DEEPSLATE_SLAB: Self = Self {
        id: 741,
        registry_key: "polished_deepslate_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_deepslate_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_deepslate_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DEEPSLATE_STAIRS: Self = Self {
        id: 724,
        registry_key: "polished_deepslate_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_deepslate_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_deepslate_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DEEPSLATE_WALL: Self = Self {
        id: 503,
        registry_key: "polished_deepslate_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_deepslate_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_deepslate_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DIORITE: Self = Self {
        id: 5,
        registry_key: "polished_diorite",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_diorite",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_diorite"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DIORITE_SLAB: Self = Self {
        id: 730,
        registry_key: "polished_diorite_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_diorite_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_diorite_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_DIORITE_STAIRS: Self = Self {
        id: 712,
        registry_key: "polished_diorite_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_diorite_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_diorite_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_GRANITE: Self = Self {
        id: 3,
        registry_key: "polished_granite",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_granite",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_granite"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_GRANITE_SLAB: Self = Self {
        id: 727,
        registry_key: "polished_granite_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_granite_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_granite_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_GRANITE_STAIRS: Self = Self {
        id: 709,
        registry_key: "polished_granite_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_granite_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_granite_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_SULFUR: Self = Self {
        id: 31,
        registry_key: "polished_sulfur",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_sulfur",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_sulfur"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_SULFUR_SLAB: Self = Self {
        id: 32,
        registry_key: "polished_sulfur_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_sulfur_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_sulfur_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_SULFUR_STAIRS: Self = Self {
        id: 33,
        registry_key: "polished_sulfur_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_sulfur_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_sulfur_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_SULFUR_WALL: Self = Self {
        id: 34,
        registry_key: "polished_sulfur_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_sulfur_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_sulfur_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_TUFF: Self = Self {
        id: 17,
        registry_key: "polished_tuff",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_tuff",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_tuff"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_TUFF_SLAB: Self = Self {
        id: 18,
        registry_key: "polished_tuff_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_tuff_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_tuff_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_TUFF_STAIRS: Self = Self {
        id: 19,
        registry_key: "polished_tuff_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_tuff_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_tuff_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POLISHED_TUFF_WALL: Self = Self {
        id: 20,
        registry_key: "polished_tuff_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.polished_tuff_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:polished_tuff_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POPPED_CHORUS_FRUIT: Self = Self {
        id: 1314,
        registry_key: "popped_chorus_fruit",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.popped_chorus_fruit",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:popped_chorus_fruit"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POPPY: Self = Self {
        id: 260,
        registry_key: "poppy",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.poppy",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:poppy"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PORKCHOP: Self = Self {
        id: 1011,
        registry_key: "porkchop",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.porkchop",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 3,
                    saturation: 1.8,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:porkchop"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POTATO: Self = Self {
        id: 1258,
        registry_key: "potato",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.potato",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 1,
                    saturation: 0.6,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:potato"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POTENT_SULFUR: Self = Self {
        id: 27,
        registry_key: "potent_sulfur",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.potent_sulfur",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:potent_sulfur"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POTION: Self = Self {
        id: 1150,
        registry_key: "potion",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.potion",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Drink,
                    sound_event: IdOr::Id(Sound::EntityGenericDrink),
                    consume_particles: false,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:potion"),
                },
            ),
            (Lore, &LoreImpl),
            (
                PotionContents,
                &PotionContentsImpl {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: Vec::new(),
                    custom_name: None,
                },
            ),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
            (UseRemainder, &UseRemainderImpl),
        ],
    };
    pub const POWDER_SNOW_BUCKET: Self = Self {
        id: 1043,
        registry_key: "powder_snow_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.powder_snow_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:powder_snow_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const POWERED_RAIL: Self = Self {
        id: 861,
        registry_key: "powered_rail",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.powered_rail",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:powered_rail"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE: Self = Self {
        id: 590,
        registry_key: "prismarine",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_BRICK_SLAB: Self = Self {
        id: 326,
        registry_key: "prismarine_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_BRICK_STAIRS: Self = Self {
        id: 594,
        registry_key: "prismarine_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_BRICKS: Self = Self {
        id: 591,
        registry_key: "prismarine_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_CRYSTALS: Self = Self {
        id: 1278,
        registry_key: "prismarine_crystals",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.prismarine_crystals",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_crystals"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_SHARD: Self = Self {
        id: 1277,
        registry_key: "prismarine_shard",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.prismarine_shard",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_shard"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_SLAB: Self = Self {
        id: 325,
        registry_key: "prismarine_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_STAIRS: Self = Self {
        id: 593,
        registry_key: "prismarine_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRISMARINE_WALL: Self = Self {
        id: 487,
        registry_key: "prismarine_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.prismarine_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prismarine_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PRIZE_POTTERY_SHERD: Self = Self {
        id: 1494,
        registry_key: "prize_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.prize_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:prize_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUFFERFISH: Self = Self {
        id: 1089,
        registry_key: "pufferfish",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pufferfish",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 1,
                    saturation: 0.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[ConsumeEffect::ApplyEffects((
                        Cow::Borrowed(&[
                            StatusEffectInstance {
                                effect_id: Cow::Borrowed("minecraft:poison"),
                                amplifier: 1i32,
                                duration: 1200i32,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                            },
                            StatusEffectInstance {
                                effect_id: Cow::Borrowed("minecraft:hunger"),
                                amplifier: 2i32,
                                duration: 300i32,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                            },
                            StatusEffectInstance {
                                effect_id: Cow::Borrowed("minecraft:nausea"),
                                amplifier: 0i32,
                                duration: 300i32,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                            },
                        ]),
                        1f32,
                    ))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pufferfish"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUFFERFISH_BUCKET: Self = Self {
        id: 1047,
        registry_key: "pufferfish_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pufferfish_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 1,
                    saturation: 0.2,
                    can_always_eat: false,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BucketEntityData, &BucketEntityDataImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pufferfish_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUFFERFISH_SPAWN_EGG: Self = Self {
        id: 1186,
        registry_key: "pufferfish_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pufferfish_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pufferfish_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUMPKIN: Self = Self {
        id: 384,
        registry_key: "pumpkin",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.pumpkin",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pumpkin"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUMPKIN_PIE: Self = Self {
        id: 1271,
        registry_key: "pumpkin_pie",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pumpkin_pie",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 8,
                    saturation: 4.8,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pumpkin_pie"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PUMPKIN_SEEDS: Self = Self {
        id: 1137,
        registry_key: "pumpkin_seeds",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.pumpkin_seeds",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:pumpkin_seeds"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_BANNER: Self = Self {
        id: 1306,
        registry_key: "purple_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_BED: Self = Self {
        id: 1125,
        registry_key: "purple_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_BUNDLE: Self = Self {
        id: 1076,
        registry_key: "purple_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.purple_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_CANDLE: Self = Self {
        id: 1440,
        registry_key: "purple_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_CARPET: Self = Self {
        id: 543,
        registry_key: "purple_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:purple_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_CONCRETE: Self = Self {
        id: 652,
        registry_key: "purple_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_CONCRETE_POWDER: Self = Self {
        id: 668,
        registry_key: "purple_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_DYE: Self = Self {
        id: 1105,
        registry_key: "purple_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.purple_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_GLAZED_TERRACOTTA: Self = Self {
        id: 636,
        registry_key: "purple_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_HARNESS: Self = Self {
        id: 876,
        registry_key: "purple_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.purple_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:purple_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_SHULKER_BOX: Self = Self {
        id: 620,
        registry_key: "purple_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_STAINED_GLASS: Self = Self {
        id: 568,
        registry_key: "purple_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_STAINED_GLASS_PANE: Self = Self {
        id: 584,
        registry_key: "purple_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_TERRACOTTA: Self = Self {
        id: 524,
        registry_key: "purple_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPLE_WOOL: Self = Self {
        id: 250,
        registry_key: "purple_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purple_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purple_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPUR_BLOCK: Self = Self {
        id: 354,
        registry_key: "purpur_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purpur_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purpur_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPUR_PILLAR: Self = Self {
        id: 355,
        registry_key: "purpur_pillar",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purpur_pillar",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purpur_pillar"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPUR_SLAB: Self = Self {
        id: 324,
        registry_key: "purpur_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purpur_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purpur_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const PURPUR_STAIRS: Self = Self {
        id: 356,
        registry_key: "purpur_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.purpur_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:purpur_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ: Self = Self {
        id: 929,
        registry_key: "quartz",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.quartz",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesTrimMaterial, &ProvidesTrimMaterialImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ_BLOCK: Self = Self {
        id: 510,
        registry_key: "quartz_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.quartz_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ_BRICKS: Self = Self {
        id: 511,
        registry_key: "quartz_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.quartz_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ_PILLAR: Self = Self {
        id: 512,
        registry_key: "quartz_pillar",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.quartz_pillar",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz_pillar"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ_SLAB: Self = Self {
        id: 321,
        registry_key: "quartz_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.quartz_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const QUARTZ_STAIRS: Self = Self {
        id: 513,
        registry_key: "quartz_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.quartz_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:quartz_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RABBIT: Self = Self {
        id: 1279,
        registry_key: "rabbit",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rabbit",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 3,
                    saturation: 1.8,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rabbit"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RABBIT_FOOT: Self = Self {
        id: 1282,
        registry_key: "rabbit_foot",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rabbit_foot",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rabbit_foot"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RABBIT_HIDE: Self = Self {
        id: 1283,
        registry_key: "rabbit_hide",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rabbit_hide",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rabbit_hide"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RABBIT_SPAWN_EGG: Self = Self {
        id: 1179,
        registry_key: "rabbit_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rabbit_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rabbit_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RABBIT_STEW: Self = Self {
        id: 1281,
        registry_key: "rabbit_stew",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rabbit_stew",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 10,
                    saturation: 12.0,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rabbit_stew"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
            (UseRemainder, &UseRemainderImpl),
        ],
    };
    pub const RAIL: Self = Self {
        id: 863,
        registry_key: "rail",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.rail",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rail"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAISER_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1473,
        registry_key: "raiser_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.raiser_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raiser_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAVAGER_SPAWN_EGG: Self = Self {
        id: 1230,
        registry_key: "ravager_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.ravager_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ravager_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_COPPER: Self = Self {
        id: 933,
        registry_key: "raw_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.raw_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_COPPER_BLOCK: Self = Self {
        id: 112,
        registry_key: "raw_copper_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.raw_copper_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_copper_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_GOLD: Self = Self {
        id: 935,
        registry_key: "raw_gold",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.raw_gold",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_gold"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_GOLD_BLOCK: Self = Self {
        id: 113,
        registry_key: "raw_gold_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.raw_gold_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_gold_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_IRON: Self = Self {
        id: 931,
        registry_key: "raw_iron",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.raw_iron",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_iron"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RAW_IRON_BLOCK: Self = Self {
        id: 111,
        registry_key: "raw_iron_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.raw_iron_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:raw_iron_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RECOVERY_COMPASS: Self = Self {
        id: 1064,
        registry_key: "recovery_compass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.recovery_compass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:recovery_compass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_BANNER: Self = Self {
        id: 1310,
        registry_key: "red_banner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_banner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_banner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_BED: Self = Self {
        id: 1129,
        registry_key: "red_bed",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_bed",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_bed"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_BUNDLE: Self = Self {
        id: 1080,
        registry_key: "red_bundle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.red_bundle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BundleContents, &BundleContentsImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_bundle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_CANDLE: Self = Self {
        id: 1444,
        registry_key: "red_candle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_candle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_candle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_CARPET: Self = Self {
        id: 547,
        registry_key: "red_carpet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_carpet",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityLlamaSwag),
                    asset_id: Some(Cow::Borrowed("minecraft:red_carpet")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[
                        &crate::entity_type::EntityType::LLAMA,
                        &crate::entity_type::EntityType::TRADER_LLAMA,
                    ]))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemLlamaCarpetUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_carpet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_CONCRETE: Self = Self {
        id: 656,
        registry_key: "red_concrete",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_concrete",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_concrete"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_CONCRETE_POWDER: Self = Self {
        id: 672,
        registry_key: "red_concrete_powder",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_concrete_powder",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_concrete_powder"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_DYE: Self = Self {
        id: 1109,
        registry_key: "red_dye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.red_dye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Dye, &DyeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_dye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_GLAZED_TERRACOTTA: Self = Self {
        id: 640,
        registry_key: "red_glazed_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_glazed_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_glazed_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_HARNESS: Self = Self {
        id: 880,
        registry_key: "red_harness",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.red_harness",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::BODY,
                    equip_sound: IdOr::Id(Sound::EntityHappyGhastEquip),
                    asset_id: Some(Cow::Borrowed("minecraft:red_harness")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_harness"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::EntityHappyGhastUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_harness"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_MUSHROOM: Self = Self {
        id: 276,
        registry_key: "red_mushroom",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_mushroom",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_mushroom"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_MUSHROOM_BLOCK: Self = Self {
        id: 416,
        registry_key: "red_mushroom_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_mushroom_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_mushroom_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_NETHER_BRICK_SLAB: Self = Self {
        id: 737,
        registry_key: "red_nether_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_nether_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_nether_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_NETHER_BRICK_STAIRS: Self = Self {
        id: 720,
        registry_key: "red_nether_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_nether_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_nether_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_NETHER_BRICK_WALL: Self = Self {
        id: 495,
        registry_key: "red_nether_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_nether_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_nether_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_NETHER_BRICKS: Self = Self {
        id: 606,
        registry_key: "red_nether_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_nether_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_nether_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SAND: Self = Self {
        id: 89,
        registry_key: "red_sand",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_sand",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_sand"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SANDSTONE: Self = Self {
        id: 597,
        registry_key: "red_sandstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_sandstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_sandstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SANDSTONE_SLAB: Self = Self {
        id: 322,
        registry_key: "red_sandstone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_sandstone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_sandstone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SANDSTONE_STAIRS: Self = Self {
        id: 600,
        registry_key: "red_sandstone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_sandstone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_sandstone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SANDSTONE_WALL: Self = Self {
        id: 488,
        registry_key: "red_sandstone_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_sandstone_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_sandstone_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_SHULKER_BOX: Self = Self {
        id: 624,
        registry_key: "red_shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_STAINED_GLASS: Self = Self {
        id: 572,
        registry_key: "red_stained_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_stained_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_stained_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_STAINED_GLASS_PANE: Self = Self {
        id: 588,
        registry_key: "red_stained_glass_pane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_stained_glass_pane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_stained_glass_pane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_TERRACOTTA: Self = Self {
        id: 528,
        registry_key: "red_terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_TULIP: Self = Self {
        id: 264,
        registry_key: "red_tulip",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_tulip",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_tulip"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RED_WOOL: Self = Self {
        id: 254,
        registry_key: "red_wool",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.red_wool",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:red_wool"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REDSTONE: Self = Self {
        id: 745,
        registry_key: "redstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.redstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:redstone"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesTrimMaterial, &ProvidesTrimMaterialImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REDSTONE_BLOCK: Self = Self {
        id: 747,
        registry_key: "redstone_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.redstone_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:redstone_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REDSTONE_LAMP: Self = Self {
        id: 775,
        registry_key: "redstone_lamp",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.redstone_lamp",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:redstone_lamp"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REDSTONE_ORE: Self = Self {
        id: 99,
        registry_key: "redstone_ore",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.redstone_ore",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:redstone_ore"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REDSTONE_TORCH: Self = Self {
        id: 746,
        registry_key: "redstone_torch",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.redstone_torch",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:redstone_torch"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REINFORCED_DEEPSLATE: Self = Self {
        id: 414,
        registry_key: "reinforced_deepslate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.reinforced_deepslate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:reinforced_deepslate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REPEATER: Self = Self {
        id: 748,
        registry_key: "repeater",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.repeater",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:repeater"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const REPEATING_COMMAND_BLOCK: Self = Self {
        id: 601,
        registry_key: "repeating_command_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.repeating_command_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:repeating_command_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BLOCK: Self = Self {
        id: 441,
        registry_key: "resin_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.resin_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BRICK: Self = Self {
        id: 1276,
        registry_key: "resin_brick",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.resin_brick",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_brick"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesTrimMaterial, &ProvidesTrimMaterialImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BRICK_SLAB: Self = Self {
        id: 444,
        registry_key: "resin_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.resin_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BRICK_STAIRS: Self = Self {
        id: 443,
        registry_key: "resin_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.resin_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BRICK_WALL: Self = Self {
        id: 445,
        registry_key: "resin_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.resin_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_BRICKS: Self = Self {
        id: 442,
        registry_key: "resin_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.resin_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESIN_CLUMP: Self = Self {
        id: 440,
        registry_key: "resin_clump",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.resin_clump",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:resin_clump"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RESPAWN_ANCHOR: Self = Self {
        id: 1428,
        registry_key: "respawn_anchor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.respawn_anchor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:respawn_anchor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const RIB_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1468,
        registry_key: "rib_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rib_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rib_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ROOTED_DIRT: Self = Self {
        id: 58,
        registry_key: "rooted_dirt",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.rooted_dirt",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rooted_dirt"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ROSE_BUSH: Self = Self {
        id: 554,
        registry_key: "rose_bush",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.rose_bush",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rose_bush"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const ROTTEN_FLESH: Self = Self {
        id: 1143,
        registry_key: "rotten_flesh",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.rotten_flesh",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 4,
                    saturation: 0.8,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[ConsumeEffect::ApplyEffects((
                        Cow::Borrowed(&[StatusEffectInstance {
                            effect_id: Cow::Borrowed("minecraft:hunger"),
                            amplifier: 0i32,
                            duration: 600i32,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                        }]),
                        0.8f32,
                    ))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:rotten_flesh"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SADDLE: Self = Self {
        id: 865,
        registry_key: "saddle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.saddle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::SADDLE,
                    equip_sound: IdOr::Id(Sound::EntityHorseSaddle),
                    asset_id: Some(Cow::Borrowed("minecraft:saddle")),
                    camera_overlay: None,
                    allowed_entities: Some(IDSet::Tag(Cow::Borrowed("can_equip_saddle"))),
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: true,
                    can_be_sheared: true,
                    shearing_sound: IdOr::Id(Sound::ItemSaddleUnequip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:saddle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SALMON: Self = Self {
        id: 1087,
        registry_key: "salmon",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.salmon",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 0.4,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:salmon"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SALMON_BUCKET: Self = Self {
        id: 1048,
        registry_key: "salmon_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.salmon_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 0.4,
                    can_always_eat: false,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BucketEntityData, &BucketEntityDataImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:salmon_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SALMON_SPAWN_EGG: Self = Self {
        id: 1187,
        registry_key: "salmon_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.salmon_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:salmon_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SAND: Self = Self {
        id: 86,
        registry_key: "sand",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sand",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sand"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SANDSTONE: Self = Self {
        id: 225,
        registry_key: "sandstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sandstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sandstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SANDSTONE_SLAB: Self = Self {
        id: 313,
        registry_key: "sandstone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sandstone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sandstone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SANDSTONE_STAIRS: Self = Self {
        id: 466,
        registry_key: "sandstone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sandstone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sandstone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SANDSTONE_WALL: Self = Self {
        id: 496,
        registry_key: "sandstone_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sandstone_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sandstone_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCAFFOLDING: Self = Self {
        id: 744,
        registry_key: "scaffolding",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.scaffolding",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:scaffolding"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCRAPE_POTTERY_SHERD: Self = Self {
        id: 1495,
        registry_key: "scrape_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.scrape_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:scrape_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCULK: Self = Self {
        id: 457,
        registry_key: "sculk",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sculk",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sculk"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCULK_CATALYST: Self = Self {
        id: 459,
        registry_key: "sculk_catalyst",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sculk_catalyst",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sculk_catalyst"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCULK_SENSOR: Self = Self {
        id: 770,
        registry_key: "sculk_sensor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sculk_sensor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sculk_sensor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCULK_SHRIEKER: Self = Self {
        id: 460,
        registry_key: "sculk_shrieker",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sculk_shrieker",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sculk_shrieker"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SCULK_VEIN: Self = Self {
        id: 458,
        registry_key: "sculk_vein",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sculk_vein",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sculk_vein"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SEA_LANTERN: Self = Self {
        id: 596,
        registry_key: "sea_lantern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sea_lantern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sea_lantern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SEA_PICKLE: Self = Self {
        id: 239,
        registry_key: "sea_pickle",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sea_pickle",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sea_pickle"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SEAGRASS: Self = Self {
        id: 238,
        registry_key: "seagrass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.seagrass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:seagrass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1459,
        registry_key: "sentry_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sentry_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sentry_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1471,
        registry_key: "shaper_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shaper_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shaper_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHEAF_POTTERY_SHERD: Self = Self {
        id: 1496,
        registry_key: "sheaf_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sheaf_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sheaf_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHEARS: Self = Self {
        id: 1134,
        registry_key: "shears",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shears",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 238 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: IDs(Cow::Borrowed(&[&Block::COBWEB])),
                            speed: Some(15f32),
                            correct_for_drops: Some(true),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:shears_extreme_breaking_speed")),
                            speed: Some(15f32),
                            correct_for_drops: None,
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:shears_major_breaking_speed")),
                            speed: Some(5f32),
                            correct_for_drops: None,
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:shears_minor_breaking_speed")),
                            speed: Some(2f32),
                            correct_for_drops: None,
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shears"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHEEP_SPAWN_EGG: Self = Self {
        id: 1162,
        registry_key: "sheep_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sheep_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sheep_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHELTER_POTTERY_SHERD: Self = Self {
        id: 1497,
        registry_key: "shelter_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shelter_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shelter_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHIELD: Self = Self {
        id: 1325,
        registry_key: "shield",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shield",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 336 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BlocksAttacks, &BlocksAttacksImpl),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::OFF_HAND,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipGeneric),
                    asset_id: None,
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: false,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (BannerPatterns, &BannerPatternsImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shield"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHORT_DRY_GRASS: Self = Self {
        id: 236,
        registry_key: "short_dry_grass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.short_dry_grass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:short_dry_grass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHORT_GRASS: Self = Self {
        id: 229,
        registry_key: "short_grass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.short_grass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:short_grass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHROOMLIGHT: Self = Self {
        id: 1408,
        registry_key: "shroomlight",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.shroomlight",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shroomlight"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHULKER_BOX: Self = Self {
        id: 609,
        registry_key: "shulker_box",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.shulker_box",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shulker_box"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHULKER_SHELL: Self = Self {
        id: 1334,
        registry_key: "shulker_shell",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shulker_shell",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shulker_shell"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SHULKER_SPAWN_EGG: Self = Self {
        id: 1246,
        registry_key: "shulker_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.shulker_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:shulker_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1472,
        registry_key: "silence_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.silence_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:silence_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SILVERFISH_SPAWN_EGG: Self = Self {
        id: 1224,
        registry_key: "silverfish_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.silverfish_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:silverfish_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SKELETON_HORSE_SPAWN_EGG: Self = Self {
        id: 1208,
        registry_key: "skeleton_horse_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.skeleton_horse_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:skeleton_horse_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SKELETON_SKULL: Self = Self {
        id: 1263,
        registry_key: "skeleton_skull",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.skeleton_skull",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[Modifier {
                        r#type: &Attributes::WAYPOINT_TRANSMIT_RANGE,
                        id: "minecraft:waypoint_transmit_range_hide",
                        amount: -1f64,
                        operation: Operation::AddMultipliedTotal,
                        slot: AttributeModifierSlot::Head,
                    }]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipGeneric),
                    asset_id: None,
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: false,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:skeleton_skull"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SKELETON_SPAWN_EGG: Self = Self {
        id: 1207,
        registry_key: "skeleton_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.skeleton_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:skeleton_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SKULL_BANNER_PATTERN: Self = Self {
        id: 1375,
        registry_key: "skull_banner_pattern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.skull_banner_pattern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:skull_banner_pattern"),
                },
            ),
            (Lore, &LoreImpl),
            (ProvidesBannerPatterns, &ProvidesBannerPatternsImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SKULL_POTTERY_SHERD: Self = Self {
        id: 1498,
        registry_key: "skull_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.skull_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:skull_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SLIME_BALL: Self = Self {
        id: 1059,
        registry_key: "slime_ball",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.slime_ball",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:slime_ball"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SLIME_BLOCK: Self = Self {
        id: 752,
        registry_key: "slime_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.slime_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:slime_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SLIME_SPAWN_EGG: Self = Self {
        id: 1225,
        registry_key: "slime_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.slime_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:slime_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMALL_AMETHYST_BUD: Self = Self {
        id: 1446,
        registry_key: "small_amethyst_bud",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.small_amethyst_bud",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:small_amethyst_bud"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMALL_DRIPLEAF: Self = Self {
        id: 296,
        registry_key: "small_dripleaf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.small_dripleaf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:small_dripleaf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMITHING_TABLE: Self = Self {
        id: 1391,
        registry_key: "smithing_table",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smithing_table",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smithing_table"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOKER: Self = Self {
        id: 1386,
        registry_key: "smoker",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smoker",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smoker"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_BASALT: Self = Self {
        id: 392,
        registry_key: "smooth_basalt",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_basalt",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_basalt"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_QUARTZ: Self = Self {
        id: 328,
        registry_key: "smooth_quartz",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_quartz",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_quartz"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_QUARTZ_SLAB: Self = Self {
        id: 734,
        registry_key: "smooth_quartz_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_quartz_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_quartz_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_QUARTZ_STAIRS: Self = Self {
        id: 717,
        registry_key: "smooth_quartz_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_quartz_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_quartz_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_RED_SANDSTONE: Self = Self {
        id: 329,
        registry_key: "smooth_red_sandstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_red_sandstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_red_sandstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_RED_SANDSTONE_SLAB: Self = Self {
        id: 728,
        registry_key: "smooth_red_sandstone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_red_sandstone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_red_sandstone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_RED_SANDSTONE_STAIRS: Self = Self {
        id: 710,
        registry_key: "smooth_red_sandstone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_red_sandstone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_red_sandstone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_SANDSTONE: Self = Self {
        id: 330,
        registry_key: "smooth_sandstone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_sandstone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_sandstone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_SANDSTONE_SLAB: Self = Self {
        id: 733,
        registry_key: "smooth_sandstone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_sandstone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_sandstone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_SANDSTONE_STAIRS: Self = Self {
        id: 716,
        registry_key: "smooth_sandstone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_sandstone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_sandstone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_STONE: Self = Self {
        id: 331,
        registry_key: "smooth_stone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_stone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_stone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SMOOTH_STONE_SLAB: Self = Self {
        id: 312,
        registry_key: "smooth_stone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.smooth_stone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:smooth_stone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNIFFER_EGG: Self = Self {
        id: 675,
        registry_key: "sniffer_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sniffer_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sniffer_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNIFFER_SPAWN_EGG: Self = Self {
        id: 1194,
        registry_key: "sniffer_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sniffer_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sniffer_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNORT_POTTERY_SHERD: Self = Self {
        id: 1499,
        registry_key: "snort_pottery_sherd",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.snort_pottery_sherd",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snort_pottery_sherd"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1467,
        registry_key: "snout_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.snout_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snout_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNOW: Self = Self {
        id: 365,
        registry_key: "snow",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.snow",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snow"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNOW_BLOCK: Self = Self {
        id: 367,
        registry_key: "snow_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.snow_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snow_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNOW_GOLEM_SPAWN_EGG: Self = Self {
        id: 1198,
        registry_key: "snow_golem_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.snow_golem_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snow_golem_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SNOWBALL: Self = Self {
        id: 1044,
        registry_key: "snowball",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.snowball",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:snowball"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SOUL_CAMPFIRE: Self = Self {
        id: 1407,
        registry_key: "soul_campfire",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.soul_campfire",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:soul_campfire"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SOUL_LANTERN: Self = Self {
        id: 1395,
        registry_key: "soul_lantern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.soul_lantern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:soul_lantern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SOUL_SAND: Self = Self {
        id: 388,
        registry_key: "soul_sand",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.soul_sand",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:soul_sand"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SOUL_SOIL: Self = Self {
        id: 389,
        registry_key: "soul_soil",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.soul_soil",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:soul_soil"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SOUL_TORCH: Self = Self {
        id: 393,
        registry_key: "soul_torch",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.soul_torch",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:soul_torch"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPAWNER: Self = Self {
        id: 357,
        registry_key: "spawner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spawner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spawner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPECTRAL_ARROW: Self = Self {
        id: 1322,
        registry_key: "spectral_arrow",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spectral_arrow",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spectral_arrow"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPIDER_EYE: Self = Self {
        id: 1151,
        registry_key: "spider_eye",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spider_eye",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 3.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[ConsumeEffect::ApplyEffects((
                        Cow::Borrowed(&[StatusEffectInstance {
                            effect_id: Cow::Borrowed("minecraft:poison"),
                            amplifier: 0i32,
                            duration: 100i32,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                        }]),
                        1f32,
                    ))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spider_eye"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPIDER_SPAWN_EGG: Self = Self {
        id: 1217,
        registry_key: "spider_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spider_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spider_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1469,
        registry_key: "spire_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spire_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spire_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPLASH_POTION: Self = Self {
        id: 1321,
        registry_key: "splash_potion",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.splash_potion",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:splash_potion"),
                },
            ),
            (Lore, &LoreImpl),
            (
                PotionContents,
                &PotionContentsImpl {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: Vec::new(),
                    custom_name: None,
                },
            ),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPONGE: Self = Self {
        id: 220,
        registry_key: "sponge",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sponge",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sponge"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPORE_BLOSSOM: Self = Self {
        id: 274,
        registry_key: "spore_blossom",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spore_blossom",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spore_blossom"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_BOAT: Self = Self {
        id: 893,
        registry_key: "spruce_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spruce_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_BUTTON: Self = Self {
        id: 780,
        registry_key: "spruce_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_CHEST_BOAT: Self = Self {
        id: 894,
        registry_key: "spruce_chest_boat",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spruce_chest_boat",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_chest_boat"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_DOOR: Self = Self {
        id: 809,
        registry_key: "spruce_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_FENCE: Self = Self {
        id: 373,
        registry_key: "spruce_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_FENCE_GATE: Self = Self {
        id: 850,
        registry_key: "spruce_fence_gate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_fence_gate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_fence_gate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_HANGING_SIGN: Self = Self {
        id: 1029,
        registry_key: "spruce_hanging_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_hanging_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_hanging_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_LEAVES: Self = Self {
        id: 210,
        registry_key: "spruce_leaves",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_leaves",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_leaves"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_LOG: Self = Self {
        id: 162,
        registry_key: "spruce_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_PLANKS: Self = Self {
        id: 64,
        registry_key: "spruce_planks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_planks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_planks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_PRESSURE_PLATE: Self = Self {
        id: 796,
        registry_key: "spruce_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_SAPLING: Self = Self {
        id: 77,
        registry_key: "spruce_sapling",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_sapling",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_sapling"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_SHELF: Self = Self {
        id: 343,
        registry_key: "spruce_shelf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_shelf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_shelf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_SIGN: Self = Self {
        id: 1017,
        registry_key: "spruce_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_SLAB: Self = Self {
        id: 299,
        registry_key: "spruce_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_STAIRS: Self = Self {
        id: 470,
        registry_key: "spruce_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_TRAPDOOR: Self = Self {
        id: 830,
        registry_key: "spruce_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPRUCE_WOOD: Self = Self {
        id: 199,
        registry_key: "spruce_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.spruce_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spruce_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SPYGLASS: Self = Self {
        id: 1084,
        registry_key: "spyglass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.spyglass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:spyglass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SQUID_SPAWN_EGG: Self = Self {
        id: 1188,
        registry_key: "squid_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.squid_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:squid_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STICK: Self = Self {
        id: 974,
        registry_key: "stick",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stick",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stick"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STICKY_PISTON: Self = Self {
        id: 751,
        registry_key: "sticky_piston",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sticky_piston",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sticky_piston"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE: Self = Self {
        id: 1,
        registry_key: "stone",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_AXE: Self = Self {
        id: 952,
        registry_key: "stone_axe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_axe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 8f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3.200000047683716f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_stone_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/axe")),
                            speed: Some(4f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_axe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_BRICK_SLAB: Self = Self {
        id: 318,
        registry_key: "stone_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_BRICK_STAIRS: Self = Self {
        id: 448,
        registry_key: "stone_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_BRICK_WALL: Self = Self {
        id: 491,
        registry_key: "stone_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_BRICKS: Self = Self {
        id: 403,
        registry_key: "stone_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_BUTTON: Self = Self {
        id: 777,
        registry_key: "stone_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_HOE: Self = Self {
        id: 953,
        registry_key: "stone_hoe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_hoe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_stone_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/hoe")),
                            speed: Some(4f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_hoe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_PICKAXE: Self = Self {
        id: 951,
        registry_key: "stone_pickaxe",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_pickaxe",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 2f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.799999952316284f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_stone_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/pickaxe")),
                            speed: Some(4f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_pickaxe"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_PRESSURE_PLATE: Self = Self {
        id: 791,
        registry_key: "stone_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_SHOVEL: Self = Self {
        id: 950,
        registry_key: "stone_shovel",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_shovel",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 2.5f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -3f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:incorrect_for_stone_tool")),
                            speed: None,
                            correct_for_drops: Some(false),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:mineable/shovel")),
                            speed: Some(4f32),
                            correct_for_drops: Some(true),
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 1,
                    can_destroy_blocks_in_creative: true,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 2,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_shovel"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_SLAB: Self = Self {
        id: 311,
        registry_key: "stone_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_SPEAR: Self = Self {
        id: 1327,
        registry_key: "stone_spear",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_spear",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 1f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.666666626930237f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (AttackRange, &AttackRangeImpl),
            (BreakSound, &BreakSoundImpl),
            (DamageType, &DamageTypeImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_spear"),
                },
            ),
            (KineticWeapon, &KineticWeaponImpl),
            (Lore, &LoreImpl),
            (MinimumAttackCharge, &MinimumAttackChargeImpl),
            (PiercingWeapon, &PiercingWeaponImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_STAIRS: Self = Self {
        id: 715,
        registry_key: "stone_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stone_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONE_SWORD: Self = Self {
        id: 949,
        registry_key: "stone_sword",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stone_sword",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 131 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 4f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.4000000953674316f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[
                        ToolRule {
                            blocks: IDs(Cow::Borrowed(&[&Block::COBWEB])),
                            speed: Some(15f32),
                            correct_for_drops: Some(true),
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:sword_instantly_mines")),
                            speed: Some(340282350000000000000000000000000000000f32),
                            correct_for_drops: None,
                        },
                        ToolRule {
                            blocks: Tag(Cow::Borrowed("minecraft:sword_efficient")),
                            speed: Some(1.5f32),
                            correct_for_drops: None,
                        },
                    ]),
                    default_mining_speed: 1.0,
                    damage_per_block: 2,
                    can_destroy_blocks_in_creative: false,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 5 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stone_sword"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STONECUTTER: Self = Self {
        id: 1392,
        registry_key: "stonecutter",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stonecutter",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stonecutter"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRAY_SPAWN_EGG: Self = Self {
        id: 1209,
        registry_key: "stray_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.stray_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stray_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIDER_SPAWN_EGG: Self = Self {
        id: 1240,
        registry_key: "strider_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.strider_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:strider_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRING: Self = Self {
        id: 976,
        registry_key: "string",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.string",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:string"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_ACACIA_LOG: Self = Self {
        id: 179,
        registry_key: "stripped_acacia_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_acacia_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_acacia_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_ACACIA_WOOD: Self = Self {
        id: 190,
        registry_key: "stripped_acacia_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_acacia_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_acacia_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_BAMBOO_BLOCK: Self = Self {
        id: 197,
        registry_key: "stripped_bamboo_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_bamboo_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_bamboo_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_BIRCH_LOG: Self = Self {
        id: 177,
        registry_key: "stripped_birch_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_birch_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_birch_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_BIRCH_WOOD: Self = Self {
        id: 188,
        registry_key: "stripped_birch_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_birch_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_birch_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_CHERRY_LOG: Self = Self {
        id: 180,
        registry_key: "stripped_cherry_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_cherry_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_cherry_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_CHERRY_WOOD: Self = Self {
        id: 191,
        registry_key: "stripped_cherry_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_cherry_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_cherry_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_CRIMSON_HYPHAE: Self = Self {
        id: 195,
        registry_key: "stripped_crimson_hyphae",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_crimson_hyphae",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_crimson_hyphae"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_CRIMSON_STEM: Self = Self {
        id: 184,
        registry_key: "stripped_crimson_stem",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_crimson_stem",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_crimson_stem"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_DARK_OAK_LOG: Self = Self {
        id: 181,
        registry_key: "stripped_dark_oak_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_dark_oak_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_dark_oak_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_DARK_OAK_WOOD: Self = Self {
        id: 192,
        registry_key: "stripped_dark_oak_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_dark_oak_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_dark_oak_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_JUNGLE_LOG: Self = Self {
        id: 178,
        registry_key: "stripped_jungle_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_jungle_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_jungle_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_JUNGLE_WOOD: Self = Self {
        id: 189,
        registry_key: "stripped_jungle_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_jungle_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_jungle_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_MANGROVE_LOG: Self = Self {
        id: 183,
        registry_key: "stripped_mangrove_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_mangrove_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_mangrove_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_MANGROVE_WOOD: Self = Self {
        id: 194,
        registry_key: "stripped_mangrove_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_mangrove_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_mangrove_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_OAK_LOG: Self = Self {
        id: 175,
        registry_key: "stripped_oak_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_oak_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_oak_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_OAK_WOOD: Self = Self {
        id: 186,
        registry_key: "stripped_oak_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_oak_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_oak_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_PALE_OAK_LOG: Self = Self {
        id: 182,
        registry_key: "stripped_pale_oak_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_pale_oak_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_pale_oak_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_PALE_OAK_WOOD: Self = Self {
        id: 193,
        registry_key: "stripped_pale_oak_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_pale_oak_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_pale_oak_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_SPRUCE_LOG: Self = Self {
        id: 176,
        registry_key: "stripped_spruce_log",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_spruce_log",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_spruce_log"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_SPRUCE_WOOD: Self = Self {
        id: 187,
        registry_key: "stripped_spruce_wood",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_spruce_wood",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_spruce_wood"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_WARPED_HYPHAE: Self = Self {
        id: 196,
        registry_key: "stripped_warped_hyphae",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_warped_hyphae",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_warped_hyphae"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRIPPED_WARPED_STEM: Self = Self {
        id: 185,
        registry_key: "stripped_warped_stem",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.stripped_warped_stem",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:stripped_warped_stem"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRUCTURE_BLOCK: Self = Self {
        id: 911,
        registry_key: "structure_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.structure_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:structure_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const STRUCTURE_VOID: Self = Self {
        id: 608,
        registry_key: "structure_void",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.structure_void",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:structure_void"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUGAR: Self = Self {
        id: 1113,
        registry_key: "sugar",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sugar",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sugar"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUGAR_CANE: Self = Self {
        id: 284,
        registry_key: "sugar_cane",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sugar_cane",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sugar_cane"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR: Self = Self {
        id: 26,
        registry_key: "sulfur",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_BRICK_SLAB: Self = Self {
        id: 36,
        registry_key: "sulfur_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_BRICK_STAIRS: Self = Self {
        id: 37,
        registry_key: "sulfur_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_BRICK_WALL: Self = Self {
        id: 38,
        registry_key: "sulfur_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_BRICKS: Self = Self {
        id: 35,
        registry_key: "sulfur_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_CUBE_BUCKET: Self = Self {
        id: 1052,
        registry_key: "sulfur_cube_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sulfur_cube_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BucketEntityData, &BucketEntityDataImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_cube_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_CUBE_SPAWN_EGG: Self = Self {
        id: 1195,
        registry_key: "sulfur_cube_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sulfur_cube_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_cube_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_SLAB: Self = Self {
        id: 28,
        registry_key: "sulfur_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_SPIKE: Self = Self {
        id: 1451,
        registry_key: "sulfur_spike",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_spike",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_spike"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_STAIRS: Self = Self {
        id: 29,
        registry_key: "sulfur_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SULFUR_WALL: Self = Self {
        id: 30,
        registry_key: "sulfur_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sulfur_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sulfur_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUNFLOWER: Self = Self {
        id: 552,
        registry_key: "sunflower",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.sunflower",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sunflower"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUSPICIOUS_GRAVEL: Self = Self {
        id: 88,
        registry_key: "suspicious_gravel",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.suspicious_gravel",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:suspicious_gravel"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUSPICIOUS_SAND: Self = Self {
        id: 87,
        registry_key: "suspicious_sand",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.suspicious_sand",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:suspicious_sand"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const SUSPICIOUS_STEW: Self = Self {
        id: 1371,
        registry_key: "suspicious_stew",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.suspicious_stew",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 6,
                    saturation: 7.2,
                    can_always_eat: true,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:suspicious_stew"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SuspiciousStewEffects, &SuspiciousStewEffectsImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
            (UseRemainder, &UseRemainderImpl),
        ],
    };
    pub const SWEET_BERRIES: Self = Self {
        id: 1404,
        registry_key: "sweet_berries",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.sweet_berries",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 2,
                    saturation: 0.4,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:sweet_berries"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TADPOLE_BUCKET: Self = Self {
        id: 1053,
        registry_key: "tadpole_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tadpole_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BucketEntityData, &BucketEntityDataImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tadpole_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TADPOLE_SPAWN_EGG: Self = Self {
        id: 1189,
        registry_key: "tadpole_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tadpole_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tadpole_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TALL_DRY_GRASS: Self = Self {
        id: 237,
        registry_key: "tall_dry_grass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tall_dry_grass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tall_dry_grass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TALL_GRASS: Self = Self {
        id: 556,
        registry_key: "tall_grass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tall_grass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tall_grass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TARGET: Self = Self {
        id: 759,
        registry_key: "target",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.target",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:target"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TERRACOTTA: Self = Self {
        id: 549,
        registry_key: "terracotta",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.terracotta",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:terracotta"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TEST_BLOCK: Self = Self {
        id: 913,
        registry_key: "test_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.test_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(Cow::Borrowed("mode"), Cow::Borrowed("start"))]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:test_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TEST_INSTANCE_BLOCK: Self = Self {
        id: 914,
        registry_key: "test_instance_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.test_instance_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:test_instance_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TIDE_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1466,
        registry_key: "tide_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tide_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tide_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TINTED_GLASS: Self = Self {
        id: 223,
        registry_key: "tinted_glass",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tinted_glass",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tinted_glass"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TIPPED_ARROW: Self = Self {
        id: 1323,
        registry_key: "tipped_arrow",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tipped_arrow",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tipped_arrow"),
                },
            ),
            (Lore, &LoreImpl),
            (
                PotionContents,
                &PotionContentsImpl {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: Vec::new(),
                    custom_name: None,
                },
            ),
            (PotionDurationScale, &PotionDurationScaleImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TNT: Self = Self {
        id: 774,
        registry_key: "tnt",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tnt",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tnt"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TNT_MINECART: Self = Self {
        id: 885,
        registry_key: "tnt_minecart",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tnt_minecart",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tnt_minecart"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TORCH: Self = Self {
        id: 350,
        registry_key: "torch",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.torch",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:torch"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TORCHFLOWER: Self = Self {
        id: 272,
        registry_key: "torchflower",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.torchflower",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:torchflower"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TORCHFLOWER_SEEDS: Self = Self {
        id: 1315,
        registry_key: "torchflower_seeds",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.torchflower_seeds",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:torchflower_seeds"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TOTEM_OF_UNDYING: Self = Self {
        id: 1333,
        registry_key: "totem_of_undying",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.totem_of_undying",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (DeathProtection, &DeathProtectionImpl),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:totem_of_undying"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRADER_LLAMA_SPAWN_EGG: Self = Self {
        id: 1199,
        registry_key: "trader_llama_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.trader_llama_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:trader_llama_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRAPPED_CHEST: Self = Self {
        id: 773,
        registry_key: "trapped_chest",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.trapped_chest",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:trapped_chest"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRIAL_KEY: Self = Self {
        id: 1533,
        registry_key: "trial_key",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.trial_key",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:trial_key"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRIAL_SPAWNER: Self = Self {
        id: 1532,
        registry_key: "trial_spawner",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.trial_spawner",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:trial_spawner"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRIDENT: Self = Self {
        id: 1362,
        registry_key: "trident",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.trident",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 250 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ATTACK_DAMAGE,
                            id: "minecraft:base_attack_damage",
                            amount: 8f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                        Modifier {
                            r#type: &Attributes::ATTACK_SPEED,
                            id: "minecraft:base_attack_speed",
                            amount: -2.9000000953674316f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::MainHand,
                        },
                    ]),
                },
            ),
            (
                Tool,
                &ToolImpl {
                    rules: Cow::Borrowed(&[]),
                    default_mining_speed: 1.0,
                    damage_per_block: 2,
                    can_destroy_blocks_in_creative: false,
                },
            ),
            (
                Weapon,
                &WeaponImpl {
                    item_damage_per_attack: 1,
                },
            ),
            (Enchantable, &EnchantableImpl { value: 1 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:trident"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TRIPWIRE_HOOK: Self = Self {
        id: 772,
        registry_key: "tripwire_hook",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tripwire_hook",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tripwire_hook"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TROPICAL_FISH: Self = Self {
        id: 1088,
        registry_key: "tropical_fish",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tropical_fish",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 1,
                    saturation: 0.2,
                    can_always_eat: false,
                },
            ),
            (
                Consumable,
                &ConsumableImpl {
                    consume_seconds: 1.6,
                    animation: ConsumeAnimation::Eat,
                    sound_event: IdOr::Id(Sound::EntityGenericEat),
                    consume_particles: true,
                    effects: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tropical_fish"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TROPICAL_FISH_BUCKET: Self = Self {
        id: 1050,
        registry_key: "tropical_fish_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tropical_fish_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                Food,
                &FoodImpl {
                    nutrition: 1,
                    saturation: 0.2,
                    can_always_eat: false,
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (BucketEntityData, &BucketEntityDataImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tropical_fish_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TROPICAL_FISH_SPAWN_EGG: Self = Self {
        id: 1190,
        registry_key: "tropical_fish_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.tropical_fish_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tropical_fish_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUBE_CORAL: Self = Self {
        id: 687,
        registry_key: "tube_coral",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tube_coral",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tube_coral"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUBE_CORAL_BLOCK: Self = Self {
        id: 682,
        registry_key: "tube_coral_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tube_coral_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tube_coral_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUBE_CORAL_FAN: Self = Self {
        id: 697,
        registry_key: "tube_coral_fan",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tube_coral_fan",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tube_coral_fan"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF: Self = Self {
        id: 12,
        registry_key: "tuff",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_BRICK_SLAB: Self = Self {
        id: 22,
        registry_key: "tuff_brick_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_brick_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_brick_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_BRICK_STAIRS: Self = Self {
        id: 23,
        registry_key: "tuff_brick_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_brick_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_brick_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_BRICK_WALL: Self = Self {
        id: 24,
        registry_key: "tuff_brick_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_brick_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_brick_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_BRICKS: Self = Self {
        id: 21,
        registry_key: "tuff_bricks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_bricks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_bricks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_SLAB: Self = Self {
        id: 13,
        registry_key: "tuff_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_STAIRS: Self = Self {
        id: 14,
        registry_key: "tuff_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TUFF_WALL: Self = Self {
        id: 15,
        registry_key: "tuff_wall",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.tuff_wall",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:tuff_wall"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TURTLE_EGG: Self = Self {
        id: 674,
        registry_key: "turtle_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.turtle_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:turtle_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TURTLE_HELMET: Self = Self {
        id: 915,
        registry_key: "turtle_helmet",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.turtle_helmet",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 275 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[
                        Modifier {
                            r#type: &Attributes::ARMOR,
                            id: "minecraft:armor.helmet",
                            amount: 2f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                        Modifier {
                            r#type: &Attributes::ARMOR_TOUGHNESS,
                            id: "minecraft:armor.helmet",
                            amount: 0f64,
                            operation: Operation::AddValue,
                            slot: AttributeModifierSlot::Head,
                        },
                    ]),
                },
            ),
            (
                Equippable,
                &EquippableImpl {
                    slot: &EquipmentSlot::HEAD,
                    equip_sound: IdOr::Id(Sound::ItemArmorEquipTurtle),
                    asset_id: Some(Cow::Borrowed("minecraft:turtle_scute")),
                    camera_overlay: None,
                    allowed_entities: None,
                    dispensable: true,
                    swappable: true,
                    damage_on_hurt: true,
                    equip_on_interact: false,
                    can_be_sheared: false,
                    shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
                },
            ),
            (Enchantable, &EnchantableImpl { value: 9 }),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:turtle_helmet"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (Repairable, &RepairableImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TURTLE_SCUTE: Self = Self {
        id: 916,
        registry_key: "turtle_scute",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.turtle_scute",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:turtle_scute"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TURTLE_SPAWN_EGG: Self = Self {
        id: 1191,
        registry_key: "turtle_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.turtle_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:turtle_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const TWISTING_VINES: Self = Self {
        id: 283,
        registry_key: "twisting_vines",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.twisting_vines",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:twisting_vines"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VAULT: Self = Self {
        id: 1535,
        registry_key: "vault",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.vault",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:vault"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VERDANT_FROGLIGHT: Self = Self {
        id: 1453,
        registry_key: "verdant_froglight",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.verdant_froglight",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:verdant_froglight"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VEX_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1465,
        registry_key: "vex_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.vex_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:vex_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VEX_SPAWN_EGG: Self = Self {
        id: 1232,
        registry_key: "vex_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.vex_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:vex_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VILLAGER_SPAWN_EGG: Self = Self {
        id: 1200,
        registry_key: "villager_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.villager_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:villager_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VINDICATOR_SPAWN_EGG: Self = Self {
        id: 1231,
        registry_key: "vindicator_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.vindicator_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:vindicator_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const VINE: Self = Self {
        id: 438,
        registry_key: "vine",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.vine",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:vine"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WANDERING_TRADER_SPAWN_EGG: Self = Self {
        id: 1201,
        registry_key: "wandering_trader_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.wandering_trader_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:wandering_trader_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARD_ARMOR_TRIM_SMITHING_TEMPLATE: Self = Self {
        id: 1463,
        registry_key: "ward_armor_trim_smithing_template",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.ward_armor_trim_smithing_template",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:ward_armor_trim_smithing_template"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARDEN_SPAWN_EGG: Self = Self {
        id: 1226,
        registry_key: "warden_spawn_egg",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.warden_spawn_egg",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (EntityData, &EntityDataImpl),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warden_spawn_egg"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_BUTTON: Self = Self {
        id: 790,
        registry_key: "warped_button",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_button",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_button"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_DOOR: Self = Self {
        id: 819,
        registry_key: "warped_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_FENCE: Self = Self {
        id: 383,
        registry_key: "warped_fence",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_fence",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_fence"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_FENCE_GATE: Self = Self {
        id: 860,
        registry_key: "warped_fence_gate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_fence_gate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_fence_gate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_FUNGUS: Self = Self {
        id: 278,
        registry_key: "warped_fungus",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_fungus",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_fungus"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_FUNGUS_ON_A_STICK: Self = Self {
        id: 888,
        registry_key: "warped_fungus_on_a_stick",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.warped_fungus_on_a_stick",
                },
            ),
            (Damage, &DamageImpl { damage: 0 }),
            (MaxDamage, &MaxDamageImpl { max_damage: 100 }),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_fungus_on_a_stick"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_HANGING_SIGN: Self = Self {
        id: 1039,
        registry_key: "warped_hanging_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_hanging_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_hanging_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_HYPHAE: Self = Self {
        id: 208,
        registry_key: "warped_hyphae",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_hyphae",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_hyphae"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_NYLIUM: Self = Self {
        id: 61,
        registry_key: "warped_nylium",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_nylium",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_nylium"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_PLANKS: Self = Self {
        id: 74,
        registry_key: "warped_planks",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_planks",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_planks"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_PRESSURE_PLATE: Self = Self {
        id: 806,
        registry_key: "warped_pressure_plate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_pressure_plate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_pressure_plate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_ROOTS: Self = Self {
        id: 280,
        registry_key: "warped_roots",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_roots",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_roots"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_SHELF: Self = Self {
        id: 344,
        registry_key: "warped_shelf",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_shelf",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (Container, &ContainerImpl { items: Vec::new() }),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_shelf"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_SIGN: Self = Self {
        id: 1027,
        registry_key: "warped_sign",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 16 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_sign",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_sign"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_SLAB: Self = Self {
        id: 310,
        registry_key: "warped_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_STAIRS: Self = Self {
        id: 481,
        registry_key: "warped_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_STEM: Self = Self {
        id: 173,
        registry_key: "warped_stem",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_stem",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_stem"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_TRAPDOOR: Self = Self {
        id: 840,
        registry_key: "warped_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WARPED_WART_BLOCK: Self = Self {
        id: 605,
        registry_key: "warped_wart_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.warped_wart_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:warped_wart_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WATER_BUCKET: Self = Self {
        id: 1041,
        registry_key: "water_bucket",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 1 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "item.minecraft.water_bucket",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:water_bucket"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_CHISELED_COPPER: Self = Self {
        id: 133,
        registry_key: "waxed_chiseled_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_chiseled_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_chiseled_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_BARS: Self = Self {
        id: 423,
        registry_key: "waxed_copper_bars",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_bars",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_bars"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_BLOCK: Self = Self {
        id: 122,
        registry_key: "waxed_copper_block",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_block",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_block"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_BULB: Self = Self {
        id: 1512,
        registry_key: "waxed_copper_bulb",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_bulb",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_bulb"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_CHAIN: Self = Self {
        id: 432,
        registry_key: "waxed_copper_chain",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_chain",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_chain"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_CHEST: Self = Self {
        id: 1520,
        registry_key: "waxed_copper_chest",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_chest",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_chest"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_DOOR: Self = Self {
        id: 824,
        registry_key: "waxed_copper_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_GOLEM_STATUE: Self = Self {
        id: 1528,
        registry_key: "waxed_copper_golem_statue",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_golem_statue",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(
                        Cow::Borrowed("copper_golem_pose"),
                        Cow::Borrowed("standing"),
                    )]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_golem_statue"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_GRATE: Self = Self {
        id: 1504,
        registry_key: "waxed_copper_grate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_grate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_grate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_LANTERN: Self = Self {
        id: 1400,
        registry_key: "waxed_copper_lantern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_lantern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_lantern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_COPPER_TRAPDOOR: Self = Self {
        id: 845,
        registry_key: "waxed_copper_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_copper_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_copper_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_CUT_COPPER: Self = Self {
        id: 141,
        registry_key: "waxed_cut_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_cut_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_cut_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_CUT_COPPER_SLAB: Self = Self {
        id: 157,
        registry_key: "waxed_cut_copper_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_cut_copper_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_cut_copper_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_CUT_COPPER_STAIRS: Self = Self {
        id: 149,
        registry_key: "waxed_cut_copper_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_cut_copper_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_cut_copper_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_CHISELED_COPPER: Self = Self {
        id: 134,
        registry_key: "waxed_exposed_chiseled_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_chiseled_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_chiseled_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER: Self = Self {
        id: 123,
        registry_key: "waxed_exposed_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_BARS: Self = Self {
        id: 424,
        registry_key: "waxed_exposed_copper_bars",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_bars",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_bars"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_BULB: Self = Self {
        id: 1513,
        registry_key: "waxed_exposed_copper_bulb",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_bulb",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_bulb"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_CHAIN: Self = Self {
        id: 433,
        registry_key: "waxed_exposed_copper_chain",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_chain",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_chain"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_CHEST: Self = Self {
        id: 1521,
        registry_key: "waxed_exposed_copper_chest",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_chest",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_chest"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_DOOR: Self = Self {
        id: 825,
        registry_key: "waxed_exposed_copper_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_GOLEM_STATUE: Self = Self {
        id: 1529,
        registry_key: "waxed_exposed_copper_golem_statue",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_golem_statue",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(
                        Cow::Borrowed("copper_golem_pose"),
                        Cow::Borrowed("standing"),
                    )]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_golem_statue"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_GRATE: Self = Self {
        id: 1505,
        registry_key: "waxed_exposed_copper_grate",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_grate",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_grate"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_LANTERN: Self = Self {
        id: 1401,
        registry_key: "waxed_exposed_copper_lantern",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_lantern",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_lantern"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_COPPER_TRAPDOOR: Self = Self {
        id: 846,
        registry_key: "waxed_exposed_copper_trapdoor",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_copper_trapdoor",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_copper_trapdoor"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_CUT_COPPER: Self = Self {
        id: 142,
        registry_key: "waxed_exposed_cut_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_cut_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_cut_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_CUT_COPPER_SLAB: Self = Self {
        id: 158,
        registry_key: "waxed_exposed_cut_copper_slab",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_cut_copper_slab",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_cut_copper_slab"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_CUT_COPPER_STAIRS: Self = Self {
        id: 150,
        registry_key: "waxed_exposed_cut_copper_stairs",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_cut_copper_stairs",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_cut_copper_stairs"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_EXPOSED_LIGHTNING_ROD: Self = Self {
        id: 766,
        registry_key: "waxed_exposed_lightning_rod",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_exposed_lightning_rod",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_exposed_lightning_rod"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_LIGHTNING_ROD: Self = Self {
        id: 765,
        registry_key: "waxed_lightning_rod",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_lightning_rod",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_lightning_rod"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_CHISELED_COPPER: Self = Self {
        id: 136,
        registry_key: "waxed_oxidized_chiseled_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_chiseled_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_chiseled_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER: Self = Self {
        id: 125,
        registry_key: "waxed_oxidized_copper",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_BARS: Self = Self {
        id: 426,
        registry_key: "waxed_oxidized_copper_bars",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_bars",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_bars"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_BULB: Self = Self {
        id: 1515,
        registry_key: "waxed_oxidized_copper_bulb",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_bulb",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_bulb"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_CHAIN: Self = Self {
        id: 435,
        registry_key: "waxed_oxidized_copper_chain",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_chain",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_chain"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_CHEST: Self = Self {
        id: 1523,
        registry_key: "waxed_oxidized_copper_chest",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_chest",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_chest"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_DOOR: Self = Self {
        id: 827,
        registry_key: "waxed_oxidized_copper_door",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_door",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_door"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
    pub const WAXED_OXIDIZED_COPPER_GOLEM_STATUE: Self = Self {
        id: 1531,
        registry_key: "waxed_oxidized_copper_golem_statue",
        components: &[
            (MaxStackSize, &MaxStackSizeImpl { size: 64 }),
            (
                ItemName,
                &ItemNameImpl {
                    name: "block.minecraft.waxed_oxidized_copper_golem_statue",
                },
            ),
            (
                AttributeModifiers,
                &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[]),
                },
            ),
            (
                BlockState,
                &BlockStateImpl {
                    properties: Cow::Borrowed(&[(
                        Cow::Borrowed("copper_golem_pose"),
                        Cow::Borrowed("standing"),
                    )]),
                },
            ),
            (BreakSound, &BreakSoundImpl),
            (
                Enchantments,
                &EnchantmentsImpl {
                    enchantment: Cow::Borrowed(&[]),
                },
            ),
            (
                ItemModel,
                &ItemModelImpl {
                    id: Cow::Borrowed("minecraft:waxed_oxidized_copper_golem_statue"),
                },
            ),
            (Lore, &LoreImpl),
            (Rarity, &RarityImpl),
            (RepairCost, &RepairCostImpl),
            (SwingAnimation, &SwingAnimationImpl),
            (TooltipDisplay, &TooltipDisplayImpl),
            (UseEffects, &UseEffectsImpl),
        ],
    };
}
