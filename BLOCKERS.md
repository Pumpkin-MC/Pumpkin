## Pumpkin blocker inventory (runtime hot paths)

This file tracks concrete crash/blocker sites in this checkout that directly impact “server stays up” and “players can join/play”.

### Highest priority: crash-safety in networking

- **Java packet serialization panics** (`unwrap()` on write failures)
  - `pumpkin/src/net/java/mod.rs`: `enqueue_packet`, `send_packet_now`
- **Bedrock frame handling can panic / UB**
  - `pumpkin/src/net/bedrock/mod.rs`: `handle_frame_set` uses `unwrap()` on `handle_frame`
  - `pumpkin/src/net/bedrock/mod.rs`: reassembly uses `unsafe unwrap_unchecked()` and `unwrap()` on map removes / option refs
- **Bedrock parsing uses unchecked reads**
  - `pumpkin/src/net/bedrock/mod.rs`: multiple `S*::read(reader).unwrap()` in play packet dispatch

### Highest priority: crash-safety in chunk scheduling / chunk system

- **Chunk generation scheduling hard panics on missing dependency chunks**
  - `pumpkin-world/src/chunk_system/schedule.rs`: `None => panic!(...)` while assembling a generation cache
- **Ticket removal can panic**
  - `pumpkin-world/src/chunk_system/chunk_loading.rs`: `Entry::Vacant(_) => panic!()`
- **Stage/state transitions use panics**
  - `pumpkin-world/src/chunk_system/chunk_state.rs`: multiple `panic!()` for invalid transitions
- **Generation cache assumes specific variants**
  - `pumpkin-world/src/chunk_system/generation_cache.rs`: `_ => panic!()` and `StagedChunkEnum::Empty => panic!(...)`

### World IO / worldgen blockers

- **Anvil chunk format has unimplemented compression variants**
  - `pumpkin-world/src/chunk/format/anvil.rs`: `Compression::Custom => todo!()`, `Self::Custom => todo!()`
- **Surface rule engine has unsupported variants**
  - `pumpkin-world/src/generation/surface/rule.rs`: `MaterialRule::Unsupported => todo!()`
- **Noise/Aquifer assumptions can panic**
  - `pumpkin-world/src/generation/noise/aquifer_sampler.rs`: `panic!("Expected Aquifer")`

### Player/chunk streaming fragility

- **Frequent unwraps in player movement/chunk streaming**
  - `pumpkin/src/entity/player.rs`
  - `pumpkin/src/world/chunker.rs`

---

### Closure policy

- Replace **panic/todo/unchecked** in runtime hot paths with:
  - **recoverable errors** (disconnect client / skip task / reschedule), and
  - **structured logs** with enough context to diagnose without crashing.
