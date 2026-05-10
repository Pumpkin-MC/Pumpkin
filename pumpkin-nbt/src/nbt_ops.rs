use crate::compound::NbtCompound;
use crate::tag::NbtTag;
use pumpkin_codecs::DataResult;
use pumpkin_codecs::DynamicOps;
use pumpkin_codecs::Lifecycle;
use pumpkin_codecs::MapLike;
use pumpkin_codecs::Number;
use pumpkin_codecs::struct_builder::{ResultStructBuilder, StringStructBuilder, StructBuilder};
use pumpkin_codecs::{impl_get_list, impl_string_struct_builder, impl_struct_builder};
use std::iter::Map;
use std::vec::IntoIter;
use tracing::warn;

/// A [`DynamicOps`] to serialize to/deserialize from NBT data.
pub struct NbtOps;

impl DynamicOps for NbtOps {
    type Value = NbtTag;
    type StructBuilder = NbtStructBuilder;

    fn empty(&self) -> Self::Value {
        NbtTag::End
    }

    fn create_number(&self, n: Number) -> Self::Value {
        NbtTag::Double(n.into())
    }

    fn create_byte(&self, data: i8) -> Self::Value {
        NbtTag::Byte(data)
    }

    fn create_short(&self, data: i16) -> Self::Value {
        NbtTag::Short(data)
    }

    fn create_int(&self, data: i32) -> Self::Value {
        NbtTag::Int(data)
    }

    fn create_long(&self, data: i64) -> Self::Value {
        NbtTag::Long(data)
    }

    fn create_float(&self, data: f32) -> Self::Value {
        NbtTag::Float(data)
    }

    fn create_double(&self, data: f64) -> Self::Value {
        NbtTag::Double(data)
    }

    fn create_bool(&self, data: bool) -> Self::Value {
        NbtTag::Byte(data.into())
    }

    fn create_string(&self, data: &str) -> Self::Value {
        NbtTag::String(data.to_string())
    }

    fn create_list<I>(&self, values: I) -> Self::Value
    where
        I: IntoIterator<Item = Self::Value>,
    {
        ListCollector::new_collector().accept_all(values).result()
    }

    fn create_map<I>(&self, entries: I) -> Self::Value
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
    {
        let mut compound = NbtCompound::new();
        for (k, v) in entries {
            if let Some(key) = k.extract_string() {
                compound.put(key, v);
            } else {
                // The Minecraft implementation just uses the string representation of the key tag,
                // but that probably isn't meant to be intentionally used, so we will just
                // log a warning.
                warn!("Invalid key tag for creating NBT compound: {k}");
            }
        }
        compound.into()
    }

    fn get_number(&self, input: &Self::Value) -> DataResult<Number> {
        match input {
            NbtTag::Byte(b) => DataResult::new_success(Number::Byte(*b)),
            NbtTag::Short(s) => DataResult::new_success(Number::Short(*s)),
            NbtTag::Int(i) => DataResult::new_success(Number::Int(*i)),
            NbtTag::Long(l) => DataResult::new_success(Number::Long(*l)),
            NbtTag::Float(f) => DataResult::new_success(Number::Float(*f)),
            NbtTag::Double(d) => DataResult::new_success(Number::Double(*d)),

            _ => DataResult::new_error("Not a number".to_string()),
        }
    }

    fn get_string(&self, input: &Self::Value) -> DataResult<String> {
        input.extract_string().map_or_else(
            || DataResult::new_error("Not a string".to_string()),
            |s| DataResult::new_success(s.to_string()),
        )
    }

    fn get_map_iter<'a>(
        &'a self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = (Self::Value, &'a Self::Value)> + 'a> {
        if let NbtTag::Compound(compound) = input {
            DataResult::new_success(
                compound
                    .child_tags
                    .iter()
                    .map(|(k, v)| (self.create_string(k), v)),
            )
        } else {
            DataResult::new_error(format!("Not a map: {input}"))
        }
    }

    fn get_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl MapLike<Value = Self::Value> + 'a> {
        if let NbtTag::Compound(compound) = input {
            DataResult::new_success(NbtMapLike { compound })
        } else {
            DataResult::new_error(format!("Not a map: {input}"))
        }
    }

    fn get_iter(&self, input: Self::Value) -> DataResult<impl Iterator<Item = Self::Value>> {
        match input {
            NbtTag::List(l) => {
                // Check the type of this list.
                // If the list contains compounds, we try unwrapping them.
                if !l.is_empty()
                    && let NbtTag::Compound(_) = l.first().unwrap()
                {
                    DataResult::new_success(NbtIter::CompoundList(l.into_iter().map(|c| {
                        if let NbtTag::Compound(compound) = c {
                            Self::try_unwrap(compound)
                        } else {
                            c.clone()
                        }
                    })))
                } else {
                    DataResult::new_success(NbtIter::List(l.into_iter()))
                }
            }

            NbtTag::ByteArray(b) => DataResult::new_success(NbtIter::ByteArray(
                b.into_iter().map(|b| Self.create_byte(b)),
            )),
            NbtTag::IntArray(i) => DataResult::new_success(NbtIter::IntArray(
                i.into_iter().map(|i| Self.create_int(i)),
            )),
            NbtTag::LongArray(l) => DataResult::new_success(NbtIter::LongArray(
                l.into_iter().map(|l| Self.create_long(l)),
            )),

            _ => DataResult::new_error(format!("Not a list: {input}")),
        }
    }

    fn get_byte_list(&self, input: Self::Value) -> DataResult<Vec<i8>> {
        if let NbtTag::ByteArray(b) = input {
            DataResult::new_success(b)
        } else {
            impl_get_list!(self, input, "bytes")
        }
    }

    fn create_byte_list(&self, buffer: Vec<i8>) -> Self::Value {
        NbtTag::ByteArray(buffer)
    }

    fn get_int_list(&self, input: Self::Value) -> DataResult<Vec<i32>> {
        if let NbtTag::IntArray(i) = input {
            DataResult::new_success(i)
        } else {
            impl_get_list!(self, input, "ints")
        }
    }

    fn create_int_list(&self, list: Vec<i32>) -> Self::Value {
        NbtTag::IntArray(list)
    }

    fn get_long_list(&self, input: Self::Value) -> DataResult<Vec<i64>> {
        if let NbtTag::LongArray(i) = input {
            DataResult::new_success(i)
        } else {
            impl_get_list!(self, input, "longs")
        }
    }

    fn create_long_list(&self, list: Vec<i64>) -> Self::Value {
        NbtTag::LongArray(list)
    }

    fn merge_into_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value> {
        ListCollector::new(list.clone()).map_or_else(
            || DataResult::new_partial_error("Not a list".to_string(), list),
            |c| DataResult::new_success(c.accept(value).result()),
        )
    }

    fn merge_values_into_list<I>(&self, list: Self::Value, values: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = Self::Value>,
    {
        ListCollector::new(list.clone()).map_or_else(
            || DataResult::new_partial_error("Not a list".to_string(), list),
            |c| DataResult::new_success(c.accept_all(values).result()),
        )
    }

    fn merge_into_map(
        &self,
        map: Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        Self::Value: Clone,
    {
        if !matches!(map, NbtTag::Compound(_) | NbtTag::End) {
            DataResult::new_partial_error(format!("Not a map: {map}"), map)
        } else if !matches!(key, NbtTag::String(_)) {
            DataResult::new_partial_error(format!("Key is not a string: {key}"), map)
        } else {
            let mut compound = if let NbtTag::Compound(c) = map {
                c
            } else {
                NbtCompound::new()
            };
            compound.put(key.extract_string().unwrap(), value);
            DataResult::new_success(compound.into())
        }
    }

    fn merge_map_like_into_map<M>(
        &self,
        map: Self::Value,
        other_map_like: M,
    ) -> DataResult<Self::Value>
    where
        M: MapLike<Value = Self::Value>,
        Self::Value: Clone,
    {
        if matches!(map, NbtTag::Compound(_) | NbtTag::End) {
            let mut compound = if let NbtTag::Compound(c) = map {
                c
            } else {
                NbtCompound::default()
            };
            let mut failed = vec![];
            other_map_like.iter().for_each(|(k, v)| {
                if let NbtTag::String(key) = k {
                    compound.put(&key, v.clone());
                } else {
                    failed.push((k, v));
                }
            });
            if failed.is_empty() {
                DataResult::new_success(compound.into())
            } else {
                DataResult::new_partial_error(
                    format!("Some keys are not strings: {failed:?}"),
                    NbtTag::Compound(compound),
                )
            }
        } else {
            DataResult::new_partial_error(format!("Not a map: {map}"), map)
        }
    }

    fn remove(&self, input: Self::Value, key: &str) -> Self::Value {
        if let NbtTag::Compound(compound) = input {
            // Try to remove any entries whose key matches with `key`.
            NbtTag::Compound(
                compound
                    .child_tags
                    .into_iter()
                    .filter(|s| s.0 != key)
                    .collect(),
            )
        } else {
            input
        }
    }

    fn convert_to<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: Self::Value) -> U {
        match input {
            NbtTag::End => out_ops.empty(),
            NbtTag::Byte(b) => out_ops.create_byte(b),
            NbtTag::Short(s) => out_ops.create_short(s),
            NbtTag::Int(i) => out_ops.create_int(i),
            NbtTag::Long(l) => out_ops.create_long(l),
            NbtTag::Float(f) => out_ops.create_float(f),
            NbtTag::Double(d) => out_ops.create_double(d),
            NbtTag::ByteArray(b) => out_ops.create_byte_list(b),
            NbtTag::String(s) => out_ops.create_string(&s),
            NbtTag::List(_) => self.convert_list(out_ops, input),
            NbtTag::Compound(_) => self.convert_map(out_ops, input),
            NbtTag::IntArray(i) => out_ops.create_int_list(i),
            NbtTag::LongArray(l) => out_ops.create_long_list(l),
        }
    }

    fn map_builder(&'static self) -> Self::StructBuilder {
        NbtStructBuilder {
            builder: DataResult::new_success_with_lifecycle(
                NbtTag::Compound(NbtCompound::new()),
                Lifecycle::Stable,
            ),
        }
    }
}

impl NbtOps {
    /// Tries to unwrap an [`NbtCompound`].
    ///
    /// If `compound` only has one element with an empty key (`""`), it returns that element.
    /// Otherwise, this simply returns a new [`NbtTag::Compound`] with `compound`.
    fn try_unwrap(mut compound: NbtCompound) -> NbtTag {
        if compound.child_tags.len() == 1 && compound.has("") {
            // Remove the element to own the contained tag.
            compound.child_tags.remove("").unwrap()
        } else {
            NbtTag::from(compound)
        }
    }
}

/// A single concrete type for an iterator of an NBT element.
enum NbtIter {
    List(IntoIter<NbtTag>),
    CompoundList(Map<IntoIter<NbtTag>, fn(NbtTag) -> NbtTag>),
    ByteArray(Map<IntoIter<i8>, fn(i8) -> NbtTag>),
    IntArray(Map<IntoIter<i32>, fn(i32) -> NbtTag>),
    LongArray(Map<IntoIter<i64>, fn(i64) -> NbtTag>),
}

impl Iterator for NbtIter {
    type Item = NbtTag;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::List(iter) => iter.next(),
            Self::CompoundList(iter) => iter.next(),
            Self::ByteArray(iter) => iter.next(),
            Self::IntArray(iter) => iter.next(),
            Self::LongArray(iter) => iter.next(),
        }
    }
}

/// An implementation of [`MapLike`] for NBT objects.
/// The lifetime is that of the referenced map.
struct NbtMapLike<'a> {
    compound: &'a NbtCompound,
}

impl MapLike for NbtMapLike<'_> {
    type Value = NbtTag;

    fn get(&self, key: &Self::Value) -> Option<&Self::Value> {
        key.extract_string().and_then(|s| self.get_str(s))
    }

    fn get_str(&self, key: &str) -> Option<&Self::Value> {
        self.compound.get(key)
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Value, &Self::Value)> + '_ {
        self.compound
            .child_tags
            .iter()
            .map(|(k, v)| (NbtTag::String(k.clone()), v))
    }
}

/// An implementation of [`StructBuilder`] for NBT objects.
pub struct NbtStructBuilder {
    builder: DataResult<NbtTag>,
}

impl ResultStructBuilder for NbtStructBuilder {
    type Result = NbtTag;

    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value> {
        match prefix {
            NbtTag::End => DataResult::new_success(builder),
            NbtTag::Compound(mut compound) => {
                match builder {
                    NbtTag::Compound(builder_compound) => {
                        for (k, v) in builder_compound {
                            compound.put(&k, v);
                        }
                    }
                    // This shouldn't happen, but just in case.
                    _ => {
                        return DataResult::new_error(format!(
                            "Expected compound in builder, found {builder}"
                        ));
                    }
                }
                DataResult::new_success(compound.into())
            }
            _ => DataResult::new_partial_error(format!("Prefix is not a map: {prefix}"), prefix),
        }
    }
}

impl StructBuilder for NbtStructBuilder {
    type Value = NbtTag;

    impl_struct_builder!(builder);
    impl_string_struct_builder!(builder, NbtOps);
}

impl StringStructBuilder for NbtStructBuilder {
    fn append(&self, key: &str, value: Self::Value, builder: Self::Result) -> Self::Result {
        if let NbtTag::Compound(mut compound) = builder {
            compound.put(key, value);
            compound.into()
        } else {
            builder
        }
    }
}

// List collectors

/// A collector object for NBT lists.
///
/// The variants of this object should not be used as that is an implementation detail.
enum ListCollector {
    Generic(InnerGenericListCollector),

    Byte(InnerByteListCollector),
    Int(InnerIntListCollector),
    Long(InnerLongListCollector),
}

impl ListCollector {
    /// Creates a new [`ListCollector`].
    ///
    /// This only returns an actual collector for [`NbtTag::End`] and all list [`NbtTag`]s.
    fn new(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::End => Some(Self::new_collector()),

            NbtTag::List(_) | NbtTag::ByteArray(_) | NbtTag::IntArray(_) | NbtTag::LongArray(_) => {
                // Try to get the length of the tag.
                let len = match &tag {
                    NbtTag::List(list) => list.len(),

                    NbtTag::ByteArray(list) => list.len(),
                    NbtTag::IntArray(list) => list.len(),
                    NbtTag::LongArray(list) => list.len(),

                    _ => unreachable!(),
                };

                if len == 0 {
                    return Some(Self::new_collector());
                }

                // From this point onwards, we know that the list is not empty.
                match tag {
                    NbtTag::List(list) => Some(Self::Generic(InnerGenericListCollector::new(list))),
                    NbtTag::ByteArray(list) => Some(Self::Byte(InnerByteListCollector::new(list))),
                    NbtTag::IntArray(list) => Some(Self::Int(InnerIntListCollector::new(list))),
                    NbtTag::LongArray(list) => Some(Self::Long(InnerLongListCollector::new(list))),

                    _ => unreachable!(),
                }
            }

            _ => None,
        }
    }

    /// Creates a new initial collector.
    /// [`NbtTag`]s can directly be added to this collector without any type worries.
    const fn new_collector() -> Self {
        Self::Generic(InnerGenericListCollector {
            result: NbtTag::List(vec![]),
        })
    }

    /// Accepts an [`NbtTag`].
    fn accept(self, tag: NbtTag) -> Self {
        match self {
            Self::Generic(c) => c.accept(tag),
            Self::Byte(c) => c.accept(tag),
            Self::Int(c) => c.accept(tag),
            Self::Long(c) => c.accept(tag),
        }
    }

    /// Accepts all [`NbtTag`]s of the provided list.
    fn accept_all(self, tags: impl IntoIterator<Item = NbtTag>) -> Self {
        let mut collector = self;
        for tag in tags {
            collector = collector.accept(tag);
        }
        collector
    }

    /// Provides the final result.
    fn result(self) -> NbtTag {
        match self {
            Self::Generic(c) => c.result(),
            Self::Byte(c) => c.result(),
            Self::Int(c) => c.result(),
            Self::Long(c) => c.result(),
        }
    }
}

/// An 'inner' list collector stored in one of the corresponding [`ListCollector`] enums.
trait InnerListCollector {
    fn accept(self, tag: NbtTag) -> ListCollector
    where
        Self: Sized;

    fn result(self) -> NbtTag;
}

/// An implementation of [`InnerListCollector`] for a generic list (of any type).
struct InnerGenericListCollector {
    result: NbtTag,
}

impl InnerListCollector for InnerGenericListCollector {
    fn accept(mut self, tag: NbtTag) -> ListCollector
    where
        Self: Sized,
    {
        match &mut self.result {
            NbtTag::List(list) => list.push(tag),
            _ => unreachable!(),
        }
        ListCollector::Generic(self)
    }

    fn result(self) -> NbtTag {
        self.result
    }
}

impl From<InnerByteListCollector> for InnerGenericListCollector {
    fn from(value: InnerByteListCollector) -> Self {
        Self {
            result: NbtTag::List(value.list.into_iter().map(NbtTag::Byte).collect()),
        }
    }
}

impl InnerGenericListCollector {
    const fn new(list: Vec<NbtTag>) -> Self {
        Self {
            result: NbtTag::List(list),
        }
    }
}

/// An implementation of [`InnerListCollector`] specifically for [`NbtTag::ByteArray`]s.
struct InnerByteListCollector {
    list: Vec<i8>,
}

impl InnerListCollector for InnerByteListCollector {
    fn accept(mut self, tag: NbtTag) -> ListCollector
    where
        Self: Sized,
    {
        if let NbtTag::Byte(byte) = tag {
            self.list.push(byte);
            ListCollector::Byte(self)
        } else {
            <Self as Into<InnerGenericListCollector>>::into(self).accept(tag)
        }
    }

    fn result(self) -> NbtTag {
        NbtTag::ByteArray(self.list)
    }
}

impl InnerByteListCollector {
    const fn new(list: Vec<i8>) -> Self {
        Self { list }
    }
}

macro_rules! add_inner_specific_array_collector_impl {
    ($name:ident, $single_type:ident, $array_type:ident, $ty:ty) => {
        #[doc = concat!("An implementation of [`InnerListCollector`] specifically for [`NbtTag::", stringify!($array_type), "`]s.")]
        struct $name {
            list: Vec<$ty>
        }

        impl InnerListCollector for $name {
            fn accept(mut self, tag: NbtTag) -> ListCollector
            where
                Self: Sized
            {
                if let NbtTag::$single_type(v) = tag {
                    self.list.push(v);
                    ListCollector::$single_type(self)
                } else {
                    <Self as Into<InnerGenericListCollector>>::into(self)
                        .accept(tag)
                }
            }

            fn result(self) -> NbtTag {
                NbtTag::$array_type(self.list)
            }
        }

        impl $name {
            const fn new(list: Vec<$ty>) -> Self {
                Self {
                    list
                }
            }
        }

        impl From<$name> for InnerGenericListCollector {
            fn from(value: $name) -> Self {
                InnerGenericListCollector {
                    result: NbtTag::List(
                        value.list.into_iter().map(|b| NbtTag::$single_type(b)).collect()
                    )
                }
            }
        }
    };
}

add_inner_specific_array_collector_impl!(InnerIntListCollector, Int, IntArray, i32);
add_inner_specific_array_collector_impl!(InnerLongListCollector, Long, LongArray, i64);

#[cfg(test)]
mod test {
    use crate::nbt_ops::ListCollector;
    use crate::tag::NbtTag;

    #[test]
    fn list_collecting() {
        // Int list collector
        let tag = NbtTag::IntArray(vec![10, 15, 20]);

        assert_eq!(
            ListCollector::new(tag)
                .expect("List collector should exist")
                .result(),
            NbtTag::IntArray(vec![10, 15, 20])
        );

        // Byte list collector
        let tag = NbtTag::ByteArray(vec![-1, 45, 100]);

        assert_eq!(
            ListCollector::new(tag)
                .expect("List collector should exist")
                .result(),
            NbtTag::ByteArray(vec![-1, 45, 100])
        );

        // Long list
        let tag = NbtTag::LongArray(vec![11_234_567_890, -986, 1, -937_238_122]);

        assert_eq!(
            ListCollector::new(tag)
                .expect("List collector should exist")
                .result(),
            NbtTag::LongArray(vec![11_234_567_890, -986, 1, -937_238_122])
        );

        // Generic list collector
        // Homogeneous elements
        let mut collector = ListCollector::new_collector();

        collector = collector.accept(NbtTag::Float(-123.4));
        collector = collector.accept(NbtTag::Float(12.5));

        assert_eq!(
            collector.result(),
            NbtTag::List(vec![NbtTag::Float(-123.4), NbtTag::Float(12.5)])
        );

        // Heterogeneous elements
        let mut collector = ListCollector::new_collector();

        collector = collector.accept(NbtTag::Byte(99));
        collector = collector.accept(NbtTag::String("99".to_string()));
        collector = collector.accept(NbtTag::LongArray(vec![1, 2, 3]));

        assert_eq!(
            collector.result(),
            NbtTag::List(vec![
                NbtTag::Byte(99),
                NbtTag::String("99".to_string()),
                NbtTag::LongArray(vec![1, 2, 3])
            ])
        );
    }
}
