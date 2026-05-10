# PNBT Specification

PNBT is a high-speed, positional binary format designed for maximum storage efficiency and extreme serialization/deserialization throughput. Unlike standard NBT, PNBT is **positional** and does not store field names or tag IDs in the stream, making it ideal for internal storage (player data, level metadata) where the schema is stable.

### Key Features
- **Zero Overhead:** No string keys or tag IDs stored in the binary stream.
- **ZigZag Varints:** Uses LEB128 encoding for all integers and lengths, with ZigZag for signed types.
- **Zero-Copy Deserialization:** Directly borrows strings and bytes from the input buffer.
- **Extreme Performance:** Specifically optimized for high-frequency internal data storage.

### Binary Layout
PNBT follows a strict positional layout defined by the Rust struct being serialized:
- **Primitives:** LEB128/ZigZag varints for integers. Fixed size for floats.
- **Strings/Bytes:** Varint length followed by raw payload.
- **Sequences/Maps:** Varint length followed by positional elements.

## Performance (vs Vanilla NBT)
- **Size Efficiency:** **~43% to 47% smaller** footprint.
- **Serialization Speed:** **~8x faster** than vanilla NBT (554 ns vs 4.51 µs).
- **Deserialization Speed:** **~3x faster** than vanilla NBT (3.41 µs vs 9.92 µs).

Note: Because PNBT is positional, any changes to the struct layout (adding/removing/reordering fields) will make existing serialized data incompatible unless handled manually (e.g. via `Option` or versioned structs).
