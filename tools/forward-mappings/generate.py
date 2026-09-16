#!/usr/bin/env python3
"""Build a forward ID mapping (older server data -> newer client) from two vanilla data reports.

ViaBackwards files map a newer version down to an older one. For a client newer than the server
data we need the opposite direction, so derive it by name from the vanilla data generator reports:

    java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports --output <dir>

The output uses the same NBT layout as the ViaVersion mapping files (direct strategy), so
`pumpkin-codegen`'s `ParsedMappings` reads it unchanged.

Usage: generate.py <old reports dir> <new reports dir> <output .nbt>
"""

import json
import struct
import sys
from pathlib import Path

# Mapping section name -> vanilla registry. Unchanged registries are skipped (identity).
REGISTRY_SECTIONS = {
    "items": "minecraft:item",
    "entities": "minecraft:entity_type",
    "sounds": "minecraft:sound_event",
    "particles": "minecraft:particle_type",
    "data_component_type": "minecraft:data_component_type",
    "argumenttypes": "minecraft:command_argument_type",
    "statistics": "minecraft:custom_stat",
    "environment_attribute": "minecraft:environment_attribute",
    "recipe_serializers": "minecraft:recipe_serializer",
    "attributes": "minecraft:attribute",
    "menus": "minecraft:menu",
    "blockentities": "minecraft:block_entity_type",
    "slot_displays": "minecraft:slot_display",
}

# Entries removed in the newer version that have a direct successor with the same network format.
RENAMES = {
    "minecraft:data_component_type": {"minecraft:swing_animation": "minecraft:attack_animation"},
}


def load(path: Path) -> dict:
    with path.open() as f:
        return json.load(f)


def registry_ids(registries: dict, name: str) -> dict[str, int]:
    entries = registries.get(name, {}).get("entries", {})
    return {key: value["protocol_id"] for key, value in entries.items()}


def block_state_ids(blocks: dict) -> dict[tuple, int]:
    ids = {}
    for name, block in blocks.items():
        for state in block["states"]:
            props = tuple(sorted(state.get("properties", {}).items()))
            ids[(name, props)] = state["id"]
    return ids


def forward(old: dict, new: dict, renames: dict) -> tuple[list[int], int] | None:
    if old == new:
        return None
    table = [-1] * (max(old.values()) + 1)
    for key, old_id in old.items():
        new_key = renames.get(key, key)
        table[old_id] = new.get(new_key, -1)
    return table, max(new.values()) + 1


def nbt_string(value: str) -> bytes:
    raw = value.encode()
    return struct.pack(">H", len(raw)) + raw


def nbt_section(name: str, table: list[int], mapped_size: int) -> bytes:
    body = b"\x01" + nbt_string("id") + b"\x00"
    body += b"\x03" + nbt_string("mappedSize") + struct.pack(">i", mapped_size)
    body += b"\x0b" + nbt_string("val") + struct.pack(">i", len(table))
    body += struct.pack(f">{len(table)}i", *table)
    return b"\x0a" + nbt_string(name) + body + b"\x00"


def main() -> None:
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    old_dir, new_dir, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])

    sections = []
    old_states = block_state_ids(load(old_dir / "blocks.json"))
    new_states = block_state_ids(load(new_dir / "blocks.json"))
    mapped = forward(old_states, new_states, {})
    if mapped:
        sections.append(("blockstates", *mapped))

    old_reg = load(old_dir / "registries.json")
    new_reg = load(new_dir / "registries.json")
    for section, registry in REGISTRY_SECTIONS.items():
        old_ids = registry_ids(old_reg, registry)
        new_ids = registry_ids(new_reg, registry)
        if not old_ids:
            continue
        mapped = forward(old_ids, new_ids, RENAMES.get(registry, {}))
        if mapped:
            sections.append((section, *mapped))

    root = b"\x0a" + nbt_string("")
    for name, table, mapped_size in sections:
        root += nbt_section(name, table, mapped_size)
        unmapped = table.count(-1)
        print(f"{name}: {len(table)} -> {mapped_size} ids, {unmapped} without successor")
    root += b"\x00"
    out.write_bytes(root)


if __name__ == "__main__":
    main()
