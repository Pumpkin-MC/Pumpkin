use std::cell::RefCell;

use zrip::{CompressContext, CompressError};

/// Level 1 compresses chunk data like C zstd `-1`. Negative levels are no
/// faster on chunk data and compress worse.
const LEVEL: i32 = 1;

thread_local! {
    // Saves run on many threads; each keeps its hash tables between chunks.
    static COMPRESSOR: RefCell<Option<CompressContext>> = const { RefCell::new(None) };
}

/// Appends one zstd frame holding `input` to `out`.
pub fn compress_append(input: &[u8], out: &mut Vec<u8>) -> Result<(), CompressError> {
    COMPRESSOR.with_borrow_mut(|slot| {
        let ctx = if let Some(ctx) = slot {
            ctx
        } else {
            let mut ctx = CompressContext::new(LEVEL)?;
            // Both formats hash their data already, and ruzstd wrote no
            // frame checksum either.
            ctx.set_content_checksum(false);
            slot.insert(ctx)
        };
        let frame = ctx.compress(input)?;
        out.reserve_exact(frame.len());
        out.extend_from_slice(&frame);
        Ok(())
    })
}
