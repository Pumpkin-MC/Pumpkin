pub trait PropertyDelegate: Sync + Send {
    fn get_property(&self, _index: i32) -> i32;
    fn set_property(&self, _index: i32, _value: i32);
    fn get_properties_size(&self) -> i32;
}

/// Trait for extracting smelting experience from cooking block entities.
pub trait ExperienceContainer: Send + Sync {
    /// Extract and reset accumulated experience, returning the total as an integer
    fn extract_experience(&self) -> i32;
}
