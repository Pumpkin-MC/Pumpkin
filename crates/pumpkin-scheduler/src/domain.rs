macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

domain_id!(WorldDomainId);
domain_id!(RegionDomainId);
domain_id!(EntityDomainId);
domain_id!(ExternalDomainId);

/// Identifies the scheduler owner responsible for executing a unit of work
///
/// A domain serializes task polls and tasks may interleave whenever they suspend
/// Completion order depends on when tasks resume and finish
/// Callers must revalidate state after suspension
///
/// An operation targeting another domain must be submitted to that domain
/// using owned inputs
/// The calling task awaits until the target domain returns an owned result
/// Domain-owned references and lock guards must be released before suspension
/// Only Global admission is currently implemented
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExecutionDomain {
    /// Owns server-wide work that cannot yet be assigned to a narrower owner
    ///
    /// All plugin-capable work initially runs here while the scheduler is
    /// introduced
    /// It also provides a safe fallback for operations spanning several worlds,
    /// regions, or entities
    ///
    /// Global work is ordered against other Global work, so excessive use of
    /// this domain limits parallelism
    Global,

    /// Owns work affecting one complete world or dimension
    ///
    /// This includes world lifecycle, weather, time, and coordination across
    /// several regions in the same world
    World(WorldDomainId),

    /// Owns state associated with one independently schedulable world region
    ///
    /// Chunk ticks, block entities, scheduled block work, spawning, and
    /// region-scoped events should execute with region ownership
    /// This means that work in the same region is ordered while work over
    /// different, unrelated regions are parallel
    Region(RegionDomainId),

    /// Owns work associated with one movable player or entity
    ///
    /// Entity ticks, movement, combat, AI, vehicles, and lifecycle operations
    /// execute through the entity's current owner
    /// Teleporting, changing dimensions, mounting, spawning, or despawning must transfer or
    /// invalidate the previous ownership generation before work continues
    Entity(EntityDomainId),

    /// Identifies scheduler-visible work owned outside normal game-state
    /// domains
    ///
    /// This can represent an external integration or completion source that
    /// participates in scheduler parking and wakeup
    /// Tokio continues to handle socket, filesystem, and other asynchronous I/O
    External(ExternalDomainId),
}
