#[cfg(target_os = "linux")]
mod imp {
    use crate::bootstrap::BootstrapProvider;

    unsafe extern "C" {
        static __start_pumpkin_bootstrap: u8;
        static __stop_pumpkin_bootstrap: u8;
    }

    #[must_use]
    pub fn builtin_providers() -> &'static [BootstrapProvider] {
        unsafe {
            // SAFETY:
            // `__start_pumpkin_bootstrap` and `__stop_pumpkin_bootstrap`
            // are linker-provided boundary symbols for the
            // `pumpkin_bootstrap` section.
            //
            // The registration mechanism guarantees that the section
            // contains contiguous, properly aligned, initialized
            // `BootstrapProvider` values.
            //
            // The section remains mapped for the lifetime of the process.
            let start = (&raw const __start_pumpkin_bootstrap).cast::<BootstrapProvider>();
            let end = &raw const __stop_pumpkin_bootstrap;

            let bytes = end.byte_offset_from(start.cast::<u8>()) as usize;

            debug_assert_eq!(bytes % size_of::<BootstrapProvider>(), 0);

            std::slice::from_raw_parts(start, bytes / size_of::<BootstrapProvider>())
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use crate::bootstrap::BootstrapProvider;

    #[repr(C, align(8))]
    struct Sentinel([u8; 16]);

    #[used]
    #[unsafe(link_section = ".pumpkin_bootstrap$a")]
    static START: Sentinel = Sentinel([0; 16]);

    #[used]
    #[unsafe(link_section = ".pumpkin_bootstrap$z")]
    static END: Sentinel = Sentinel([0; 16]);

    #[must_use]
    pub fn builtin_providers() -> &'static [BootstrapProvider] {
        unsafe {
            // SAFETY:
            // COFF orders contributions to `.pumpkin_bootstrap$*`
            // lexicographically by their `$` suffix. `START` is placed in
            // `$a`, providers are placed between it and `$z`, and `END` is
            // placed in `$z`.
            //
            // Therefore the memory immediately following `START` up to `END`
            // contains only contiguous, initialized `BootstrapProvider`
            // values.
            //
            // `START` has sufficient alignment for the provider section and
            // the provider registration mechanism must ensure that no padding
            // or unrelated objects occur between provider entries.
            //
            // These statics and the section containing them are part of the
            // loaded executable and therefore remain valid for the entire
            // duration of the program, allowing the resulting slice to have
            // a `'static` lifetime.
            let start = (&raw const START).add(1).cast::<BootstrapProvider>();

            let end = (&raw const END).cast::<BootstrapProvider>();

            let bytes = end.byte_offset_from(start) as usize;

            debug_assert_eq!(bytes % size_of::<BootstrapProvider>(), 0);

            std::slice::from_raw_parts(start, bytes / size_of::<BootstrapProvider>())
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use crate::bootstrap::BootstrapProvider;
    use std::{
        ffi::{c_char, c_ulong},
        slice,
    };

    #[repr(C)]
    struct MachHeader64 {
        _private: [u8; 0],
    }

    unsafe extern "C" {
        static _mh_execute_header: MachHeader64;

        fn getsectiondata(
            mhp: *const MachHeader64,
            segname: *const c_char,
            sectname: *const c_char,
            size: *mut c_ulong,
        ) -> *mut u8;
    }

    #[must_use]
    pub fn builtin_providers() -> &'static [BootstrapProvider] {
        unsafe {
            // SAFETY:
            // `_mh_execute_header` is provided by the Mach-O loader and
            // identifies the currently executing image.
            //
            // `getsectiondata` returns the address and size of the requested
            // Mach-O section. The bootstrap registration mechanism guarantees
            // that `__DATA,__pumpkin_boot` contains only contiguous,
            // initialized `BootstrapProvider` values and that the section has
            // the alignment required by `BootstrapProvider`.
            //
            // The section belongs to the loaded executable image and remains
            // mapped for the lifetime of the process, so a slice constructed
            // from it may have a `'static` lifetime.
            //
            // A null return value means that the section does not exist, in
            // which case there are no built-in providers.
            let mut size = 0;

            let ptr = getsectiondata(
                &_mh_execute_header,
                c"__DATA".as_ptr(),
                c"__pumpkin_boot".as_ptr(),
                &mut size,
            );

            if ptr.is_null() {
                return &[];
            }

            let size = size as usize;

            assert_eq!(size % std::mem::size_of::<BootstrapProvider>(), 0,);

            slice::from_raw_parts(
                ptr.cast::<BootstrapProvider>(),
                size / std::mem::size_of::<BootstrapProvider>(),
            )
        }
    }
}

pub use imp::builtin_providers;
