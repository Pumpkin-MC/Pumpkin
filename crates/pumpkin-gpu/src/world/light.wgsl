// GPU-accelerated initial light scanning for chunk lighting.
//
// Entry points:
// - `scan_sky_light`: computes per-block sky light by sweeping downwards from the
//   heightmap per column, subtracting opacity at each step. Each column is independent.
// - `scan_block_light`: copies luminance values into the block-light buffer. A trivial
//   copy kernel; the CPU-side savings come from batching the block-state reads.
//
// Storage buffers hold one u8 per u32 element — bytemuck packs the u8 slices on the
// Rust side as `&[u8]` reinterpreted as `&[u32]`.

struct LightDims {
    num_columns: u32,  // 18 × 18 = 324
    height: u32,        // number of Y levels (max_y - bottom_y)
    bottom_y: i32,
    padding: u32,       // WGSL struct alignment to 16 bytes
}

// ── Sky light ──────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> sky_dims: LightDims;
@group(0) @binding(1) var<storage, read> opacity: array<u32>;
@group(0) @binding(2) var<storage, read> heightmap: array<i32>;
@group(0) @binding(3) var<storage, read_write> sky_light_out: array<u32>;

@compute @workgroup_size(64)
fn scan_sky_light(@builtin(global_invocation_id) gid: vec3<u32>) {
    let col = gid.x;
    if col >= sky_dims.num_columns { return; }

    let top_y = heightmap[col];
    let col_base = col * sky_dims.height;

    // Fill air above the surface with full sky light (15).
    // I32 calculation for top_y + 1 - bottom_y, then clamp to u32 range.
    let air_start_i32 = top_y + 1 - sky_dims.bottom_y;
    var y: u32 = select(0u, u32(air_start_i32), air_start_i32 > 0);
    while y < sky_dims.height {
        sky_light_out[col_base + y] = 15u;
        y += 1u;
    }

    // Scan downwards from the surface, reducing light by opacity.
    // In the initial scan, light only decreases when passing through a non-air
    // block — air (opacity 0) does not attenuate. This differs from the later
    // BFS propagation pass, where every step costs at least 1.
    var light: u32 = 15u;
    let scan_start_i32 = top_y - sky_dims.bottom_y;
    y = u32(clamp(scan_start_i32, 0, i32(sky_dims.height) - 1));
    loop {
        let idx = col_base + y;
        let op = opacity[idx];
        if op > 0u {
            light = select(0u, light - op, light > op);
        }
        sky_light_out[idx] = min(light, 15u);
        if light == 0u || y == 0u {
            break;
        }
        y -= 1u;
    }
}

// ── Block light ────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> block_dims: LightDims;
@group(0) @binding(1) var<storage, read> luminance: array<u32>;
@group(0) @binding(2) var<storage, read_write> block_light_out: array<u32>;

@compute @workgroup_size(64)
fn scan_block_light(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let total = block_dims.num_columns * block_dims.height;
    if idx >= total { return; }

    block_light_out[idx] = luminance[idx];
}
