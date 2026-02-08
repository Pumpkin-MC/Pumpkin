# Session 005 — Architect — DTO Architecture for 1.16.5 Support

**Date:** 2026-02-07
**Agent:** Architect
**Role:** Scout (scoping unknown territory) + Contract Specialist (cross-agent boundary analysis)

## Preamble

Read all sessions from 2026-02-07 (001_redstone through 004_architect). Read decisions/architect.md through ARCH-016. No requests from other agents that affect this scoping work.

## What This Session Does

Scopes the DTO (Data Transfer Object) layer needed to support Minecraft 1.16.5 (protocol 754) clients connecting to Pumpkin's 1.21.11 (protocol 774) server. Produces an architecture proposal, not code.

## Key Discovery: Pumpkin Already Has Multi-Version Infrastructure

The codebase is much better prepared than expected:

| Feature | Where | Status |
|---|---|---|
| `MinecraftVersion` enum with 1.16.x variants | `pumpkin-util/src/version.rs:38-43` | Done |
| Per-client version tracking | `net/java/mod.rs:68` (`JavaClient.version`) | Done |
| Version set in handshake | `net/java/handshake.rs:13` | Done |
| `PacketId` struct with version mapping | `pumpkin-data/build/packet.rs:38-54` | Done (1.21.7 + 1.21.11) |
| JSON-driven packet ID generation | `assets/packet/` | Done (extensible) |
| Version-conditional logic | `net/java/config.rs:182` | Precedent exists |
| `ClientPacket::write_packet_data(_version)` | `pumpkin-protocol/src/packet.rs:26` | Signature ready, param unused |

## The 5 Major Translation Layers (1.16.5 → 1.21.11)

### Layer 1: Connection State Machine (CRITICAL)

**Problem:** 1.16.5 has NO `Config` state. Connection flow is:
- 1.16.5: `Handshake → Login → Play`
- 1.21.11: `Handshake → Login → Config → Play`

Pumpkin's entire post-login flow sends registry data, tags, known packs, and finish config during the Config phase. For 1.16.5 clients, all of this must happen during Login or early Play.

**DTO approach:** Version-aware state machine in `JavaClient`. For protocol < 764 (pre-1.20.2), skip Config state entirely:
- After login success, synthesize the Join Game packet (1.16.5 format) which includes registry/dimension data inline
- Fire tags during early Play instead of Config
- Skip `CFinishConfig` / `SAcknowledgeFinishConfig` entirely

**Impact:** `net/java/mod.rs` (connection state transitions), `net/java/config.rs` (must be skippable), `net/java/login.rs` (must handle registry data for old clients).

### Layer 2: Chunk Format (HIGH)

**Problem:**
- 1.16.5: World height 0-255 (16 sections), biomes per-chunk (1024 ints, 4×4×4 grid)
- 1.18+: World height -64..320 (24 sections), biomes per-section (palette-based)
- 1.16.5 chunk packet includes a bitmask of present sections; 1.18+ uses a different encoding

**DTO approach:** `ChunkDataDto` that translates Pumpkin's internal 24-section chunks to 16-section chunks for 1.16.5:
- Clamp Y range to 0-255 (drop sections below 0 and above 255)
- Convert palette-based biomes back to 1024-int array
- Generate section bitmask from present sections
- Different heightmap format

**Impact:** `pumpkin-protocol/src/java/client/play/` chunk packets, `pumpkin-world/` chunk serialization.

### Layer 3: Item Components → NBT (HIGH)

**Problem:** 1.20.5 replaced item NBT with "Item Components" — a structured, typed system. 1.16.5 uses the old NBT format for all item data (enchantments, display name, lore, damage, etc.).

**DTO approach:** `ItemStackDto` that converts between:
- Internal: Component-based ItemStack (1.21.11 format)
- Wire: NBT-based ItemStack (1.16.5 format)

Key translations: `minecraft:enchantments` component → `{Enchantments:[{id:"...",lvl:1s}]}` NBT, `minecraft:custom_name` component → `{display:{Name:"..."}}` NBT, etc.

**Impact:** Every inventory/container packet, creative mode, item entity packets.

### Layer 4: Packet ID Remapping (MEDIUM)

**Problem:** Packet IDs shifted significantly between 1.16.5 and 1.21.11. Many packets were added (damage event, hurt animation, player info update replacing player info), some removed, some split.

**DTO approach:** Extend existing `PacketId` struct:
```rust
pub struct PacketId {
    pub latest_id: i32,
    pub v1_21_7_id: i32,
    pub v1_16_5_id: i32,  // NEW
}
```
Add `1_16_5_packets.json` to `assets/packet/`. The build system already handles this pattern — just add a third version file.

For packets that don't exist in 1.16.5 (return -1), the DTO layer must either:
- Suppress them (e.g., `CChunkBatchStart` doesn't exist in 1.16.5)
- Translate them to 1.16.5 equivalents (e.g., `CPlayerInfoUpdate` → old `CPlayerInfo`)

**Impact:** `pumpkin-data/build/packet.rs`, dispatch in `net/java/mod.rs`.

### Layer 5: Registry and Login Packets (MEDIUM)

**Problem:**
- 1.16.5 sends dimension codec + dimension data inside the Join Game packet
- 1.21.11 sends registries during Config, then Join Game references them by ID
- 1.16.5 has no Known Packs, no Feature Flags, no Cookie system

**DTO approach:** `JoinGameDto` that constructs the 1.16.5 Join Game packet with inline registry data:
- Embed dimension codec NBT (overworld, nether, end dimension types)
- Embed biome registry NBT
- Include hashed seed, game mode, previous game mode, world names, etc.
- Different field ordering than 1.21.11's Login packet

**Impact:** `net/java/login.rs`, `pumpkin-protocol/src/java/client/play/` login packet.

## Secondary Concerns (LOWER priority)

| Concern | Version introduced | Complexity |
|---|---|---|
| Signed chat system | 1.19 | Medium — 1.16.5 uses simple unsigned chat |
| Damage event packet | 1.19.4 | Low — translate to entity status + sound |
| Hurt animation packet | 1.19.4 | Low — translate to entity animation |
| Player Info Update | 1.19.3 | Medium — replaced single Player Info packet |
| Simulation distance | 1.18 | Low — just don't send |
| Bundle delimiter | 1.19.4 | Low — just don't send |
| Chunk batch system | 1.20.2 | Low — just don't send, send chunks immediately |

## Proposed Architecture

```
                    ┌──────────────────────────┐
                    │    Internal Game State    │
                    │  (1.21.11 canonical model)│
                    └──────────┬───────────────┘
                               │
                    ┌──────────▼───────────────┐
                    │     DTO Translation       │
                    │  pumpkin-protocol/src/dto/ │
                    │                           │
                    │  match client.version {   │
                    │    V_1_21_x => passthrough │
                    │    V_1_16_5 => translate() │
                    │  }                        │
                    └──────────┬───────────────┘
                               │
                    ┌──────────▼───────────────┐
                    │    Wire Format + Send     │
                    │   (per-version encoding)  │
                    └──────────────────────────┘
```

### File Structure

```
pumpkin-protocol/src/dto/
├── mod.rs              # VersionAdapter trait
├── v1_21.rs            # Passthrough (identity transform)
├── v1_16_5/
│   ├── mod.rs          # 1.16.5 adapter
│   ├── chunks.rs       # Chunk format translation
│   ├── items.rs        # Item component → NBT
│   ├── login.rs        # Join Game with inline registries
│   └── player_info.rs  # Player info packet translation
```

### Key Trait

```rust
pub trait VersionAdapter: Send + Sync {
    fn version(&self) -> MinecraftVersion;

    /// Translate outgoing packet for this client's version
    fn adapt_outgoing(&self, packet: &dyn ClientPacket) -> Box<dyn ClientPacket>;

    /// Translate incoming packet from this client's version
    fn adapt_incoming(&self, packet_id: i32, payload: &[u8]) -> Result<RawPacket, ReadingError>;

    /// Whether this version has the Config connection state
    fn has_config_state(&self) -> bool;

    /// Build the Join Game packet for this version (different formats)
    fn build_login_packet(&self, game_join_data: &GameJoinData) -> Box<dyn ClientPacket>;
}
```

### Integration Point

In `JavaClient`, after handshake stores the version:

```rust
// net/java/mod.rs — after handshake
let adapter = VersionAdapter::for_version(self.version.load());
self.version_adapter.store(adapter);
```

Then in packet sending:
```rust
// Instead of: self.send_packet(&some_packet)
// Becomes:    self.send_versioned_packet(&some_packet)
// Which internally calls adapter.adapt_outgoing()
```

## What Agents Need To Know

- **Protocol agent:** You'll own `pumpkin-protocol/src/dto/`. The VersionAdapter trait and all version-specific translations live in your scope.
- **Core agent:** Connection state machine needs version branching. Config state is skipped for pre-1.20.2 clients.
- **WorldGen agent:** Chunk serialization must support 16-section format for old clients.
- **Items agent:** ItemStack wire format differs (components vs NBT). You'll need to provide the translation logic.
- **Entity agent:** Entity metadata indices changed between versions. Some entity types don't exist in 1.16.5.

## Effort Estimate

| Layer | Complexity | Files touched | Depends on |
|---|---|---|---|
| Packet ID remapping | Low | 2 (build.rs + JSON) | PrismarineJS packet data |
| Connection state machine | High | 3-4 (net/java/) | Layer 4 + 5 |
| Chunk format | High | 2-3 (protocol + world) | — |
| Item components → NBT | High | 2-3 (protocol + inventory) | — |
| Registry/Login packets | Medium | 2 (protocol) | Layer 1 |

**Total:** ~15-20 files, ~2000-3000 lines of DTO translation code.

## Decisions Made

**REVISED:** Priority order changed from "1.16.5 first" to tiered rollback: **1.18 > 1.16.5 > (1.14.x > 1.12)**. Work backwards from current — smallest delta first.

### ARCH-017: Tiered implementation — 1.18 first, then 1.16.5
**Decision:** Tier 1 is 1.18.2 (protocol 758) — closest to current, validates DTO plumbing with minimal delta (packet ID remapping + minor field changes). Tier 2 is 1.16.5 (protocol 754) — adds Config state bypass, item component→NBT, dimension codec. Tier 3 is 1.14.x (protocol 477) — lower priority. Tier 4 is 1.12.2 (protocol 340) — pre-Flattening stretch goal.
**Rationale:** Smallest delta first. Each tier adds one more translation layer incrementally. Avoids debugging all 5 layers at once.
**Affects:** Protocol, Core, all agents
**Status:** active — DEFERRED (Phase 2, scoped)

### ARCH-018: Config state bypass for pre-1.20.2 clients
**Decision:** Clients with protocol < 764 (pre-1.20.2) skip the Config connection state entirely. Registry data, tags, and feature flags are embedded in the Join Game packet or sent during early Play, matching vanilla 1.16.5 behavior.
**Rationale:** The Config state was added in 1.20.2. Trying to force old clients through it would break the connection flow. The DTO adapter handles this transparently.
**Affects:** Protocol, Core
**Status:** active — DEFERRED (Phase 2)

### ARCH-019: DTO module lives in pumpkin-protocol/src/dto/
**Decision:** All version translation code lives in `pumpkin-protocol/src/dto/`. Protocol agent owns it. The DTO module imports internal types but does not modify them. Existing packet definitions stay untouched (ARCH-011 compliant).
**Rationale:** The DTO is a protocol concern — it translates wire formats. Keeping it in pumpkin-protocol/ maintains the single-crate boundary for network serialization. New module, no existing code changes.
**Affects:** Protocol
**Status:** active — DEFERRED (Phase 2)

## What I Need From Others

- **Protocol:** When Phase 2 activates, you'll need to source `1_16_5_packets.json` from PrismarineJS or wiki.vg data. The format matches existing `1_21_7_packets.json`.
- **Items:** You'll need to document the Component→NBT mapping for all item types you've implemented.
- **WorldGen:** You'll need to provide a chunk serialization method that outputs 1.16.5 format (16 sections, biome array).

## What Others Should Know

- The existing `PacketId` system is already multi-version-ready. Adding 1.16.5 = adding one JSON file + one struct field.
- `ClientPacket::write_packet_data` already takes `&MinecraftVersion` but ignores it. Phase 2 activates this parameter.
- `JavaClient.version` is already per-client and set during handshake. No new infrastructure needed for version tracking.
- The Config state bypass (ARCH-018) is the single hardest problem. Everything else is field-level translation.
