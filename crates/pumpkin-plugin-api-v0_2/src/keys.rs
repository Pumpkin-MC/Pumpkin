pub fn qualify(key: &str) -> String {
    if key.contains(':') {
        key.to_owned()
    } else {
        format!("minecraft:{key}")
    }
}
