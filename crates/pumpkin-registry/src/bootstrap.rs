use crate::{RegistryConfig, error::BootstrapError};
pub use linkme::{self as __linkme, distributed_slice};
use pumpkin_util::identifier::Identifier;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    any::TypeId,
    borrow::Cow,
    ops::Deref,
    sync::{Arc, Weak},
};

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
#[derive(Clone)]
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
            #[linkme(crate = $crate::bootstrap::__linkme)]
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
            #[linkme(crate = $crate::bootstrap::__linkme)]
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

pub type ProviderSet = Cow<'static, [BootstrapProvider]>;

pub enum ProviderRef {
    Builtin(&'static BootstrapProvider),
    Dynamic {
        source: Arc<ProviderSet>,
        index: usize,
    },
}

impl Deref for ProviderRef {
    type Target = BootstrapProvider;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Builtin(provider) => provider,
            Self::Dynamic { source, index } => &source[*index],
        }
    }
}

pub struct BootstrapManager {
    sources: Vec<Weak<ProviderSet>>,
}

impl BootstrapManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_providers(&mut self, providers: &Arc<ProviderSet>) {
        self.sources.push(Arc::downgrade(providers));
    }

    pub fn providers(&self) -> impl Iterator<Item = ProviderRef> + '_ {
        let builtin = PROVIDERS.iter().map(ProviderRef::Builtin);

        let dynamic = self
            .sources
            .iter()
            .filter_map(Weak::upgrade)
            .flat_map(|source| {
                let len = source.len();

                (0..len).map(move |index| ProviderRef::Dynamic {
                    source: Arc::clone(&source),
                    index,
                })
            });

        builtin.chain(dynamic)
    }

    pub fn providers_for<'a>(
        &'a self,
        registry: &'a Identifier,
    ) -> impl Iterator<Item = ProviderRef> + 'a {
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
        self.populate_with_config(registry, RegistryConfig::default())
    }

    pub fn populate_with_config<T>(
        &self,
        registry: &Identifier,
        config: RegistryConfig,
    ) -> Result<(Vec<T>, FxHashMap<Identifier, usize>), BootstrapError>
    where
        T: Send + 'static,
    {
        let sources: Vec<_> = self.sources.iter().filter_map(Weak::upgrade).collect();

        let builtin_entries = populate_providers::<T>(&PROVIDERS, registry)?;

        let added_entries: Vec<Vec<RegistryEntry<T>>> = sources
            .par_iter()
            .flat_map_iter(|source| source.iter())
            .filter(|provider| provider.registry() == *registry)
            .map(populate_provider)
            .collect::<Result<_, _>>()?;

        let capacity = builtin_entries
            .iter()
            .chain(&added_entries)
            .map(Vec::len)
            .sum();

        let mut entries = Vec::with_capacity(capacity);
        let mut mapping = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);

        for source_entries in builtin_entries.into_iter().chain(added_entries) {
            for entry in source_entries {
                if let Some(&id) = mapping.get(&entry.identifier) {
                    if !config.allow_overwrites {
                        return Err(BootstrapError::DuplicateEntry {
                            registry: registry.clone(),
                            identifier: entry.identifier,
                        });
                    }

                    entries[id] = entry.value;
                    continue;
                }

                let id = entries.len();
                mapping.insert(entry.identifier, id);
                entries.push(entry.value);
            }
        }

        Ok((entries, mapping))
    }
}

impl Default for BootstrapManager {
    fn default() -> Self {
        Self::new()
    }
}

fn populate_providers<T>(
    providers: &[BootstrapProvider],
    registry: &Identifier,
) -> Result<Vec<Vec<RegistryEntry<T>>>, BootstrapError>
where
    T: Send + 'static,
{
    providers
        .par_iter()
        .filter(|provider| provider.registry() == *registry)
        .map(populate_provider)
        .collect()
}

fn populate_provider<T>(
    provider: &BootstrapProvider,
) -> Result<Vec<RegistryEntry<T>>, BootstrapError>
where
    T: Send + 'static,
{
    let erased = provider.populate();
    let actual = erased.type_id();

    erased
        .into_vec::<RegistryEntry<T>>()
        .map_err(|_| BootstrapError::TypeMismatch {
            registry: provider.registry(),
            expected: TypeId::of::<RegistryEntry<T>>(),
            actual,
        })
}
