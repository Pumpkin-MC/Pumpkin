use std::str::FromStr;
use std::sync::OnceLock;

use crate::locale::Locale;

/// Global logging locale, set by the pumpkin server crate during startup.
static SERVER_LOGGING_LOCALE: OnceLock<Locale> = OnceLock::new();
/// Global command locale, set by the pumpkin server crate during startup.
static SERVER_COMMAND_LOCALE: OnceLock<Locale> = OnceLock::new();

/// Returns the server logging locale, falling back to [`Locale::EnUs`].
#[must_use]
pub fn server_locale() -> Locale {
    *SERVER_LOGGING_LOCALE.get().unwrap_or(&Locale::EnUs)
}

/// Returns the server command locale, falling back to [`server_locale`].
#[must_use]
pub fn server_command_locale() -> Locale {
    SERVER_COMMAND_LOCALE
        .get()
        .copied()
        .unwrap_or_else(server_locale)
}

/// Sets the server logging locale. Called from the pumpkin server crate during
/// initialization.
pub fn set_server_locale(locale: Locale) {
    let _ = SERVER_LOGGING_LOCALE.set(locale);
}

/// Sets the server command locale. Called from the pumpkin server crate during
/// initialization.
pub fn set_server_command_locale(locale: Locale) {
    let _ = SERVER_COMMAND_LOCALE.set(locale);
}

/// Detects the system locale using platform-specific APIs.
///
/// # Platform behaviour
/// * **Linux / macOS / FreeBSD / Android** — reads `LANG`, `LC_ALL`, `LC_MESSAGES`
///   environment variables in order. Extracts the language portion
///   (e.g. `"zh_CN"` from `"zh_CN.UTF-8"`).
/// * **Windows** — calls `GetUserDefaultLocaleName` to retrieve the
///   user's preferred locale (returns BCP‑47 tags like `"zh-CN"`).
///
/// Falls back to [`Locale::EnUs`] if detection fails on any platform.
///
/// # Returns
/// The detected system [`Locale`].
#[must_use]
pub fn detect_system_locale() -> Locale {
    detect_platform_locale()
}

#[cfg(unix)]
fn detect_platform_locale() -> Locale {
    let raw = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    if raw.is_empty() || raw == "C" || raw == "POSIX" {
        return Locale::EnUs;
    }

    // Extract language part before the first '.'
    // e.g. "zh_CN.UTF-8" -> "zh_CN"
    let lang = raw.split('.').next().unwrap_or("en_us");
    Locale::from_str(lang).unwrap_or(Locale::EnUs)
}

#[cfg(windows)]
fn detect_platform_locale() -> Locale {
    // LOCALE_NAME_MAX_LENGTH is 85 on Windows
    const BUF_SIZE: usize = 85;

    unsafe extern "system" {
        fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
    }

    let mut buffer: [u16; BUF_SIZE] = [0; BUF_SIZE];
    let result = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), BUF_SIZE as i32) };

    if result <= 0 {
        return Locale::EnUs;
    }

    let len = result as usize;
    let raw = String::from_utf16_lossy(&buffer[..len])
        .trim_end_matches('\0')
        .to_owned();

    if raw.is_empty() {
        return Locale::EnUs;
    }

    Locale::from_str(&raw).unwrap_or(Locale::EnUs)
}

#[cfg(not(any(unix, windows)))]
fn detect_platform_locale() -> Locale {
    // Unknown platform – no locale detection available.
    Locale::EnUs
}

/// Resolves the server-side locale based on the configuration value.
///
/// # Arguments
/// * `config_value` — The locale configuration string, either `"auto"` or a locale code.
///
/// # Returns
/// The resolved [`Locale`]. If `"auto"`, calls [`detect_system_locale`].
/// Otherwise parses the config value as a locale, falling back to [`Locale::EnUs`].
#[must_use]
pub fn resolve_server_locale(config_value: &str) -> Locale {
    if config_value.eq_ignore_ascii_case("auto") {
        return detect_system_locale();
    }
    crate::parse_locale_value(config_value)
}
