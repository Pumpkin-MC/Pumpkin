use crate::compound::NbtCompound;
use crate::tag::NbtTag;
use crate::{COMPOUND_ID, END_ID};
use pumpkin_datafixerupper::serialization::Number;
use pumpkin_datafixerupper::serialization::data_result::DataResult;
use pumpkin_datafixerupper::serialization::dynamic_ops::DynamicOps;
use pumpkin_datafixerupper::serialization::lifecycle::Lifecycle;
use pumpkin_datafixerupper::serialization::map_like::MapLike;
use pumpkin_datafixerupper::serialization::struct_builder::{
    ResultStructBuilder, StringStructBuilder, StructBuilder,
};
use pumpkin_datafixerupper::{impl_get_list, impl_string_struct_builder, impl_struct_builder};
use std::iter::Map;
use std::vec::IntoIter;

/// A [`DynamicOps`] to serialize to/deserialize from NBT data.
pub struct NbtOps;

/// An instance of [`NbtOps`], which serializes/deserializes NBT data.
pub static INSTANCE: NbtOps = NbtOps;

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
        ListCollector::new_initial_collector()
            .accept_all(values)
            .result()
    }

    fn create_map<I>(&self, entries: I) -> Self::Value
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
    {
        let mut compound = NbtCompound::new();
        for (k, v) in entries {
            if let Some(key) = k.extract_string() {
                compound.put(key, v);
            }
        }
        compound.into()
    }

    fn get_number(&self, input: &Self::Value) -> DataResult<Number> {
        match input {
            NbtTag::Byte(b) => DataResult::success(Number::Byte(*b)),
            NbtTag::Short(s) => DataResult::success(Number::Short(*s)),
            NbtTag::Int(i) => DataResult::success(Number::Int(*i)),
            NbtTag::Long(l) => DataResult::success(Number::Long(*l)),
            NbtTag::Float(f) => DataResult::success(Number::Float(*f)),
            NbtTag::Double(d) => DataResult::success(Number::Double(*d)),

            _ => DataResult::error("Not a number".to_string()),
        }
    }

    fn get_string(&self, input: &Self::Value) -> DataResult<String> {
        input.extract_string().map_or_else(
            || DataResult::error("Not a string".to_string()),
            |s| DataResult::success(s.to_string()),
        )
    }

    fn get_map_iter<'a>(
        &'a self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = (Self::Value, &'a Self::Value)> + 'a> {
        if let NbtTag::Compound(compound) = input {
            DataResult::success(
                compound
                    .child_tags
                    .iter()
                    .map(|(k, v)| (self.create_string(k), v)),
            )
        } else {
            DataResult::error(format!("Not a map: {input}"))
        }
    }

    fn get_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl MapLike<Value = Self::Value> + 'a> {
        if let NbtTag::Compound(compound) = input {
            DataResult::success(NbtMapLike { compound })
        } else {
            DataResult::error(format!("Not a map: {input}"))
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
                    DataResult::success(NbtIter::CompoundList(l.into_iter().map(|c| {
                        if let NbtTag::Compound(compound) = c {
                            Self::try_unwrap(compound)
                        } else {
                            c.clone()
                        }
                    })))
                } else {
                    DataResult::success(NbtIter::List(l.into_iter()))
                }
            }

            NbtTag::ByteArray(b) => DataResult::success(NbtIter::ByteArray(
                b.into_iter().map(|b| Self.create_byte(b as i8)),
            )),
            NbtTag::IntArray(i) => {
                DataResult::success(NbtIter::IntArray(i.into_iter().map(|i| Self.create_int(i))))
            }
            NbtTag::LongArray(l) => DataResult::success(NbtIter::LongArray(
                l.into_iter().map(|l| Self.create_long(l)),
            )),

            _ => DataResult::error(format!("Not a list: {input}")),
        }
    }

    fn get_byte_buffer(&self, input: Self::Value) -> DataResult<Box<[u8]>> {
        if let NbtTag::ByteArray(b) = input {
            DataResult::success(b)
        } else {
            impl_get_list!(box self, input, "bytes")
        }
    }

    fn create_byte_buffer(&self, buffer: Vec<u8>) -> Self::Value {
        NbtTag::ByteArray(buffer.into_boxed_slice())
    }

    fn get_int_list(&self, input: Self::Value) -> DataResult<Vec<i32>> {
        if let NbtTag::IntArray(i) = input {
            DataResult::success(i)
        } else {
            impl_get_list!(self, input, "ints")
        }
    }

    fn create_int_list(&self, list: Vec<i32>) -> Self::Value {
        NbtTag::IntArray(list)
    }

    fn get_long_list(&self, input: Self::Value) -> DataResult<Vec<i64>> {
        if let NbtTag::LongArray(i) = input {
            DataResult::success(i)
        } else {
            impl_get_list!(self, input, "longs")
        }
    }

    fn create_long_list(&self, list: Vec<i64>) -> Self::Value {
        NbtTag::LongArray(list)
    }

    fn merge_into_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value> {
        ListCollector::new(list.clone()).map_or_else(
            || DataResult::partial_error("Not a list".to_string(), list),
            |c| DataResult::success(c.accept(value).result()),
        )
    }

    fn merge_values_into_list<I>(&self, list: Self::Value, values: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = Self::Value>,
    {
        ListCollector::new(list.clone()).map_or_else(
            || DataResult::partial_error("Not a list".to_string(), list),
            |c| DataResult::success(c.accept_all(values).result()),
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
            DataResult::partial_error(format!("Not a map: {map}"), map)
        } else if !matches!(key, NbtTag::String(_)) {
            DataResult::partial_error(format!("Key is not a string: {key}"), map)
        } else {
            let mut compound = if let NbtTag::Compound(c) = map {
                c
            } else {
                NbtCompound::new()
            };
            compound.put(key.extract_string().unwrap(), value);
            DataResult::success(compound.into())
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
                DataResult::success(compound.into())
            } else {
                DataResult::partial_error(
                    format!("Some keys are not strings: {failed:?}"),
                    NbtTag::Compound(compound),
                )
            }
        } else {
            DataResult::partial_error(format!("Not a map: {map}"), map)
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
            NbtTag::ByteArray(b) => out_ops.create_byte_buffer(b.to_vec()),
            NbtTag::String(s) => out_ops.create_string(&s),
            NbtTag::List(_) => self.convert_list(out_ops, input),
            NbtTag::Compound(_) => self.convert_map(out_ops, input),
            NbtTag::IntArray(i) => out_ops.create_int_list(i),
            NbtTag::LongArray(l) => out_ops.create_long_list(l),
        }
    }

    fn map_builder(&'static self) -> Self::StructBuilder {
        NbtStructBuilder {
            builder: DataResult::success_with_lifecycle(
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
        if compound.child_tags.len() == 1
            && let Some(_) = compound.get("")
        {
            // Remove the element to own the contained tag.
            compound.child_tags.remove(0).1
        } else {
            NbtTag::from(compound)
        }
    }
}

/// A single concrete type for an iterator of an NBT element.
enum NbtIter {
    List(IntoIter<NbtTag>),
    CompoundList(Map<IntoIter<NbtTag>, fn(NbtTag) -> NbtTag>),
    ByteArray(Map<IntoIter<u8>, fn(u8) -> NbtTag>),
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
            NbtTag::End => DataResult::success(builder),
            NbtTag::Compound(mut compound) => {
                match builder {
                    NbtTag::Compound(builder_compound) => {
                        for (k, v) in builder_compound {
                            compound.put(&k, v);
                        }
                    }
                    _ => unreachable!(),
                }
                DataResult::success(compound.into())
            }
            _ => DataResult::partial_error(format!("Prefix is not a map: {prefix}"), prefix),
        }
    }
}

impl StructBuilder for NbtStructBuilder {
    type Value = NbtTag;

    impl_struct_builder!(builder);
    impl_string_struct_builder!(builder, INSTANCE);
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
    Heterogeneous(InnerHeterogeneousListCollector),
    Homogeneous(InnerHomogeneousListCollector),
    Initial(InnerInitialListCollector),

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
            NbtTag::End => Some(Self::new_initial_collector()),

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
                    return Some(Self::new_initial_collector());
                }

                // From this point onwards, we know that the list is not empty.
                match tag {
                    NbtTag::List(list) => {
                        // Check the type of element this list has.
                        match list.first().unwrap().get_type_id() {
                            END_ID => Some(Self::new_initial_collector()),
                            COMPOUND_ID => Some(Self::Heterogeneous(
                                InnerHeterogeneousListCollector::new(list),
                            )),
                            _ => Some(Self::Homogeneous(InnerHomogeneousListCollector::new(list))),
                        }
                    }

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
    const fn new_initial_collector() -> Self {
        Self::Initial(InnerInitialListCollector)
    }

    /// Accepts an [`NbtTag`].
    fn accept(self, tag: NbtTag) -> Self {
        match self {
            Self::Heterogeneous(c) => c.accept(tag),
            Self::Homogeneous(c) => c.accept(tag),
            Self::Initial(c) => c.accept(tag),

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
            Self::Heterogeneous(c) => c.result(),
            Self::Homogeneous(c) => c.result(),
            Self::Initial(c) => c.result(),

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

/// An implementation of [`InnerListCollector`] for a list with more than 1 type of element.
///
/// Internally, each element is wrapped in a [`NbtTag::Compound`] to preserve the same type for the
/// underlying list.
struct InnerHeterogeneousListCollector {
    result: Vec<NbtTag>,
}

impl InnerListCollector for InnerHeterogeneousListCollector {
    fn accept(mut self, tag: NbtTag) -> ListCollector
    where
        Self: Sized,
    {
        self.result.push(Self::wrap_if_required(tag));
        ListCollector::Heterogeneous(self)
    }

    fn result(self) -> NbtTag {
        NbtTag::List(self.result)
    }
}

impl InnerHeterogeneousListCollector {
    const fn new(list: Vec<NbtTag>) -> Self {
        Self { result: list }
    }

    const fn is_wrapper(tag: &NbtTag) -> bool {
        if let NbtTag::Compound(compound) = tag {
            compound.child_tags.len() == 1
        } else {
            false
        }
    }
    fn wrap_if_required(tag: NbtTag) -> NbtTag {
        if !Self::is_wrapper(&tag)
            && let NbtTag::Compound(compound) = tag
        {
            compound.into()
        } else {
            Self::wrap_element(tag)
        }
    }
    fn wrap_element(tag: NbtTag) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put("", tag);
        compound.into()
    }
}

impl From<InnerHomogeneousListCollector> for InnerHeterogeneousListCollector {
    fn from(collector: InnerHomogeneousListCollector) -> Self {
        Self {
            result: collector.result,
        }
    }
}

impl From<InnerByteListCollector> for InnerHeterogeneousListCollector {
    fn from(value: InnerByteListCollector) -> Self {
        Self {
            result: value
                .list
                .into_iter()
                .map(|b| Self::wrap_element(NbtTag::Byte(b)))
                .collect(),
        }
    }
}

/// An implementation of [`InnerListCollector`] for a list with exactly 1 type of element.
struct InnerHomogeneousListCollector {
    result: Vec<NbtTag>,
}

impl InnerListCollector for InnerHomogeneousListCollector {
    fn accept(mut self, tag: NbtTag) -> ListCollector
    where
        Self: Sized,
    {
        if tag.get_type_id() == self.element_type() {
            self.result.push(tag);
            ListCollector::Homogeneous(self)
        } else {
            // Switch to a heterogeneous collector.
            self.result.push(tag);
            ListCollector::Heterogeneous(self.into())
        }
    }

    fn result(self) -> NbtTag {
        NbtTag::List(self.result)
    }
}

impl InnerHomogeneousListCollector {
    /// Returns the element type of list of this collector.
    fn element_type(&self) -> u8 {
        if self.result.is_empty() {
            END_ID
        } else {
            // Check the first element's type.
            self.result[0].get_type_id()
        }
    }

    const fn new(list: Vec<NbtTag>) -> Self {
        Self { result: list }
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
            <Self as Into<InnerHeterogeneousListCollector>>::into(self).accept(tag)
        }
    }

    fn result(self) -> NbtTag {
        NbtTag::ByteArray(
            self.list
                .into_iter()
                .map(|i| i as u8)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

impl InnerByteListCollector {
    fn new(list: Box<[u8]>) -> Self {
        Self {
            list: list.into_iter().map(|i| i as i8).collect(),
        }
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
                    <Self as Into<InnerHeterogeneousListCollector>>::into(self)
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

        impl From<$name> for InnerHeterogeneousListCollector {
            fn from(value: $name) -> Self {
                InnerHeterogeneousListCollector {
                    result: value.list.into_iter().map(|b| Self::wrap_element(NbtTag::$single_type(b))).collect()
                }
            }
        }
    };
}

add_inner_specific_array_collector_impl!(InnerIntListCollector, Int, IntArray, i32);
add_inner_specific_array_collector_impl!(InnerLongListCollector, Long, LongArray, i64);

/// An implementation of [`InnerListCollector`] to start with.
/// This list collector turns into another collector depending on the type of tag given to it.
struct InnerInitialListCollector;

impl InnerListCollector for InnerInitialListCollector {
    fn accept(self, tag: NbtTag) -> ListCollector
    where
        Self: Sized,
    {
        match tag {
            NbtTag::Compound(_) => {
                ListCollector::Heterogeneous(InnerHeterogeneousListCollector { result: vec![tag] })
            }
            NbtTag::Byte(b) => ListCollector::Byte(InnerByteListCollector { list: vec![b] }),
            NbtTag::Int(i) => ListCollector::Int(InnerIntListCollector { list: vec![i] }),
            NbtTag::Long(l) => ListCollector::Long(InnerLongListCollector { list: vec![l] }),
            _ => ListCollector::Homogeneous(InnerHomogeneousListCollector { result: vec![tag] }),
        }
    }

    fn result(self) -> NbtTag {
        NbtTag::List(vec![])
    }
}

#[cfg(test)]
mod test {
    use crate::compound::NbtCompound;
    use crate::nbt_ops::{INSTANCE, ListCollector};
    use crate::tag::NbtTag;
    use pumpkin_datafixerupper::serialization::codec::{
        BOOL_CODEC, BYTE_BUFFER_CODEC, BYTE_CODEC, ComapFlatMapCodec, DOUBLE_CODEC,
        DefaultedFieldCodec, FieldMapCodec, INT_CODEC, INT_STREAM_CODEC, LONG_CODEC,
        LONG_STREAM_CODEC, SHORT_CODEC, STRING_CODEC, UBYTE_CODEC, UINT_CODEC, UbyteCodec,
        UintCodec, comap_flat_map, field, optional_field_with_default, unbounded_list,
        unbounded_map, validate,
    };
    use pumpkin_datafixerupper::serialization::codecs::list::ListCodec;
    use pumpkin_datafixerupper::serialization::codecs::primitive::{ByteBufferCodec, StringCodec};
    use pumpkin_datafixerupper::serialization::codecs::unbounded_map::UnboundedMapCodec;
    use pumpkin_datafixerupper::serialization::codecs::validated::ValidatedCodec;
    use pumpkin_datafixerupper::serialization::coders::{Decoder, Encoder};
    use pumpkin_datafixerupper::serialization::data_result::DataResult;
    use pumpkin_datafixerupper::serialization::map_codec::for_getter;
    use pumpkin_datafixerupper::serialization::struct_codecs::{StructCodec2, StructCodec3};
    use pumpkin_datafixerupper::struct_codec;
    use std::collections::HashMap;

    /// Convenience function to easily create an [`NbtTag::Compound`].
    macro_rules! nbt_compound_tag {
        ( { $($key:literal : $tag:expr),+ $(,)* } ) => {
            {
                let mut compound = NbtCompound::new();
                $( compound.put($key, $tag); )+
                NbtTag::Compound(compound)
            }
        };
        // For empty compounds
        ( {} ) => {
            NbtTag::Compound(NbtCompound::new())
        };
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn primitives() {
        // Simple types
        assert_eq!(
            INT_CODEC
                .encode_start(&45, &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::Int(45)
        );
        assert_eq!(
            BOOL_CODEC
                .encode_start(&true, &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::Byte(1)
        );
        assert_eq!(
            BYTE_CODEC
                .encode_start(&-89, &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::Byte(-89)
        );
        assert_eq!(
            DOUBLE_CODEC
                .encode_start(&1.0, &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::Double(1.0)
        );

        assert_eq!(
            STRING_CODEC
                .encode_start(&"Sample Text".to_string(), &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::String("Sample Text".to_string())
        );

        assert_eq!(
            INT_CODEC
                .parse(NbtTag::Int(50), &INSTANCE)
                .expect("Decoding should succeed"),
            50
        );
        assert_eq!(
            SHORT_CODEC
                .parse(NbtTag::Short(-1235), &INSTANCE)
                .expect("Decoding should succeed"),
            -1235
        );
        assert_eq!(
            LONG_CODEC
                .parse(NbtTag::Long(53234), &INSTANCE)
                .expect("Decoding should succeed"),
            53234
        );

        // Packed array types
        let byte_vec = vec![
            1u8, 45u8, 100u8, 170u8, 203u8, 98u8, 245u8, 255u8, 0u8, 13u8,
        ];

        assert_eq!(
            BYTE_BUFFER_CODEC
                .encode_start(&Box::from(&byte_vec[0..3]), &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::ByteArray(Box::from(vec![1, 45, 100]))
        );
        assert_eq!(
            BYTE_BUFFER_CODEC
                .encode_start(&Box::from(&byte_vec[2..7]), &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::ByteArray(Box::from(vec![100, 170, 203, 98, 245]))
        );

        assert_eq!(
            INT_STREAM_CODEC
                .encode_start(&vec![-100, 1234, 23948], &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::IntArray(vec![-100, 1234, 23948])
        );
        assert_eq!(
            INT_STREAM_CODEC
                .encode_start(&vec![1, 120938, 1231909999], &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::IntArray(vec![1, 120938, 1231909999])
        );

        assert_eq!(
            LONG_STREAM_CODEC
                .encode_start(&vec![10_000_000_000, -99_999_999_999], &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::LongArray(vec![10_000_000_000, -99_999_999_999])
        );
        assert_eq!(
            LONG_STREAM_CODEC
                .encode_start(&vec![123_456_789_012_345, 66], &INSTANCE)
                .expect("Encoding should succeed"),
            NbtTag::LongArray(vec![123_456_789_012_345, 66])
        );

        assert_eq!(
            BYTE_BUFFER_CODEC
                .parse(NbtTag::ByteArray(Box::new([1, 4])), &INSTANCE)
                .expect("Decoding should succeed"),
            vec![1, 4].into_boxed_slice()
        );
        // All `get_...` packed array functions allow any arbitrary number array.
        assert_eq!(
            BYTE_BUFFER_CODEC
                .parse(NbtTag::IntArray(vec![120]), &INSTANCE)
                .expect("Decoding should succeed"),
            vec![120].into_boxed_slice()
        );
        assert_eq!(
            INT_STREAM_CODEC
                .parse(NbtTag::LongArray(vec![1, 2, 3]), &INSTANCE)
                .expect("Decoding should succeed"),
            vec![1, 2, 3]
        );
        assert_eq!(
            LONG_STREAM_CODEC
                .parse(NbtTag::IntArray(vec![0, 0]), &INSTANCE)
                .expect("Decoding should succeed"),
            vec![0, 0]
        );
    }

    #[test]
    fn list_collecting() {
        // Int list collector
        let mut collector = ListCollector::new_initial_collector();

        collector = collector.accept(NbtTag::Int(10));
        collector = collector.accept(NbtTag::Int(15));
        collector = collector.accept(NbtTag::Int(20));

        assert_eq!(collector.result(), NbtTag::IntArray(vec![10, 15, 20]));

        // Byte list collector
        let mut collector = ListCollector::new_initial_collector();

        collector = collector.accept(NbtTag::Byte(-1));
        collector = collector.accept(NbtTag::Byte(5));
        collector = collector.accept(NbtTag::Byte(10));

        assert_eq!(
            collector.result(),
            NbtTag::ByteArray(Box::new([-1i8 as u8, 5i8 as u8, 10i8 as u8]))
        );

        // Long list collector
        let mut collector = ListCollector::new_initial_collector();

        collector = collector.accept(NbtTag::Long(11_234_567_890));
        collector = collector.accept(NbtTag::Long(-986));
        collector = collector.accept(NbtTag::Long(1));
        collector = collector.accept(NbtTag::Long(-937_238_122));

        assert_eq!(
            collector.result(),
            NbtTag::LongArray(vec![11_234_567_890, -986, 1, -937_238_122])
        );

        // Homogeneous list collector
        let mut collector = ListCollector::new_initial_collector();

        collector = collector.accept(NbtTag::Float(-123.4));
        collector = collector.accept(NbtTag::Float(12.5));

        assert_eq!(
            collector.result(),
            NbtTag::List(vec![NbtTag::Float(-123.4), NbtTag::Float(12.5)])
        );

        // Heterogeneous list collector
        let mut collector = ListCollector::new_initial_collector();

        collector = collector.accept(NbtTag::Byte(99));

        // The list collector should change its type after accepting this string.
        collector = collector.accept(NbtTag::String("99".to_string()));
        collector = collector.accept(NbtTag::LongArray(vec![1, 2, 3]));

        let result = collector.result();

        if let NbtTag::List(list) = result {
            for tag in list {
                // We expect each element to be wrapped in a compound tag with only 1 key, an empty string with our value.
                let compound = tag.extract_compound().expect("Expected NBT compound");

                assert_eq!(compound.child_tags.len(), 1);

                assert_eq!(compound.child_tags[0].0, "");
            }
        } else {
            panic!("Expected an NBT list, got {result:?}");
        }
    }

    // Specific codec tests

    #[test]
    fn employee() {
        /// A struct to store a single employee.
        /// The `name` and `department` of the employee should not be empty.
        #[derive(Debug, PartialEq)]
        struct Employee {
            name: String,
            department: String,
            salary: u32,
        }

        pub type NonEmptyStringCodec = ValidatedCodec<StringCodec>;
        /// Convenience codec for only encoding/decoding non-empty strings.
        pub static NON_EMPTY_STRING_CODEC: NonEmptyStringCodec = validate(&STRING_CODEC, |s| {
            if s.is_empty() {
                Err("String should not be empty".to_string())
            } else {
                Ok(())
            }
        });

        pub type EmployeeCodec = StructCodec3<
            Employee,
            FieldMapCodec<NonEmptyStringCodec>,
            FieldMapCodec<NonEmptyStringCodec>,
            FieldMapCodec<UintCodec>,
        >;
        pub static EMPLOYEE_CODEC: EmployeeCodec = struct_codec!(
            for_getter(field(&NON_EMPTY_STRING_CODEC, "name"), |s: &Employee| &s
                .name),
            for_getter(
                field(&NON_EMPTY_STRING_CODEC, "department"),
                |s: &Employee| &s.department
            ),
            for_getter(field(&UINT_CODEC, "salary"), |s: &Employee| &s.salary),
            |name, department, salary| Employee {
                name,
                department,
                salary
            }
        );

        // Encoding

        assert_eq!(
            EMPLOYEE_CODEC
                .encode_start(
                    &Employee {
                        name: "John Doe".to_string(),
                        department: "Marketing".to_string(),
                        salary: 82_000
                    },
                    &INSTANCE
                )
                .expect("Encoding should succeed"),
            nbt_compound_tag!({
                "name": NbtTag::String("John Doe".to_string()),
                "department": NbtTag::String("Marketing".to_string()),
                "salary": NbtTag::Int(82_000)
            })
        );

        assert_eq!(
            EMPLOYEE_CODEC
                .encode_start(
                    &Employee {
                        name: "Linna Hall".to_string(),
                        // Department is empty.
                        department: String::new(),
                        salary: 90_000
                    },
                    &INSTANCE
                )
                .get_message()
                .expect("Encoding should fail"),
            "String should not be empty"
        );

        // Decoding

        assert_eq!(
            EMPLOYEE_CODEC
                .parse(
                    nbt_compound_tag!({
                        "name": NbtTag::String("Kelly Peak".to_string()),
                        "department": NbtTag::String("Sales".to_string()),
                        "salary": NbtTag::Int(72_000)
                    }),
                    &INSTANCE
                )
                .expect("Decoding should succeed"),
            Employee {
                name: "Kelly Peak".to_string(),
                department: "Sales".to_string(),
                salary: 72_000
            }
        );

        assert_eq!(
            EMPLOYEE_CODEC
                .parse(
                    nbt_compound_tag!({
                        "name": NbtTag::String(String::new()),
                        "department": NbtTag::String("Information Technology".to_string()),
                        "salary": NbtTag::Int(100_000)
                    }),
                    &INSTANCE
                )
                .get_message()
                .expect("Decoding should fail"),
            "String should not be empty"
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn text() {
        /// Alignments of a line of text.
        #[derive(Debug, PartialEq, Clone)]
        enum TextAlignment {
            Left,
            Center,
            Right,
        }

        impl From<&TextAlignment> for String {
            fn from(value: &TextAlignment) -> Self {
                match value {
                    TextAlignment::Left => "left",
                    TextAlignment::Center => "center",
                    TextAlignment::Right => "right",
                }
                .to_string()
            }
        }

        struct InvalidTextAlignmentError;

        impl TryFrom<String> for TextAlignment {
            type Error = InvalidTextAlignmentError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                match value.as_str() {
                    "left" => Ok(Self::Left),
                    "center" => Ok(Self::Center),
                    "right" => Ok(Self::Right),

                    _ => Err(InvalidTextAlignmentError),
                }
            }
        }

        pub type TextAlignmentCodec = ComapFlatMapCodec<TextAlignment, StringCodec>;

        // The transformer codec:
        // - always converts   `TextAlignment` -> `String`
        // - but only converts `String` -> `TextAlignment` if the string is valid.
        pub static TEXT_ALIGNMENT_CODEC: TextAlignmentCodec = comap_flat_map(
            &STRING_CODEC,
            |string| {
                string.clone().try_into().map_or_else(
                    |_| DataResult::error(format!("Invalid alignment: {string}")),
                    DataResult::success,
                )
            },
            |modifier: &TextAlignment| modifier.into(),
        );

        /// A single piece of text.
        #[derive(Debug, PartialEq, Clone)]
        struct Text {
            content: String,
            /// Optional field, defaults to `Left` alignment.
            alignment: TextAlignment,
        }

        pub type TextCodec =
            StructCodec2<Text, FieldMapCodec<StringCodec>, DefaultedFieldCodec<TextAlignmentCodec>>;
        pub static TEXT_CODEC: TextCodec = struct_codec!(
            for_getter(field(&STRING_CODEC, "content"), |t: &Text| &t.content),
            for_getter(
                optional_field_with_default(&TEXT_ALIGNMENT_CODEC, "alignment", || {
                    TextAlignment::Left
                }),
                |t| &t.alignment
            ),
            |content, alignment| Text { content, alignment }
        );

        // Encoding

        assert_eq!(
            TEXT_CODEC
                .encode_start(
                    &Text {
                        content: "Lorem ipsum".to_string(),
                        alignment: TextAlignment::Left
                    },
                    &INSTANCE
                )
                .expect("Encoding should succeed"),
            nbt_compound_tag!({
                "content": NbtTag::String("Lorem ipsum".to_string()),
                // Since "left" is the default, it will not be included.
            })
        );

        assert_eq!(
            TEXT_CODEC
                .encode_start(
                    &Text {
                        content: "An apple a day keeps the doctor away".to_string(),
                        alignment: TextAlignment::Center
                    },
                    &INSTANCE
                )
                .expect("Encoding should succeed"),
            nbt_compound_tag!({
                "content": NbtTag::String("An apple a day keeps the doctor away".to_string()),
                "alignment": NbtTag::String("center".to_string())
            })
        );

        // Decoding

        assert_eq!(
            TEXT_CODEC
                .parse(
                    nbt_compound_tag!({
                        "content": NbtTag::String("Surprise Sample Text".to_string()),
                        "alignment": NbtTag::String("right".to_string())
                    }),
                    &INSTANCE
                )
                .expect("Decoding should succeed"),
            Text {
                content: "Surprise Sample Text".to_string(),
                alignment: TextAlignment::Right
            }
        );

        assert_eq!(
            TEXT_CODEC
                .parse(
                    nbt_compound_tag!({
                        "content": NbtTag::String("Will the test succeed?".to_string()),
                        // Alignment omitted; it will default to `Left`.
                    }),
                    &INSTANCE
                )
                .expect("Decoding should succeed"),
            Text {
                content: "Will the test succeed?".to_string(),
                alignment: TextAlignment::Left
            }
        );

        assert!(
            TEXT_CODEC
                .parse(
                    nbt_compound_tag!({
                        "content": NbtTag::String("Some random document".to_string()),
                        // Unfortunately, we don't have *justify* in our possible alignments.
                        "alignment": NbtTag::String("justify".to_string())
                    }),
                    &INSTANCE
                )
                .get_message()
                .expect("Decoding should fail")
                .starts_with("Invalid alignment")
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn dog_park() {
        /// Represents an arbitrary dog.
        #[derive(Debug, PartialEq, Clone)]
        struct Dog {
            breed: String,
            age: u8,
            // Optional, defaults to an empty `Vec`.
            tricks: Vec<String>,
        }

        /// A dog park representation.
        #[derive(Debug, PartialEq)]
        struct DogPark {
            name: String,
            /// Each key of this map is the dog's name.
            dogs: HashMap<String, Dog>,
        }

        pub type DogCodec = StructCodec3<
            Dog,
            FieldMapCodec<StringCodec>,
            FieldMapCodec<UbyteCodec>,
            DefaultedFieldCodec<ListCodec<StringCodec>>,
        >;
        pub static DOG_CODEC: DogCodec = struct_codec!(
            for_getter(field(&STRING_CODEC, "breed"), |t: &Dog| &t.breed),
            for_getter(field(&UBYTE_CODEC, "age"), |t: &Dog| &t.age),
            for_getter(
                optional_field_with_default(&unbounded_list(&STRING_CODEC), "tricks", Vec::new),
                |t: &Dog| &t.tricks
            ),
            |breed, age, tricks| Dog { breed, age, tricks }
        );

        pub type DogParkCodec = StructCodec2<
            DogPark,
            FieldMapCodec<StringCodec>,
            FieldMapCodec<UnboundedMapCodec<StringCodec, DogCodec>>,
        >;
        pub static DOG_PARK_CODEC: DogParkCodec = struct_codec!(
            for_getter(field(&STRING_CODEC, "name"), |p: &DogPark| &p.name),
            for_getter(
                field(&unbounded_map(&STRING_CODEC, &DOG_CODEC), "dogs"),
                |p| &p.dogs
            ),
            |name, dogs| DogPark { name, dogs }
        );

        // Encoding

        let mut dogs = HashMap::new();
        dogs.insert(
            "Rodrick".to_string(),
            Dog {
                breed: "German Shepherd".to_string(),
                age: 4,
                tricks: vec!["spin".to_string()],
            },
        );
        dogs.insert(
            "Lucy".to_string(),
            Dog {
                breed: "Beagle".to_string(),
                age: 6,
                tricks: vec!["fetch".to_string(), "sit".to_string()],
            },
        );
        dogs.insert(
            "Dan".to_string(),
            Dog {
                breed: "Chihuahua".to_string(),
                age: 3,
                tricks: vec![],
            },
        );

        let serialized_park = DOG_PARK_CODEC
            .encode_start(
                &DogPark {
                    name: "Sunny Side Park".to_string(),
                    dogs,
                },
                &INSTANCE,
            )
            .expect("Encoding should succeed");

        let compound = serialized_park
            .extract_compound()
            .expect("Tag should be a compound");

        assert_eq!(
            compound
                .clone()
                .get_string("name")
                .expect("Compound tag should have a 'name' key"),
            "Sunny Side Park"
        );

        for (k, v) in compound
            .get_compound("dogs")
            .expect("Compound tag should have a 'dogs' key")
            .clone()
        {
            match k.as_str() {
                "Rodrick" => assert_eq!(
                    v,
                    nbt_compound_tag!({
                        "breed": NbtTag::String("German Shepherd".to_string()),
                        "age": NbtTag::Byte(4),
                        "tricks": NbtTag::List(vec![NbtTag::String("spin".to_string())])
                    })
                ),
                "Lucy" => assert_eq!(
                    v,
                    nbt_compound_tag!({
                        "breed": NbtTag::String("Beagle".to_string()),
                        "age": NbtTag::Byte(6),
                        "tricks": NbtTag::List(vec![NbtTag::String("fetch".to_string()), NbtTag::String("sit".to_string())])
                    })
                ),
                "Dan" => assert_eq!(
                    v,
                    nbt_compound_tag!({
                        "breed": NbtTag::String("Chihuahua".to_string()),
                        "age": NbtTag::Byte(3),
                        // 'tricks' will be omitted for an empty list.
                    })
                ),
                _ => panic!("Unexpected dog {k} found"),
            }
        }

        // Decoding

        let deserialized_park = DOG_PARK_CODEC
            .parse(
                nbt_compound_tag!({
                    "name": NbtTag::String("Lighthouse Meadow Park".to_string()),
                    "dogs": nbt_compound_tag!({
                        "Adam": nbt_compound_tag!({
                            "breed": NbtTag::String("Bulldog".to_string()),
                            "age": NbtTag::Byte(8),
                            "tricks": NbtTag::List(vec![NbtTag::String("catch".to_string())])
                        })
                    })
                }),
                &INSTANCE,
            )
            .expect("Decoding should succeed");

        assert_eq!(deserialized_park.name, "Lighthouse Meadow Park");
        assert_eq!(deserialized_park.dogs.len(), 1);
        assert_eq!(
            deserialized_park
                .dogs
                .get("Adam")
                .expect("No dog 'Adam' in dogs"),
            &Dog {
                breed: "Bulldog".to_string(),
                age: 8,
                tricks: vec!["catch".to_string()]
            }
        );

        assert!(
            DOG_PARK_CODEC
                .parse(
                    nbt_compound_tag!({
                        "name": NbtTag::String("Dark Park".to_string()),
                        "dogs": nbt_compound_tag!({
                            "Adam": nbt_compound_tag!({
                                "breed": NbtTag::String("Poodle".to_string()),
                                // Negative ages are not allowed.
                                "age": NbtTag::Byte(-2)
                            })
                        })
                    }),
                    &INSTANCE
                )
                .get_message()
                .expect("Decoding should fail")
                .starts_with("Could not fit i8")
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn packed_color() {
        /// A color stored using 4 bytes, one each for red, green, blue and alpha.
        #[derive(Debug, PartialEq, Clone)]
        struct PackedColor {
            r: u8,
            g: u8,
            b: u8,
            /// Optional field, defaults to `255` (full alpha).
            a: u8,
        }

        pub type PackedColorCodec = ComapFlatMapCodec<PackedColor, ByteBufferCodec>;
        pub static PACKED_COLOR_CODEC: PackedColorCodec = comap_flat_map(
            &BYTE_BUFFER_CODEC,
            |v| {
                // While decoding, our codec only accept byte buffers (arrays) with exactly 3 or 4 elements.
                if v.len() == 4 {
                    DataResult::success(PackedColor {
                        r: v[0],
                        g: v[1],
                        b: v[2],
                        a: v[3],
                    })
                } else if v.len() == 3 {
                    // Alpha defaults to 255.
                    DataResult::success(PackedColor {
                        r: v[0],
                        g: v[1],
                        b: v[2],
                        a: 255,
                    })
                } else {
                    DataResult::error(format!("Invalid byte buffer for color: {v:?}"))
                }
            },
            |c| vec![c.r, c.g, c.b, c.a].into_boxed_slice(),
        );

        // Encoding

        assert_eq!(
            PACKED_COLOR_CODEC
                .encode_start(
                    &PackedColor {
                        r: 100,
                        g: 121,
                        b: 89,
                        a: 201
                    },
                    &INSTANCE
                )
                .expect("Encoding should succeed"),
            NbtTag::ByteArray(Box::new([100, 121, 89, 201]))
        );

        assert_eq!(
            PACKED_COLOR_CODEC
                .encode_start(
                    &PackedColor {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255
                    },
                    &INSTANCE
                )
                .expect("Encoding should succeed"),
            NbtTag::ByteArray(Box::new([0, 0, 0, 255]))
        );

        // Decoding

        assert_eq!(
            PACKED_COLOR_CODEC
                .parse(NbtTag::ByteArray(Box::new([100, 121, 89, 201])), &INSTANCE)
                .expect("Decoding should succeed"),
            PackedColor {
                r: 100,
                g: 121,
                b: 89,
                a: 201
            }
        );

        assert_eq!(
            PACKED_COLOR_CODEC
                .parse(NbtTag::ByteArray(Box::new([255, 255, 0])), &INSTANCE)
                .expect("Decoding should succeed"),
            PackedColor {
                r: 255,
                g: 255,
                b: 0,
                a: 255
            }
        );

        assert!(
            PACKED_COLOR_CODEC
                .parse(NbtTag::ByteArray(Box::new([120])), &INSTANCE)
                .get_message()
                .expect("Decoding should fail")
                .starts_with("Invalid byte buffer for color")
        );

        // Even other number array types will work.
        assert_eq!(
            PACKED_COLOR_CODEC
                .parse(NbtTag::IntArray(vec![1, 2, 3, 4]), &INSTANCE)
                .expect("Decoding should succeed"),
            PackedColor {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
        );
    }
}
