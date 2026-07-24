# Upstream 3103e97 split adaptation plan

Source: Pumpkin-MC/Pumpkin `3103e97` ("ci: remove macos-15-intel" — actually a multi-area commit).

## Principle
Take independent slices; never force the full zero-copy `ServerPacket<'a>` chain in one shot.
Keep our gameplay/entity/i18n/redstone stack intact.

## Layers

| Layer | Scope | Risk | Status |
|-------|--------|------|--------|
| 0 | `packet_decoder` `split_to` buffer reuse | Low | Done (`d1d6a24`) |
| 1 | Java net write-path: replace `.unwrap()` with log+return/continue; keep_alive `unwrap_or_default` | Low | Done (`809bde2`) |
| 2 | Optional CI: drop macos-15-intel only (workflow yaml) | Low | Optional / skip |
| 3 | `packet_encoder` header-buffer write (no trait change) | Medium | Next candidate |
| 4 | pumpkin-nbt `from_slice` zero-alloc (lifetime on NbtReadHelper) | Medium-High | Later |
| 5 | protocol `ServerPacket<'a>` + serialization `NetworkReadExt` + all packet reads | High (80+ files) | Later on `adapt/upstream-3103e97` |
| 6 | net/java+bedrock consumers of lifetime packet types | High (depends on 5) | With layer 5 |
| 7 | world chat/login signature + TextComponent lifetime | High (depends on 5) | With layer 5 |
| 8 | Cargo.lock/codegen mass churn | Avoid unless needed for 5 | Skip until forced |

## Do not take wholesale
- Full Cargo.lock rewrite without need
- README / pure cosmetics from the same commit message
