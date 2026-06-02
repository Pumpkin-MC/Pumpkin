# Bedrock Assets

This directory contains a number of different data files used to help support connecting via Bedrock Edition clients (including Java => Bedrock remapping).

- `block_states.nbt`
    - mined from BDS (taken from where? `BetterAltay`? `dragonfly`?)
    - Provides a listing of all blocks and block states that exist in Bedrock. Used to build a mapping from Java block states to Bedrock ones by matching (string) identifiers and data components.
- `item_components.nbt`
    - mined from BDS (file hosted at [CloudburstMC/Data](https://github.com/CloudburstMC/Data))
- `runtime_item_states.nbt`
    - mined from BDS (file hosted at [CloudburstMC/Data](https://github.com/CloudburstMC/Data))
- `item_data_overrides.json`
    - adapted from `GeyserMC/mappings` `items.json`.
    - Strips everything except the `bedrock_data` field (making it the value of each corresponding top-level key), omitting any `0` values.
    - Most of `items.json` is automatically generated, but that value appears to be manually maintained by the Geyser team. We separate that out for our own use, while keeping the rest generated.
