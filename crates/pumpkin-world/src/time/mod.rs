use pumpkin_registry::RegistryResolvable;

pub struct WorldClock;

pub struct Timeline {
    clock: RegistryResolvable<WorldClock>,
    period: Option<u32>,
}
