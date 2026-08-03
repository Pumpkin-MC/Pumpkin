# Protocol correctness: a real bug class to hunt for

Source: `net.minecraft.network.protocol.**` (structure only — the actual bug found this session
was in Pumpkin's own packet-writer code, not a vanilla-source question).

## The bug class: presence flag written, payload not

Found in `pumpkin-protocol/src/java/client/config/add_resource_pack.rs`:
`CConfigAddResourcePack::write_packet_data` had a branch that wrote the boolean "has prompt
message" flag as `true`, then wrote **zero bytes** for the actual NBT-encoded text component that
should have followed it — a `// TODO` stub left in place of the real encode call. Any server
config with a non-empty `prompt_message` would desync the client's packet parser at that point
(client reads the `true` flag, then tries to decode a text component from bytes that belong to
the *next* packet).

This is a structurally easy bug to introduce and easy to miss in review, because:

- `cargo check`/`clippy` can't catch it — the code compiles fine, it's just semantically wrong.
- It's only reachable when the optional field is actually populated (`Some(...)`), so a
  server running with defaults (empty prompt message) never triggers it — the bug can sit latent
  through normal testing.
- The fix (`write.write_slice(&prompt.encode())?`) is one line once found.

## Where to hunt for more of these

Grep `pumpkin-protocol/src/**` for `write_bool(true)` (or any presence-flag write) followed by a
comment (`// TODO`, `// unimplemented`, or just nothing) instead of an actual payload write for
the `Some` branch. Also worth checking the *reverse* direction — packet **readers** that read a
presence flag but don't actually consume the payload bytes when the flag is `false`/`true`
inconsistently with what the writer produces.

General checklist when reviewing any packet's read/write pair:

1. **Field order** — does the writer write fields in the exact order the reader reads them?
2. **VarInt vs fixed-width** — Minecraft's protocol mixes `VarInt`, `VarLong`, and fixed-width
   ints inconsistently across packet types; using the wrong one for a given field silently
   corrupts everything after it in the same packet, not just that field.
3. **Optional-field presence flags** — for every `Option<T>` field encoded as (bool flag) + (T
   payload if true), verify BOTH the flag write AND the payload write exist, and that they're
   only skipped together, never independently (this is exactly the bug class above).
4. **NBT encoding** — text components, item NBT, and similar structured payloads need their
   actual encoder called, not just a length-prefix or type-tag stub.

If you find another instance of this exact bug class, cite this file in the commit body as the
reason you knew to check for it.
