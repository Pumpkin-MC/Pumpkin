use crate::{FrozenRegistry, TypedRegistry as _};
use pumpkin_util::identifier::Identifier;
use std::{any::Any, marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::RwLockReadGuard;

pub enum ErasedRegistryRef<'a> {
    Borrowed(&'a dyn Any),
    Locked(RwLockReadGuard<'a, dyn Any>),
}

impl Deref for ErasedRegistryRef<'_> {
    type Target = dyn Any;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(v) => *v,
            Self::Locked(v) => &**v,
        }
    }
}

pub struct DataKeyRef<'a, T> {
    // Must be dropped before `_registry`, because this may borrow from it.
    pub(crate) _guards: Vec<ErasedRegistryRef<'a>>,

    // Keeps a nested registry alive when the resolved value is not in the
    // borrowed root registry.
    pub(crate) _registry: Option<Arc<dyn crate::Registry>>,

    // Points into the last guard.
    pub(crate) value: *const T,

    pub(crate) marker: PhantomData<&'a T>,
}

impl<T> Deref for DataKeyRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        //
        // `value` points into one of `guards`.
        // That guard remains owned by this DataKeyRef for the entire
        // lifetime of the returned reference.
        unsafe { &*self.value }
    }
}

pub struct DynIterator<'a, T> {
    inner: Box<dyn Iterator<Item = T> + 'a>,
}

impl<'a, T> DynIterator<'a, T> {
    pub fn new<I>(iterator: I) -> Self
    where
        I: Iterator<Item = T> + 'a,
    {
        Self {
            inner: Box::new(iterator),
        }
    }
}

impl<T> Iterator for DynIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct LockedIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    // Must be dropped before `guard`.
    iterator: DynIterator<'a, (&'a Identifier, &'a T)>,

    // Keeps all references yielded by `iterator` valid.
    _guard: RwLockReadGuard<'a, FrozenRegistry<T>>,
}

impl<'a, T> LockedIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    pub(crate) fn new(guard: RwLockReadGuard<'a, FrozenRegistry<T>>) -> Self {
        let iterator = guard.iter();

        // SAFETY:
        //
        // `iterator` only contains references into the FrozenRegistry protected
        // by `guard`.
        //
        // `guard` is stored in this object for at least as long as `iterator`,
        // preventing the FrozenRegistry from being replaced.
        //
        // `iterator` is declared before `_guard`, so it is dropped first.
        //
        // Moving RwLockReadGuard does not move the protected FrozenRegistry.
        let iterator = unsafe {
            std::mem::transmute::<
                DynIterator<'_, (&Identifier, &T)>,
                DynIterator<'a, (&'a Identifier, &'a T)>,
            >(DynIterator::new(iterator))
        };

        Self {
            iterator,
            _guard: guard,
        }
    }
}

impl<'a, T> Iterator for LockedIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    type Item = (&'a Identifier, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}
