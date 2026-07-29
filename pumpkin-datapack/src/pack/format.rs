/// Pack format versioning (major.minor, e.g. 81.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackFormat {
    pub major: u32,
    pub minor: u32,
}

impl PackFormat {
    /// The current server data pack format (26.2 -> major=81, minor=1).
    pub const CURRENT: Self = Self {
        major: 81,
        minor: 1,
    };

    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Check if `other` is contained within this format range.
    /// A single format `(maj, min)` is contained if major matches and minor >= min.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.major == other.major && self.minor <= other.minor
    }
}

/// A range of pack formats (inclusive on both ends).
#[derive(Debug, Clone)]
pub enum FormatRange {
    Single(PackFormat),
    Range { min: PackFormat, max: PackFormat },
}

impl FormatRange {
    /// Check if the given pack format is within this range.
    #[must_use]
    pub const fn contains(&self, format: PackFormat) -> bool {
        match self {
            Self::Single(f) => f.contains(format),
            Self::Range { min, max } => {
                format.major == min.major
                    && format.major == max.major
                    && format.minor >= min.minor
                    && format.minor <= max.minor
            }
        }
    }

    /// Check if this range matches a specific pack format exactly.
    ///
    /// - `Single(f)` matches if `f == other`
    /// - `Range { min, max }` matches if `min <= other <= max`
    #[must_use]
    pub fn matches(&self, other: &PackFormat) -> bool {
        match self {
            Self::Single(f) => f == other,
            Self::Range { min, max } => {
                other.major >= min.major
                    && other.major <= max.major
                    && (other.major > min.major || other.minor >= min.minor)
                    && (other.major < max.major || other.minor <= max.minor)
            }
        }
    }
}

/// Compatibility level between a pack and the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackCompatibility {
    Compatible,
    TooOld,
    TooNew,
}

impl PackCompatibility {
    #[must_use]
    pub const fn check(pack_format: u32, supported: &FormatRange) -> Self {
        let pf = PackFormat::new(pack_format, 0);
        if supported.contains(pf) {
            Self::Compatible
        } else if pack_format < PackFormat::CURRENT.major {
            Self::TooOld
        } else {
            Self::TooNew
        }
    }
}
