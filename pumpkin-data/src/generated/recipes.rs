/* This file is generated. Do not edit manually. */
use crate::item::Item;
use crate::tag::Taggable;
#[derive(Clone, Debug)]
pub enum CraftingRecipeTypes {
    CraftingShaped {
        category: RecipeCategoryTypes,
        group: Option<&'static str>,
        show_notification: bool,
        key: &'static [(char, RecipeIngredientTypes)],
        pattern: &'static [&'static str],
        result: RecipeResultStruct,
    },
    CraftingShapeless {
        category: RecipeCategoryTypes,
        group: Option<&'static str>,
        ingredients: &'static [RecipeIngredientTypes],
        result: RecipeResultStruct,
    },
    CraftingTransmute {
        category: RecipeCategoryTypes,
        group: Option<&'static str>,
        input: RecipeIngredientTypes,
        material: RecipeIngredientTypes,
        result: RecipeResultStruct,
    },
    CraftingDecoratedPot {
        category: RecipeCategoryTypes,
    },
    CraftingSpecial,
}
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CookingRecipe {
    #[doc = r#" Vanilla-compatible recipe ID (e.g., "minecraft:iron_ingot_from_smelting_iron_ore")"#]
    pub recipe_id: &'static str,
    pub category: RecipeCategoryTypes,
    pub group: Option<&'static str>,
    pub ingredient: RecipeIngredientTypes,
    pub cookingtime: i32,
    pub experience: f32,
    pub result: RecipeResultStruct,
}
#[derive(Clone, Debug)]
pub enum CookingRecipeType {
    Blasting(CookingRecipe),
    Smelting(CookingRecipe),
    Smoking(CookingRecipe),
    CampfireCooking(CookingRecipe),
}
#[derive(Clone, Debug)]
pub enum CookingRecipeKind {
    Blasting,
    Smelting,
    Smoking,
    CampfireCooking,
}
impl From<&CookingRecipeType> for CookingRecipeKind {
    fn from(recipe_type: &CookingRecipeType) -> Self {
        match recipe_type {
            CookingRecipeType::Blasting(_) => CookingRecipeKind::Blasting,
            CookingRecipeType::Smelting(_) => CookingRecipeKind::Smelting,
            CookingRecipeType::Smoking(_) => CookingRecipeKind::Smoking,
            CookingRecipeType::CampfireCooking(_) => CookingRecipeKind::CampfireCooking,
        }
    }
}
impl From<CookingRecipeType> for CookingRecipeKind {
    fn from(recipe_type: CookingRecipeType) -> Self {
        match recipe_type {
            CookingRecipeType::Blasting(_) => CookingRecipeKind::Blasting,
            CookingRecipeType::Smelting(_) => CookingRecipeKind::Smelting,
            CookingRecipeType::Smoking(_) => CookingRecipeKind::Smoking,
            CookingRecipeType::CampfireCooking(_) => CookingRecipeKind::CampfireCooking,
        }
    }
}
impl CookingRecipeKind {
    pub fn to_type(self, recipe: CookingRecipe) -> CookingRecipeType {
        match self {
            CookingRecipeKind::Blasting => CookingRecipeType::Blasting(recipe),
            CookingRecipeKind::Smelting => CookingRecipeType::Smelting(recipe),
            CookingRecipeKind::Smoking => CookingRecipeType::Smoking(recipe),
            CookingRecipeKind::CampfireCooking => CookingRecipeType::CampfireCooking(recipe),
        }
    }
}
#[derive(Clone, Debug)]
pub struct RecipeResultStruct {
    pub id: &'static str,
    pub count: u8,
}
#[derive(Clone, Debug)]
pub enum RecipeIngredientTypes {
    Simple(&'static str),
    Tagged(&'static str),
    OneOf(&'static [&'static str]),
}
impl RecipeIngredientTypes {
    pub fn match_item(&self, item: &Item) -> bool {
        match self {
            RecipeIngredientTypes::Simple(ingredient) => {
                let name = format!("minecraft:{}", item.registry_key);
                name == *ingredient
            }
            RecipeIngredientTypes::Tagged(tag) => item
                .is_tagged_with(tag)
                .expect("Crafting recipe used invalid tag"),
            RecipeIngredientTypes::OneOf(ingredients) => {
                let name = format!("minecraft:{}", item.registry_key);
                ingredients.contains(&name.as_str())
            }
        }
    }
}
#[derive(Clone, Debug)]
pub enum RecipeCategoryTypes {
    Equipment,
    Building,
    Restone,
    Misc,
    Food,
    Blocks,
}
pub static RECIPES_CRAFTING: &[CraftingRecipeTypes] = &[
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:acacia_planks")],
        result: RecipeResultStruct {
            id: "minecraft:acacia_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:acacia_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:acacia_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_acacia_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:acacia_logs")],
        result: RecipeResultStruct {
            id: "minecraft:acacia_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_acacia_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:acacia_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:acacia_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:acacia_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:acacia_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:redstone_torch"),
            ),
            ('S', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["XSX", "X#X", "XSX"],
        result: RecipeResultStruct {
            id: "minecraft:activator_rail",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:amethyst_shard"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:amethyst_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:diorite"),
            RecipeIngredientTypes::Simple("minecraft:cobblestone"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:andesite",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:andesite"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:andesite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:andesite"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:andesite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:andesite"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:andesite_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_block")),
            ('i', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["III", " i ", "iii"],
        result: RecipeResultStruct {
            id: "minecraft:anvil",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('/', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                '_',
                RecipeIngredientTypes::Simple("minecraft:smooth_stone_slab"),
            ),
        ],
        pattern: &["///", " / ", "/_/"],
        result: RecipeResultStruct {
            id: "minecraft:armor_stand",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:flint")),
            ('Y', RecipeIngredientTypes::Simple("minecraft:feather")),
        ],
        pattern: &["X", "#", "Y"],
        result: RecipeResultStruct {
            id: "minecraft:arrow",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
            RecipeIngredientTypes::Simple("minecraft:bamboo"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:bamboo_planks")],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:bamboo_raft"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_chest_raft",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_bamboo_block"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bamboo_slab"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_mosaic",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_mosaic"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_mosaic_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_mosaic"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_mosaic_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:bamboo_blocks")],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_planks",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_raft",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_bamboo_block"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:bamboo_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bamboo_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('P', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            (
                'S',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_slabs"),
            ),
        ],
        pattern: &["PSP", "P P", "PSP"],
        result: RecipeResultStruct {
            id: "minecraft:barrel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('O', RecipeIngredientTypes::Simple("minecraft:obsidian")),
            ('S', RecipeIngredientTypes::Simple("minecraft:nether_star")),
        ],
        pattern: &["GGG", "GSG", "OOO"],
        result: RecipeResultStruct {
            id: "minecraft:beacon",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('H', RecipeIngredientTypes::Simple("minecraft:honeycomb")),
            ('P', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["PPP", "HHH", "PPP"],
        result: RecipeResultStruct {
            id: "minecraft:beehive",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
            RecipeIngredientTypes::Simple("minecraft:beetroot"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:beetroot_soup",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:birch_planks")],
        result: RecipeResultStruct {
            id: "minecraft:birch_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:birch_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:birch_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:birch_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('W', RecipeIngredientTypes::Simple("minecraft:birch_planks")),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:birch_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('W', RecipeIngredientTypes::Simple("minecraft:birch_planks")),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:birch_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_birch_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:birch_logs")],
        result: RecipeResultStruct {
            id: "minecraft:birch_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:birch_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_birch_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:birch_planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:birch_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_planks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:birch_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:birch_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:birch_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:black_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:black_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:black_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:black_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:black_dye"),
        result: RecipeResultStruct {
            id: "minecraft:black_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:black_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:black_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("black_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:ink_sac")],
        result: RecipeResultStruct {
            id: "minecraft:black_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("black_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:wither_rose")],
        result: RecipeResultStruct {
            id: "minecraft:black_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:black_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:black_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:black_dye"),
        result: RecipeResultStruct {
            id: "minecraft:black_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:black_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:black_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:black_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:black_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:black_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:black_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:black_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:black_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:blackstone"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:blackstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:blackstone"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blackstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:blackstone"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blackstone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:smooth_stone")),
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:furnace")),
        ],
        pattern: &["III", "IXI", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blast_furnace",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:blaze_rod")],
        result: RecipeResultStruct {
            id: "minecraft:blaze_powder",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:blue_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:blue_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:blue_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:blue_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:blue_dye"),
        result: RecipeResultStruct {
            id: "minecraft:blue_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:blue_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:blue_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("blue_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:lapis_lazuli")],
        result: RecipeResultStruct {
            id: "minecraft:blue_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("blue_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:cornflower")],
        result: RecipeResultStruct {
            id: "minecraft:blue_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:blue_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:blue_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
            RecipeIngredientTypes::Simple("minecraft:packed_ice"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_ice",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:blue_dye"),
        result: RecipeResultStruct {
            id: "minecraft:blue_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:blue_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blue_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:blue_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blue_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:blue_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blue_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:blue_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:blue_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            (
                'C',
                RecipeIngredientTypes::OneOf(&[
                    "minecraft:copper_block",
                    "minecraft:waxed_copper_block",
                ]),
            ),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:bolt_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bolt_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bone_meal"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bone_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bonemeal"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:bone")],
        result: RecipeResultStruct {
            id: "minecraft:bone_meal",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bonemeal"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:bone_block")],
        result: RecipeResultStruct {
            id: "minecraft:bone_meal",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:leather"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:book",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:book")),
        ],
        pattern: &["###", "XXX", "###"],
        result: RecipeResultStruct {
            id: "minecraft:bookshelf",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:vine"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:bordure_indented_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &[" #X", "# X", " #X"],
        result: RecipeResultStruct {
            id: "minecraft:bow",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Tagged("#minecraft:planks"))],
        pattern: &["# #", " # "],
        result: RecipeResultStruct {
            id: "minecraft:bowl",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:wheat"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:bread",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Tagged("#minecraft:stone_crafting_materials"),
            ),
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
        ],
        pattern: &[" B ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brewing_stand",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bricks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bricks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bricks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:brick"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:brown_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:brown_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:brown_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:brown_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:brown_dye"),
        result: RecipeResultStruct {
            id: "minecraft:brown_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:brown_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:brown_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("brown_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:cocoa_beans")],
        result: RecipeResultStruct {
            id: "minecraft:brown_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:brown_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:brown_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:brown_dye"),
        result: RecipeResultStruct {
            id: "minecraft:brown_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:brown_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brown_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:brown_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brown_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:brown_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brown_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:brown_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:brown_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot")),
            ('I', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:feather")),
        ],
        pattern: &["X", "#", "I"],
        result: RecipeResultStruct {
            id: "minecraft:brush",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["# #", " # "],
        result: RecipeResultStruct {
            id: "minecraft:bucket",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:leather")),
            ('-', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["-", "#"],
        result: RecipeResultStruct {
            id: "minecraft:bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('A', RecipeIngredientTypes::Simple("minecraft:milk_bucket")),
            ('B', RecipeIngredientTypes::Simple("minecraft:sugar")),
            ('C', RecipeIngredientTypes::Simple("minecraft:wheat")),
            ('E', RecipeIngredientTypes::Tagged("#minecraft:eggs")),
        ],
        pattern: &["AAA", "BEB", "CCC"],
        result: RecipeResultStruct {
            id: "minecraft:cake",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:amethyst_shard"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:sculk_sensor")),
        ],
        pattern: &[" # ", "#X#"],
        result: RecipeResultStruct {
            id: "minecraft:calibrated_sculk_sensor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('C', RecipeIngredientTypes::Tagged("#minecraft:coals")),
            ('L', RecipeIngredientTypes::Tagged("#minecraft:logs")),
            ('S', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &[" S ", "SCS", "LLL"],
        result: RecipeResultStruct {
            id: "minecraft:campfire",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('H', RecipeIngredientTypes::Simple("minecraft:honeycomb")),
            ('S', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["S", "H"],
        result: RecipeResultStruct {
            id: "minecraft:candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:fishing_rod")),
            ('X', RecipeIngredientTypes::Simple("minecraft:carrot")),
        ],
        pattern: &["# ", " X"],
        result: RecipeResultStruct {
            id: "minecraft:carrot_on_a_stick",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('@', RecipeIngredientTypes::Simple("minecraft:paper")),
        ],
        pattern: &["@@", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cartography_table",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["# #", "# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cauldron",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:cherry_planks")],
        result: RecipeResultStruct {
            id: "minecraft:cherry_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:cherry_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cherry_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_cherry_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:cherry_logs")],
        result: RecipeResultStruct {
            id: "minecraft:cherry_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_cherry_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:cherry_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cherry_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cherry_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cherry_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Tagged("#minecraft:planks"))],
        pattern: &["###", "# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:minecart"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:chest_minecart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_slabs"),
            ),
        ],
        pattern: &["###", "XXX", "###"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_bookshelf",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cut_copper_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_deepslate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:nether_brick_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_nether_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_polished_blackstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:quartz_slab"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_quartz_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_sandstone_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_red_sandstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:resin_brick_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_resin_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:sandstone_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_sandstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stone_brick_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_stone_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff_slab"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_tuff",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:tuff_brick_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:chiseled_tuff_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:clay_ball"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:clay",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gold_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" # ", "#X#", " # "],
        result: RecipeResultStruct {
            id: "minecraft:clock",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:coal_block")],
        result: RecipeResultStruct {
            id: "minecraft:coal",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:coal"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:coal_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('D', RecipeIngredientTypes::Simple("minecraft:dirt")),
            ('G', RecipeIngredientTypes::Simple("minecraft:gravel")),
        ],
        pattern: &["DG", "GD"],
        result: RecipeResultStruct {
            id: "minecraft:coarse_dirt",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:coast_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:coast_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cobbled_deepslate_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cobbled_deepslate_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cobbled_deepslate_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cobblestone"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cobblestone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cobblestone"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cobblestone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cobblestone"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cobblestone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:redstone_torch"),
            ),
            ('I', RecipeIngredientTypes::Simple("minecraft:stone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:quartz")),
        ],
        pattern: &[" # ", "#X#", "III"],
        result: RecipeResultStruct {
            id: "minecraft:comparator",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" # ", "#X#", " # "],
        result: RecipeResultStruct {
            id: "minecraft:compass",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Tagged("#minecraft:wooden_slabs"),
        )],
        pattern: &["# #", "# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:composter",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:nautilus_shell"),
            ),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:heart_of_the_sea"),
            ),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:conduit",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:wheat")),
            ('X', RecipeIngredientTypes::Simple("minecraft:cocoa_beans")),
        ],
        pattern: &["#X#"],
        result: RecipeResultStruct {
            id: "minecraft:cookie",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:copper_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:copper_bars",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:copper_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:copper_boots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            ('C', RecipeIngredientTypes::Simple("minecraft:copper_block")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('I', RecipeIngredientTypes::Simple("minecraft:copper_ingot")),
            (
                'N',
                RecipeIngredientTypes::Simple("minecraft:copper_nugget"),
            ),
        ],
        pattern: &["N", "I", "N"],
        result: RecipeResultStruct {
            id: "minecraft:copper_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:chest")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:copper_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["X X", "XXX", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:copper_chestplate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:copper_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("copper_grate"),
        show_notification: true,
        key: &[('M', RecipeIngredientTypes::Simple("minecraft:copper_block"))],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:copper_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:copper_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:copper_block")],
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:copper_nugget"),
        )],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredients: &[RecipeIngredientTypes::Simple(
            "minecraft:waxed_copper_block",
        )],
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:copper_torch")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:copper_nugget"),
            ),
        ],
        pattern: &["XXX", "X#X", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:copper_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["XXX", "X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:copper_leggings",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:copper_ingot")],
        result: RecipeResultStruct {
            id: "minecraft:copper_nugget",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:copper_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:copper_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:copper_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:copper_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:copper_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:copper_nugget"),
            ),
            (
                'X',
                RecipeIngredientTypes::OneOf(&["minecraft:coal", "minecraft:charcoal"]),
            ),
        ],
        pattern: &["C", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:copper_torch",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:copper_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:crafting_table"),
            ),
            ('D', RecipeIngredientTypes::Simple("minecraft:dropper")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["###", "#C#", "RDR"],
        result: RecipeResultStruct {
            id: "minecraft:crafter",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: false,
        key: &[('#', RecipeIngredientTypes::Tagged("#minecraft:planks"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:crafting_table",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('L', RecipeIngredientTypes::Simple("minecraft:pale_oak_log")),
            ('R', RecipeIngredientTypes::Simple("minecraft:resin_block")),
        ],
        pattern: &[" L ", " R ", " L "],
        result: RecipeResultStruct {
            id: "minecraft:creaking_heart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:creeper_head"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:creeper_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:crimson_planks")],
        result: RecipeResultStruct {
            id: "minecraft:crimson_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_crimson_stem"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:crimson_stem"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_hyphae",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:crimson_stems")],
        result: RecipeResultStruct {
            id: "minecraft:crimson_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_crimson_stem"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:crimson_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:crimson_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:crimson_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                '$',
                RecipeIngredientTypes::Simple("minecraft:tripwire_hook"),
            ),
            ('&', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('~', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["#&#", "~$~", " # "],
        result: RecipeResultStruct {
            id: "minecraft:crossbow",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_block"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cut_copper"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cut_copper"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_sandstone"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cut_red_sandstone",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cut_red_sandstone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cut_red_sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:sandstone"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:cut_sandstone",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cut_sandstone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:cut_sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cyan_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:cyan_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cyan_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
        result: RecipeResultStruct {
            id: "minecraft:cyan_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:cyan_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("cyan_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("cyan_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:pitcher_plant")],
        result: RecipeResultStruct {
            id: "minecraft:cyan_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cyan_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
        result: RecipeResultStruct {
            id: "minecraft:cyan_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:cyan_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:cyan_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:cyan_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:cyan_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:cyan_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:dark_oak_planks")],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:dark_oak_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_dark_oak_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:dark_oak_logs")],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_dark_oak_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_oak_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:dark_oak_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:dark_oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('I', RecipeIngredientTypes::Simple("minecraft:black_dye")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            ),
        ],
        pattern: &["SSS", "SIS", "SSS"],
        result: RecipeResultStruct {
            id: "minecraft:dark_prismarine",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_prismarine"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_prismarine_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:dark_prismarine"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dark_prismarine_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('Q', RecipeIngredientTypes::Simple("minecraft:quartz")),
            (
                'W',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_slabs"),
            ),
        ],
        pattern: &["GGG", "QQQ", "WWW"],
        result: RecipeResultStruct {
            id: "minecraft:daylight_detector",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingDecoratedPot {
        category: RecipeCategoryTypes::Misc,
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:brick"))],
        pattern: &[" # ", "# #", " # "],
        result: RecipeResultStruct {
            id: "minecraft:decorated_pot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'S',
            RecipeIngredientTypes::Simple("minecraft:polished_deepslate"),
        )],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_tiles"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_tile_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_tiles"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_tile_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:deepslate_tiles"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_tile_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'S',
            RecipeIngredientTypes::Simple("minecraft:deepslate_bricks"),
        )],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:deepslate_tiles",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stone_pressure_plate"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["X X", "X#X", "XRX"],
        result: RecipeResultStruct {
            id: "minecraft:detector_rail",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:diamond_block")],
        result: RecipeResultStruct {
            id: "minecraft:diamond",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:diamond"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:diamond"))],
        pattern: &["X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_boots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:diamond"))],
        pattern: &["X X", "XXX", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_chestplate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:diamond"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:diamond"))],
        pattern: &["XXX", "X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_leggings",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:diamond_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:diamond_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:diamond_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:diamond_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('C', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('Q', RecipeIngredientTypes::Simple("minecraft:quartz")),
        ],
        pattern: &["CQ", "QC"],
        result: RecipeResultStruct {
            id: "minecraft:diorite",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:diorite"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:diorite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:diorite"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:diorite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:diorite"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:diorite_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:bow")),
        ],
        pattern: &["###", "#X#", "#R#"],
        result: RecipeResultStruct {
            id: "minecraft:dispenser",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("dry_ghast"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:ghast_tear")),
            ('X', RecipeIngredientTypes::Simple("minecraft:soul_sand")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dried_ghast",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:dried_kelp_block")],
        result: RecipeResultStruct {
            id: "minecraft:dried_kelp",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:dried_kelp"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dried_kelp_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pointed_dripstone"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:dripstone_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["###", "# #", "#R#"],
        result: RecipeResultStruct {
            id: "minecraft:dropper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:sandstone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:dune_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:dune_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:black_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:blue_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:brown_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cyan_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:cyan_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:yellow_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:yellow_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:yellow_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:yellow_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:yellow_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:yellow_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:yellow_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:yellow_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_bed",
                "minecraft:blue_bed",
                "minecraft:brown_bed",
                "minecraft:cyan_bed",
                "minecraft:gray_bed",
                "minecraft:green_bed",
                "minecraft:light_blue_bed",
                "minecraft:light_gray_bed",
                "minecraft:lime_bed",
                "minecraft:magenta_bed",
                "minecraft:orange_bed",
                "minecraft:pink_bed",
                "minecraft:purple_bed",
                "minecraft:red_bed",
                "minecraft:white_bed",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_carpet",
                "minecraft:blue_carpet",
                "minecraft:brown_carpet",
                "minecraft:cyan_carpet",
                "minecraft:gray_carpet",
                "minecraft:green_carpet",
                "minecraft:light_blue_carpet",
                "minecraft:light_gray_carpet",
                "minecraft:lime_carpet",
                "minecraft:magenta_carpet",
                "minecraft:orange_carpet",
                "minecraft:pink_carpet",
                "minecraft:purple_carpet",
                "minecraft:red_carpet",
                "minecraft:white_carpet",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_carpet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_harness",
                "minecraft:blue_harness",
                "minecraft:brown_harness",
                "minecraft:cyan_harness",
                "minecraft:gray_harness",
                "minecraft:green_harness",
                "minecraft:light_blue_harness",
                "minecraft:light_gray_harness",
                "minecraft:lime_harness",
                "minecraft:magenta_harness",
                "minecraft:orange_harness",
                "minecraft:pink_harness",
                "minecraft:purple_harness",
                "minecraft:red_harness",
                "minecraft:white_harness",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("wool"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
            RecipeIngredientTypes::OneOf(&[
                "minecraft:black_wool",
                "minecraft:blue_wool",
                "minecraft:brown_wool",
                "minecraft:cyan_wool",
                "minecraft:gray_wool",
                "minecraft:green_wool",
                "minecraft:light_blue_wool",
                "minecraft:light_gray_wool",
                "minecraft:lime_wool",
                "minecraft:magenta_wool",
                "minecraft:orange_wool",
                "minecraft:pink_wool",
                "minecraft:purple_wool",
                "minecraft:red_wool",
                "minecraft:white_wool",
            ]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:emerald_block")],
        result: RecipeResultStruct {
            id: "minecraft:emerald",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:emerald"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:emerald_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:obsidian")),
            ('B', RecipeIngredientTypes::Simple("minecraft:book")),
            ('D', RecipeIngredientTypes::Simple("minecraft:diamond")),
        ],
        pattern: &[" B ", "D#D", "###"],
        result: RecipeResultStruct {
            id: "minecraft:enchanting_table",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('E', RecipeIngredientTypes::Simple("minecraft:ender_eye")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('T', RecipeIngredientTypes::Simple("minecraft:ghast_tear")),
        ],
        pattern: &["GGG", "GEG", "GTG"],
        result: RecipeResultStruct {
            id: "minecraft:end_crystal",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:popped_chorus_fruit"),
            ),
            ('/', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
        ],
        pattern: &["/", "#"],
        result: RecipeResultStruct {
            id: "minecraft:end_rod",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:end_stone_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:end_stone_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:end_stone_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:end_stone_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:end_stone_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:end_stone_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:end_stone"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:end_stone_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:obsidian")),
            ('E', RecipeIngredientTypes::Simple("minecraft:ender_eye")),
        ],
        pattern: &["###", "#E#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:ender_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:ender_pearl"),
            RecipeIngredientTypes::Simple("minecraft:blaze_powder"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:ender_eye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:exposed_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("exposed_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:exposed_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:exposed_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("exposed_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:exposed_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:exposed_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:exposed_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:exposed_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:exposed_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:exposed_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:end_stone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:eye_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:eye_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:spider_eye"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:sugar"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:fermented_spider_eye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:bricks"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:field_masoned_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gunpowder"),
            RecipeIngredientTypes::Simple("minecraft:blaze_powder"),
            RecipeIngredientTypes::OneOf(&["minecraft:coal", "minecraft:charcoal"]),
        ],
        result: RecipeResultStruct {
            id: "minecraft:fire_charge",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gunpowder"),
            RecipeIngredientTypes::Simple("minecraft:paper"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:firework_rocket",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["  #", " #X", "# X"],
        result: RecipeResultStruct {
            id: "minecraft:fishing_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('@', RecipeIngredientTypes::Simple("minecraft:flint")),
        ],
        pattern: &["@@", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:fletching_table",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:iron_ingot"),
            RecipeIngredientTypes::Simple("minecraft:flint"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:flint_and_steel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:breeze_rod")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:flow_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:flow_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:oxeye_daisy"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:flower_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:brick"))],
        pattern: &["# #", " # "],
        result: RecipeResultStruct {
            id: "minecraft:flower_pot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Tagged("#minecraft:stone_crafting_materials"),
        )],
        pattern: &["###", "# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:furnace",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:furnace"),
            RecipeIngredientTypes::Simple("minecraft:minecart"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:furnace_minecart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:glass"))],
        pattern: &["# #", " # "],
        result: RecipeResultStruct {
            id: "minecraft:glass_bottle",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:glass"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gold_nugget")),
            ('X', RecipeIngredientTypes::Simple("minecraft:melon_slice")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:glistering_melon_slice",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:item_frame"),
            RecipeIngredientTypes::Simple("minecraft:glow_ink_sac"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:glow_item_frame",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:glowstone_dust"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:glowstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gold_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:gold_block")],
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:gold_nugget"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:gold_ingot")],
        result: RecipeResultStruct {
            id: "minecraft:gold_nugget",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gold_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:apple")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:golden_apple",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:golden_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:golden_boots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gold_nugget")),
            ('X', RecipeIngredientTypes::Simple("minecraft:carrot")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:golden_carrot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["X X", "XXX", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:golden_chestplate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:golden_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:golden_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["XXX", "X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:golden_leggings",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:golden_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:golden_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:golden_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:gold_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:golden_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:diorite"),
            RecipeIngredientTypes::Simple("minecraft:quartz"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:granite",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:granite"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:granite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:granite"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:granite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:granite"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:granite_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gray_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:gray_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gray_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:gray_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:gray_dye"),
        result: RecipeResultStruct {
            id: "minecraft:gray_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:gray_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:gray_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("gray_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:gray_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("gray_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:closed_eyeblossom")],
        result: RecipeResultStruct {
            id: "minecraft:gray_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:gray_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:gray_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:gray_dye"),
        result: RecipeResultStruct {
            id: "minecraft:gray_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:gray_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gray_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:gray_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gray_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:gray_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gray_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:gray_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:gray_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:green_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:green_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:green_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:green_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:green_dye"),
        result: RecipeResultStruct {
            id: "minecraft:green_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:green_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:green_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:green_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:green_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:green_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:green_dye"),
        result: RecipeResultStruct {
            id: "minecraft:green_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:green_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:green_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:green_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:green_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:green_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:green_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:green_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:green_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('-', RecipeIngredientTypes::Simple("minecraft:stone_slab")),
            ('I', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["I-I", "# #"],
        result: RecipeResultStruct {
            id: "minecraft:grindstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:hay_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:heavy_weighted_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:honey_bottle"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:honey_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:honey_block"),
            RecipeIngredientTypes::Simple("minecraft:glass_bottle"),
            RecipeIngredientTypes::Simple("minecraft:glass_bottle"),
            RecipeIngredientTypes::Simple("minecraft:glass_bottle"),
            RecipeIngredientTypes::Simple("minecraft:glass_bottle"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:honey_bottle",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:honeycomb"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:honeycomb_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('C', RecipeIngredientTypes::Simple("minecraft:chest")),
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["I I", "ICI", " I "],
        result: RecipeResultStruct {
            id: "minecraft:hopper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:hopper"),
            RecipeIngredientTypes::Simple("minecraft:minecart"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:hopper_minecart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:host_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:host_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:iron_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:iron_bars",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:iron_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:iron_boots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('N', RecipeIngredientTypes::Simple("minecraft:iron_nugget")),
        ],
        pattern: &["N", "I", "N"],
        result: RecipeResultStruct {
            id: "minecraft:iron_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["X X", "XXX", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:iron_chestplate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:iron_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:iron_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:iron_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:iron_block")],
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_nugget"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["XXX", "X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:iron_leggings",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:iron_ingot")],
        result: RecipeResultStruct {
            id: "minecraft:iron_nugget",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:iron_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:iron_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:iron_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:iron_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:iron_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:iron_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:item_frame",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            (
                'A',
                RecipeIngredientTypes::Simple("minecraft:carved_pumpkin"),
            ),
            ('B', RecipeIngredientTypes::Simple("minecraft:torch")),
        ],
        pattern: &["A", "B"],
        result: RecipeResultStruct {
            id: "minecraft:jack_o_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:diamond")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jukebox",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:jungle_planks")],
        result: RecipeResultStruct {
            id: "minecraft:jungle_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:jungle_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:jungle_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_jungle_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:jungle_logs")],
        result: RecipeResultStruct {
            id: "minecraft:jungle_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_jungle_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:jungle_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:jungle_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:jungle_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:jungle_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stick"))],
        pattern: &["# #", "###", "# #"],
        result: RecipeResultStruct {
            id: "minecraft:ladder",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:torch")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_nugget")),
        ],
        pattern: &["XXX", "X#X", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:lapis_lazuli"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:lapis_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:lapis_block")],
        result: RecipeResultStruct {
            id: "minecraft:lapis_lazuli",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('~', RecipeIngredientTypes::Simple("minecraft:string"))],
        pattern: &["~~ ", "~~ ", "  ~"],
        result: RecipeResultStruct {
            id: "minecraft:lead",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:rabbit_hide"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:leather",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:leather"))],
        pattern: &["X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:leather_boots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:leather"))],
        pattern: &["X X", "XXX", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:leather_chestplate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:leather"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:leather_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:leather"))],
        pattern: &["X X", "XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:leather_horse_armor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:leather"))],
        pattern: &["XXX", "X X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:leather_leggings",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:bookshelf")),
            (
                'S',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_slabs"),
            ),
        ],
        pattern: &["SSS", " B ", " S "],
        result: RecipeResultStruct {
            id: "minecraft:lectern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:lever",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_blue_wool"),
            ),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_blue_wool"),
            ),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
        result: RecipeResultStruct {
            id: "minecraft:light_blue_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:light_blue_wool"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_blue_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:blue_orchid")],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_blue_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_blue_wool"),
            ),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
        result: RecipeResultStruct {
            id: "minecraft:light_blue_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            ),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:light_blue_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            (
                '$',
                RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            ),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:light_blue_dye"),
            ),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_blue_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_gray_wool"),
            ),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_gray_wool"),
            ),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
        result: RecipeResultStruct {
            id: "minecraft:light_gray_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:light_gray_wool"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_gray_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:azure_bluet")],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_gray_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:black_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_dye",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_gray_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:gray_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_gray_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:oxeye_daisy")],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("light_gray_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:white_tulip")],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:light_gray_wool"),
            ),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
        result: RecipeResultStruct {
            id: "minecraft:light_gray_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            ),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:light_gray_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            (
                '$',
                RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            ),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:light_gray_dye"),
            ),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:light_gray_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:gold_ingot"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:light_weighted_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:copper_ingot"))],
        pattern: &["#", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:lightning_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:lime_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:lime_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:lime_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:lime_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:lime_dye"),
        result: RecipeResultStruct {
            id: "minecraft:lime_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:lime_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:lime_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lime_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:green_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:lime_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:lime_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:lime_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:lime_dye"),
        result: RecipeResultStruct {
            id: "minecraft:lime_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:lime_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:lime_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:lime_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:lime_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:lime_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:lime_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:lime_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:lime_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:chiseled_stone_bricks"),
            ),
        ],
        pattern: &["SSS", "S#S", "SSS"],
        result: RecipeResultStruct {
            id: "minecraft:lodestone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('@', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["@@", "##"],
        result: RecipeResultStruct {
            id: "minecraft:loom",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:heavy_core")),
            ('I', RecipeIngredientTypes::Simple("minecraft:breeze_rod")),
        ],
        pattern: &[" # ", " I "],
        result: RecipeResultStruct {
            id: "minecraft:mace",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:magenta_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:magenta_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:magenta_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
        result: RecipeResultStruct {
            id: "minecraft:magenta_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:magenta_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("magenta_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:allium")],
        result: RecipeResultStruct {
            id: "minecraft:magenta_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("magenta_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_dye",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("magenta_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_dye",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("magenta_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:lilac")],
        result: RecipeResultStruct {
            id: "minecraft:magenta_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("magenta_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magenta_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:magenta_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:magenta_dye"),
        result: RecipeResultStruct {
            id: "minecraft:magenta_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:magenta_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:magenta_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:magenta_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:magenta_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:magenta_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:magma_cream"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:magma_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blaze_powder"),
            RecipeIngredientTypes::Simple("minecraft:slime_ball"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:magma_cream",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:mangrove_planks")],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:mangrove_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_mangrove_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:mangrove_logs")],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_mangrove_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mangrove_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:mangrove_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:mangrove_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:paper")),
            ('X', RecipeIngredientTypes::Simple("minecraft:compass")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:map",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
            RecipeIngredientTypes::Simple("minecraft:melon_slice"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:melon",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:melon_slice")],
        result: RecipeResultStruct {
            id: "minecraft:melon_seeds",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:minecart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:enchanted_golden_apple"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mojang_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:moss_block"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:moss_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("mossy_cobblestone"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cobblestone"),
            RecipeIngredientTypes::Simple("minecraft:moss_block"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mossy_cobblestone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("mossy_cobblestone"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cobblestone"),
            RecipeIngredientTypes::Simple("minecraft:vine"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mossy_cobblestone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_cobblestone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_cobblestone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_cobblestone"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_cobblestone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_cobblestone"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_cobblestone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_stone_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_stone_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_stone_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_stone_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:mossy_stone_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mossy_stone_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("mossy_stone_bricks"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:stone_bricks"),
            RecipeIngredientTypes::Simple("minecraft:moss_block"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mossy_stone_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("mossy_stone_bricks"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:stone_bricks"),
            RecipeIngredientTypes::Simple("minecraft:vine"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mossy_stone_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:mud_bricks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:mud_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:mud_bricks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mud_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:mud_bricks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:mud_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:packed_mud"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:mud_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:mud"),
            RecipeIngredientTypes::Simple("minecraft:mangrove_roots"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:muddy_mangrove_roots",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:bowl"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:mushroom_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
            RecipeIngredientTypes::Simple("minecraft:disc_fragment_5"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:music_disc_5",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:nether_brick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:nether_bricks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:nether_brick_fence",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:nether_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:nether_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:nether_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:nether_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:nether_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:nether_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:nether_brick"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:nether_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
            RecipeIngredientTypes::Simple("minecraft:nether_wart"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:nether_wart_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:netherite_ingot"),
        )],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:netherite_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("netherite_ingot"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:netherite_scrap"),
            RecipeIngredientTypes::Simple("minecraft:netherite_scrap"),
            RecipeIngredientTypes::Simple("minecraft:netherite_scrap"),
            RecipeIngredientTypes::Simple("minecraft:netherite_scrap"),
            RecipeIngredientTypes::Simple("minecraft:gold_ingot"),
            RecipeIngredientTypes::Simple("minecraft:gold_ingot"),
            RecipeIngredientTypes::Simple("minecraft:gold_ingot"),
            RecipeIngredientTypes::Simple("minecraft:gold_ingot"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:netherite_ingot",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("netherite_ingot"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:netherite_block")],
        result: RecipeResultStruct {
            id: "minecraft:netherite_ingot",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:netherrack")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:netherite_upgrade_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:netherite_upgrade_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:note_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:oak_planks")],
        result: RecipeResultStruct {
            id: "minecraft:oak_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:oak_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:oak_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:oak_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('W', RecipeIngredientTypes::Simple("minecraft:oak_planks")),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:oak_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('W', RecipeIngredientTypes::Simple("minecraft:oak_planks")),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:oak_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_oak_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:oak_logs")],
        result: RecipeResultStruct {
            id: "minecraft:oak_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:oak_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_oak_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:oak_planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:oak_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_planks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oak_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:oak_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('Q', RecipeIngredientTypes::Simple("minecraft:quartz")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["###", "RRQ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:observer",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:orange_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:orange_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:orange_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:orange_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:orange_dye"),
        result: RecipeResultStruct {
            id: "minecraft:orange_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:orange_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:orange_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:orange_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("orange_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:open_eyeblossom")],
        result: RecipeResultStruct {
            id: "minecraft:orange_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("orange_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:orange_tulip")],
        result: RecipeResultStruct {
            id: "minecraft:orange_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("orange_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:orange_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("orange_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:torchflower")],
        result: RecipeResultStruct {
            id: "minecraft:orange_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:orange_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:orange_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:orange_dye"),
        result: RecipeResultStruct {
            id: "minecraft:orange_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:orange_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:orange_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:orange_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:orange_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:orange_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:orange_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:orange_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:orange_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("oxidized_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:oxidized_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("oxidized_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:oxidized_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
            RecipeIngredientTypes::Simple("minecraft:ice"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:packed_ice",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:mud"),
            RecipeIngredientTypes::Simple("minecraft:wheat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:packed_mud",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:wool")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:painting",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_moss_block"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:pale_moss_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:pale_oak_planks")],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:pale_oak_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_pale_oak_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:pale_oak_logs")],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_pale_oak_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pale_oak_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:pale_oak_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:pale_oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:sugar_cane"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:paper",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:pink_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:pink_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:pink_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:pink_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:pink_dye"),
        result: RecipeResultStruct {
            id: "minecraft:pink_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:pink_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:pink_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pink_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("pink_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:cactus_flower")],
        result: RecipeResultStruct {
            id: "minecraft:pink_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("pink_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:peony")],
        result: RecipeResultStruct {
            id: "minecraft:pink_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("pink_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:pink_petals")],
        result: RecipeResultStruct {
            id: "minecraft:pink_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("pink_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:pink_tulip")],
        result: RecipeResultStruct {
            id: "minecraft:pink_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("pink_dye"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pink_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:pink_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:pink_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:pink_dye"),
        result: RecipeResultStruct {
            id: "minecraft:pink_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:pink_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pink_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:pink_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pink_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:pink_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pink_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:pink_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:pink_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
            ('T', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["TTT", "#X#", "#R#"],
        result: RecipeResultStruct {
            id: "minecraft:piston",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:andesite"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_andesite",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_andesite"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_andesite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_andesite"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_andesite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:basalt"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_basalt",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:blackstone"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'S',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone"),
        )],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple(
            "minecraft:polished_blackstone",
        )],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_blackstone"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_blackstone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'S',
            RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
        )],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_deepslate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_deepslate"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_deepslate_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_deepslate"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_deepslate_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_deepslate"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_deepslate_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:diorite"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_diorite",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_diorite"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_diorite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_diorite"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_diorite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:granite"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_granite",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_granite"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_granite_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_granite"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_granite_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('S', RecipeIngredientTypes::Simple("minecraft:tuff"))],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:polished_tuff",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_tuff"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_tuff_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_tuff"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_tuff_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:polished_tuff"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:polished_tuff_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:gold_ingot")),
        ],
        pattern: &["X X", "X#X", "XRX"],
        result: RecipeResultStruct {
            id: "minecraft:powered_rail",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:prismarine_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:prismarine_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:prismarine"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:prismarine"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:prismarine"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:prismarine_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:pumpkin"),
            RecipeIngredientTypes::Simple("minecraft:sugar"),
            RecipeIngredientTypes::Tagged("#minecraft:eggs"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:pumpkin_pie",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:pumpkin")],
        result: RecipeResultStruct {
            id: "minecraft:pumpkin_seeds",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:purple_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:purple_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:purple_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:purple_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:purple_dye"),
        result: RecipeResultStruct {
            id: "minecraft:purple_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:purple_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:purple_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:purple_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:blue_dye"),
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:purple_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:purple_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:purple_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:purple_dye"),
        result: RecipeResultStruct {
            id: "minecraft:purple_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:purple_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:purple_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:purple_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:purple_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:purple_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:purple_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:purple_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:purple_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'F',
            RecipeIngredientTypes::Simple("minecraft:popped_chorus_fruit"),
        )],
        pattern: &["FF", "FF"],
        result: RecipeResultStruct {
            id: "minecraft:purpur_block",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:purpur_slab"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:purpur_pillar",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&["minecraft:purpur_block", "minecraft:purpur_pillar"]),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:purpur_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&["minecraft:purpur_block", "minecraft:purpur_pillar"]),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:purpur_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:quartz"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:quartz_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:quartz_block"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:quartz_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:quartz_block"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:quartz_pillar",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&[
                "minecraft:chiseled_quartz_block",
                "minecraft:quartz_block",
                "minecraft:quartz_pillar",
            ]),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:quartz_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&[
                "minecraft:chiseled_quartz_block",
                "minecraft:quartz_block",
                "minecraft:quartz_pillar",
            ]),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:quartz_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("rabbit_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:baked_potato"),
            RecipeIngredientTypes::Simple("minecraft:cooked_rabbit"),
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:carrot"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:rabbit_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("rabbit_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:baked_potato"),
            RecipeIngredientTypes::Simple("minecraft:cooked_rabbit"),
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:carrot"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:rabbit_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["X X", "X#X", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:rail",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:raiser_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:raiser_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:raw_copper_block")],
        result: RecipeResultStruct {
            id: "minecraft:raw_copper",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:raw_copper"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:raw_copper_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:raw_gold_block")],
        result: RecipeResultStruct {
            id: "minecraft:raw_gold",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:raw_gold"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:raw_gold_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:raw_iron_block")],
        result: RecipeResultStruct {
            id: "minecraft:raw_iron",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:raw_iron"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:raw_iron_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('C', RecipeIngredientTypes::Simple("minecraft:compass")),
            ('S', RecipeIngredientTypes::Simple("minecraft:echo_shard")),
        ],
        pattern: &["SSS", "SCS", "SSS"],
        result: RecipeResultStruct {
            id: "minecraft:recovery_compass",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:red_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:red_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:red_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:red_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:red_dye"),
        result: RecipeResultStruct {
            id: "minecraft:red_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:red_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:red_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:red_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:red_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("red_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:beetroot")],
        result: RecipeResultStruct {
            id: "minecraft:red_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("red_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:poppy")],
        result: RecipeResultStruct {
            id: "minecraft:red_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("red_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:rose_bush")],
        result: RecipeResultStruct {
            id: "minecraft:red_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("red_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:red_tulip")],
        result: RecipeResultStruct {
            id: "minecraft:red_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:red_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:red_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_nether_bricks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:red_nether_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_nether_bricks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_nether_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_nether_bricks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_nether_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('N', RecipeIngredientTypes::Simple("minecraft:nether_brick")),
            ('W', RecipeIngredientTypes::Simple("minecraft:nether_wart")),
        ],
        pattern: &["NW", "WN"],
        result: RecipeResultStruct {
            id: "minecraft:red_nether_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:red_sand"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:red_sandstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&[
                "minecraft:red_sandstone",
                "minecraft:chiseled_red_sandstone",
            ]),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:red_sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&[
                "minecraft:red_sandstone",
                "minecraft:chiseled_red_sandstone",
                "minecraft:cut_red_sandstone",
            ]),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_sandstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_sandstone"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_sandstone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:red_dye"),
        result: RecipeResultStruct {
            id: "minecraft:red_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:red_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:red_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:red_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:red_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:red_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:redstone_block")],
        result: RecipeResultStruct {
            id: "minecraft:redstone",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:redstone"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:redstone_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('G', RecipeIngredientTypes::Simple("minecraft:glowstone")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" R ", "RGR", " R "],
        result: RecipeResultStruct {
            id: "minecraft:redstone_lamp",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            ('X', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:redstone_torch",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:redstone_torch"),
            ),
            ('I', RecipeIngredientTypes::Simple("minecraft:stone")),
            ('X', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &["#X#", "III"],
        result: RecipeResultStruct {
            id: "minecraft:repeater",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:resin_clump"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:resin_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:resin_bricks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:resin_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:resin_bricks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:resin_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:resin_bricks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:resin_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:resin_brick"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:resin_bricks",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:resin_block")],
        result: RecipeResultStruct {
            id: "minecraft:resin_clump",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('G', RecipeIngredientTypes::Simple("minecraft:glowstone")),
            (
                'O',
                RecipeIngredientTypes::Simple("minecraft:crying_obsidian"),
            ),
        ],
        pattern: &["OOO", "GGG", "OOO"],
        result: RecipeResultStruct {
            id: "minecraft:respawn_anchor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:netherrack")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:rib_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:rib_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('X', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &[" X ", "X#X"],
        result: RecipeResultStruct {
            id: "minecraft:saddle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:sand"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:sandstone",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&["minecraft:sandstone", "minecraft:chiseled_sandstone"]),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::OneOf(&[
                "minecraft:sandstone",
                "minecraft:chiseled_sandstone",
                "minecraft:cut_sandstone",
            ]),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:sandstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:sandstone"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:sandstone_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('I', RecipeIngredientTypes::Simple("minecraft:bamboo")),
            ('~', RecipeIngredientTypes::Simple("minecraft:string")),
        ],
        pattern: &["I~I", "I I", "I I"],
        result: RecipeResultStruct {
            id: "minecraft:scaffolding",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:prismarine_crystals"),
            ),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:prismarine_shard"),
            ),
        ],
        pattern: &["SCS", "CCC", "SCS"],
        result: RecipeResultStruct {
            id: "minecraft:sea_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:sentry_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:sentry_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:shaper_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:shaper_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:iron_ingot"))],
        pattern: &[" #", "# "],
        result: RecipeResultStruct {
            id: "minecraft:shears",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            (
                'W',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
            ('o', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["WoW", "WWW", " W "],
        result: RecipeResultStruct {
            id: "minecraft:shield",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:chest")),
            (
                '-',
                RecipeIngredientTypes::Simple("minecraft:shulker_shell"),
            ),
        ],
        pattern: &["-", "#", "-"],
        result: RecipeResultStruct {
            id: "minecraft:shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
            ),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:silence_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:silence_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:paper"),
            RecipeIngredientTypes::Simple("minecraft:wither_skeleton_skull"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:skull_banner_pattern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:slime_block")],
        result: RecipeResultStruct {
            id: "minecraft:slime_ball",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:slime_ball"))],
        pattern: &["###", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:slime_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('@', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &["@@", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:smithing_table",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:logs")),
            ('X', RecipeIngredientTypes::Simple("minecraft:furnace")),
        ],
        pattern: &[" # ", "#X#", " # "],
        result: RecipeResultStruct {
            id: "minecraft:smoker",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_quartz"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_quartz_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_quartz"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_quartz_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_red_sandstone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_red_sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_red_sandstone"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_red_sandstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_sandstone"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_sandstone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:smooth_sandstone"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_sandstone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:smooth_stone"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:smooth_stone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:blackstone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:snout_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:snout_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:snow_block"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:snow",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:snowball"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:snow_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Tagged("#minecraft:soul_fire_base_blocks"),
            ),
            ('L', RecipeIngredientTypes::Tagged("#minecraft:logs")),
            ('S', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &[" S ", "S#S", "LLL"],
        result: RecipeResultStruct {
            id: "minecraft:soul_campfire",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:soul_torch")),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_nugget")),
        ],
        pattern: &["XXX", "X#X", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:soul_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'S',
                RecipeIngredientTypes::Tagged("#minecraft:soul_fire_base_blocks"),
            ),
            (
                'X',
                RecipeIngredientTypes::OneOf(&["minecraft:coal", "minecraft:charcoal"]),
            ),
        ],
        pattern: &["X", "#", "S"],
        result: RecipeResultStruct {
            id: "minecraft:soul_torch",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:glowstone_dust"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:arrow")),
        ],
        pattern: &[" # ", "#X#", " # "],
        result: RecipeResultStruct {
            id: "minecraft:spectral_arrow",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:purpur_block")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:spire_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spire_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("boat"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["# #", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:spruce_planks")],
        result: RecipeResultStruct {
            id: "minecraft:spruce_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("chest_boat"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:spruce_boat"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:spruce_chest_boat",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_spruce_log"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:spruce_logs")],
        result: RecipeResultStruct {
            id: "minecraft:spruce_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_spruce_log"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:spruce_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:spruce_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:spruce_log"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:spruce_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:amethyst_shard"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:copper_ingot")),
        ],
        pattern: &[" # ", " X ", " X "],
        result: RecipeResultStruct {
            id: "minecraft:spyglass",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("sticks"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Tagged("#minecraft:planks"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:stick",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("sticks"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:bamboo"))],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:stick",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('P', RecipeIngredientTypes::Simple("minecraft:piston")),
            ('S', RecipeIngredientTypes::Simple("minecraft:slime_ball")),
        ],
        pattern: &["S", "P"],
        result: RecipeResultStruct {
            id: "minecraft:sticky_piston",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:stone_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone_bricks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:stone_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone_bricks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:stone_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone_bricks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:stone_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stone_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:stone")],
        result: RecipeResultStruct {
            id: "minecraft:stone_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:stone_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:stone_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:stone_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:stone_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:stone_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:stone_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:stone"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:stone_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:stone_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:stone_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stone")),
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
        ],
        pattern: &[" I ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:stonecutter",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_acacia_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_acacia_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_birch_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_birch_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_cherry_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_cherry_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_crimson_stem"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_crimson_hyphae",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_dark_oak_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_dark_oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_jungle_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_jungle_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_mangrove_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_mangrove_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_oak_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_pale_oak_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_pale_oak_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_spruce_log"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_spruce_wood",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_warped_stem"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:stripped_warped_hyphae",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("sugar"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:honey_bottle")],
        result: RecipeResultStruct {
            id: "minecraft:sugar",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("sugar"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:sugar_cane")],
        result: RecipeResultStruct {
            id: "minecraft:sugar",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:allium"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:azure_bluet"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:blue_orchid"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:closed_eyeblossom"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:cornflower"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:dandelion"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:lily_of_the_valley"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:open_eyeblossom"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:orange_tulip"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:oxeye_daisy"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:pink_tulip"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:poppy"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_tulip"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:torchflower"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:white_tulip"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("suspicious_stew"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:bowl"),
            RecipeIngredientTypes::Simple("minecraft:brown_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:red_mushroom"),
            RecipeIngredientTypes::Simple("minecraft:wither_rose"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:suspicious_stew",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('H', RecipeIngredientTypes::Simple("minecraft:hay_block")),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" R ", "RHR", " R "],
        result: RecipeResultStruct {
            id: "minecraft:target",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:prismarine")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:tide_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:tide_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:amethyst_shard"),
            ),
        ],
        pattern: &[" S ", "SGS", " S "],
        result: RecipeResultStruct {
            id: "minecraft:tinted_glass",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::OneOf(&["minecraft:sand", "minecraft:red_sand"]),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:gunpowder")),
        ],
        pattern: &["X#X", "#X#", "X#X"],
        result: RecipeResultStruct {
            id: "minecraft:tnt",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:tnt"),
            RecipeIngredientTypes::Simple("minecraft:minecart"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:tnt_minecart",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::OneOf(&["minecraft:coal", "minecraft:charcoal"]),
            ),
        ],
        pattern: &["X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:torch",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chest"),
            RecipeIngredientTypes::Simple("minecraft:tripwire_hook"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:trapped_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Tagged("#minecraft:planks")),
            ('I', RecipeIngredientTypes::Simple("minecraft:iron_ingot")),
            ('S', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["I", "S", "#"],
        result: RecipeResultStruct {
            id: "minecraft:tripwire_hook",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff_bricks"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_brick_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff_bricks"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_brick_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff_bricks"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_brick_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            'S',
            RecipeIngredientTypes::Simple("minecraft:polished_tuff"),
        )],
        pattern: &["SS", "SS"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_bricks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff"))],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff"))],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:tuff"))],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:tuff_wall",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[('X', RecipeIngredientTypes::Simple("minecraft:turtle_scute"))],
        pattern: &["XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:turtle_helmet",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:cobblestone")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:vex_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:vex_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
            ),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:ward_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:ward_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_button"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:warped_planks")],
        result: RecipeResultStruct {
            id: "minecraft:warped_button",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_door"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:warped_planks"),
        )],
        pattern: &["##", "##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:warped_door",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_fence"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:warped_planks"),
            ),
        ],
        pattern: &["W#W", "W#W"],
        result: RecipeResultStruct {
            id: "minecraft:warped_fence",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_fence_gate"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'W',
                RecipeIngredientTypes::Simple("minecraft:warped_planks"),
            ),
        ],
        pattern: &["#W#", "#W#"],
        result: RecipeResultStruct {
            id: "minecraft:warped_fence_gate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:fishing_rod")),
            (
                'X',
                RecipeIngredientTypes::Simple("minecraft:warped_fungus"),
            ),
        ],
        pattern: &["# ", " X"],
        result: RecipeResultStruct {
            id: "minecraft:warped_fungus_on_a_stick",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("hanging_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:stripped_warped_stem"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:iron_chain")),
        ],
        pattern: &["X X", "###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:warped_hanging_sign",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("bark"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:warped_stem"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:warped_hyphae",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("planks"),
        ingredients: &[RecipeIngredientTypes::Tagged("#minecraft:warped_stems")],
        result: RecipeResultStruct {
            id: "minecraft:warped_planks",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_pressure_plate"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:warped_planks"),
        )],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:warped_pressure_plate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("shelf"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:stripped_warped_stem"),
        )],
        pattern: &["###", "   ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:warped_shelf",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("wooden_sign"),
        show_notification: true,
        key: &[
            (
                '#',
                RecipeIngredientTypes::Simple("minecraft:warped_planks"),
            ),
            ('X', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " X "],
        result: RecipeResultStruct {
            id: "minecraft:warped_sign",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:warped_planks"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:warped_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("wooden_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:warped_planks"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:warped_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("wooden_trapdoor"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:warped_planks"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:warped_trapdoor",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_chiseled_copper"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_cut_copper_slab"),
        )],
        pattern: &[" M ", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_chiseled_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:chiseled_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_bar"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_bars"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_bars",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_block"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_block"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_block",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:waxed_copper_block"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_bulb"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_bulb"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_bulb",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chain"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_chain"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chest"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_chest"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_door"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_door"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_door",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_golem_statue"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_golem_statue"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_golem_statue",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_copper_block"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_grate"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_grate"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_grate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_lantern"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_lantern"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_trapdoor"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:copper_trapdoor"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_copper_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_copper_block"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cut_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper_slab"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cut_copper_slab"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper_slab",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_cut_copper_stairs"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:cut_copper_stairs"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_cut_copper_stairs",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_chiseled_copper"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_exposed_cut_copper_slab"),
        )],
        pattern: &[" M ", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_chiseled_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_chiseled_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_bar"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_bars"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_bars",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_exposed_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:waxed_exposed_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_exposed_copper_bulb"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_bulb"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_bulb",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chain"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_chain"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chest"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_chest"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_door"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_door"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_door",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_block"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_golem_statue"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_golem_statue"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_golem_statue",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_exposed_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_copper_grate"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_grate"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_grate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_lantern"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_lantern"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_trapdoor"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_copper_trapdoor"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_copper_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_exposed_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_exposed_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper_slab"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper_slab"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper_slab",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_exposed_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_exposed_cut_copper_stairs"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_cut_copper_stairs"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_cut_copper_stairs",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_lightning_rod"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:exposed_lightning_rod"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_exposed_lightning_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_lightning_rod"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:lightning_rod"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_lightning_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_chiseled_copper"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_cut_copper_slab"),
        )],
        pattern: &[" M ", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_chiseled_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_chiseled_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_bar"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_bars"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_bars",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_oxidized_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_oxidized_copper_bulb"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_bulb"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_bulb",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chain"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_chain"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chest"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_chest"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_door"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_door"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_door",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_block"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_golem_statue"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_golem_statue"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_golem_statue",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_copper_grate"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_grate"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_grate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_lantern"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_lantern"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_trapdoor"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_copper_trapdoor"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_copper_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper_slab"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper_slab"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper_slab",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_oxidized_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_oxidized_cut_copper_stairs"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_cut_copper_stairs"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_cut_copper_stairs",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_lightning_rod"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:oxidized_lightning_rod"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_oxidized_lightning_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_chiseled_copper"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_weathered_cut_copper_slab"),
        )],
        pattern: &[" M ", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_chiseled_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_chiseled_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_bar"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_bars"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_bars",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_weathered_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:waxed_weathered_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_weathered_copper_bulb"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_bulb"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_bulb",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chain"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_chain"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_chain",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_chest"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_chest"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_chest",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_door"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_door"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_door",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_block"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_golem_statue"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_golem_statue"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_golem_statue",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:waxed_weathered_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_copper_grate"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_grate"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_grate",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_copper_lantern"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_lantern"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_lantern",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Restone,
        group: Some("waxed_copper_trapdoor"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_copper_trapdoor"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_copper_trapdoor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_weathered_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper_slab"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_weathered_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper_slab"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper_slab"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper_slab",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper_stairs"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:waxed_weathered_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_weathered_cut_copper_stairs"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper_stairs"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_cut_copper_stairs",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("waxed_lightning_rod"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:weathered_lightning_rod"),
            RecipeIngredientTypes::Simple("minecraft:honeycomb"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:waxed_weathered_lightning_rod",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            ('C', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:wayfinder_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:wayfinder_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper_slab"),
        )],
        pattern: &["#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:weathered_chiseled_copper",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Restone,
        group: Some("weathered_copper_bulb"),
        show_notification: true,
        key: &[
            ('B', RecipeIngredientTypes::Simple("minecraft:blaze_rod")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:weathered_copper"),
            ),
            ('R', RecipeIngredientTypes::Simple("minecraft:redstone")),
        ],
        pattern: &[" C ", "CBC", " R "],
        result: RecipeResultStruct {
            id: "minecraft:weathered_copper_bulb",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("weathered_copper_grate"),
        show_notification: true,
        key: &[(
            'M',
            RecipeIngredientTypes::Simple("minecraft:weathered_copper"),
        )],
        pattern: &[" M ", "M M", " M "],
        result: RecipeResultStruct {
            id: "minecraft:weathered_copper_grate",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:weathered_copper"),
        )],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:weathered_cut_copper",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper"),
        )],
        pattern: &["###"],
        result: RecipeResultStruct {
            id: "minecraft:weathered_cut_copper_slab",
            count: 6u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:weathered_cut_copper"),
        )],
        pattern: &["#  ", "## ", "###"],
        result: RecipeResultStruct {
            id: "minecraft:weathered_cut_copper_stairs",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:hay_block")],
        result: RecipeResultStruct {
            id: "minecraft:wheat",
            count: 9u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:white_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:white_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:white_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:white_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:white_dye"),
        result: RecipeResultStruct {
            id: "minecraft:white_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:white_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:white_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:white_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:white_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("white_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:bone_meal")],
        result: RecipeResultStruct {
            id: "minecraft:white_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("white_dye"),
        ingredients: &[RecipeIngredientTypes::Simple(
            "minecraft:lily_of_the_valley",
        )],
        result: RecipeResultStruct {
            id: "minecraft:white_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:white_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:white_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:white_dye"),
        result: RecipeResultStruct {
            id: "minecraft:white_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:white_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:white_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:white_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:white_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:white_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:white_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:white_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:white_terracotta",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: None,
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:string"))],
        pattern: &["##", "##"],
        result: RecipeResultStruct {
            id: "minecraft:white_wool",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:diamond")),
            (
                'C',
                RecipeIngredientTypes::Simple("minecraft:mossy_cobblestone"),
            ),
            (
                'S',
                RecipeIngredientTypes::Simple("minecraft:wild_armor_trim_smithing_template"),
            ),
        ],
        pattern: &["#S#", "#C#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:wild_armor_trim_smithing_template",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:breeze_rod")],
        result: RecipeResultStruct {
            id: "minecraft:wind_charge",
            count: 4u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[(
            'X',
            RecipeIngredientTypes::Simple("minecraft:armadillo_scute"),
        )],
        pattern: &["X  ", "XXX", "X X"],
        result: RecipeResultStruct {
            id: "minecraft:wolf_armor",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["XX", "X#", " #"],
        result: RecipeResultStruct {
            id: "minecraft:wooden_axe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["XX", " #", " #"],
        result: RecipeResultStruct {
            id: "minecraft:wooden_hoe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["XXX", " # ", " # "],
        result: RecipeResultStruct {
            id: "minecraft:wooden_pickaxe",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["X", "#", "#"],
        result: RecipeResultStruct {
            id: "minecraft:wooden_shovel",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["  X", " # ", "#  "],
        result: RecipeResultStruct {
            id: "minecraft:wooden_spear",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: None,
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:stick")),
            (
                'X',
                RecipeIngredientTypes::Tagged("#minecraft:wooden_tool_materials"),
            ),
        ],
        pattern: &["X", "X", "#"],
        result: RecipeResultStruct {
            id: "minecraft:wooden_sword",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:book"),
            RecipeIngredientTypes::Simple("minecraft:ink_sac"),
            RecipeIngredientTypes::Simple("minecraft:feather"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:writable_book",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("banner"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:yellow_wool")),
            ('|', RecipeIngredientTypes::Simple("minecraft:stick")),
        ],
        pattern: &["###", "###", " | "],
        result: RecipeResultStruct {
            id: "minecraft:yellow_banner",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("bed"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:yellow_wool")),
            ('X', RecipeIngredientTypes::Tagged("#minecraft:planks")),
        ],
        pattern: &["###", "XXX"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_bed",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Equipment,
        group: Some("bundle_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:bundles"),
        material: RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
        result: RecipeResultStruct {
            id: "minecraft:yellow_bundle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("dyed_candle"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:candle"),
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_candle",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("carpet"),
        show_notification: true,
        key: &[('#', RecipeIngredientTypes::Simple("minecraft:yellow_wool"))],
        pattern: &["##"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_carpet",
            count: 3u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Building,
        group: Some("concrete_powder"),
        ingredients: &[
            RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:sand"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
            RecipeIngredientTypes::Simple("minecraft:gravel"),
        ],
        result: RecipeResultStruct {
            id: "minecraft:yellow_concrete_powder",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("yellow_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:dandelion")],
        result: RecipeResultStruct {
            id: "minecraft:yellow_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("yellow_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:sunflower")],
        result: RecipeResultStruct {
            id: "minecraft:yellow_dye",
            count: 2u8,
        },
    },
    CraftingRecipeTypes::CraftingShapeless {
        category: RecipeCategoryTypes::Misc,
        group: Some("yellow_dye"),
        ingredients: &[RecipeIngredientTypes::Simple("minecraft:wildflowers")],
        result: RecipeResultStruct {
            id: "minecraft:yellow_dye",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Equipment,
        group: Some("harness"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:yellow_wool")),
            ('G', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('L', RecipeIngredientTypes::Simple("minecraft:leather")),
        ],
        pattern: &["LLL", "G#G"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_harness",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingTransmute {
        category: RecipeCategoryTypes::Misc,
        group: Some("shulker_box_dye"),
        input: RecipeIngredientTypes::Tagged("#minecraft:shulker_boxes"),
        material: RecipeIngredientTypes::Simple("minecraft:yellow_dye"),
        result: RecipeResultStruct {
            id: "minecraft:yellow_shulker_box",
            count: 1u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_glass"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass")),
            ('X', RecipeIngredientTypes::Simple("minecraft:yellow_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_stained_glass",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[(
            '#',
            RecipeIngredientTypes::Simple("minecraft:yellow_stained_glass"),
        )],
        pattern: &["###", "###"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_stained_glass_pane",
            count: 16u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Misc,
        group: Some("stained_glass_pane"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:glass_pane")),
            ('$', RecipeIngredientTypes::Simple("minecraft:yellow_dye")),
        ],
        pattern: &["###", "#$#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_stained_glass_pane",
            count: 8u8,
        },
    },
    CraftingRecipeTypes::CraftingShaped {
        category: RecipeCategoryTypes::Building,
        group: Some("stained_terracotta"),
        show_notification: true,
        key: &[
            ('#', RecipeIngredientTypes::Simple("minecraft:terracotta")),
            ('X', RecipeIngredientTypes::Simple("minecraft:yellow_dye")),
        ],
        pattern: &["###", "#X#", "###"],
        result: RecipeResultStruct {
            id: "minecraft:yellow_terracotta",
            count: 8u8,
        },
    },
];
pub static RECIPES_COOKING: &[CookingRecipeType] = &[
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:baked_potato_from_smelting_potato",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:potato"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:baked_potato",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:baked_potato_from_campfire_cooking_potato",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:potato"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:baked_potato",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:baked_potato_from_smoking_potato",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:potato"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:baked_potato",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:black_glazed_terracotta_from_smelting_black_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:black_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:black_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:blue_glazed_terracotta_from_smelting_blue_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:blue_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:blue_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:brick_from_smelting_clay_ball",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:clay_ball"),
        cookingtime: 200i32,
        experience: 0.3f32,
        result: RecipeResultStruct {
            id: "minecraft:brick",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:brown_glazed_terracotta_from_smelting_brown_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:brown_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:brown_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:charcoal_from_smelting_minecraft_logs_that_burn",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Tagged("#minecraft:logs_that_burn"),
        cookingtime: 200i32,
        experience: 0.15f32,
        result: RecipeResultStruct {
            id: "minecraft:charcoal",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:coal_from_blasting_coal_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("coal"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:coal_ore"),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:coal",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:coal_from_blasting_deepslate_coal_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("coal"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_coal_ore"),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:coal",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:coal_from_smelting_coal_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("coal"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:coal_ore"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:coal",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:coal_from_smelting_deepslate_coal_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("coal"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_coal_ore"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:coal",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_beef_from_smelting_beef",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:beef"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_beef",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_beef_from_campfire_cooking_beef",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:beef"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_beef",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_beef_from_smoking_beef",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:beef"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_beef",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_chicken_from_smelting_chicken",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:chicken"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_chicken",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_chicken_from_campfire_cooking_chicken",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:chicken"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_chicken",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_chicken_from_smoking_chicken",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:chicken"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_chicken",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_cod_from_smelting_cod",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cod"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_cod",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_cod_from_campfire_cooking_cod",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cod"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_cod",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_cod_from_smoking_cod",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cod"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_cod",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_mutton_from_smelting_mutton",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:mutton"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_mutton",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_mutton_from_campfire_cooking_mutton",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:mutton"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_mutton",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_mutton_from_smoking_mutton",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:mutton"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_mutton",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_porkchop_from_smelting_porkchop",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:porkchop"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_porkchop",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_porkchop_from_campfire_cooking_porkchop",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:porkchop"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_porkchop",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_porkchop_from_smoking_porkchop",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:porkchop"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_porkchop",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_rabbit_from_smelting_rabbit",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:rabbit"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_rabbit",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_rabbit_from_campfire_cooking_rabbit",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:rabbit"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_rabbit",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_rabbit_from_smoking_rabbit",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:rabbit"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_rabbit",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cooked_salmon_from_smelting_salmon",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:salmon"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_salmon",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:cooked_salmon_from_campfire_cooking_salmon",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:salmon"),
        cookingtime: 600i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_salmon",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:cooked_salmon_from_smoking_salmon",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:salmon"),
        cookingtime: 100i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:cooked_salmon",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_blasting_copper_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:copper_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_blasting_deepslate_copper_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_copper_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_blasting_raw_copper",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_copper"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_smelting_copper_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:copper_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_smelting_deepslate_copper_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_copper_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:copper_ingot_from_smelting_raw_copper",
        category: RecipeCategoryTypes::Misc,
        group: Some("copper_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_copper"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:copper_nugget_from_blasting_copper_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:copper_pickaxe",
            "minecraft:copper_shovel",
            "minecraft:copper_axe",
            "minecraft:copper_hoe",
            "minecraft:copper_sword",
            "minecraft:copper_spear",
            "minecraft:copper_helmet",
            "minecraft:copper_chestplate",
            "minecraft:copper_leggings",
            "minecraft:copper_boots",
            "minecraft:copper_horse_armor",
            "minecraft:copper_nautilus_armor",
        ]),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:copper_nugget_from_smelting_copper_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:copper_pickaxe",
            "minecraft:copper_shovel",
            "minecraft:copper_axe",
            "minecraft:copper_hoe",
            "minecraft:copper_sword",
            "minecraft:copper_spear",
            "minecraft:copper_helmet",
            "minecraft:copper_chestplate",
            "minecraft:copper_leggings",
            "minecraft:copper_boots",
            "minecraft:copper_horse_armor",
            "minecraft:copper_nautilus_armor",
        ]),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:copper_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cracked_deepslate_bricks_from_smelting_deepslate_bricks",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_bricks"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cracked_deepslate_bricks",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cracked_deepslate_tiles_from_smelting_deepslate_tiles",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_tiles"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cracked_deepslate_tiles",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cracked_nether_bricks_from_smelting_nether_bricks",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:nether_bricks"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cracked_nether_bricks",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cracked_polished_blackstone_bricks_from_smelting_polished_blackstone_bricks",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:polished_blackstone_bricks"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cracked_polished_blackstone_bricks",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cracked_stone_bricks_from_smelting_stone_bricks",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:stone_bricks"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cracked_stone_bricks",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:cyan_glazed_terracotta_from_smelting_cyan_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cyan_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:cyan_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:deepslate_from_smelting_cobbled_deepslate",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cobbled_deepslate"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:deepslate",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:diamond_from_blasting_deepslate_diamond_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("diamond"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_diamond_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:diamond",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:diamond_from_blasting_diamond_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("diamond"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:diamond_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:diamond",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:diamond_from_smelting_deepslate_diamond_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("diamond"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_diamond_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:diamond",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:diamond_from_smelting_diamond_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("diamond"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:diamond_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:diamond",
            count: 1u8,
        },
    }),
    CookingRecipeType::CampfireCooking(CookingRecipe {
        recipe_id: "minecraft:dried_kelp_from_campfire_cooking_kelp",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:kelp"),
        cookingtime: 600i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:dried_kelp",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:dried_kelp_from_smelting_kelp",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:kelp"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:dried_kelp",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smoking(CookingRecipe {
        recipe_id: "minecraft:dried_kelp_from_smoking_kelp",
        category: RecipeCategoryTypes::Food,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:kelp"),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:dried_kelp",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:emerald_from_blasting_deepslate_emerald_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("emerald"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_emerald_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:emerald",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:emerald_from_blasting_emerald_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("emerald"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:emerald_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:emerald",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:emerald_from_smelting_deepslate_emerald_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("emerald"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_emerald_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:emerald",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:emerald_from_smelting_emerald_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("emerald"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:emerald_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:emerald",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:glass_from_smelting_minecraft_smelts_to_glass",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Tagged("#minecraft:smelts_to_glass"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:glass",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_blasting_deepslate_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_gold_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_blasting_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:gold_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_blasting_nether_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:nether_gold_ore"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_blasting_raw_gold",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_gold"),
        cookingtime: 100i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_smelting_deepslate_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_gold_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_smelting_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:gold_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_smelting_nether_gold_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:nether_gold_ore"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gold_ingot_from_smelting_raw_gold",
        category: RecipeCategoryTypes::Misc,
        group: Some("gold_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_gold"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:gold_nugget_from_blasting_golden_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:golden_pickaxe",
            "minecraft:golden_shovel",
            "minecraft:golden_axe",
            "minecraft:golden_hoe",
            "minecraft:golden_sword",
            "minecraft:golden_spear",
            "minecraft:golden_helmet",
            "minecraft:golden_chestplate",
            "minecraft:golden_leggings",
            "minecraft:golden_boots",
            "minecraft:golden_horse_armor",
            "minecraft:golden_nautilus_armor",
        ]),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gold_nugget_from_smelting_golden_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:golden_pickaxe",
            "minecraft:golden_shovel",
            "minecraft:golden_axe",
            "minecraft:golden_hoe",
            "minecraft:golden_sword",
            "minecraft:golden_spear",
            "minecraft:golden_helmet",
            "minecraft:golden_chestplate",
            "minecraft:golden_leggings",
            "minecraft:golden_boots",
            "minecraft:golden_horse_armor",
            "minecraft:golden_nautilus_armor",
        ]),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:gold_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:gray_glazed_terracotta_from_smelting_gray_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:gray_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:gray_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:green_dye_from_smelting_cactus",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cactus"),
        cookingtime: 200i32,
        experience: 1f32,
        result: RecipeResultStruct {
            id: "minecraft:green_dye",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:green_glazed_terracotta_from_smelting_green_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:green_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:green_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_blasting_deepslate_iron_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_iron_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_blasting_iron_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:iron_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_blasting_raw_iron",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_iron"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_smelting_deepslate_iron_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_iron_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_smelting_iron_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:iron_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:iron_ingot_from_smelting_raw_iron",
        category: RecipeCategoryTypes::Misc,
        group: Some("iron_ingot"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:raw_iron"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_ingot",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:iron_nugget_from_blasting_iron_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:iron_pickaxe",
            "minecraft:iron_shovel",
            "minecraft:iron_axe",
            "minecraft:iron_hoe",
            "minecraft:iron_sword",
            "minecraft:iron_spear",
            "minecraft:iron_helmet",
            "minecraft:iron_chestplate",
            "minecraft:iron_leggings",
            "minecraft:iron_boots",
            "minecraft:iron_horse_armor",
            "minecraft:iron_nautilus_armor",
            "minecraft:chainmail_helmet",
            "minecraft:chainmail_chestplate",
            "minecraft:chainmail_leggings",
            "minecraft:chainmail_boots",
        ]),
        cookingtime: 100i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:iron_nugget_from_smelting_iron_pickaxe",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::OneOf(&[
            "minecraft:iron_pickaxe",
            "minecraft:iron_shovel",
            "minecraft:iron_axe",
            "minecraft:iron_hoe",
            "minecraft:iron_sword",
            "minecraft:iron_spear",
            "minecraft:iron_helmet",
            "minecraft:iron_chestplate",
            "minecraft:iron_leggings",
            "minecraft:iron_boots",
            "minecraft:iron_horse_armor",
            "minecraft:chainmail_helmet",
            "minecraft:chainmail_chestplate",
            "minecraft:chainmail_leggings",
            "minecraft:chainmail_boots",
            "minecraft:iron_nautilus_armor",
        ]),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:iron_nugget",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:lapis_lazuli_from_blasting_deepslate_lapis_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("lapis_lazuli"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_lapis_ore"),
        cookingtime: 100i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:lapis_lazuli",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:lapis_lazuli_from_blasting_lapis_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("lapis_lazuli"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:lapis_ore"),
        cookingtime: 100i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:lapis_lazuli",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:lapis_lazuli_from_smelting_deepslate_lapis_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("lapis_lazuli"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_lapis_ore"),
        cookingtime: 200i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:lapis_lazuli",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:lapis_lazuli_from_smelting_lapis_ore",
        category: RecipeCategoryTypes::Misc,
        group: Some("lapis_lazuli"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:lapis_ore"),
        cookingtime: 200i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:lapis_lazuli",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:leaf_litter_from_smelting_minecraft_leaves",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Tagged("#minecraft:leaves"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:leaf_litter",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:light_blue_glazed_terracotta_from_smelting_light_blue_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:light_blue_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:light_blue_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:light_gray_glazed_terracotta_from_smelting_light_gray_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:light_gray_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:light_gray_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:lime_dye_from_smelting_sea_pickle",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:sea_pickle"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:lime_dye",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:lime_glazed_terracotta_from_smelting_lime_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:lime_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:lime_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:magenta_glazed_terracotta_from_smelting_magenta_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:magenta_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:magenta_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:nether_brick_from_smelting_netherrack",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:netherrack"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:nether_brick",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:netherite_scrap_from_smelting_ancient_debris",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:ancient_debris"),
        cookingtime: 200i32,
        experience: 2f32,
        result: RecipeResultStruct {
            id: "minecraft:netherite_scrap",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:netherite_scrap_from_blasting_ancient_debris",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:ancient_debris"),
        cookingtime: 100i32,
        experience: 2f32,
        result: RecipeResultStruct {
            id: "minecraft:netherite_scrap",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:orange_glazed_terracotta_from_smelting_orange_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:orange_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:orange_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:pink_glazed_terracotta_from_smelting_pink_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:pink_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:pink_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:popped_chorus_fruit_from_smelting_chorus_fruit",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:chorus_fruit"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:popped_chorus_fruit",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:purple_glazed_terracotta_from_smelting_purple_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:purple_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:purple_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:quartz_from_smelting_nether_quartz_ore",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:nether_quartz_ore"),
        cookingtime: 200i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:quartz",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:quartz_from_blasting_nether_quartz_ore",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:nether_quartz_ore"),
        cookingtime: 100i32,
        experience: 0.2f32,
        result: RecipeResultStruct {
            id: "minecraft:quartz",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:red_glazed_terracotta_from_smelting_red_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:red_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:red_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:redstone_from_blasting_deepslate_redstone_ore",
        category: RecipeCategoryTypes::Blocks,
        group: Some("redstone"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_redstone_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:redstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Blasting(CookingRecipe {
        recipe_id: "minecraft:redstone_from_blasting_redstone_ore",
        category: RecipeCategoryTypes::Blocks,
        group: Some("redstone"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:redstone_ore"),
        cookingtime: 100i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:redstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:redstone_from_smelting_deepslate_redstone_ore",
        category: RecipeCategoryTypes::Blocks,
        group: Some("redstone"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:deepslate_redstone_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:redstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:redstone_from_smelting_redstone_ore",
        category: RecipeCategoryTypes::Blocks,
        group: Some("redstone"),
        ingredient: RecipeIngredientTypes::Simple("minecraft:redstone_ore"),
        cookingtime: 200i32,
        experience: 0.7f32,
        result: RecipeResultStruct {
            id: "minecraft:redstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:resin_brick_from_smelting_resin_clump",
        category: RecipeCategoryTypes::Misc,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:resin_clump"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:resin_brick",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:smooth_basalt_from_smelting_basalt",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:basalt"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:smooth_basalt",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:smooth_quartz_from_smelting_quartz_block",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:quartz_block"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:smooth_quartz",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:smooth_red_sandstone_from_smelting_red_sandstone",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:red_sandstone"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:smooth_red_sandstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:smooth_sandstone_from_smelting_sandstone",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:sandstone"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:smooth_sandstone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:smooth_stone_from_smelting_stone",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:stone"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:smooth_stone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:sponge_from_smelting_wet_sponge",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:wet_sponge"),
        cookingtime: 200i32,
        experience: 0.15f32,
        result: RecipeResultStruct {
            id: "minecraft:sponge",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:stone_from_smelting_cobblestone",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:cobblestone"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:stone",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:terracotta_from_smelting_clay",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:clay"),
        cookingtime: 200i32,
        experience: 0.35f32,
        result: RecipeResultStruct {
            id: "minecraft:terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:white_glazed_terracotta_from_smelting_white_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:white_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:white_glazed_terracotta",
            count: 1u8,
        },
    }),
    CookingRecipeType::Smelting(CookingRecipe {
        recipe_id: "minecraft:yellow_glazed_terracotta_from_smelting_yellow_terracotta",
        category: RecipeCategoryTypes::Blocks,
        group: None,
        ingredient: RecipeIngredientTypes::Simple("minecraft:yellow_terracotta"),
        cookingtime: 200i32,
        experience: 0.1f32,
        result: RecipeResultStruct {
            id: "minecraft:yellow_glazed_terracotta",
            count: 1u8,
        },
    }),
];
pub fn get_cooking_recipe_with_ingredient(
    ingredient: &Item,
    recipe_type: CookingRecipeKind,
) -> Option<&'static CookingRecipe> {
    RECIPES_COOKING
        .iter()
        .find_map(|recipe| match (recipe, &recipe_type) {
            (CookingRecipeType::Blasting(cooking_recipe), CookingRecipeKind::Blasting)
            | (CookingRecipeType::Smelting(cooking_recipe), CookingRecipeKind::Smelting)
            | (CookingRecipeType::Smoking(cooking_recipe), CookingRecipeKind::Smoking)
            | (
                CookingRecipeType::CampfireCooking(cooking_recipe),
                CookingRecipeKind::CampfireCooking,
            ) => cooking_recipe
                .ingredient
                .match_item(ingredient)
                .then_some(cooking_recipe),
            _ => None,
        })
}
#[doc = r" Get the experience value for a recipe by its recipe ID."]
#[doc = r" Used for calculating XP when extracting from furnace."]
#[doc = r#" Recipe IDs are in vanilla format like "minecraft:iron_ingot_from_smelting_iron_ore""#]
pub fn get_recipe_experience(recipe_id: &str) -> Option<f32> {
    RECIPES_COOKING.iter().find_map(|recipe| {
        let cooking_recipe = match recipe {
            CookingRecipeType::Blasting(r)
            | CookingRecipeType::Smelting(r)
            | CookingRecipeType::Smoking(r)
            | CookingRecipeType::CampfireCooking(r) => r,
        };
        (cooking_recipe.recipe_id == recipe_id).then_some(cooking_recipe.experience)
    })
}
