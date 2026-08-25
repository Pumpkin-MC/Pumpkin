use crate::{StaticRegistry, TypedRegistry as _};
use std::{any::Any, marker::PhantomData, ops::Deref, sync::Arc};

pub enum ErasedRegistryRef<'a> {
    Borrowed(&'a dyn Any),
    Snapshot {
        value: *const dyn Any,
        _snapshot: Arc<dyn Any + Send + Sync>,
    },
}

impl ErasedRegistryRef<'_> {
    pub(crate) fn from_snapshot<T>(snapshot: Arc<StaticRegistry<T>>, id: usize) -> Option<Self>
    where
        T: Send + Sync + 'static,
    {
        let value = snapshot.by_id(id)? as &dyn Any;
        let value = std::ptr::from_ref(value);
        let snapshot: Arc<dyn Any + Send + Sync> = snapshot;

        Some(Self::Snapshot {
            value,
            _snapshot: snapshot,
        })
    }
}

impl Deref for ErasedRegistryRef<'_> {
    type Target = dyn Any;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(value) => *value,
            Self::Snapshot { value, .. } => {
                // SAFETY:
                //
                // `value` points into `_snapshot`, which owns the StaticRegistry
                // containing the value for the entire lifetime of this reference.
                unsafe { &**value }
            }
        }
    }
}

pub struct SnapshotRef<T>
where
    T: Send + Sync + 'static,
{
    value: *const T,
    _snapshot: Arc<StaticRegistry<T>>,
}

impl<T> SnapshotRef<T>
where
    T: Send + Sync + 'static,
{
    pub(crate) fn new(snapshot: Arc<StaticRegistry<T>>, id: usize) -> Option<Self> {
        let value = std::ptr::from_ref(snapshot.by_id(id)?);
        Some(Self {
            value,
            _snapshot: snapshot,
        })
    }
}

impl<T> Deref for SnapshotRef<T>
where
    T: Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        //
        // `value` points into `_snapshot`, and `_snapshot` is retained by this
        // wrapper for at least as long as the returned reference.
        unsafe { &*self.value }
    }
}

pub struct DataKeyRef<'a, T> {
    pub(crate) _guards: Vec<ErasedRegistryRef<'a>>,
    pub(crate) _registry: Option<Arc<dyn crate::Registry>>,
    pub(crate) value: *const T,
    pub(crate) marker: PhantomData<&'a T>,
}

impl<T> Deref for DataKeyRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        //
        // `value` points into one of `_guards`, and that guard remains owned by
        // this DataKeyRef for the lifetime of the returned reference.
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
