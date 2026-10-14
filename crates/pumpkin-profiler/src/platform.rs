use serde_json::Value;
use std::{io, path::Path};

pub const MAX_NATIVE_BYTES: usize = 1024 * 1024;

pub struct Process {
    pub rss_bytes: u64,
    pub virtual_bytes: u64,
    pub native: Value,
    pub smaps_rollup_error: Option<String>,
}

pub fn capture(path: &Path) -> io::Result<Process> {
    let memory = memory_stats::memory_stats()
        .ok_or_else(|| io::Error::other("cannot read process memory statistics"))?;
    let (native, smaps_rollup_error) = native_capture(path)?;
    Ok(Process {
        rss_bytes: memory.physical_mem as u64,
        virtual_bytes: memory.virtual_mem as u64,
        native,
        smaps_rollup_error,
    })
}

#[cfg(target_os = "linux")]
fn native_capture(path: &Path) -> io::Result<(Value, Option<String>)> {
    use crate::capture::write_file;
    use std::{fs::File, io::Read};
    fn read_proc(path: &str) -> io::Result<Vec<u8>> {
        const MAX_PROC_BYTES: usize = 4 * 1024 * 1024;
        let mut bytes = Vec::new();
        File::open(path)?
            .take(MAX_PROC_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_PROC_BYTES {
            return Err(io::Error::other("process memory report exceeds size limit"));
        }
        Ok(bytes)
    }
    for name in ["status", "maps"] {
        write_file(
            &path.join(format!("{name}.txt")),
            &read_proc(&format!("/proc/self/{name}"))?,
        )?;
    }
    write_file(&path.join("glibc.xml"), &native_xml()?)?;
    let error = match read_proc("/proc/self/smaps_rollup") {
        Ok(bytes) => {
            write_file(&path.join("smaps_rollup.txt"), &bytes)?;
            None
        }
        Err(error) => Some(error.to_string()),
    };
    Ok((
        serde_json::json!({"os": "linux", "allocator": "glibc"}),
        error,
    ))
}

#[cfg(target_os = "linux")]
fn native_xml() -> io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; MAX_NATIVE_BYTES + 1];
    // SAFETY: The fixed buffer stays allocated and exclusively borrowed until fclose.
    let stream = unsafe { libc::fmemopen(buffer.as_mut_ptr().cast(), buffer.len(), c"w".as_ptr()) };
    if stream.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: stream is an open writable FILE. All paths close it before releasing buffer.
    let result = unsafe {
        let reported = libc::malloc_info(0, stream);
        let flushed = libc::fflush(stream);
        let position = libc::ftell(stream);
        let error = (reported != 0 || flushed != 0 || libc::ferror(stream) != 0 || position < 0)
            .then(io::Error::last_os_error);
        let closed = libc::fclose(stream);
        error.map_or_else(
            || {
                if closed != 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(position)
                }
            },
            Err,
        )
    }?;
    let size = usize::try_from(result).map_err(io::Error::other)?;
    if size > MAX_NATIVE_BYTES {
        return Err(io::Error::other("glibc arena report exceeds size limit"));
    }
    buffer.truncate(size);
    Ok(buffer)
}

#[cfg(target_os = "macos")]
fn native_capture(path: &Path) -> io::Result<(Value, Option<String>)> {
    use serde_json::json;
    use std::mem::MaybeUninit;
    let mut usage = MaybeUninit::<libc::rusage_info_v4>::uninit();
    // SAFETY: The output buffer matches RUSAGE_INFO_V4 and is read only on success.
    let result = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            usage.as_mut_ptr().cast(),
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: proc_pid_rusage successfully initialized the entire v4 structure.
    let usage = unsafe { usage.assume_init() };
    let mut stats = MaybeUninit::<libc::malloc_statistics_t>::uninit();
    // SAFETY: A null zone sums all malloc zones into the correctly sized output buffer.
    unsafe { libc::malloc_zone_statistics(std::ptr::null_mut(), stats.as_mut_ptr()) };
    // SAFETY: malloc_zone_statistics initializes all fields of stats.
    let stats = unsafe { stats.assume_init() };
    let native = json!({
        "os": "macos", "allocator": "darwin_malloc", "rusage_flavor": libc::RUSAGE_INFO_V4,
        "physical_footprint_bytes": usage.ri_phys_footprint,
        "peak_physical_footprint_bytes": usage.ri_lifetime_max_phys_footprint,
        "wired_bytes": usage.ri_wired_size, "pageins": usage.ri_pageins,
        "user_cpu_nanos": usage.ri_user_time, "system_cpu_nanos": usage.ri_system_time,
        "disk_read_bytes": usage.ri_diskio_bytesread, "disk_written_bytes": usage.ri_diskio_byteswritten,
        "logical_write_bytes": usage.ri_logical_writes,
        "interrupt_wakeups": usage.ri_interrupt_wkups, "idle_wakeups": usage.ri_pkg_idle_wkups,
        "instructions": usage.ri_instructions, "cycles": usage.ri_cycles,
        "malloc": { "blocks_in_use": stats.blocks_in_use, "bytes_in_use": stats.size_in_use,
            "max_bytes_in_use": stats.max_size_in_use, "bytes_allocated": stats.size_allocated }
    });
    crate::capture::write_json(&path.join("darwin.json"), &native)?;
    Ok((native, None))
}
