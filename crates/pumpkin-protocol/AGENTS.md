# pumpkin-protocol

The root [AGENTS.md](../../AGENTS.md) applies here too. This file covers adding and changing packets.

## Packet flow

Keep Alive is a small example that touches every layer:

1. **Packet ids** come from `assets/packets.json` through codegen. The packet struct uses `#[java_packet(KEEP_ALIVE)]` with the generated constant, never a literal id.
2. **Serverbound packets** (`S...`, client to server) live in `src/java/server/<phase>/` and implement `ServerPacket::read()`. **Clientbound packets** (`C...`) live in `src/java/client/<phase>/` and implement `ClientPacket::write_packet_data()`. Both have to be exported from the phase's `mod.rs`.
3. **Dispatch** of play packets happens in `handle_play_packet()` in `crates/pumpkin/src/net/java/mod.rs`, which matches on `to_id(version)`. Earlier phases (handshake, status, login, configuration) are handled in `crates/pumpkin/src/net/java/pending.rs`, not in the play dispatcher.
4. **The handler** goes in its own file under `crates/pumpkin/src/net/java/play/`.

## Changing a packet

- Check the field order, types and length limits against vanilla.
- Bound every length and allocation you read from the wire. A client controls those bytes.
- Add a round-trip test next to the existing ones. Where you can, also compare against known bytes from vanilla, because a round trip passes when the reader and writer share the same mistake.
- Bedrock packets are a separate implementation. Changing a Java packet doesn't mean the Bedrock side needs the same change, and the reverse is also true.
