use std::{
    cmp::Reverse,
    collections::HashMap,
    fmt::{self, Write as _},
    io,
    time::Duration,
};

use prost::Message;
use serde::Serialize;

use crate::{
    allocator::sampling_weight,
    collector::{MAX_FRAMES, Sample},
};

const MAX_LOCATIONS: usize = 65_536;
const MAX_FUNCTIONS: usize = 65_536;
const MAX_LINES: usize = 131_072;
const MAX_STRING_BYTES: usize = 4 * 1024 * 1024;
const MAX_SYMBOL_BYTES: usize = 512;

#[derive(Message)]
pub struct Profile {
    #[prost(message, repeated, tag = "1")]
    sample_type: Vec<ValueType>,
    #[prost(message, repeated, tag = "2")]
    sample: Vec<ProfileSample>,
    #[prost(message, repeated, tag = "4")]
    location: Vec<Location>,
    #[prost(message, repeated, tag = "5")]
    function: Vec<Function>,
    #[prost(string, repeated, tag = "6")]
    string_table: Vec<String>,
    #[prost(int64, tag = "7")]
    drop_frames: i64,
    #[prost(int64, tag = "9")]
    time_nanos: i64,
    #[prost(int64, tag = "10")]
    duration_nanos: i64,
    #[prost(int64, repeated, tag = "13")]
    comment: Vec<i64>,
    #[prost(int64, tag = "14")]
    default_sample_type: i64,
}

#[derive(Message)]
struct ValueType {
    #[prost(int64, tag = "1")]
    r#type: i64,
    #[prost(int64, tag = "2")]
    unit: i64,
}

#[derive(Message)]
struct ProfileSample {
    #[prost(uint64, repeated, tag = "1")]
    location_id: Vec<u64>,
    #[prost(int64, repeated, tag = "2")]
    value: Vec<i64>,
}

#[derive(Message)]
struct Location {
    #[prost(uint64, tag = "1")]
    id: u64,
    #[prost(uint64, tag = "3")]
    address: u64,
    #[prost(message, repeated, tag = "4")]
    line: Vec<Line>,
}

#[derive(Message)]
struct Line {
    #[prost(uint64, tag = "1")]
    function_id: u64,
    #[prost(int64, tag = "2")]
    line: i64,
}

#[derive(Message)]
struct Function {
    #[prost(uint64, tag = "1")]
    id: u64,
    #[prost(int64, tag = "2")]
    name: i64,
    #[prost(int64, tag = "3")]
    system_name: i64,
    #[prost(int64, tag = "4")]
    filename: i64,
}

#[derive(Default, Serialize)]
pub struct HeapStats {
    sampled_live_allocations: usize,
    sampled_live_bytes: usize,
    estimated_live_allocations: usize,
    pub estimated_live_bytes: usize,
    stacks_without_symbols: usize,
    unresolved_frames: usize,
    truncated_stacks: usize,
    pub dropped_samples: u64,
    estimates_complete: bool,
    pub top_allocations: Vec<AllocationSite>,
}

#[derive(Serialize)]
pub struct AllocationSite {
    pub name: String,
    pub bytes: usize,
}

struct Builder {
    profile: Profile,
    strings: HashMap<String, i64>,
    string_bytes: usize,
    locations: HashMap<usize, (u64, bool)>,
    functions: HashMap<(i64, i64), u64>,
    lines: usize,
}

impl Builder {
    fn new(timestamp: i64, elapsed: Duration, dropped: u64) -> io::Result<Self> {
        let mut builder = Self {
            profile: Profile {
                time_nanos: timestamp,
                duration_nanos: elapsed.as_nanos().min(i64::MAX as u128) as i64,
                ..Profile::default()
            },
            strings: HashMap::new(),
            string_bytes: 0,
            locations: HashMap::new(),
            functions: HashMap::new(),
            lines: 0,
        };
        builder.intern(String::new())?;
        let objects = builder.intern("inuse_objects".to_owned())?;
        let count = builder.intern("count".to_owned())?;
        let bytes = builder.intern("inuse_space".to_owned())?;
        let unit = builder.intern("bytes".to_owned())?;
        builder.profile.sample_type = vec![
            ValueType {
                r#type: objects,
                unit: count,
            },
            ValueType {
                r#type: bytes,
                unit,
            },
        ];
        builder.profile.default_sample_type = bytes;
        builder.profile.drop_frames = builder.intern(
            r"pumpkin_profiler::allocator::.*|<pumpkin_profiler::allocator::TrackingAllocator as .*|__rustc::__rust_(alloc|alloc_zeroed|realloc)".to_owned(),
        )?;
        if dropped != 0 {
            let comment =
                builder.intern(format!("Incomplete estimates: dropped {dropped} samples"))?;
            builder.profile.comment.push(comment);
        }
        Ok(builder)
    }

    fn intern(&mut self, value: String) -> io::Result<i64> {
        if let Some(&index) = self.strings.get(&value) {
            return Ok(index);
        }
        if value.len() > MAX_STRING_BYTES - self.string_bytes {
            return Err(io::Error::other(
                "heap profile exceeds symbol string budget",
            ));
        }
        self.string_bytes += value.len();
        let index = self.profile.string_table.len() as i64;
        self.strings.insert(value.clone(), index);
        self.profile.string_table.push(value);
        Ok(index)
    }

    fn line(&mut self, name: String, filename: String, line: i64) -> io::Result<Line> {
        if self.lines == MAX_LINES {
            return Err(io::Error::other("heap profile exceeds inline frame budget"));
        }
        let name = self.intern(name)?;
        let filename = self.intern(filename)?;
        let key = (name, filename);
        let function_id = if let Some(&id) = self.functions.get(&key) {
            id
        } else {
            if self.functions.len() == MAX_FUNCTIONS {
                return Err(io::Error::other("heap profile exceeds function budget"));
            }
            let id = self.profile.function.len() as u64 + 1;
            self.profile.function.push(Function {
                id,
                name,
                system_name: name,
                filename,
            });
            self.functions.insert(key, id);
            id
        };
        self.lines += 1;
        Ok(Line { function_id, line })
    }

    fn location(&mut self, address: usize) -> io::Result<(u64, bool)> {
        if let Some(&location) = self.locations.get(&address) {
            return Ok(location);
        }
        if self.locations.len() == MAX_LOCATIONS {
            return Err(io::Error::other(
                "heap profile exceeds distinct stack address budget",
            ));
        }
        let mut symbols = Vec::new();
        // Backtrace PCs point past the call instruction.
        backtrace::resolve(
            address.saturating_sub(1) as *mut std::ffi::c_void,
            |symbol| {
                if symbols.len() == MAX_FRAMES {
                    return;
                }
                if let Some(name) = symbol.name() {
                    symbols.push((
                        bounded_text(name),
                        symbol
                            .filename()
                            .map_or_else(String::new, |path| bounded_text(path.display())),
                        i64::from(symbol.lineno().unwrap_or(0)),
                    ));
                }
            },
        );
        let resolved = !symbols.is_empty();
        if !resolved {
            symbols.push((format!("0x{address:x}"), String::new(), 0));
        }
        let lines = symbols
            .into_iter()
            .map(|(name, filename, line)| self.line(name, filename, line))
            .collect::<io::Result<Vec<_>>>()?;
        let id = self.profile.location.len() as u64 + 1;
        self.profile.location.push(Location {
            id,
            address: address as u64,
            line: lines,
        });
        self.locations.insert(address, (id, resolved));
        Ok((id, resolved))
    }

    fn allocation_site(&self, ids: &[u64]) -> Option<u64> {
        let functions = || {
            ids.iter()
                .flat_map(|&id| self.profile.location[id as usize - 1].line.iter())
                .map(|line| line.function_id)
        };
        functions()
            .find(|&id| {
                let name = &self.profile.string_table
                    [self.profile.function[id as usize - 1].name as usize];
                name.contains("pumpkin_") && !name.contains("pumpkin_profiler::")
            })
            .or_else(|| {
                functions().find(|&id| {
                    let name = &self.profile.string_table
                        [self.profile.function[id as usize - 1].name as usize];
                    ![
                        "std::",
                        "core::",
                        "alloc::",
                        "__rustc::",
                        "pumpkin_profiler::",
                        "<std::",
                        "<core::",
                        "<alloc::",
                        "<pumpkin_profiler::",
                        "0x",
                    ]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                })
            })
            .or_else(|| {
                functions().find(|&id| {
                    let name = &self.profile.string_table
                        [self.profile.function[id as usize - 1].name as usize];
                    !name.contains("pumpkin_profiler::") && !name.starts_with("__rustc::")
                })
            })
    }
}

pub fn build(
    samples: &[Sample],
    dropped: u64,
    timestamp: i64,
    elapsed: Duration,
) -> io::Result<(Profile, HeapStats)> {
    let mut builder = Builder::new(timestamp, elapsed, dropped)?;
    let mut sites = HashMap::<u64, usize>::new();
    let mut stats = HeapStats {
        sampled_live_allocations: samples.len(),
        dropped_samples: dropped,
        estimates_complete: dropped == 0,
        ..HeapStats::default()
    };
    for sample in samples {
        let frames = if sample.frame_count == 0 {
            &[0][..]
        } else {
            &sample.frames[..sample.frame_count]
        };
        let mut ids = Vec::with_capacity(frames.len());
        let mut has_symbol = false;
        for &address in frames {
            let (id, resolved) = builder.location(address)?;
            ids.push(id);
            has_symbol |= resolved;
            stats.unresolved_frames += usize::from(!resolved);
        }
        stats.stacks_without_symbols += usize::from(!has_symbol);
        stats.truncated_stacks += usize::from(sample.frame_count == MAX_FRAMES);
        let weight = sampling_weight(sample.size);
        let bytes = sample
            .size
            .checked_mul(weight)
            .ok_or_else(|| io::Error::other("estimated sample size overflow"))?;
        stats.sampled_live_bytes = stats
            .sampled_live_bytes
            .checked_add(sample.size)
            .ok_or_else(|| io::Error::other("sampled byte total overflow"))?;
        stats.estimated_live_bytes = stats
            .estimated_live_bytes
            .checked_add(bytes)
            .ok_or_else(|| io::Error::other("estimated byte total overflow"))?;
        stats.estimated_live_allocations += weight;
        if let Some(site) = builder.allocation_site(&ids) {
            *sites.entry(site).or_default() += bytes;
        }
        builder.profile.sample.push(ProfileSample {
            location_id: ids,
            value: vec![
                weight as i64,
                i64::try_from(bytes).map_err(io::Error::other)?,
            ],
        });
    }
    let mut sites: Vec<_> = sites.into_iter().collect();
    sites.sort_unstable_by_key(|&(_, bytes)| Reverse(bytes));
    stats.top_allocations = sites
        .into_iter()
        .take(5)
        .map(|(id, bytes)| {
            let function = &builder.profile.function[id as usize - 1];
            AllocationSite {
                name: builder.profile.string_table[function.name as usize].clone(),
                bytes,
            }
        })
        .collect();
    Ok((builder.profile, stats))
}

struct LimitedText(String);

impl fmt::Write for LimitedText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let mut end = value.len().min(MAX_SYMBOL_BYTES - self.0.len());
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.0.push_str(&value[..end]);
        if end == value.len() {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

fn bounded_text(value: impl fmt::Display) -> String {
    let mut text = LimitedText(String::with_capacity(MAX_SYMBOL_BYTES));
    let _ = write!(text, "{value:#}");
    text.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_power_of_two_allocations_have_integer_object_weights() {
        let samples = [393_216, 98_304].map(|size| Sample {
            address: 1,
            size,
            frames: [0; MAX_FRAMES],
            frame_count: 0,
        });
        let (profile, _) = build(&samples, 0, 42, Duration::from_secs(1)).unwrap();
        assert_eq!(profile.sample[0].value, [1, 393_216]);
        assert_eq!(profile.sample[1].value, [4, 393_216]);
    }
}
