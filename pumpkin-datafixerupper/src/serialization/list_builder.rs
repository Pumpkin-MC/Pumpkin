use crate::serialization::{data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait for building lists. If anything goes wrong while adding elements, the stored [`DataResult`] becomes an error.
pub trait ListBuilder {
    type Value;

    /// Builds the final list and returns the result.
    #[must_use]
    fn build(self, prefix: Self::Value) -> DataResult<Self::Value>;

    /// Adds a direct value to this [`ListBuilder`].
    #[must_use]
    fn add(self, value: Self::Value) -> Self;

    #[must_use]
    /// Adds a [`DataResult`] to this [`ListBuilder`].
    fn add_data_result(self, value: DataResult<Self::Value>) -> Self;
}

/// An implementation of [`ListBuilder`].
pub struct ListBuilderImpl<T, O: 'static> {
    elements: DataResult<Vec<T>>,
    ops: &'static O,
}

pub fn new_list_builder_impl<T>(
    ops: &'static impl DynamicOps<Value = T>,
) -> impl ListBuilder<Value = T> {
    ListBuilderImpl {
        elements: DataResult::success(vec![]),
        ops,
    }
}

impl<T, O> ListBuilder for ListBuilderImpl<T, O>
where
    O: DynamicOps<Value = T>,
{
    type Value = T;

    fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
        self.elements
            .flat_map(|e| self.ops.merge_values_into_list(prefix, e))
    }

    fn add(mut self, value: Self::Value) -> Self {
        self.elements = self.elements.map(|mut e: Vec<T>| {
            e.push(value);
            e
        });
        self
    }

    fn add_data_result(mut self, value: DataResult<T>) -> Self {
        self.elements = self.elements.apply_2_and_make_stable(
            |mut e, v| {
                e.push(v);
                e
            },
            value,
        );
        self
    }
}
