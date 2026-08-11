use crate::error::BootstrapError;
pub use linkme::distributed_slice;
use pumpkin_util::identifier::Identifier;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::any::TypeId;

pub struct RegistryEntry<T> {
    identifier: Identifier,
    value: T,
}

impl<T> RegistryEntry<T> {
    pub const fn new(identifier: Identifier, value: T) -> Self {
        Self { identifier, value }
    }

    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

#[repr(C)]
pub struct ErasedVec {
    ptr: *mut (),
    len: usize,
    capacity: usize,
    type_id: TypeId,
}

impl ErasedVec {
    #[must_use]
    pub fn from_vec<T: 'static>(vec: Vec<T>) -> Self {
        let (ptr, len, capacity) = vec.into_raw_parts();

        Self {
            ptr: ptr.cast(),
            len,
            capacity,
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn into_vec<T: 'static>(self) -> Result<Vec<T>, Self> {
        if self.type_id != TypeId::of::<T>() {
            return Err(self);
        }

        // SAFETY:
        // `self.type_id == TypeId::of::<T>()` guarantees that this allocation was
        // originally created from a `Vec<T>`.
        //
        // Therefore:
        //
        // - `self.ptr` was allocated for `T` and has the alignment required by `T`;
        // - `self.ptr` was allocated using the allocator expected by `Vec<T>`;
        // - `self.len` is the number of initialized `T` values currently stored;
        // - `self.capacity` is the original allocation capacity in units of `T`;
        // - `self.len <= self.capacity`;
        // - the allocation has not already been freed or reconstructed into
        //   another owning container;
        // - ownership of the allocation is transferred to the returned `Vec<T>`,
        //   so it will be deallocated exactly once.
        //
        // These invariants are established when this erased allocation is created,
        // and `type_id` is never changed independently of `ptr`, `len`, or
        // `capacity`.
        Ok(unsafe { Vec::from_raw_parts(self.ptr.cast::<T>(), self.len, self.capacity) })
    }

    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }
}

#[repr(C)]
pub struct BootstrapProvider {
    registry: &'static str,
    populate: fn() -> ErasedVec,
}

impl BootstrapProvider {
    pub const fn new(registry: &'static str, populate: fn() -> ErasedVec) -> Self {
        Self { registry, populate }
    }

    #[must_use]
    pub const fn registry(&self) -> Identifier {
        Identifier::parse_static(self.registry)
    }

    #[must_use]
    pub fn populate(&self) -> ErasedVec {
        (self.populate)()
    }
}

#[macro_export]
macro_rules! bootstrap_provider {
    (
        $name:ident : $ty:ty => $registry:expr => {
            $(
                $identifier:literal => $value:expr
            ),* $(,)?
        }
    ) => {
        const _: () = {
            fn populate() -> Vec<$crate::bootstrap::RegistryEntry<$ty>> {
                vec![
                    $(
                        $crate::bootstrap::RegistryEntry::new(
                            pumpkin_util::identifier::Identifier::parse_static(
                                $identifier
                            ),
                            $value
                        ),
                    )*
                ]
            }

            fn populate_erased() -> $crate::bootstrap::ErasedVec {
                $crate::bootstrap::ErasedVec::from_vec(populate())
            }

            #[$crate::bootstrap::distributed_slice($crate::bootstrap::PROVIDERS)]
            static $name: $crate::bootstrap::BootstrapProvider =
                $crate::bootstrap::BootstrapProvider::new(
                    $registry,
                    populate_erased,
                );
        };
    };

    (
        $name:ident : $ty:ty => $registry:expr,
        $populate:expr
        $(,)?
    ) => {
        const _: () = {
            fn populate() -> Vec<$crate::bootstrap::RegistryEntry<$ty>> {
                ($populate)()
            }

            fn populate_erased() -> $crate::bootstrap::ErasedVec {
                $crate::bootstrap::ErasedVec::from_vec(populate())
            }

            #[$crate::bootstrap::distributed_slice($crate::bootstrap::PROVIDERS)]
            static $name: $crate::bootstrap::BootstrapProvider =
                $crate::bootstrap::BootstrapProvider::new(
                    $registry,
                    populate_erased,
                );
        };
    };
}

#[distributed_slice]
pub static PROVIDERS: [BootstrapProvider];

pub struct BootstrapManager<'a> {
    sources: Vec<&'a [BootstrapProvider]>,
}

impl<'a> BootstrapManager<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_providers(&mut self, providers: &'a [BootstrapProvider]) {
        self.sources.push(providers);
    }

    pub fn providers(&self) -> impl Iterator<Item = &BootstrapProvider> {
        PROVIDERS
            .iter()
            .chain(self.sources.iter().flat_map(|source| source.iter()))
    }

    pub fn providers_for<'b>(
        &'b self,
        registry: &'b Identifier,
    ) -> impl Iterator<Item = &'b BootstrapProvider> {
        self.providers()
            .filter(move |provider| provider.registry() == *registry)
    }

    pub fn populate<T>(
        &self,
        registry: &Identifier,
    ) -> Result<(Vec<T>, FxHashMap<Identifier, usize>), BootstrapError>
    where
        T: Send + 'static,
    {
        let builtin_entries = populate_sources::<T>(std::slice::from_ref(&&*PROVIDERS), registry)?;

        let added_entries = populate_sources::<T>(&self.sources, registry)?;

        let capacity = builtin_entries
            .iter()
            .chain(&added_entries)
            .map(Vec::len)
            .sum();

        let mut entries = Vec::with_capacity(capacity);
        let mut mapping = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);

        for source_entries in builtin_entries.into_iter().chain(added_entries) {
            for entry in source_entries {
                let id = entries.len();

                if mapping.insert(entry.identifier.clone(), id).is_some() {
                    return Err(BootstrapError::DuplicateEntry {
                        registry: registry.clone(),
                        identifier: entry.identifier,
                    });
                }

                entries.push(entry.value);
            }
        }

        Ok((entries, mapping))
    }
}

impl Default for BootstrapManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn populate_sources<T>(
    sources: &[&[BootstrapProvider]],
    registry: &Identifier,
) -> Result<Vec<Vec<RegistryEntry<T>>>, BootstrapError>
where
    T: Send + 'static,
{
    sources
        .par_iter()
        .flat_map_iter(|source| source.iter())
        .filter(|provider| provider.registry() == *registry)
        .map(|provider| {
            let erased = provider.populate();
            let actual = erased.type_id();

            erased
                .into_vec::<RegistryEntry<T>>()
                .map_err(|_| BootstrapError::TypeMismatch {
                    registry: provider.registry(),
                    expected: TypeId::of::<RegistryEntry<T>>(),
                    actual,
                })
        })
        .collect()
}
