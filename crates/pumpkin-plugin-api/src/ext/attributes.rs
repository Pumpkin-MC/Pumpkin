use crate::wit::pumpkin::plugin::attributes::{AttributeModifier, ModifierOperation};

impl AttributeModifier {
    /// Creates a new attribute modifier.
    #[must_use]
    pub fn new(id: impl Into<String>, amount: f64, operation: ModifierOperation) -> Self {
        Self {
            id: id.into(),
            amount,
            operation,
        }
    }

    /// Creates an additive attribute modifier.
    #[must_use]
    pub fn add(id: impl Into<String>, amount: f64) -> Self {
        Self::new(id, amount, ModifierOperation::Add)
    }

    /// Creates a multiplicative base attribute modifier (`base * (1 + amount)`).
    #[must_use]
    pub fn multiply_base(id: impl Into<String>, amount: f64) -> Self {
        Self::new(id, amount, ModifierOperation::MultiplyBase)
    }

    /// Creates a multiplicative total attribute modifier (`total * (1 + amount)`).
    #[must_use]
    pub fn multiply_total(id: impl Into<String>, amount: f64) -> Self {
        Self::new(id, amount, ModifierOperation::MultiplyTotal)
    }
}
