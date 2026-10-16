#![cfg(all(
    any(target_os = "macos", all(target_os = "linux", target_env = "gnu")),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{
    alloc::Layout, fs, io::Read, mem::ManuallyDrop, os::unix::fs::PermissionsExt, path::Path,
};

use flate2::read::GzDecoder;
use prost::Message;
use pumpkin_profiler::{Operation, PROFILER, TrackingAllocator, register_allocator};
use serde_json::json;

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

#[derive(Message)]
struct HeapProfile {
    #[prost(message, repeated, tag = "2")]
    samples: Vec<HeapSample>,
}

#[derive(Message)]
struct HeapSample {
    #[prost(int64, repeated, tag = "2")]
    values: Vec<i64>,
}

fn live_allocations(path: &Path, size: usize) -> usize {
    let file = fs::File::open(path.join("heap.pb.gz")).unwrap();
    let mut bytes = Vec::new();
    GzDecoder::new(file).read_to_end(&mut bytes).unwrap();
    HeapProfile::decode(bytes.as_slice())
        .unwrap()
        .samples
        .iter()
        .filter(|sample| sample.values == [1, size as i64])
        .count()
}

#[inline(never)]
fn retained_allocation(size: usize) -> Vec<u8> {
    std::hint::black_box(vec![7; size])
}

#[test]
fn live_profiles_follow_realloc_free_session_restart_and_failed_output() {
    register_allocator();
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let run =
        |operation| PROFILER.execute(operation, root.path(), "lifetime-regression", &json!({}));
    let old = retained_allocation(4 * 1024 * 1024);
    run(Operation::Start).unwrap();
    let directory = fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        live_allocations(&directory.join("snapshot-000"), old.capacity()),
        0
    );

    let allocation = ManuallyDrop::new(retained_allocation(2 * 1024 * 1024));
    let size = allocation.capacity();
    let layout = Layout::from_size_align(size, 1).unwrap();
    let pointer = allocation.as_ptr().cast_mut();
    run(Operation::Snapshot).unwrap();
    assert_eq!(
        fs::metadata(directory.join("snapshot-001"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(live_allocations(&directory.join("snapshot-001"), size), 1);

    // SAFETY: This is the live global allocation with its actual layout. isize::MAX
    // is a valid layout size at alignment 1, but cannot fit in the process address space.
    let failed = unsafe { std::alloc::realloc(pointer, layout, isize::MAX as usize) };
    assert!(failed.is_null());
    run(Operation::Snapshot).unwrap();
    assert_eq!(live_allocations(&directory.join("snapshot-002"), size), 1);

    let new_size = 3 * 1024 * 1024;
    // SAFETY: Failed realloc left the original pointer live; this layout still matches.
    let grown = unsafe { std::alloc::realloc(pointer, layout, new_size) };
    assert!(!grown.is_null());
    run(Operation::Snapshot).unwrap();
    assert_eq!(live_allocations(&directory.join("snapshot-003"), size), 0);
    assert_eq!(
        live_allocations(&directory.join("snapshot-003"), new_size),
        1
    );

    // SAFETY: Successful realloc returned this pointer with the updated layout.
    unsafe { std::alloc::dealloc(grown, Layout::from_size_align(new_size, 1).unwrap()) };
    run(Operation::Stop).unwrap();
    assert_eq!(
        live_allocations(&directory.join("snapshot-004"), new_size),
        0
    );

    let survivor = retained_allocation(size);
    run(Operation::Start).unwrap();
    let restarted = fs::read_dir(root.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path != &directory)
        .unwrap();
    assert_eq!(
        live_allocations(&restarted.join("snapshot-000"), survivor.capacity()),
        0
    );
    let current = retained_allocation(new_size);
    drop(survivor);

    let moved = root.path().join("saved-session");
    fs::rename(&restarted, &moved).unwrap();
    fs::write(&restarted, b"blocked output path").unwrap();
    assert!(run(Operation::Stop).is_err());
    assert!(run(Operation::Status).unwrap().contains("active"));
    fs::remove_file(&restarted).unwrap();
    fs::rename(&moved, &restarted).unwrap();
    run(Operation::Stop).unwrap();
    assert_eq!(
        live_allocations(&restarted.join("snapshot-001"), current.capacity()),
        1
    );
    drop(current);
    drop(old);
}
