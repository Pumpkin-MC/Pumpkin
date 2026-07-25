# INSTRUCTIONS.md — Rust & Protocol Implementation Standards

## 1. Rust Safety & Idioms

- **Unsafe Code:** `unsafe` Rust is prohibited unless explicitly approved and accompanied by a detailed `// SAFETY:` block explaining memory safety invariants.
- **Zero-Copy Parsing:** Leverage `bytes::Bytes` and zero-copy slicing where possible to minimize heap allocations during packet reading and broadcasting.
- **Derives:** Structs representing network payloads should derive `Debug`, `Clone`, and relevant serialization traits.

---

## 2. Packet & Protocol Guidelines

### Packet Lifecycles
Every network packet follows three distinct phases:
1. **Decode:** Parse raw bytes into a strongly-typed Rust struct. Validate boundary constraints (e.g., string lengths, array sizes).
2. **Handle:** Process packet intent inside the appropriate connection state (`Handshaking`, `Status`, `Login`, or `Play`).
3. **Encode:** Serialize state changes or outgoing packets into `BytesMut` buffers for writing.

### Bound Checking & Defense
- Validate all incoming array/string sizes against protocol limits before allocating memory to protect against buffer exhaustion attacks.
- Gracefully close connections on invalid or corrupted protocol packets rather than crashing the worker thread.

---

## 3. Concurrency & State Management

- **Player State:** Store player session state in dedicated connection actor tasks or thread-safe atomic references (`Arc<RwLock<...>>`).
- **World Ticking:** Keep the main tick loop predictable. Decouple network read/write worker threads from world tick processing.

---

## 4. Code Formatting & Cleanliness

- Maintain clean, minimal code diffs. Do not reformulate unrelated code blocks or refactor modules outside the requested scope.
- Write clear inline comments explaining non-obvious mathematical or protocol-specific logic.
- Avoid introducing external dependencies without checking if existing workspace dependencies satisfy the requirement.
