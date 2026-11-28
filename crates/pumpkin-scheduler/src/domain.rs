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

/// Selects the execution domain requested for a task.
///
/// [`GlobalScheduler`](crate::GlobalScheduler) accepts only [`Self::Global`].
/// The remaining variants reserve names for future ownership and routing work;
/// they do not provide world, region, or entity isolation today.
///
/// Ordering applies to individual task polls. A task that suspends can resume
/// after another task has changed shared state, so callers must release locks
/// before waiting on dependent work and revalidate state when they resume.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExecutionDomain {
    /// Server-wide work submitted to a Global scheduler.
    ///
    /// Only work submitted to that scheduler participates in its ordering.
    /// Access to authoritative game state remains the caller's responsibility.
    Global,

    /// Reserved for world-wide work, such as weather, time, or world lifecycle.
    World(WorldDomainId),

    /// Reserved for work owned by one independently scheduled world region.
    Region(RegionDomainId),

    /// Reserved for work routed to a player or entity's current owner.
    ///
    /// Routing and ownership transfer during movement are not implemented.
    Entity(EntityDomainId),

    /// Reserved for work owned by an external integration or completion source.
    ///
    /// Async I/O still runs on the application's I/O executor.
    External(ExternalDomainId),
}
