
# Pumpkin save format

Pumpkin save format - its an optimized version of minecraft world saving format.
Its a much simplier and optimized, because its uses bitvec.

# So how it works?

First of all, we dont use region files.
We save one chunk in one file and rely on user file system.
We dont use location tables and timestamp tables for optimization.

First byte in file its a compression alghoritm, next goes compressed data or normal chunk.
Chunk separated by 24 subchunks. First in all subchunk goes palette.
Bits per block in palette picking same as anvil format.
After goes 2 separation bytes, which are `u16::MAX`.
After goes block array, which is 4096 block, but its can be changed in future for using rle compression or other alghoritms.

All file layout of chunk looks like this:
`compression_byte``packed_data`

packed_data:
array of `subchunk` with len of 24.

subchunk:
`palette``2_separation_bytes``block_array`