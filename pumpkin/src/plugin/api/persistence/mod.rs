pub mod persistent_data_container;

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Dummy;

    fn test_key() -> NamespacedKey {
        NamespacedKey::new("example", "test_key")
    }

    #[test]
    fn test_save_and_get_data() {
        let dummy = Dummy;
        let holder = PersistentDataHolder::new(&dummy);

        let key = test_key();
        holder.save_data(key.clone(), PersistentValue::String("hello".to_string()));

        let retrieved = holder.get_data(&key);
        assert_eq!(
            retrieved,
            Some(PersistentValue::String("hello".to_string()))
        );
    }

    #[test]
    fn test_clear_data() {
        let dummy = Dummy;
        let holder = PersistentDataHolder::new(&dummy);

        let key = test_key();
        holder.save_data(key.clone(), PersistentValue::Bool(true));
        holder.clear();

        assert!(holder.get_data(&key).is_none());
    }

    #[test]
    fn test_destroy_container() {
        let dummy = Dummy;
        let mut holder = PersistentDataHolder::new(&dummy);

        let key = test_key();
        holder.save_data(key.clone(), PersistentValue::Bool(true));

        holder.destroy_container();

        assert!(holder.get_data(&key).is_none());
        assert!(holder.container.is_none());
    }
}
