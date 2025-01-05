use std::{collections::HashMap, sync::LazyLock};

pub static LANGUAGE_TOKENS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/lang/en_us.json"))
        .expect("Could not parse assets/lang/e_us.json")
});

pub fn get_translation(key: String) -> String {
    LANGUAGE_TOKENS.get(&key).cloned().unwrap_or(key)
}

// Format using %s as placeholder for arguments
pub fn format_with_vec<F, A>(format_str: F, args: &[A]) -> String
where
    F: AsRef<str>,
    A: AsRef<str>,
{
    let parts: Vec<&str> = format_str.as_ref().split("%s").collect();
    let mut result = String::new();

    let mut args_iter = args.iter();

    for (i, part) in parts.iter().enumerate() {
        result.push_str(part);

        if i < parts.len() - 1 {
            if let Some(arg) = args_iter.next() {
                result.push_str(arg.as_ref());
            } else {
                // when no args arte left keep the %s,
                // this way we know somethings wrong
                result.push_str("%s");
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::format_with_vec;

    #[test]
    fn format_str_single() {
        let formatted = format_with_vec("Hello %s", &["World"]);
        assert_eq!(formatted, "Hello World")
    }

    #[test]
    fn format_str_multiple() {
        let formatted = format_with_vec("%s %s", &["Hello", "World"]);
        assert_eq!(formatted, "Hello World")
    }

    #[test]
    fn format_too_many() {
        let formatted = format_with_vec("%s %s %s", &["Hello", "World"]);
        assert_eq!(formatted, "Hello World %s")
    }
}
