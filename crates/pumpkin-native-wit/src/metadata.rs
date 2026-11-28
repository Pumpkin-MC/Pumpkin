//! Load-time WIT world discovery and structural ABI verification for native plugins.

use std::collections::HashSet;
use std::path::Path;

use object::{NameOrOrdinal, Object, ObjectSymbol};
use wit_component::DecodedWasm;
use wit_parser::{
    Function, FunctionKind, Handle, InterfaceId, PackageId, Resolve, Type, TypeDefKind, TypeId,
    WorldId, WorldItem, WorldKey,
};

const WORLD_SYMBOL_PREFIX: &str = "__wit_bindgen_world_";
const PUMPKIN_WORLD_PREFIX: &str = "__wit_bindgen_world_pumpkinX3ApluginX40";

/// Returns `true` if `path` is a native shared library exporting any `wit-bindgen` world metadata
/// symbol (`__wit_bindgen_world_*`), without executing library constructors via `dlopen`.
#[must_use]
pub fn has_native_wit_world_symbol(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    has_native_wit_world_symbol_in_bytes(&bytes)
}

/// Inspects a native shared library's exported symbols to detect the `pumpkin:plugin@<ver>/plugin`
/// WIT package version without `dlopen`.
#[must_use]
pub fn detect_native_wit_world_version(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    detect_native_wit_world_version_in_bytes(&bytes)
}

fn normalized_symbol_name(name: &str) -> &str {
    // Mach-O (and 32-bit COFF) prepend a leading underscore to C symbols.
    name.strip_prefix('_').map_or(name, |stripped| {
        if stripped.starts_with(WORLD_SYMBOL_PREFIX) {
            stripped
        } else {
            name
        }
    })
}

fn export_name_str<'a>(export: &'a object::Export<'a>) -> Option<&'a str> {
    let NameOrOrdinal::Name(bytes) = export.name() else {
        return None;
    };
    std::str::from_utf8(bytes).ok()
}

fn has_native_wit_world_symbol_in_bytes(bytes: &[u8]) -> bool {
    let Ok(file) = object::File::parse(bytes) else {
        return false;
    };
    file.exports().is_ok_and(|exports| {
        exports.flatten().any(|export| {
            export_name_str(&export)
                .is_some_and(|name| normalized_symbol_name(name).starts_with(WORLD_SYMBOL_PREFIX))
        })
    }) || file.dynamic_symbols().any(|sym| {
        sym.is_definition()
            && sym
                .name()
                .is_ok_and(|name| normalized_symbol_name(name).starts_with(WORLD_SYMBOL_PREFIX))
    })
}

fn detect_native_wit_world_version_in_bytes(bytes: &[u8]) -> Option<String> {
    let file = object::File::parse(bytes).ok()?;

    if let Ok(exports) = file.exports() {
        for export in exports.flatten() {
            if let Some(name) = export_name_str(&export)
                && let Some(version) = parse_pumpkin_world_version(normalized_symbol_name(name))
            {
                return Some(version);
            }
        }
    }

    for sym in file.dynamic_symbols() {
        if sym.is_definition()
            && let Ok(name) = sym.name()
            && let Some(version) = parse_pumpkin_world_version(normalized_symbol_name(name))
        {
            return Some(version);
        }
    }

    None
}

fn parse_pumpkin_world_version(symbol: &str) -> Option<String> {
    let rest = symbol.strip_prefix(PUMPKIN_WORLD_PREFIX)?;
    let (encoded_version, world) = rest.split_once("X2F")?;
    (world == "plugin")
        .then(|| decode_external_component(encoded_version))
        .flatten()
}

fn decode_external_component(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut out = String::with_capacity(encoded.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'X' && i + 2 < bytes.len() {
            let hi = hex_value(bytes[i + 1])?;
            let lo = hex_value(bytes[i + 2])?;
            out.push(char::from((hi << 4) | lo));
            i += 3;
        } else {
            out.push(char::from(bytes[i]));
            i += 1;
        }
    }
    Some(out)
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// Validates that the guest library's `wit_component::metadata::encode` bytes are ABI-compatible
/// with the host's encoded WIT world.
///
/// Every export the host expects must exist in the guest world with a matching signature, and
/// every import the guest declares must exist in the host world with a matching signature.
pub fn validate_world_metadata(guest_bytes: &[u8], host_bytes: &[u8]) -> wasmtime::Result<()> {
    let (host_resolve, host_pkg, host_world) = decode_world(host_bytes, "host")?;
    let (guest_resolve, guest_pkg, guest_world) = decode_world(guest_bytes, "guest")?;

    let host_pkg_name = &host_resolve.packages[host_pkg].name;
    let guest_pkg_name = &guest_resolve.packages[guest_pkg].name;
    if host_pkg_name != guest_pkg_name {
        return Err(wasmtime::Error::msg(format!(
            "native WIT package mismatch: host expects `{host_pkg_name}`, plugin exports `{guest_pkg_name}`"
        )));
    }

    let hw = &host_resolve.worlds[host_world];
    let gw = &guest_resolve.worlds[guest_world];
    if hw.name != gw.name {
        return Err(wasmtime::Error::msg(format!(
            "native WIT world mismatch: host expects `{}`, plugin exports `{}`",
            hw.name, gw.name
        )));
    }

    // Every export expected by the host must be provided by the guest with a compatible signature.
    for (host_key, host_item) in &hw.exports {
        let key_name = host_resolve.name_world_key(host_key);
        let Some(guest_item) = find_world_item(&guest_resolve, &gw.exports, &key_name) else {
            return Err(wasmtime::Error::msg(format!(
                "native WIT plugin is missing required export `{key_name}`"
            )));
        };
        check_world_item_compatible(
            &host_resolve,
            host_item,
            &guest_resolve,
            guest_item,
            &key_name,
            "export",
            false,
        )?;
    }

    // Every import declared by the guest must be supplied by the host with a compatible signature.
    for (guest_key, guest_item) in &gw.imports {
        let key_name = guest_resolve.name_world_key(guest_key);
        let Some(host_item) = find_world_item(&host_resolve, &hw.imports, &key_name) else {
            return Err(wasmtime::Error::msg(format!(
                "native WIT plugin imports `{key_name}`, which the host does not provide"
            )));
        };
        check_world_item_compatible(
            &host_resolve,
            host_item,
            &guest_resolve,
            guest_item,
            &key_name,
            "import",
            true,
        )?;
    }

    Ok(())
}

fn decode_world(bytes: &[u8], side: &str) -> wasmtime::Result<(Resolve, PackageId, WorldId)> {
    let decoded = wit_component::decode(bytes).map_err(|err| {
        wasmtime::Error::msg(format!("failed to decode {side} WIT world metadata: {err}"))
    })?;
    let DecodedWasm::WitPackage(resolve, pkg_id) = decoded else {
        return Err(wasmtime::Error::msg(format!(
            "{side} WIT metadata did not decode as a WIT package"
        )));
    };
    let world_id = resolve.select_world(&[pkg_id], None).map_err(|err| {
        wasmtime::Error::msg(format!(
            "failed to select world from {side} WIT metadata: {err}"
        ))
    })?;
    Ok((resolve, pkg_id, world_id))
}

fn find_world_item<'a>(
    resolve: &Resolve,
    items: impl IntoIterator<Item = (&'a WorldKey, &'a WorldItem)>,
    name: &str,
) -> Option<&'a WorldItem> {
    items
        .into_iter()
        .find(|(k, _)| resolve.name_world_key(k) == name)
        .map(|(_, item)| item)
}

fn check_world_item_compatible(
    host_resolve: &Resolve,
    host_item: &WorldItem,
    guest_resolve: &Resolve,
    guest_item: &WorldItem,
    key_name: &str,
    direction: &str,
    guest_is_subset: bool,
) -> wasmtime::Result<()> {
    match (host_item, guest_item) {
        (WorldItem::Function(hf), WorldItem::Function(gf)) => {
            check_function_compatible(host_resolve, hf, guest_resolve, gf, key_name, direction)
        }
        (WorldItem::Interface { id: host_id, .. }, WorldItem::Interface { id: guest_id, .. }) => {
            check_interface_compatible(
                host_resolve,
                *host_id,
                guest_resolve,
                *guest_id,
                key_name,
                direction,
                guest_is_subset,
            )
        }
        (WorldItem::Type { id: host_ty, .. }, WorldItem::Type { id: guest_ty, .. }) => {
            let mut visited = HashSet::new();
            if types_compatible(
                host_resolve,
                Type::Id(*host_ty),
                guest_resolve,
                Type::Id(*guest_ty),
                &mut visited,
            ) {
                Ok(())
            } else {
                Err(wasmtime::Error::msg(format!(
                    "native WIT {direction} type `{key_name}` differs between host and plugin"
                )))
            }
        }
        _ => Err(wasmtime::Error::msg(format!(
            "native WIT {direction} `{key_name}` has incompatible item kind between host and plugin"
        ))),
    }
}

fn check_interface_compatible(
    host_resolve: &Resolve,
    host_id: InterfaceId,
    guest_resolve: &Resolve,
    guest_id: InterfaceId,
    iface_name: &str,
    direction: &str,
    guest_is_subset: bool,
) -> wasmtime::Result<()> {
    let host_iface = &host_resolve.interfaces[host_id];
    let guest_iface = &guest_resolve.interfaces[guest_id];

    if guest_is_subset {
        // For imports, every function the guest imports must exist in the host interface.
        for (func_name, gf) in &guest_iface.functions {
            let Some(hf) = host_iface.functions.get(func_name) else {
                return Err(wasmtime::Error::msg(format!(
                    "native WIT plugin imports `{iface_name}#{func_name}`, which the host does not provide"
                )));
            };
            check_function_compatible(
                host_resolve,
                hf,
                guest_resolve,
                gf,
                &format!("{iface_name}#{func_name}"),
                direction,
            )?;
        }
    } else {
        // For exports, every function the host expects must exist in the guest interface.
        for (func_name, hf) in &host_iface.functions {
            let Some(gf) = guest_iface.functions.get(func_name) else {
                return Err(wasmtime::Error::msg(format!(
                    "native WIT plugin is missing required export `{iface_name}#{func_name}`"
                )));
            };
            check_function_compatible(
                host_resolve,
                hf,
                guest_resolve,
                gf,
                &format!("{iface_name}#{func_name}"),
                direction,
            )?;
        }
    }

    Ok(())
}

fn check_function_compatible(
    host_resolve: &Resolve,
    hf: &Function,
    guest_resolve: &Resolve,
    gf: &Function,
    full_name: &str,
    direction: &str,
) -> wasmtime::Result<()> {
    if !function_kinds_compatible(&hf.kind, &gf.kind) {
        return Err(wasmtime::Error::msg(format!(
            "native WIT {direction} `{full_name}` kind mismatch (sync/async/method)"
        )));
    }
    if hf.params.len() != gf.params.len() {
        return Err(wasmtime::Error::msg(format!(
            "native WIT {direction} `{full_name}` parameter count mismatch: host has {}, plugin has {}",
            hf.params.len(),
            gf.params.len()
        )));
    }
    let mut visited = HashSet::new();
    for (idx, (hp, gp)) in hf.params.iter().zip(gf.params.iter()).enumerate() {
        if !types_compatible(host_resolve, hp.ty, guest_resolve, gp.ty, &mut visited) {
            return Err(wasmtime::Error::msg(format!(
                "native WIT {direction} `{full_name}` parameter #{idx} (`{}`) type mismatch",
                hp.name
            )));
        }
    }
    match (hf.result, gf.result) {
        (None, None) => Ok(()),
        (Some(hr), Some(gr)) => {
            if types_compatible(host_resolve, hr, guest_resolve, gr, &mut visited) {
                Ok(())
            } else {
                Err(wasmtime::Error::msg(format!(
                    "native WIT {direction} `{full_name}` return type mismatch"
                )))
            }
        }
        _ => Err(wasmtime::Error::msg(format!(
            "native WIT {direction} `{full_name}` return presence mismatch"
        ))),
    }
}

const fn function_kinds_compatible(a: &FunctionKind, b: &FunctionKind) -> bool {
    matches!(
        (a, b),
        (FunctionKind::Freestanding, FunctionKind::Freestanding)
            | (
                FunctionKind::AsyncFreestanding,
                FunctionKind::AsyncFreestanding
            )
            | (FunctionKind::Method(_), FunctionKind::Method(_))
            | (FunctionKind::AsyncMethod(_), FunctionKind::AsyncMethod(_))
            | (FunctionKind::Static(_), FunctionKind::Static(_))
            | (FunctionKind::AsyncStatic(_), FunctionKind::AsyncStatic(_))
            | (FunctionKind::Constructor(_), FunctionKind::Constructor(_))
    )
}

fn dealias(resolve: &Resolve, mut ty: Type) -> Type {
    while let Type::Id(id) = ty {
        if let TypeDefKind::Type(inner) = resolve.types[id].kind {
            ty = inner;
        } else {
            break;
        }
    }
    ty
}

fn canonical_resource_id(resolve: &Resolve, mut id: TypeId) -> TypeId {
    while let TypeDefKind::Type(Type::Id(inner)) = resolve.types[id].kind {
        id = inner;
    }
    id
}

fn resource_identity(resolve: &Resolve, id: TypeId) -> (Option<String>, Option<&str>) {
    let id = canonical_resource_id(resolve, id);
    let ty = &resolve.types[id];
    let owner = match ty.owner {
        wit_parser::TypeOwner::Interface(iface_id) => resolve.id_of(iface_id),
        wit_parser::TypeOwner::World(world_id) => Some(resolve.worlds[world_id].name.clone()),
        wit_parser::TypeOwner::None => None,
    };
    (owner, ty.name.as_deref())
}

fn types_compatible(
    hr: &Resolve,
    ht: Type,
    gr: &Resolve,
    gt: Type,
    visited: &mut HashSet<(TypeId, TypeId)>,
) -> bool {
    let ht = dealias(hr, ht);
    let gt = dealias(gr, gt);
    match (ht, gt) {
        (Type::Bool, Type::Bool)
        | (Type::U8, Type::U8)
        | (Type::U16, Type::U16)
        | (Type::U32, Type::U32)
        | (Type::U64, Type::U64)
        | (Type::S8, Type::S8)
        | (Type::S16, Type::S16)
        | (Type::S32, Type::S32)
        | (Type::S64, Type::S64)
        | (Type::F32, Type::F32)
        | (Type::F64, Type::F64)
        | (Type::Char, Type::Char)
        | (Type::String, Type::String)
        | (Type::ErrorContext, Type::ErrorContext) => true,
        (Type::Id(hid), Type::Id(gid)) => {
            if !visited.insert((hid, gid)) {
                return true;
            }
            typedefs_compatible(hr, hid, gr, gid, visited)
        }
        _ => false,
    }
}

fn typedefs_compatible(
    hr: &Resolve,
    hid: TypeId,
    gr: &Resolve,
    gid: TypeId,
    visited: &mut HashSet<(TypeId, TypeId)>,
) -> bool {
    match (&hr.types[hid].kind, &gr.types[gid].kind) {
        (TypeDefKind::Record(a), TypeDefKind::Record(b)) => {
            a.fields.len() == b.fields.len()
                && a.fields.iter().zip(&b.fields).all(|(fa, fb)| {
                    fa.name == fb.name && types_compatible(hr, fa.ty, gr, fb.ty, visited)
                })
        }
        (TypeDefKind::Variant(a), TypeDefKind::Variant(b)) => {
            a.cases.len() == b.cases.len()
                && a.cases.iter().zip(&b.cases).all(|(ca, cb)| {
                    ca.name == cb.name
                        && match (ca.ty, cb.ty) {
                            (None, None) => true,
                            (Some(ta), Some(tb)) => types_compatible(hr, ta, gr, tb, visited),
                            _ => false,
                        }
                })
        }
        (TypeDefKind::Enum(a), TypeDefKind::Enum(b)) => {
            a.cases.len() == b.cases.len()
                && a.cases
                    .iter()
                    .zip(&b.cases)
                    .all(|(ca, cb)| ca.name == cb.name)
        }
        (TypeDefKind::Flags(a), TypeDefKind::Flags(b)) => {
            a.flags.len() == b.flags.len()
                && a.flags
                    .iter()
                    .zip(&b.flags)
                    .all(|(fa, fb)| fa.name == fb.name)
        }
        (TypeDefKind::Tuple(a), TypeDefKind::Tuple(b)) => {
            a.types.len() == b.types.len()
                && a.types
                    .iter()
                    .zip(&b.types)
                    .all(|(ta, tb)| types_compatible(hr, *ta, gr, *tb, visited))
        }
        (TypeDefKind::Option(a), TypeDefKind::Option(b))
        | (TypeDefKind::List(a), TypeDefKind::List(b)) => types_compatible(hr, *a, gr, *b, visited),
        (TypeDefKind::Result(a), TypeDefKind::Result(b)) => {
            let ok = match (a.ok, b.ok) {
                (None, None) => true,
                (Some(ta), Some(tb)) => types_compatible(hr, ta, gr, tb, visited),
                _ => false,
            };
            let err = match (a.err, b.err) {
                (None, None) => true,
                (Some(ta), Some(tb)) => types_compatible(hr, ta, gr, tb, visited),
                _ => false,
            };
            ok && err
        }
        (TypeDefKind::Handle(ha), TypeDefKind::Handle(hb)) => match (ha, hb) {
            (Handle::Own(ra), Handle::Own(rb)) | (Handle::Borrow(ra), Handle::Borrow(rb)) => {
                resource_identity(hr, *ra) == resource_identity(gr, *rb)
            }
            _ => false,
        },
        (TypeDefKind::Resource, TypeDefKind::Resource) => {
            resource_identity(hr, hid) == resource_identity(gr, gid)
        }
        (TypeDefKind::Map(ka, va), TypeDefKind::Map(kb, vb)) => {
            types_compatible(hr, *ka, gr, *kb, visited)
                && types_compatible(hr, *va, gr, *vb, visited)
        }
        (TypeDefKind::Future(a), TypeDefKind::Future(b))
        | (TypeDefKind::Stream(a), TypeDefKind::Stream(b)) => match (a, b) {
            (None, None) => true,
            (Some(ta), Some(tb)) => types_compatible(hr, *ta, gr, *tb, visited),
            _ => false,
        },
        (TypeDefKind::FixedLengthList(ta, la), TypeDefKind::FixedLengthList(tb, lb)) => {
            la == lb && types_compatible(hr, *ta, gr, *tb, visited)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_pumpkin_world_version, validate_world_metadata};
    use wit_component::StringEncoding;
    use wit_parser::{Resolve, UnresolvedPackageGroup};

    fn encode_inline_wit(wit: &str, world_name: &str) -> wasmtime::Result<Vec<u8>> {
        let mut resolve = Resolve::new();
        let pkg = UnresolvedPackageGroup::parse("test.wit", wit)
            .map_err(|(_, e)| wasmtime::Error::msg(e.to_string()))?;
        let pkg_id = resolve
            .push_group(pkg)
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        let world_id = resolve
            .select_world(&[pkg_id], Some(world_name))
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        wit_component::metadata::encode(&resolve, world_id, StringEncoding::UTF8, None, false)
            .map_err(|e| wasmtime::Error::msg(e.to_string()))
    }

    #[test]
    fn parses_encoded_pumpkin_world_version_symbol() {
        assert_eq!(
            parse_pumpkin_world_version(
                "__wit_bindgen_world_pumpkinX3ApluginX400X2E1X2E0X2Fplugin"
            )
            .as_deref(),
            Some("0.1.0")
        );
        assert_eq!(
            parse_pumpkin_world_version(
                "__wit_bindgen_world_pumpkinX3ApluginX400X2E2X2E0X2Fplugin"
            )
            .as_deref(),
            Some("0.2.0")
        );
        assert_eq!(parse_pumpkin_world_version("PUMPKIN_API_VERSION"), None);
    }

    #[test]
    fn matching_worlds_validate() -> wasmtime::Result<()> {
        let wit = r"
            package test:demo@0.1.0;
            interface host {
                record item { name: string, count: u32 }
                echo: func(i: item) -> string;
            }
            world plugin {
                import host;
                export run: func(x: u32) -> result<string, string>;
            }
        ";
        let bytes = encode_inline_wit(wit, "plugin")?;
        validate_world_metadata(&bytes, &bytes)
    }

    #[test]
    fn mismatched_record_field_fails_validation() -> wasmtime::Result<()> {
        let host = encode_inline_wit(
            r"
            package test:demo@0.1.0;
            interface host {
                record item { name: string, count: u32 }
                echo: func(i: item) -> string;
            }
            world plugin {
                import host;
                export run: func(x: u32) -> string;
            }
            ",
            "plugin",
        )?;
        let guest = encode_inline_wit(
            r"
            package test:demo@0.1.0;
            interface host {
                record item { name: string, count: u64 }
                echo: func(i: item) -> string;
            }
            world plugin {
                import host;
                export run: func(x: u32) -> string;
            }
            ",
            "plugin",
        )?;
        let err = validate_world_metadata(&guest, &host)
            .err()
            .ok_or_else(|| wasmtime::Error::msg("type mismatch must fail"))?;
        assert!(err.to_string().contains("parameter #0"));
        Ok(())
    }

    #[test]
    fn missing_required_export_fails_validation() -> wasmtime::Result<()> {
        let host = encode_inline_wit(
            r"
            package test:demo@0.1.0;
            world plugin {
                export on-load: func();
                export on-unload: func();
            }
            ",
            "plugin",
        )?;
        let guest = encode_inline_wit(
            r"
            package test:demo@0.1.0;
            world plugin {
                export on-load: func();
            }
            ",
            "plugin",
        )?;
        let err = validate_world_metadata(&guest, &host)
            .err()
            .ok_or_else(|| wasmtime::Error::msg("missing export must fail"))?;
        assert!(err.to_string().contains("on-unload"));
        Ok(())
    }

    #[test]
    fn version_mismatch_fails_validation() -> wasmtime::Result<()> {
        let host = encode_inline_wit("package test:demo@0.1.0; world plugin {}", "plugin")?;
        let guest = encode_inline_wit("package test:demo@0.2.0; world plugin {}", "plugin")?;
        let err = validate_world_metadata(&guest, &host)
            .err()
            .ok_or_else(|| wasmtime::Error::msg("version mismatch must fail"))?;
        assert!(err.to_string().contains("package mismatch"));
        Ok(())
    }

    #[test]
    fn future_and_stream_types_validate() -> wasmtime::Result<()> {
        let host = encode_inline_wit(
            r"
            package test:demo@0.2.0;
            interface host {
                idle: func() -> result<future, string>;
                events: func() -> stream<u32>;
            }
            world plugin {
                import host;
            }
            ",
            "plugin",
        )?;
        validate_world_metadata(&host, &host)?;

        let mismatched = encode_inline_wit(
            r"
            package test:demo@0.2.0;
            interface host {
                idle: func() -> result<future<u32>, string>;
                events: func() -> stream<u32>;
            }
            world plugin {
                import host;
            }
            ",
            "plugin",
        )?;
        let err = validate_world_metadata(&mismatched, &host)
            .err()
            .ok_or_else(|| wasmtime::Error::msg("future payload mismatch must fail"))?;
        assert!(err.to_string().contains("return type mismatch"));
        Ok(())
    }

    #[test]
    fn non_plugin_world_suffix_is_rejected() {
        assert_eq!(
            parse_pumpkin_world_version("__wit_bindgen_world_pumpkinX3ApluginX400X2E2X2E0X2Fother"),
            None
        );
    }

    #[test]
    fn resource_handles_require_matching_owning_interface() -> wasmtime::Result<()> {
        let host = encode_inline_wit(
            r"
            package test:demo@0.2.0;
            interface left {
                resource token;
            }
            interface right {
                resource token;
            }
            interface host {
                use left.{token as tok};
                consume: func(t: borrow<tok>);
            }
            world plugin {
                import left;
                import right;
                import host;
            }
            ",
            "plugin",
        )?;
        let matching_guest = encode_inline_wit(
            r"
            package test:demo@0.2.0;
            interface left {
                resource token;
            }
            interface host {
                use left.{token};
                consume: func(t: borrow<token>);
            }
            world plugin {
                import left;
                import host;
            }
            ",
            "plugin",
        )?;
        validate_world_metadata(&matching_guest, &host)?;

        let mismatched_guest = encode_inline_wit(
            r"
            package test:demo@0.2.0;
            interface right {
                resource token;
            }
            interface host {
                use right.{token};
                consume: func(t: borrow<token>);
            }
            world plugin {
                import right;
                import host;
            }
            ",
            "plugin",
        )?;
        assert!(validate_world_metadata(&mismatched_guest, &host).is_err());
        Ok(())
    }
}
