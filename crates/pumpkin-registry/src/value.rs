use pumpkin_util::identifier::Identifier;
use std::{any::Any, marker::PhantomData, ops::Deref};
use tokio::sync::RwLockReadGuard;

use crate::builder::RegistryBuilder;

pub enum RegistryRef<'a, T: ?Sized> {
    Borrowed(&'a T),
    Locked(RwLockReadGuard<'a, T>),
}

impl<T: ?Sized> Deref for RegistryRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(value) => value,
            Self::Locked(guard) => guard,
        }
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for RegistryRef<'_, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(formatter)
    }
}

pub struct ErasedRegistryRef<'a> {
    inner: Box<dyn Deref<Target = dyn Any + Send + Sync> + Send + Sync + 'a>,
}

impl<'a> ErasedRegistryRef<'a> {
    pub(crate) fn new<T>(value: RegistryRef<'a, T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            inner: Box::new(ErasedRegistryRefInner(value)),
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.deref().downcast_ref()
    }
}

impl Deref for ErasedRegistryRef<'_> {
    type Target = dyn Any + Send + Sync;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

struct ErasedRegistryRefInner<'a, T>(RegistryRef<'a, T>);

impl<T> Deref for ErasedRegistryRefInner<'_, T>
where
    T: Send + Sync + 'static,
{
    type Target = dyn Any + Send + Sync;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

pub struct DataKeyRef<'a, T> {
    // Keeps every registry/value lock guard alive.
    pub(crate) _guards: Vec<ErasedRegistryRef<'a>>,

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

pub struct LockedIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    // Must be dropped before `guard`.
    iterator: Box<dyn Iterator<Item = (&'a Identifier, &'a T)> + 'a>,
    // Keeps the underlying registry alive and prevents mutation.
    _guard: RwLockReadGuard<'a, RegistryBuilder<T>>,
}

impl<'a, T> LockedIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    pub fn new(guard: RwLockReadGuard<'a, RegistryBuilder<T>>) -> Self {
        let iterator = guard.iter();

        // SAFETY:
        //
        // `iterator` contains references into the RegistryBuilder protected by
        // `guard`.
        //
        // The guard is stored in the returned LockedIterator, so:
        // - the RegistryBuilder cannot be mutated while iteration is active;
        // - the referenced RegistryBuilder remains valid;
        // - `iterator` is dropped before `guard`, due to field declaration order.
        //
        // Moving the guard does not move the RegistryBuilder itself.
        let iterator = unsafe {
            std::mem::transmute::<
                Box<dyn Iterator<Item = (&Identifier, &T)> + '_>,
                Box<dyn Iterator<Item = (&'a Identifier, &'a T)> + 'a>,
            >(Box::new(iterator))
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
