use indexmap::IndexMap;
use semver::Version;
use serde::Deserialize;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

#[derive(Deserialize)]
struct CustomStatisticEntry {
    id: i32,
}

#[derive(Deserialize)]
struct StatisticData {
    id: i32,
    registry: String,
    entries: IndexMap<String, CustomStatisticEntry>,
}

pub fn build() -> String {
    let stats_json =
        fs::read_to_string("../../assets/stats.json").expect("Failed to read stats.json");
    let stats_data: IndexMap<String, StatisticData> =
        serde_json::from_str(&stats_json).expect("Failed to parse stats.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("statistics");

    // Category enum
    let mut category_enum = Enum::empty();
    let mut category_entries: Vec<(&String, &StatisticData)> = stats_data.iter().collect();
    category_entries.sort_by_key(|(_, data)| data.id);

    for (raw_name, _) in category_entries {
        let name = raw_name
            .strip_prefix("minecraft:")
            .unwrap_or(raw_name)
            .replace('_', "-");
        category_enum.case(name);
    }

    interface.type_def(TypeDef::new(
        "statistic-category",
        TypeDefKind::Enum(category_enum),
    ));

    // Custom statistics enum
    if let Some(custom_data) = stats_data.get("minecraft:custom") {
        let mut custom_enum = Enum::empty();
        let mut entries: Vec<(&String, &CustomStatisticEntry)> =
            custom_data.entries.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.id);

        for (raw_name, _) in entries {
            let name = raw_name
                .strip_prefix("minecraft:")
                .unwrap_or(raw_name)
                .replace('_', "-");
            custom_enum.case(name);
        }

        interface.type_def(TypeDef::new(
            "custom-statistic",
            TypeDefKind::Enum(custom_enum),
        ));
    }

    package.interface(interface);
    package.to_string()
}
