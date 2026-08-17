use std::collections::BTreeMap;
use std::fs;

pub fn build() -> String {
    let enchantments: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&fs::read_to_string("../../assets/enchantments.json").unwrap())
            .expect("Failed to parse enchantments.json");

    let mut enchantment_vec = enchantments.keys().collect::<Vec<_>>();
    enchantment_vec.sort();

    let mut cases = String::new();
    for raw_name in enchantment_vec {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-");
        cases.push_str(&format!("    {name},\n"));
    }

    format!(
        r##"package pumpkin:plugin@0.1.0;

interface enchantments {{
  use text.{{text-component}};

  /// Equipment slot where an enchantment is active.
  enum attribute-modifier-slot {{
    any,
    main-hand,
    off-hand,
    hand,
    feet,
    legs,
    chest,
    head,
    armor,
    body,
    saddle,
  }}

  /// Vanilla enchantments enum.
  enum enchantment {{
{cases}  }}

  /// Represents a custom enchantment definition.
  record custom-enchantment {{
    /// Unique identifier for the enchantment (e.g. "my_plugin:lifesteal").
    id: string,
    /// Description or display name of the enchantment.
    description: text-component,
    /// Maximum level of the enchantment (e.g. 1..=10).
    max-level: u32,
    /// Base anvil repair/combination cost multiplier.
    anvil-cost: u32,
    /// Tag or item pattern for supported items (e.g. "#minecraft:enchantable/weapon").
    supported-items: string,
    /// Weight / rarity of the enchantment (higher = more common, default 5).
    weight: u32,
    /// Equipment slots where this enchantment is active.
    slots: list<attribute-modifier-slot>,
    /// List of exclusive/conflicting enchantment IDs.
    exclusive-set: list<string>,
  }}

  /// Global manager for registering and querying custom enchantments.
  resource enchantment-manager {{
    /// Registers a new custom enchantment with the server.
    register-enchantment: func(enchantment: custom-enchantment) -> result<_, string>;

    /// Gets an enchantment definition by its ID.
    get-enchantment: func(id: string) -> option<custom-enchantment>;

    /// Checks if an enchantment ID is registered.
    has-enchantment: func(id: string) -> bool;

    /// Returns all registered custom enchantment IDs.
    get-all-enchantment-ids: func() -> list<string>;
  }}
}}
"##
    )
}
