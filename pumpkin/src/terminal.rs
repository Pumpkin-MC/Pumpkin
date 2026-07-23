//! Restores the terminal to its original state when the process exits.
//!
//! The console thread reads input through `rustyline`, which switches the
//! terminal into raw mode while a `readline` call is active. `rustyline`
//! only restores the previous terminal state when that call returns, so if
//! the process exits while the console thread is still blocked inside
//! `readline` (for example after a panic in another thread), the terminal
//! is left in raw mode: typed characters stop echoing until the user runs
//! `reset`. See <https://github.com/Pumpkin-MC/Pumpkin/issues/2441>.
//!
//! To avoid this, the original terminal attributes are saved before the
//! line editor is created, and an `atexit` handler restores them on any
//! exit path, including `std::process::exit` from the panic hook.

#[cfg(unix)]
mod imp {
    use std::mem::MaybeUninit;
    use std::sync::OnceLock;

    static ORIGINAL_TERMIOS: OnceLock<Option<libc::termios>> = OnceLock::new();

    extern "C" fn restore_terminal() {
        if let Some(&Some(original)) = ORIGINAL_TERMIOS.get() {
            // SAFETY: `original` was produced by a successful `tcgetattr`
            // call on stdin, so it is a valid `termios` for this stream.
            unsafe {
                let _ = libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw const original);
            }
        }
    }

    /// Saves the current terminal attributes of stdin and registers an
    /// `atexit` handler that restores them when the process exits.
    ///
    /// Only the first call has an effect; subsequent calls are no-ops.
    /// Does nothing if stdin is not a terminal.
    pub fn save_original_state() {
        ORIGINAL_TERMIOS.get_or_init(|| {
            // SAFETY: `isatty` and `tcgetattr` are called with a valid file
            // descriptor, and the `termios` value is only read after
            // `tcgetattr` reported success by returning 0.
            let termios = unsafe {
                if libc::isatty(libc::STDIN_FILENO) == 0 {
                    return None;
                }
                let mut termios = MaybeUninit::<libc::termios>::uninit();
                if libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr()) != 0 {
                    return None;
                }
                termios.assume_init()
            };

            // SAFETY: `restore_terminal` only performs async-signal-safe
            // work and is safe to run during process shutdown.
            unsafe {
                let _ = libc::atexit(restore_terminal);
            }

            Some(termios)
        });
    }
}

#[cfg(not(unix))]
mod imp {
    /// Saving and restoring terminal attributes is only needed on Unix,
    /// where leaked raw mode outlives the process; this is a no-op here.
    pub const fn save_original_state() {}
}

pub use imp::save_original_state;
