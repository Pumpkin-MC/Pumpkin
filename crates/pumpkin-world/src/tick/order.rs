//! Sequential ordering for work recorded while entities tick in parallel.

use std::cell::Cell;

/// Orders one entity can take per tick, small enough for the counter to last years.
pub const ORDERS_PER_ENTITY: u64 = 1 << 20;

thread_local! {
    static NEXT_ORDER: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Makes scheduled ticks and spawns on this thread take orders from `first`; `None` resets.
pub fn set_entity_order(first: Option<u64>) {
    NEXT_ORDER.set(first);
}

/// Next order of the entity ticking on this thread, if any.
#[must_use]
pub fn next_entity_order() -> Option<u64> {
    NEXT_ORDER.with(|next| {
        let order = next.get()?;
        next.set(Some(order + 1));
        Some(order)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_follow_the_ticking_entity() {
        assert_eq!(next_entity_order(), None);
        set_entity_order(Some(2 * ORDERS_PER_ENTITY));
        assert_eq!(next_entity_order(), Some(2 * ORDERS_PER_ENTITY));
        assert_eq!(next_entity_order(), Some(2 * ORDERS_PER_ENTITY + 1));
        set_entity_order(None);
        assert_eq!(next_entity_order(), None);
    }
}
