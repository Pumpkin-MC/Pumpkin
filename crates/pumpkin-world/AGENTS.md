# pumpkin-world

The root [AGENTS.md](../../AGENTS.md) applies here too. This file covers chunk loading, world generation and how to check changes against vanilla.

## Chunk loading

Chunk loading uses a ticket and level system modelled on vanilla, in `src/chunk_system/`. The level numbers don't match vanilla's, so look up Pumpkin's constants in `chunk_loading.rs` instead of reusing numbers from the Java code. A ticket at the wrong level can load or generate far more chunks than you meant to, so check the effect of any new ticket, not only that the chunk arrives.

## World generation

- Compare output with vanilla, not only with the previous Pumpkin build. `src/generation/proto_chunk_test.rs` checks generation against vanilla chunk dumps in `assets/tests/`. Keep those tests passing, and use the same approach for new stages.
- Worldgen values come from the vanilla datapack in `assets/` through codegen: noise settings, density functions, features, structures. Don't copy numbers from decompiled code when the datapack defines them.
- Read heights and limits from the generation context or the dimension, never from a guess like 256 or 384. Datapacks change them.

## Performance

Chunk generation is the hottest path in the server. For changes here, run the Criterion benches in `benches/` on `master` and on your branch, and put both numbers in the PR. If generation gets slower, compare the data and the codegen output between the two commits before you blame threading. The cause is often not where it looks.
