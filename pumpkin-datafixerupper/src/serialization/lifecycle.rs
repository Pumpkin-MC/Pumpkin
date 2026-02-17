/// A marker to convey the lifecycle of some object.
/// It could be stable, experimental or deprecated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    /// Represents something stable.
    Stable,
    /// Represents something experimental.
    Experimental,
    /// Represents something deprecated. The `u32` with this lifecycle represents the *date* from which it was marked to be deprecated.
    ///
    /// If the number is smaller, it was deprecated earlier, whereas if it is bigger, it was deprecated later.
    Deprecated(u32),
}

impl Lifecycle {
    /// Adds a lifecycle to another, returning a resultant lifecycle which matches the more restrictive of the two.
    ///
    /// This function follows this order:
    /// - If at least one of the lifecycles is *experimental*, [`Lifecycle::Experimental`] is returned.
    /// - If both lifecycles are *deprecated*, this returns the earlier-deprecated one (where the *date* is lower).
    /// - If exactly one lifecycle is *deprecated*, that lifecycle is returned.
    /// - If nothing above matches, [`Lifecycle::Stable`] is returned.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Experimental, _) | (_, Self::Experimental) => Self::Experimental,

            (d1 @ Self::Deprecated(s1), d2 @ Self::Deprecated(s2)) => {
                if s1 < s2 {
                    d1
                } else {
                    d2
                }
            }

            (d @ Self::Deprecated(_), _) | (_, d @ Self::Deprecated(_)) => d,

            _ => Self::Stable,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::serialization::lifecycle::Lifecycle;

    #[test]
    fn add_lifecycles() {
        assert_eq!(Lifecycle::Stable.add(Lifecycle::Stable), Lifecycle::Stable);

        assert_eq!(
            Lifecycle::Experimental.add(Lifecycle::Deprecated(10)),
            Lifecycle::Experimental
        );

        assert_eq!(
            Lifecycle::Deprecated(10).add(Lifecycle::Deprecated(15)),
            Lifecycle::Deprecated(10)
        );
    }
}
