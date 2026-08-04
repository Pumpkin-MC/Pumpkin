/// Pack format versioning (major.minor, e.g. 107.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackFormat {
    pub major: u32,
    pub minor: u32,
}

impl PackFormat {
    /// The current server data pack format (26.2 -> 107.1).
    pub const CURRENT: Self = Self {
        major: 107,
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

impl PartialOrd for PackFormat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PackFormat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor).cmp(&(other.major, other.minor))
    }
}

/// A range of pack formats (inclusive on both ends).
#[derive(Debug, Clone)]
pub enum FormatRange {
    Single(PackFormat),
    Range { min: PackFormat, max: PackFormat },
}

impl FormatRange {
    /// The inclusive lower and upper bounds of this range.
    #[must_use]
    pub const fn bounds(&self) -> (PackFormat, PackFormat) {
        match self {
            Self::Single(f) => (*f, *f),
            Self::Range { min, max } => (*min, *max),
        }
    }

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
    /// The pack declares no usable format information.
    Unknown,
}

impl PackCompatibility {
    #[must_use]
    pub fn check(declared: &FormatRange, game: PackFormat) -> Self {
        let (min, max) = declared.bounds();
        if min.major == u32::MAX {
            Self::Unknown
        } else if max < game {
            Self::TooOld
        } else if game < min {
            Self::TooNew
        } else {
            Self::Compatible
        }
    }
}
