# Bedrock Assets

This directory contains a number of different data files used to help support connecting via Bedrock Edition clients (including Java => Bedrock remapping).

- `block_states.nbt`
    - mined from BDS (file hosted at [pmmp/BedrockData](https://github.com/pmmp/BedrockData/blob/master/canonical_block_states.nbt), `canonical_block_states.nbt`)
    - Provides a listing of all blocks and block states that exist in Bedrock. Used to build a mapping from Java block states to Bedrock ones by matching (string) identifiers and data components.
- `blocks.nbt`
    - downloaded from [GeyserMC/mappings](https://github.com/GeyserMC/mappings)
    - Defines the exact Bedrock block identifier and property mappings for every Java Edition block state ID. Used in code generation to translate Java block states to Bedrock counterparts.
- `item_components.nbt`
    - mined from BDS (file hosted at [CloudburstMC/Data](https://github.com/CloudburstMC/Data))
- `runtime_item_states.nbt`
    - mined from BDS (file hosted at [CloudburstMC/Data](https://github.com/CloudburstMC/Data))
- `item_data_overrides.json`
    - adapted from `GeyserMC/mappings` `items.json`.
    - Strips everything except the `bedrock_data` field (making it the value of each corresponding top-level key), omitting any `0` values.
    - Most of `items.json` is automatically generated, but that value appears to be manually maintained by the Geyser team. We separate that out for our own use, while keeping the rest generated.
- `biomes.json`
    - downloaded from [GeyserMC/mappings](https://github.com/GeyserMC/mappings)
    - Maps Java Edition biome identifiers to their corresponding Bedrock Edition biome ID. Used in code generation to translate Java biomes to Bedrock counterparts.
- `biome_definitions.nbt`
    - downloaded from [Kaooot/bedrock-network-data](https://github.com/Kaooot/bedrock-network-data/blob/master/release/1.26.40/biome_definitions.nbt)
    - Contains the gzip-compressed vanilla biome registry extracted from Bedrock Dedicated Server 1.26.40.
    - Validated and converted by `pumpkin-codegen` into the static `BiomeDefinitionList` wire payload used during Bedrock world initialization.
- `player_geometry.json`
    - adapted from [GeyserMC/Geyser](https://github.com/GeyserMC/Geyser/blob/master/core/src/main/resources/bedrock/geometries/geo.json) (MIT License)
    - Provides valid standard wide and slim Bedrock player geometry for Java Edition player skins.
    - See `LICENSE-GEYSER` for the source license and copyright notice.

The Java-to-Bedrock skin conversion also uses Geyser's required-opacity masks under the same license.
