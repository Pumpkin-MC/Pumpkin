# PrismarineJS Protocol Data

> ProtoDef-format packet definitions from PrismarineJS/minecraft-data.
> Covers 5 MC versions matching our gameplay data harvest.

## Files

| File | Description |
|------|-------------|
| `protocolVersions.json` | MC version string → protocol version number mapping (all known versions) |
| `versions.json` | List of all PrismarineJS-supported MC version strings |

## Per-Version Protocol Data

Located at `.claude/specs/data/{version}/prismarine/protocol/`:

| File | Description |
|------|-------------|
| `protocol.json` | Full ProtoDef packet definitions (types, states, packets, field schemas) |
| `proto.yml` | Human-readable YAML version of protocol.json |
| `version.json` | Version metadata (version string, protocol number) |

## Cross-Version Packet Count Matrix

| State | Dir | 1.12.2 | 1.14.4 | 1.16.5 | 1.18.2 | 1.21.4 |
|-------|-----|--------|--------|--------|--------|--------|
| handshaking | S | 2 | 2 | 2 | 2 | 2 |
| status | C | 2 | 2 | 2 | 2 | 2 |
| status | S | 2 | 2 | 2 | 2 | 2 |
| login | C | 4 | 5 | 5 | 5 | 6 |
| login | S | 2 | 3 | 3 | 3 | 5 |
| configuration | C | — | — | — | — | 17 |
| configuration | S | — | — | — | — | 10 |
| play | C | 80 | 93 | 92 | 104 | 131 |
| play | S | 33 | 46 | 48 | 48 | 62 |
| **Total** | **C** | **86** | **100** | **99** | **111** | **156** |
| **Total** | **S** | **39** | **53** | **55** | **55** | **81** |
| **Grand Total** | | **125** | **153** | **154** | **166** | **237** |

C = clientbound (toClient), S = serverbound (toServer)

## Shared Type Count

| Version | Shared Types |
|---------|-------------|
| 1.12.2 | 31 |
| 1.14.4 | 38 |
| 1.16.5 | 39 |
| 1.18.2 | 42 |
| 1.21.4 | 84 |

## Play Packet Delta vs 1.21.4

| Version | Clientbound | | | Serverbound | | |
|---------|-------|-------|---------|-------|-------|---------|
| | Shared | Added | Removed | Shared | Added | Removed |
| 1.12.2 | 66 | 65 | 14 | 29 | 33 | 4 |
| 1.14.4 | 80 | 51 | 13 | 41 | 21 | 5 |
| 1.16.5 | 80 | 51 | 12 | 44 | 18 | 4 |
| 1.18.2 | 96 | 35 | 8 | 45 | 17 | 3 |

## Protocol States

1. **handshaking** — Initial connection (set_protocol + legacy_server_list_ping)
2. **status** — Server list ping (ping_start, server_info, ping)
3. **login** — Authentication, encryption, compression setup
4. **configuration** — Post-1.20.2 configuration phase (registry data, resource packs, feature flags)
5. **play** — Main gameplay (entities, chunks, movement, combat, inventory, etc.)

## ProtoDef Format

Packets are defined as ProtoDef containers:
```json
["container", [
  {"name": "entityId", "type": "varint"},
  {"name": "objectUUID", "type": "UUID"},
  {"name": "x", "type": "f64"},
  ...
]]
```

Packet ID mapping uses a mapper type:
```json
["mapper", {"type": "varint", "mappings": {"0x00": "bundle_delimiter", "0x01": "spawn_entity", ...}}]
```

## Inheritance

Not all versions have their own protocol.json. Inheritance from `dataPaths.json`:
- 1.12.2: own protocol
- 1.14.4: own protocol
- 1.16.5: inherits from 1.16.2
- 1.18.2: own protocol
- 1.21.4: own protocol
