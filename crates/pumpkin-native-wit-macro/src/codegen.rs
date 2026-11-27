//! Host-side Rust code generator for native (`cdylib`) WIT worlds.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::mem;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use wit_bindgen_core::abi::AbiVariant;
use wit_component::StringEncoding;
use wit_parser::{
    Docs, Enum, Flags, Function, FunctionKind, Handle, InterfaceId, PackageId, Record, Resolve,
    Result_, SizeAlign, Tuple, Type, TypeDefKind, TypeId, TypeOwner, Variant, WorldId, WorldItem,
    WorldKey,
};

use crate::abi_gen;
use crate::rust::{RustGenerator, TypeNames, render_type, to_rust_ident, to_rust_upper_camel_case};
use crate::symbol_name;
use crate::types::{TypeInfo, Types, dealias_id};

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct ImportMode {
    pub is_async: bool,
    pub with_store: bool,
    pub trappable: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Opts {
    pub store_data: String,
    pub concurrent: bool,
    pub default_import: ImportMode,
    pub imports: HashMap<String, ImportMode>,
}

impl Opts {
    pub fn generate(&self, resolve: &mut Resolve, world: WorldId) -> anyhow::Result<String> {
        resolve.generate_nominal_type_ids(world);
        let mut generator = Generator {
            opts: self.clone(),
            ..Generator::default()
        };
        generator.generate(resolve, world)
    }

    fn mode_for(&self, iface_id: &str, func_name: &str) -> ImportMode {
        let exact = format!("{iface_id}.{func_name}");
        if let Some(&mode) = self.imports.get(&exact) {
            return mode;
        }
        if let Some((unversioned, _)) = iface_id.split_once('@') {
            let unversioned_exact = format!("{unversioned}.{func_name}");
            if let Some(&mode) = self.imports.get(&unversioned_exact) {
                return mode;
            }
            if let Some(&mode) = self.imports.get(unversioned) {
                return mode;
            }
        }
        if let Some(&mode) = self.imports.get(iface_id) {
            return mode;
        }
        self.default_import
    }
}

#[derive(Default)]
struct Generator {
    src: String,
    opts: Opts,
    import_interfaces: IndexMap<InterfaceId, ImportInterface>,
    world_export_functions: Vec<(Option<String>, Function)>,
    types: Types,
    interface_names: IndexMap<InterfaceId, Vec<String>>,
    wasmtime_export_only: HashSet<InterfaceId>,
    import_registrations: Vec<ImportRegistration>,
    world_metadata_symbol: String,
    world_wit_name: String,
    sizes: SizeAlign,
}

struct ImportInterface {
    contents: String,
    path: Vec<String>,
}

#[derive(Clone)]
struct ImportRegistration {
    callback: String,
    module: String,
    name: String,
    uses_plugin_slot: bool,
}

struct ImportFn {
    wasmtime_name: String,
    is_resource_method: bool,
    is_wit_async: bool,
    is_drop: bool,
    explicit_param_idents: Vec<String>,
    explicit_param_types: Vec<Type>,
    source: Option<Function>,
    result_type: Option<Type>,
    symbol: String,
    module: String,
    name: String,
    config_name: String,
}

impl Generator {
    const fn wasmtime_types(&self) -> WasmtimeTypes<'_> {
        WasmtimeTypes {
            names: &self.interface_names,
            export_only: &self.wasmtime_export_only,
        }
    }

    fn generate_interface_name(resolve: &Resolve, id: InterfaceId, name: &WorldKey) -> Vec<String> {
        let mut path = Vec::new();
        match name {
            WorldKey::Name(name) => {
                path.push(to_rust_ident(name));
            }
            WorldKey::Interface(_) => {
                let iface = &resolve.interfaces[id];
                if let Some(pkg_id) = iface.package {
                    let pkgname = &resolve.packages[pkg_id].name;
                    path.push(to_rust_ident(&pkgname.namespace));
                    path.push(Self::name_package_module(resolve, pkg_id));
                }
                if let Some(iface_name) = iface.name.as_deref() {
                    path.push(to_rust_ident(iface_name));
                }
            }
        }
        path
    }

    fn name_package_module(resolve: &Resolve, id: PackageId) -> String {
        let pkg = &resolve.packages[id];
        let versions_with_same_name = resolve
            .packages
            .iter()
            .filter(|(_, p)| p.name.namespace == pkg.name.namespace && p.name.name == pkg.name.name)
            .count();
        let base = to_rust_ident(&pkg.name.name);
        if versions_with_same_name == 1 {
            return base;
        }
        let Some(version) = &pkg.name.version else {
            return base;
        };
        let version = version
            .to_string()
            .replace(['.', '-', '+'], "_")
            .to_snake_case();
        format!("{base}{version}")
    }

    fn generate(&mut self, resolve: &Resolve, world_id: WorldId) -> anyhow::Result<String> {
        self.types.analyze(resolve, world_id);
        self.sizes
            .fill(resolve)
            .map_err(|err| anyhow::anyhow!("failed to compute WIT type sizes: {err}"))?;
        self.world_metadata_symbol = symbol_name::world_metadata_symbol(resolve, world_id);
        self.world_wit_name = symbol_name::world_wit_name(resolve, world_id);

        self.discover_world_items(resolve, world_id)?;
        self.emit_interface_modules(resolve);
        self.emit_host_world_metadata(resolve, world_id)?;
        self.emit_reliable_type_aliases(resolve);
        self.emit_type_conversion_fns(resolve);
        self.generate_import_callbacks(resolve, world_id)?;
        self.generate_plugin_struct(resolve, world_id);

        let modules = mem::take(&mut self.import_interfaces);
        self.emit_modules(
            modules
                .into_iter()
                .map(|(_, item)| (item.contents, item.path))
                .collect(),
        );

        let body = mem::take(&mut self.src);
        Ok(format!(
            "#[allow(\n\
             clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo,\n\
             clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable,\n\
             clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr,\n\
             clippy::empty_enums, clippy::use_self, clippy::redundant_clone,\n\
             clippy::if_then_some_else_none, clippy::option_if_let_else,\n\
             clippy::needless_pass_by_ref_mut, clippy::needless_collect,\n\
             dead_code, unused_variables, unused_imports, unused_mut, unused_parens,\n\
             non_snake_case, non_camel_case_types\n\
             )]\n\
             const _: () = {{}};\n\
             {body}"
        ))
    }

    fn discover_world_items(&mut self, resolve: &Resolve, world_id: WorldId) -> anyhow::Result<()> {
        let world = &resolve.worlds[world_id];
        let mut processed_interfaces: HashSet<InterfaceId> = HashSet::new();
        let mut worklist: Vec<InterfaceId> = Vec::new();

        // Discover directly imported interfaces and interfaces referenced by world-level `use`.
        for (key, item) in &world.imports {
            match item {
                WorldItem::Interface { id, .. } => {
                    if processed_interfaces.insert(*id) {
                        self.register_interface(resolve, key, *id);
                        worklist.push(*id);
                    }
                }
                WorldItem::Type { id, .. } => {
                    let target = dealias_id(resolve, *id);
                    if let TypeOwner::Interface(owner_id) = resolve.types[target].owner
                        && processed_interfaces.insert(owner_id)
                    {
                        self.register_interface(resolve, &WorldKey::Interface(owner_id), owner_id);
                        worklist.push(owner_id);
                    }
                    let mut iface_gen = InterfaceGenerator::new(self, resolve);
                    if let WorldKey::Name(name) = key {
                        iface_gen.define_type(name, *id);
                    }
                    let body = mem::take(&mut iface_gen.src);
                    self.src.push_str(&body);
                }
                WorldItem::Function(func) => {
                    anyhow::bail!(
                        "pumpkin-native-wit does not support world-level imported function `{}`",
                        func.name
                    );
                }
            }
        }

        // Transitively discover all interfaces referenced via `use` across type definitions.
        while let Some(iface_id) = worklist.pop() {
            self.enqueue_interface_deps(
                resolve,
                iface_id,
                &mut processed_interfaces,
                &mut worklist,
            );
        }

        // Identify export-only interfaces (which Wasmtime places under `exports::`).
        let imported_or_used = processed_interfaces.clone();
        self.wasmtime_export_only = world
            .exports
            .values()
            .filter_map(|item| match item {
                WorldItem::Interface { id, .. } => Some(*id),
                _ => None,
            })
            .filter(|id| !imported_or_used.contains(id))
            .collect();

        for (key, export) in &world.exports {
            match export {
                WorldItem::Function(f) => {
                    self.world_export_functions.push((None, f.clone()));
                }
                WorldItem::Interface { id, .. } => {
                    if let Some((rname, _)) = resolve.interfaces[*id]
                        .types
                        .iter()
                        .find(|(_, tid)| matches!(resolve.types[**tid].kind, TypeDefKind::Resource))
                    {
                        anyhow::bail!(
                            "pumpkin-native-wit does not support exported resource `{rname}` in `{}`",
                            resolve.name_world_key(key)
                        );
                    }
                    if processed_interfaces.insert(*id) {
                        self.register_interface(resolve, key, *id);
                    }
                    let module = resolve.name_world_key(key);
                    for f in resolve.interfaces[*id].functions.values() {
                        self.world_export_functions
                            .push((Some(module.clone()), f.clone()));
                    }
                }
                WorldItem::Type { .. } => {}
            }
        }

        self.validate_world_exports(resolve)
    }

    fn validate_world_exports(&self, resolve: &Resolve) -> anyhow::Result<()> {
        for (_, func) in &self.world_export_functions {
            let mut visited = HashSet::new();
            for ty in func
                .params
                .iter()
                .map(|p| &p.ty)
                .chain(func.result.as_ref())
            {
                if let Some((type_name, owner_world)) =
                    find_world_owned_named_type(resolve, *ty, &mut visited)
                {
                    let world_name = &resolve.worlds[owner_world].name;
                    anyhow::bail!(
                        "pumpkin-native-wit does not support world-defined named type \
                         `{type_name}` from world `{world_name}` in exported function `{}`",
                        func.name
                    );
                }
            }
        }
        Ok(())
    }

    fn enqueue_interface_deps(
        &mut self,
        resolve: &Resolve,
        iface_id: InterfaceId,
        processed: &mut HashSet<InterfaceId>,
        worklist: &mut Vec<InterfaceId>,
    ) {
        let type_ids: Vec<TypeId> = resolve.interfaces[iface_id]
            .types
            .values()
            .copied()
            .collect();
        for tid in type_ids {
            self.enqueue_referenced_interfaces(resolve, Type::Id(tid), processed, worklist);
        }
        let funcs: Vec<Function> = resolve.interfaces[iface_id]
            .functions
            .values()
            .cloned()
            .collect();
        for func in funcs {
            for param in &func.params {
                self.enqueue_referenced_interfaces(resolve, param.ty, processed, worklist);
            }
            if let Some(res) = func.result {
                self.enqueue_referenced_interfaces(resolve, res, processed, worklist);
            }
        }
    }

    fn emit_interface_modules(&mut self, resolve: &Resolve) {
        let ordered_interfaces: Vec<InterfaceId> = self.interface_names.keys().copied().collect();
        for iface_id in ordered_interfaces {
            let path = self.interface_names[&iface_id].clone();
            let mut iface_gen = InterfaceGenerator::new(self, resolve);
            iface_gen.current_interface = Some((iface_id, path.len()));
            iface_gen.types(iface_id);
            let module_body = mem::take(&mut iface_gen.src);
            let mod_name = path.last().cloned().unwrap_or_else(|| "iface".to_string());
            let contents = format!("pub mod {mod_name} {{\n{module_body}\n}}\n");
            self.import_interfaces
                .insert(iface_id, ImportInterface { contents, path });
        }
    }

    fn register_interface(&mut self, resolve: &Resolve, key: &WorldKey, id: InterfaceId) {
        let path = Self::generate_interface_name(resolve, id, key);
        self.interface_names.insert(id, path);
    }

    fn enqueue_referenced_interfaces(
        &mut self,
        resolve: &Resolve,
        ty: Type,
        processed: &mut HashSet<InterfaceId>,
        worklist: &mut Vec<InterfaceId>,
    ) {
        let Type::Id(id) = ty else {
            return;
        };
        let def = &resolve.types[id];
        if let TypeOwner::Interface(owner_id) = def.owner
            && processed.insert(owner_id)
        {
            self.register_interface(resolve, &WorldKey::Interface(owner_id), owner_id);
            worklist.push(owner_id);
        }
        match &def.kind {
            TypeDefKind::Type(inner)
            | TypeDefKind::Option(inner)
            | TypeDefKind::List(inner)
            | TypeDefKind::FixedLengthList(inner, _) => {
                self.enqueue_referenced_interfaces(resolve, *inner, processed, worklist);
            }
            TypeDefKind::Record(r) => {
                for f in &r.fields {
                    self.enqueue_referenced_interfaces(resolve, f.ty, processed, worklist);
                }
            }
            TypeDefKind::Variant(v) => {
                for c in &v.cases {
                    if let Some(t) = c.ty {
                        self.enqueue_referenced_interfaces(resolve, t, processed, worklist);
                    }
                }
            }
            TypeDefKind::Tuple(t) => {
                for elem in &t.types {
                    self.enqueue_referenced_interfaces(resolve, *elem, processed, worklist);
                }
            }
            TypeDefKind::Result(r) => {
                if let Some(ok) = r.ok {
                    self.enqueue_referenced_interfaces(resolve, ok, processed, worklist);
                }
                if let Some(err) = r.err {
                    self.enqueue_referenced_interfaces(resolve, err, processed, worklist);
                }
            }
            TypeDefKind::Handle(Handle::Own(rid) | Handle::Borrow(rid)) => {
                self.enqueue_referenced_interfaces(resolve, Type::Id(*rid), processed, worklist);
            }
            TypeDefKind::Map(k, v) => {
                self.enqueue_referenced_interfaces(resolve, *k, processed, worklist);
                self.enqueue_referenced_interfaces(resolve, *v, processed, worklist);
            }
            _ => {}
        }
    }

    fn emit_host_world_metadata(
        &mut self,
        resolve: &Resolve,
        world_id: WorldId,
    ) -> anyhow::Result<()> {
        let bytes =
            wit_component::metadata::encode(resolve, world_id, StringEncoding::UTF8, None, false)?;
        let _ = writeln!(self.src, "const HOST_WORLD_METADATA: &[u8] = &[");
        for chunk in bytes.chunks(20) {
            for b in chunk {
                let _ = write!(self.src, "{b},");
            }
            self.src.push('\n');
        }
        self.src.push_str("];\n");
        Ok(())
    }

    fn emit_reliable_type_aliases(&mut self, resolve: &Resolve) {
        let mut already_aliased: HashSet<TypeId> = HashSet::new();
        let interface_ids: Vec<InterfaceId> = self.interface_names.keys().copied().collect();
        for iface_id in interface_ids {
            for &tid in resolve.interfaces[iface_id].types.values() {
                let real_id = dealias_id(resolve, tid);
                if !matches!(
                    resolve.types[real_id].kind,
                    TypeDefKind::Record(_) | TypeDefKind::Variant(_) | TypeDefKind::Enum(_)
                ) {
                    continue;
                }
                if !already_aliased.insert(real_id) {
                    continue;
                }
                let TypeOwner::Interface(owner_id) = resolve.types[real_id].owner else {
                    continue;
                };
                let Some(path) = self.interface_names.get(&owner_id) else {
                    continue;
                };
                let alias_name = abi_gen::type_name(resolve, real_id);
                let type_camel =
                    to_rust_upper_camel_case(resolve.types[real_id].name.as_deref().unwrap_or(""));
                let _ = writeln!(
                    self.src,
                    "type {alias_name} = {}::{type_camel};",
                    path.join("::")
                );
            }
        }
    }

    /// Emits shared conversion helpers per named type so large shared types (like `Event` or
    /// packet enums) are compiled once instead of duplicated at every call site.
    fn emit_type_conversion_fns(&mut self, resolve: &Resolve) {
        let wt = self.wasmtime_types();
        let mut seen = HashSet::new();
        let interface_ids: Vec<InterfaceId> = self.interface_names.keys().copied().collect();
        let mut type_ids = Vec::new();
        for iface_id in interface_ids {
            for &tid in resolve.interfaces[iface_id].types.values() {
                let real_id = dealias_id(resolve, tid);
                if matches!(
                    resolve.types[real_id].kind,
                    TypeDefKind::Record(_)
                        | TypeDefKind::Variant(_)
                        | TypeDefKind::Enum(_)
                        | TypeDefKind::Flags(_)
                ) && seen.insert(real_id)
                {
                    type_ids.push(real_id);
                }
            }
        }

        let concurrent = self.opts.concurrent;
        let mut out = String::new();
        for id in type_ids {
            let ty = Type::Id(id);
            let fn_suffix = conv_fn_suffix(resolve, id);
            let native_ty = abi_gen::rust_type_for_native(resolve, &ty);
            let wasmtime_ty = rust_type_for_wasmtime(resolve, wt, &ty);

            let body_to_wt = convert_body(
                resolve,
                wt,
                id,
                "v",
                Dir::ToWasmtime,
                Src::Owned,
                concurrent,
            );
            let _ = writeln!(
                out,
                "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, dead_code, unused_variables, unused_mut)]\n\
                 fn __to_wt_{fn_suffix}(v: {native_ty}) -> {wasmtime_ty} {{ {body_to_wt} }}"
            );

            let body_to_native =
                convert_body(resolve, wt, id, "v", Dir::ToNative, Src::Owned, concurrent);
            let _ = writeln!(
                out,
                "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, dead_code, unused_variables, unused_mut)]\n\
                 fn __to_native_{fn_suffix}(v: {wasmtime_ty}) -> {native_ty} {{ {body_to_native} }}"
            );

            let body_to_native_ref = convert_body(
                resolve,
                wt,
                id,
                "v",
                Dir::ToNative,
                Src::ThroughRef,
                concurrent,
            );
            let _ = writeln!(
                out,
                "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, dead_code, unused_variables, unused_mut)]\n\
                 fn __to_native_ref_{fn_suffix}(v: &{wasmtime_ty}) -> {native_ty} {{ {body_to_native_ref} }}"
            );
        }
        self.src.push_str(&out);
    }

    fn generate_import_callbacks(
        &mut self,
        resolve: &Resolve,
        world_id: WorldId,
    ) -> anyhow::Result<()> {
        let imports: Vec<(WorldKey, InterfaceId)> = resolve.worlds[world_id]
            .imports
            .iter()
            .filter_map(|(key, item)| match item {
                WorldItem::Interface { id, .. } => Some((key.clone(), *id)),
                _ => None,
            })
            .collect();

        for (key, iface_id) in imports {
            let iface = &resolve.interfaces[iface_id];
            if iface.name.is_none() {
                continue;
            }
            let wasmtime_path = Self::generate_interface_name(resolve, iface_id, &key).join("::");
            let iface_key_str = resolve.name_world_key(&key);

            let mut free_fns: Vec<&Function> = Vec::new();
            let mut resource_fns: IndexMap<TypeId, Vec<&Function>> = IndexMap::new();
            for func in iface.functions.values() {
                match &func.kind {
                    FunctionKind::Freestanding | FunctionKind::AsyncFreestanding => {
                        free_fns.push(func);
                    }
                    FunctionKind::Method(rid)
                    | FunctionKind::AsyncMethod(rid)
                    | FunctionKind::Constructor(rid)
                    | FunctionKind::Static(rid)
                    | FunctionKind::AsyncStatic(rid) => {
                        resource_fns.entry(*rid).or_default().push(func);
                    }
                    other => {
                        anyhow::bail!(
                            "pumpkin-native-wit does not yet support WIT import `{iface_key_str}#{}` ({other:?})",
                            func.name
                        );
                    }
                }
            }

            if !free_fns.is_empty() {
                let mut import_fns = Vec::with_capacity(free_fns.len());
                for func in free_fns {
                    if let Some(import_fn) = ImportFn::from_function(resolve, &key, func)? {
                        import_fns.push(import_fn);
                    }
                }
                if !import_fns.is_empty() {
                    self.emit_import_group(
                        resolve,
                        "Host",
                        "HostWithStore",
                        &wasmtime_path,
                        &iface_key_str,
                        &import_fns,
                    );
                }
            }

            // Every resource defined in this interface needs a destructor plus any methods.
            let resources: Vec<(String, TypeId)> = iface
                .types
                .iter()
                .filter(|&(_, &tid)| matches!(resolve.types[tid].kind, TypeDefKind::Resource))
                .map(|(name, &tid)| (name.clone(), tid))
                .collect();

            for (res_name, rid) in resources {
                let camel = to_rust_upper_camel_case(&res_name);
                let trait_name = format!("Host{camel}");
                let store_trait_name = format!("Host{camel}WithStore");
                let mut import_fns = Vec::new();
                if let Some(fns) = resource_fns.swap_remove(&rid) {
                    for func in fns {
                        if let Some(import_fn) = ImportFn::from_function(resolve, &key, func)? {
                            import_fns.push(import_fn);
                        }
                    }
                }
                import_fns.push(ImportFn::resource_drop(&iface_key_str, &res_name));
                self.emit_import_group(
                    resolve,
                    &trait_name,
                    &store_trait_name,
                    &wasmtime_path,
                    &iface_key_str,
                    &import_fns,
                );
            }
        }

        Ok(())
    }

    fn emit_import_group(
        &mut self,
        resolve: &Resolve,
        trait_name: &str,
        store_trait_name: &str,
        wasmtime_path: &str,
        iface_key_str: &str,
        fns: &[ImportFn],
    ) {
        let mut out = String::new();
        let mut regs = Vec::with_capacity(fns.len());

        for f in fns {
            let helper_name = format!("__host_call_{}", f.symbol);
            let cb_name = format!("__import_cb_{}", f.symbol);
            self.emit_import_helper(
                &mut out,
                resolve,
                trait_name,
                store_trait_name,
                wasmtime_path,
                iface_key_str,
                f,
                &helper_name,
            );
            self.emit_import_cabi_wrapper(&mut out, resolve, f, &helper_name, &cb_name);

            regs.push(ImportRegistration {
                callback: cb_name,
                module: f.module.clone(),
                name: f.name.clone(),
                uses_plugin_slot: self.opts.concurrent && !f.is_resource_method && !f.is_drop,
            });
        }

        self.src.push_str(&out);
        self.import_registrations.extend(regs);
    }

    fn build_wt_args(&self, resolve: &Resolve, f: &ImportFn) -> Vec<String> {
        let concurrent = self.opts.concurrent;
        let mut wt_args = Vec::new();
        if f.is_resource_method || f.is_drop {
            if let Some(src) = f.source.as_ref() {
                wt_args.push(convert_expr(
                    resolve,
                    &src.params[0].ty,
                    "rep",
                    Dir::ToWasmtime,
                    Src::Owned,
                    concurrent,
                ));
            } else {
                let to_rep = if concurrent {
                    "tagged_handle_to_rep"
                } else {
                    "handle_to_rep"
                };
                wt_args.push(format!(
                    "::pumpkin_native_wit::wasmtime::component::Resource::new_own(\
                     ::pumpkin_native_wit::{to_rep}(rep))"
                ));
            }
        }
        for (ident, ty) in f
            .explicit_param_idents
            .iter()
            .zip(f.explicit_param_types.iter())
        {
            wt_args.push(convert_expr(
                resolve,
                ty,
                ident,
                Dir::ToWasmtime,
                Src::Owned,
                concurrent,
            ));
        }
        wt_args
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Codegen helper forwards interface trait context and target buffer"
    )]
    fn emit_import_helper(
        &self,
        out: &mut String,
        resolve: &Resolve,
        trait_name: &str,
        store_trait_name: &str,
        wasmtime_path: &str,
        iface_key_str: &str,
        f: &ImportFn,
        helper_name: &str,
    ) {
        if self.opts.concurrent {
            self.emit_concurrent_import_helper(
                out,
                resolve,
                trait_name,
                store_trait_name,
                wasmtime_path,
                iface_key_str,
                f,
                helper_name,
            );
            return;
        }

        let store_data = &self.opts.store_data;
        let mode = self.opts.mode_for(iface_key_str, &f.config_name);
        let import_label = format!("{}#{}", f.module, f.name);
        let (native_params, native_ret) = native_helper_signature(resolve, f);
        let wt_args = self.build_wt_args(resolve, f);

        let raw_call = if mode.with_store {
            let mut all_args = vec![
                "::pumpkin_native_wit::wasmtime::component::Access::new(\
                 ::pumpkin_native_wit::wasmtime::AsContextMut::as_context_mut(__store), \
                 |__s| __s)"
                    .to_string(),
            ];
            all_args.extend(wt_args);
            format!(
                "<::pumpkin_native_wit::wasmtime::component::HasSelf<{store_data}> \
                 as super::{wasmtime_path}::{store_trait_name}<{store_data}>>::{}({})",
                f.wasmtime_name,
                all_args.join(", ")
            )
        } else {
            let mut all_args = vec!["__store.data_mut()".to_string()];
            all_args.extend(wt_args);
            format!(
                "<{store_data} as super::{wasmtime_path}::{trait_name}>::{}({})",
                f.wasmtime_name,
                all_args.join(", ")
            )
        };

        let evaluated = if mode.is_async {
            format!("::pumpkin_native_wit::block_on({raw_call})")
        } else {
            raw_call
        };

        let _ = writeln!(
            out,
            "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, dead_code, unused_variables, unused_mut)]\n\
             fn {helper_name}({native_params}){native_ret} {{\n\
             unsafe {{ ::pumpkin_native_wit::with_active_store::<{store_data}, _>(\"{import_label}\", |__store| {{"
        );
        if f.is_drop {
            let _ = writeln!(out, "let _ = {evaluated};");
        } else {
            let unwrapped = if mode.trappable {
                format!(
                    "{evaluated}.unwrap_or_else(|e| panic!(\"native WIT host call `{import_label}` trapped: {{e}}\"))"
                )
            } else {
                evaluated
            };
            if let Some(result_ty) = f.result_type.as_ref() {
                let _ = writeln!(out, "let __r = {unwrapped};");
                let back =
                    convert_expr(resolve, result_ty, "__r", Dir::ToNative, Src::Owned, false);
                let _ = writeln!(out, "{back}");
            } else {
                let _ = writeln!(out, "{unwrapped};");
            }
        }
        out.push_str("}) }\n}\n");
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Codegen helper forwards interface trait context and target buffer"
    )]
    fn emit_concurrent_import_helper(
        &self,
        out: &mut String,
        resolve: &Resolve,
        trait_name: &str,
        store_trait_name: &str,
        wasmtime_path: &str,
        iface_key_str: &str,
        f: &ImportFn,
        helper_name: &str,
    ) {
        let store_data = &self.opts.store_data;
        let mode = self.opts.mode_for(iface_key_str, &f.config_name);
        let use_with_store = mode.with_store || f.is_wit_async;
        let is_async = mode.is_async || f.is_wit_async;
        let import_label = format!("{}#{}", f.module, f.name);
        let (native_params, native_ret) = native_helper_signature(resolve, f);
        let wt_args = self.build_wt_args(resolve, f);

        let _ = writeln!(
            out,
            "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, dead_code, unused_variables, unused_mut)]\n\
             fn {helper_name}({native_params}){native_ret} {{"
        );
        let mut arg_names = Vec::with_capacity(wt_args.len());
        for (idx, expr) in wt_args.iter().enumerate() {
            let name = format!("__arg{idx}");
            let _ = writeln!(out, "let {name} = {expr};");
            arg_names.push(name);
        }

        let dispatch_expr = if use_with_store {
            let mut all_args = vec![if is_async { "__accessor" } else { "__access" }.to_string()];
            all_args.extend(arg_names);
            let await_suffix = if is_async { ".await" } else { "" };
            let call = format!(
                "<::pumpkin_native_wit::wasmtime::component::HasSelf<{store_data}> \
                 as super::{wasmtime_path}::{store_trait_name}<{store_data}>>::{}({}){await_suffix}",
                f.wasmtime_name,
                all_args.join(", ")
            );
            let expr = if is_async {
                call
            } else {
                format!("__accessor.with(|__access| {call})")
            };
            let body = if mode.trappable {
                expr
            } else {
                format!("::core::result::Result::Ok({expr})")
            };
            format!(
                "::pumpkin_native_wit::with_concurrent_async::<{store_data}, _>(\
                 \"{import_label}\", \
                 move |__accessor| ::std::boxed::Box::pin(async move {{ {body} }}))"
            )
        } else {
            let mut all_args = vec!["__state".to_string()];
            all_args.extend(arg_names);
            let call = format!(
                "<{store_data} as super::{wasmtime_path}::{trait_name}>::{}({})",
                f.wasmtime_name,
                all_args.join(", ")
            );
            let call = if is_async {
                format!("::pumpkin_native_wit::block_on({call})")
            } else {
                call
            };
            let raw = format!(
                "::pumpkin_native_wit::with_concurrent_sync::<{store_data}, _>(\
                 \"{import_label}\", move |__state| {call})"
            );
            if mode.trappable {
                format!("{raw}.and_then(|__inner| __inner)")
            } else {
                raw
            }
        };

        if f.is_drop {
            let _ = writeln!(out, "let _ = {dispatch_expr};");
        } else {
            let unwrapped = format!(
                "{dispatch_expr}.unwrap_or_else(|e| panic!(\"native WIT host call `{import_label}` trapped: {{e}}\"))"
            );
            if let Some(result_ty) = f.result_type.as_ref() {
                let _ = writeln!(out, "let __r = {unwrapped};");
                let back = convert_expr(resolve, result_ty, "__r", Dir::ToNative, Src::Owned, true);
                let _ = writeln!(out, "{back}");
            } else {
                let _ = writeln!(out, "{unwrapped};");
            }
        }
        out.push_str("}\n");
    }

    fn emit_import_cabi_wrapper(
        &self,
        out: &mut String,
        resolve: &Resolve,
        f: &ImportFn,
        helper_name: &str,
        cb_name: &str,
    ) {
        let store_data = &self.opts.store_data;
        let import_label = format!("{}#{}", f.module, f.name);
        if let Some(func) = f.source.as_ref() {
            let sig = resolve.wasm_signature(AbiVariant::GuestImport, func);
            let mut cabi_params: Vec<String> = sig
                .params
                .iter()
                .enumerate()
                .map(|(i, ty)| format!("arg{i}: {}", abi_gen::wasm_type_rust(*ty)))
                .collect();
            let cabi_ret = if sig.retptr || sig.results.is_empty() {
                String::new()
            } else {
                format!(" -> {}", abi_gen::wasm_type_rust(sig.results[0]))
            };
            let last_idx = cabi_params.len().saturating_sub(1);
            if sig.retptr
                && let Some(last) = cabi_params.last_mut()
            {
                *last = format!("arg{last_idx}: *mut u8");
            }
            let args: Vec<String> = (0..cabi_params.len()).map(|i| format!("arg{i}")).collect();
            let body = abi_gen::generate_import_body(
                resolve,
                &self.sizes,
                func,
                args.clone(),
                helper_name.to_string(),
            );
            if self.opts.concurrent && !f.is_resource_method {
                let inner_params = if cabi_params.is_empty() {
                    "__plugin_id: u8".to_string()
                } else {
                    format!("__plugin_id: u8, {}", cabi_params.join(", "))
                };
                let forward_args = if args.is_empty() {
                    "__PLUGIN_ID".to_string()
                } else {
                    format!("__PLUGIN_ID, {}", args.join(", "))
                };
                let _ = writeln!(
                    out,
                    "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr, dead_code, unused_variables, unused_mut)]\n\
                     fn {cb_name}__inner({inner_params}){cabi_ret} {{\n\
                     ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {{\n\
                     let _concurrent_guard = ::pumpkin_native_wit::enter_concurrent_import_for_plugin::<{store_data}>(__plugin_id, \"{import_label}\");\n\
                     unsafe {{ {body} }}\n\
                     }})).unwrap_or_else(|e| ::pumpkin_native_wit::ffi_panic_guard(e))\n\
                     }}\n\
                     #[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr, dead_code, unused_variables, unused_mut)]\n\
                     unsafe extern \"C\" fn {cb_name}<const __PLUGIN_ID: u8>({}){cabi_ret} {{\n\
                     {cb_name}__inner({forward_args})\n\
                     }}",
                    cabi_params.join(", ")
                );
            } else {
                let concurrent_guard = if self.opts.concurrent {
                    format!(
                        "let _concurrent_guard = ::pumpkin_native_wit::enter_concurrent_import::<{store_data}>(::core::option::Option::Some(arg0 as u32), \"{import_label}\");\n"
                    )
                } else {
                    String::new()
                };
                let _ = writeln!(
                    out,
                    "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr, dead_code, unused_variables, unused_mut)]\n\
                     unsafe extern \"C\" fn {cb_name}({}){cabi_ret} {{\n\
                     ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {{\n\
                     {concurrent_guard}{body}\n\
                     }})).unwrap_or_else(|e| ::pumpkin_native_wit::ffi_panic_guard(e))\n\
                     }}",
                    cabi_params.join(", ")
                );
            }
        } else {
            let borrow_check = if self.opts.concurrent {
                "if ::pumpkin_native_wit::concurrent_handle_is_borrow(rep as u32) { return; }\n"
            } else {
                "if ::pumpkin_native_wit::handle_is_borrow(rep as u32) { return; }\n"
            };
            let concurrent_guard = if self.opts.concurrent {
                format!(
                    "let _concurrent_guard = ::pumpkin_native_wit::enter_concurrent_import::<{store_data}>(::core::option::Option::Some(rep as u32), \"{import_label}\");\n"
                )
            } else {
                String::new()
            };
            let _ = writeln!(
                out,
                "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, dead_code, unused_variables)]\n\
                 unsafe extern \"C\" fn {cb_name}(rep: i32) {{\n\
                 ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {{\n\
                 {borrow_check}{concurrent_guard}{helper_name}(rep as u32);\n\
                 }})).unwrap_or_else(|e| ::pumpkin_native_wit::ffi_panic_guard(e));\n\
                 }}"
            );
        }
    }

    fn generate_plugin_struct(&mut self, resolve: &Resolve, world_id: WorldId) {
        let store_data = &self.opts.store_data;
        let world_name = to_rust_upper_camel_case(&resolve.worlds[world_id].name);
        let export_pointers: Vec<ExportPointer> = self
            .world_export_functions
            .iter()
            .map(|(module, func)| ExportPointer::new(resolve, module.as_deref(), func))
            .collect();

        let mut out = String::new();
        let _ = writeln!(
            out,
            "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr, dead_code, unused_variables, unused_mut)]\n\
             pub struct {world_name} {{\n\
             _lib: ::std::sync::Arc<::pumpkin_native_wit::libloading::Library>,\n\
             __cabi_realloc: ::pumpkin_native_wit::CabiRealloc,"
        );
        if self.opts.concurrent {
            let _ = writeln!(
                out,
                "__bridge: ::std::sync::Arc<::pumpkin_native_wit::ConcurrentBridge<{store_data}>>,"
            );
        }
        for p in &export_pointers {
            let _ = writeln!(out, "{}: {},", p.field, p.fn_ty);
            if let Some((field, fn_ty, _)) = &p.post_return {
                let _ = writeln!(out, "{field}: {fn_ty},");
            }
        }
        out.push_str("}\n");

        let _ = writeln!(
            out,
            "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::undocumented_unsafe_blocks, clippy::cast_ptr_alignment, clippy::ptr_as_ptr, dead_code, unused_variables, unused_mut)]\n\
             impl {world_name} {{"
        );
        self.emit_plugin_new(&mut out, &export_pointers);
        let _ = writeln!(
            out,
            "pub fn library(&self) -> ::std::sync::Arc<::pumpkin_native_wit::libloading::Library> {{\n\
             ::std::sync::Arc::clone(&self._lib)\n\
             }}"
        );
        if self.opts.concurrent {
            let _ = writeln!(
                out,
                "pub fn attach_concurrent_store(&self, runner: ::std::sync::Arc<dyn ::pumpkin_native_wit::ConcurrentStoreRunner<{store_data}>>) {{\n\
                 self.__bridge.attach_runner(runner);\n\
                 }}"
            );
        }
        for (module, func) in &self.world_export_functions {
            self.emit_plugin_export_method(&mut out, resolve, module.as_deref(), func);
        }
        out.push_str("}\n");
        self.src.push_str(&out);
    }

    fn emit_plugin_new(&self, out: &mut String, export_pointers: &[ExportPointer]) {
        let world_marker = &self.world_metadata_symbol;
        let world_wit_name = &self.world_wit_name;
        let _ = writeln!(
            out,
            "pub unsafe fn new(path: &::std::path::Path) -> ::pumpkin_native_wit::wasmtime::Result<Self> {{\n\
             let lib = unsafe {{ ::pumpkin_native_wit::libloading::Library::new(path) }}\n\
             .map_err(|e| ::pumpkin_native_wit::wasmtime::Error::msg(format!(\"failed to load native WIT library {{}}: {{e}}\", path.display())))?;\n\
             let world_meta_fn: ::pumpkin_native_wit::WorldMetadataFn = unsafe {{\n\
             *lib.get::<::pumpkin_native_wit::WorldMetadataFn>(b\"{world_marker}\\0\")\n\
             .map_err(|e| ::pumpkin_native_wit::wasmtime::Error::msg(format!(\n\
             \"plugin does not export WIT world `{world_wit_name}` (symbol `{world_marker}` not found): {{e}}\")))?\n\
             }};\n\
             let mut meta_len: usize = 0;\n\
             let meta_ptr = unsafe {{ world_meta_fn(&mut meta_len) }};\n\
             if meta_ptr.is_null() || meta_len == 0 {{\n\
             return Err(::pumpkin_native_wit::wasmtime::Error::msg(\n\
             \"plugin returned empty WIT world metadata for `{world_wit_name}`\"));\n\
             }}\n\
             let guest_metadata = unsafe {{ ::core::slice::from_raw_parts(meta_ptr, meta_len) }};\n\
             ::pumpkin_native_wit::validate_world_metadata(guest_metadata, HOST_WORLD_METADATA)?;\n\
             let __cabi_realloc: ::pumpkin_native_wit::CabiRealloc = unsafe {{\n\
             *lib.get::<::pumpkin_native_wit::CabiRealloc>(::pumpkin_native_wit::CABI_REALLOC_SYMBOL)\n\
             .map_err(|e| ::pumpkin_native_wit::wasmtime::Error::msg(format!(\n\
             \"plugin does not export native cabi_realloc entrypoint: {{e}}\")))?\n\
             }};"
        );
        let mut fields = vec![
            "_lib: ::std::sync::Arc::new(lib)".to_string(),
            "__cabi_realloc".to_string(),
        ];
        if self.opts.concurrent {
            let store_data = &self.opts.store_data;
            let _ = writeln!(
                out,
                "let __bridge = ::pumpkin_native_wit::ConcurrentBridge::<{store_data}>::new(__cabi_realloc)?;"
            );
            fields.push("__bridge".to_string());
        }
        self.emit_import_resolver(out);
        for p in export_pointers {
            let what = format!("export `{}`", p.canonical);
            ExportPointer::emit_resolution(out, &p.field, &p.fn_ty, &p.symbol, &what);
            fields.push(p.field.clone());
            if let Some((field, fn_ty, symbol)) = &p.post_return {
                let what = format!("post-return for export `{}`", p.canonical);
                ExportPointer::emit_resolution(out, field, fn_ty, symbol, &what);
                fields.push(field.clone());
            }
        }
        let _ = writeln!(out, "Ok(Self {{ {} }})", fields.join(", "));
        out.push_str("}\n");
    }

    fn emit_import_resolver(&self, out: &mut String) {
        let _ = writeln!(
            out,
            "unsafe extern \"C\" fn __wit_import_resolver(\n\
             ctx: *mut (),\n\
             module_ptr: *const u8,\n\
             module_len: usize,\n\
             name_ptr: *const u8,\n\
             name_len: usize,\n\
             ) -> *mut () {{\n\
             let module = if module_len == 0 {{ &[][..] }} else {{ unsafe {{ ::core::slice::from_raw_parts(module_ptr, module_len) }} }};\n\
             let name = if name_len == 0 {{ &[][..] }} else {{ unsafe {{ ::core::slice::from_raw_parts(name_ptr, name_len) }} }};"
        );
        if self.opts.concurrent {
            let slots: String = (1..=64u8)
                .map(|i| format!("$f::<{i}> as *mut ()"))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(
                out,
                "let __slot_idx = (ctx as usize).checked_sub(1);\n\
                 macro_rules! __slot_ptr {{ ($f:ident) => {{{{ \
                 const TABLE: [*mut (); ::pumpkin_native_wit::MAX_CONCURRENT_PLUGINS] = [{slots}]; \
                 __slot_idx.and_then(|i| TABLE.get(i).copied()).unwrap_or(::core::ptr::null_mut()) \
                 }}}}; }}"
            );
        }
        out.push_str("match (module, name) {\n");
        for reg in &self.import_registrations {
            if reg.uses_plugin_slot {
                let _ = writeln!(
                    out,
                    "(b\"{}\", b\"{}\") => __slot_ptr!({}),",
                    reg.module, reg.name, reg.callback
                );
            } else {
                let _ = writeln!(
                    out,
                    "(b\"{}\", b\"{}\") => {} as *mut (),",
                    reg.module, reg.name, reg.callback
                );
            }
        }
        let resolver_ctx = if self.opts.concurrent {
            "usize::from(__bridge.plugin_id()) as *mut ()"
        } else {
            "::core::ptr::null_mut()"
        };
        let _ = writeln!(
            out,
            "_ => ::core::ptr::null_mut(),\n\
             }}\n\
             }}\n\
             unsafe {{\n\
             let set_resolver: ::pumpkin_native_wit::SetImportResolverFn = \
             *lib.get::<::pumpkin_native_wit::SetImportResolverFn>(::pumpkin_native_wit::SET_IMPORT_RESOLVER_SYMBOL)\n\
             .map_err(|e| ::pumpkin_native_wit::wasmtime::Error::msg(format!(\n\
             \"plugin does not export native import resolver entrypoint: {{e}}\")))?;\n\
             set_resolver(__wit_import_resolver, {resolver_ctx});\n\
             }}"
        );
    }

    fn emit_plugin_export_method(
        &self,
        out: &mut String,
        resolve: &Resolve,
        module: Option<&str>,
        func: &Function,
    ) {
        let store_data = &self.opts.store_data;
        let concurrent = self.opts.concurrent;
        let wt = self.wasmtime_types();
        let call_name = format!("call_{}", rust_function_name(func));
        let api_params: Vec<String> = func
            .params
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    to_rust_ident(&p.name),
                    wasmtime_param_type(resolve, wt, &p.ty)
                )
            })
            .collect();
        let api_ret = func.result.map_or_else(
            || "()".to_string(),
            |ty| rust_type_for_wasmtime(resolve, wt, &ty),
        );

        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| {
                let ident = to_rust_ident(&p.name);
                convert_expr(
                    resolve,
                    &p.ty,
                    &ident,
                    Dir::ToNative,
                    Src::Borrowed,
                    concurrent,
                )
            })
            .collect();
        let body = abi_gen::generate_export_body(resolve, &self.sizes, func, params, module);
        let ffi_expr = func.result.as_ref().map_or_else(
            || format!("unsafe {{ {body} }}"),
            |ty| {
                let lifted =
                    convert_expr(resolve, ty, "r", Dir::ToWasmtime, Src::Owned, concurrent);
                if lifted == "r" {
                    format!("unsafe {{ {body} }}")
                } else {
                    format!("unsafe {{ {body} }}.map(|r| {lifted})")
                }
            },
        );

        let extra_params = if api_params.is_empty() {
            String::new()
        } else {
            format!(", {}", api_params.join(", "))
        };
        if concurrent {
            let _ = writeln!(
                out,
                "pub fn {call_name}(&self{extra_params}) -> ::pumpkin_native_wit::wasmtime::Result<{api_ret}> {{\n\
                 ::pumpkin_native_wit::run_export(|| {{\n\
                 let _allocator = ::pumpkin_native_wit::install_allocator(self.__cabi_realloc);\n\
                 let _bridge = ::pumpkin_native_wit::install_concurrent_bridge(::std::sync::Arc::clone(&self.__bridge));\n\
                 {ffi_expr}\n\
                 }})\n\
                 }}"
            );
        } else {
            let _ = writeln!(
                out,
                "pub fn {call_name}(&self, mut store: ::pumpkin_native_wit::wasmtime::StoreContextMut<'_, {store_data}>{extra_params}) -> ::pumpkin_native_wit::wasmtime::Result<{api_ret}> {{\n\
                 ::pumpkin_native_wit::run_export(|| {{\n\
                 let _allocator = ::pumpkin_native_wit::install_allocator(self.__cabi_realloc);\n\
                 let _store = unsafe {{ ::pumpkin_native_wit::install_store(&mut store) }};\n\
                 {ffi_expr}\n\
                 }})\n\
                 }}"
            );
        }
    }

    fn emit_modules(&mut self, modules: Vec<(String, Vec<String>)>) {
        #[derive(Default)]
        struct Module {
            submodules: BTreeMap<String, Self>,
            contents: Vec<String>,
        }

        fn emit(dst: &mut String, module: Module) {
            for (name, sub) in module.submodules {
                let _ = writeln!(
                    dst,
                    "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::empty_enums, dead_code, unused_imports)]\npub mod {name} {{"
                );
                emit(dst, sub);
                dst.push_str("}\n");
            }
            for content in module.contents {
                let _ = writeln!(
                    dst,
                    "#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo, clippy::empty_enums, dead_code, unused_imports)]\n{content}"
                );
            }
        }

        let mut root = Module::default();
        for (module, path) in modules {
            let mut cur = &mut root;
            for name in &path[..path.len().saturating_sub(1)] {
                cur = cur.submodules.entry(name.clone()).or_default();
            }
            cur.contents.push(module);
        }

        emit(&mut self.src, root);
    }
}

struct InterfaceGenerator<'a> {
    src: String,
    generator: &'a mut Generator,
    resolve: &'a Resolve,
    current_interface: Option<(InterfaceId, usize)>,
}

impl<'a> InterfaceGenerator<'a> {
    const fn new(generator: &'a mut Generator, resolve: &'a Resolve) -> Self {
        Self {
            src: String::new(),
            generator,
            resolve,
            current_interface: None,
        }
    }

    fn types(&mut self, id: InterfaceId) {
        for (name, &tid) in &self.resolve.interfaces[id].types {
            self.define_type(name, tid);
        }
    }

    fn define_type(&mut self, name: &str, id: TypeId) {
        let ty = &self.resolve.types[id];
        match &ty.kind {
            TypeDefKind::Record(record) => self.type_record(id, name, record, &ty.docs),
            TypeDefKind::Flags(_)
            | TypeDefKind::Unknown
            | TypeDefKind::Future(_)
            | TypeDefKind::Stream(_)
            | TypeDefKind::FixedLengthList(..) => {}
            TypeDefKind::Handle(_) | TypeDefKind::Resource => self.type_handle(id, name, &ty.docs),
            TypeDefKind::Tuple(tuple) => self.type_tuple(id, name, tuple, &ty.docs),
            TypeDefKind::Enum(enum_) => self.type_enum(id, name, enum_, &ty.docs),
            TypeDefKind::Variant(variant) => self.type_variant(id, name, variant, &ty.docs),
            TypeDefKind::Option(t) => self.type_option(id, name, t, &ty.docs),
            TypeDefKind::Result(r) => self.type_result(id, name, r, &ty.docs),
            TypeDefKind::List(t) => self.type_list(id, name, t, &ty.docs),
            TypeDefKind::Type(t) => self.type_alias(id, name, t, &ty.docs),
            TypeDefKind::Map(k, v) => self.type_map(id, name, k, v, &ty.docs),
        }
    }

    fn type_record(&mut self, id: TypeId, _name: &str, record: &Record, _docs: &Docs) {
        let info = self.info(id);
        let name = self.type_name(id);
        if info.is_copy() {
            self.push_str("#[derive(Copy, Clone, Debug)]\n");
        } else if info.is_clone() {
            self.push_str("#[derive(Clone, Debug)]\n");
        } else {
            self.push_str("#[derive(Debug)]\n");
        }
        let _ = writeln!(self.src, "pub struct {name} {{");
        for field in &record.fields {
            let field_name = to_rust_ident(&field.name);
            let field_ty = self.ty(&field.ty);
            let _ = writeln!(self.src, "    pub {field_name}: {field_ty},");
        }
        self.push_str("}\n");
    }

    fn type_tuple(&mut self, id: TypeId, _name: &str, tuple: &Tuple, _docs: &Docs) {
        let name = self.type_name(id);
        let _ = write!(self.src, "pub type {name} = (");
        for ty in &tuple.types {
            self.print_ty(ty);
            self.push_str(",");
        }
        self.push_str(");\n");
    }

    fn type_variant(&mut self, id: TypeId, _name: &str, variant: &Variant, _docs: &Docs) {
        let info = self.info(id);
        let name = self.type_name(id);
        if info.is_copy() {
            self.push_str("#[derive(Copy, Clone)]\n");
        } else if info.is_clone() {
            self.push_str("#[derive(Clone)]\n");
        }
        let _ = writeln!(self.src, "pub enum {name} {{");
        for case in &variant.cases {
            let case_name = case.name.to_upper_camel_case();
            if let Some(ty) = &case.ty {
                let ty_str = self.ty(ty);
                let _ = writeln!(self.src, "    {case_name}({ty_str}),");
            } else {
                let _ = writeln!(self.src, "    {case_name},");
            }
        }
        self.push_str("}\n");
        self.print_rust_enum_debug(
            &name,
            variant
                .cases
                .iter()
                .map(|c| (c.name.to_upper_camel_case(), c.ty.as_ref())),
        );
    }

    fn type_enum(&mut self, id: TypeId, _name: &str, enum_: &Enum, _docs: &Docs) {
        let name = self.type_name(id);
        self.push_str("#[derive(Clone, Copy, PartialEq, Eq, Debug)]\n");
        let _ = writeln!(self.src, "pub enum {name} {{");
        for case in &enum_.cases {
            let case_name = case.name.to_upper_camel_case();
            let _ = writeln!(self.src, "    {case_name},");
        }
        self.push_str("}\n");
    }

    fn type_option(&mut self, id: TypeId, _name: &str, payload: &Type, _docs: &Docs) {
        let name = self.type_name(id);
        let inner = self.ty(payload);
        let _ = writeln!(
            self.src,
            "pub type {name} = ::std::option::Option<{inner}>;"
        );
    }

    fn type_result(&mut self, id: TypeId, _name: &str, result: &Result_, _docs: &Docs) {
        let name = self.type_name(id);
        let ok = self.optional_ty(result.ok.as_ref());
        let err = self.optional_ty(result.err.as_ref());
        let _ = writeln!(
            self.src,
            "pub type {name} = ::std::result::Result<{ok}, {err}>;"
        );
    }

    fn type_alias(&mut self, id: TypeId, _name: &str, ty: &Type, _docs: &Docs) {
        let name = self.type_name(id);
        let rendered = self.ty(ty);
        let _ = writeln!(self.src, "pub type {name} = {rendered};");
    }

    fn type_list(&mut self, id: TypeId, _name: &str, ty: &Type, _docs: &Docs) {
        let name = self.type_name(id);
        let _ = write!(self.src, "pub type {name} = ");
        self.print_list(ty);
        self.push_str(";\n");
    }

    fn type_handle(&mut self, id: TypeId, _name: &str, _docs: &Docs) {
        let name = self.type_name(id);
        let _ = writeln!(self.src, "pub type {name} = u32;");
    }

    fn type_map(&mut self, id: TypeId, _name: &str, key: &Type, value: &Type, _docs: &Docs) {
        let name = self.type_name(id);
        let k = self.ty(key);
        let v = self.ty(value);
        let _ = writeln!(
            self.src,
            "pub type {name} = ::std::collections::HashMap<{k}, {v}>;"
        );
    }

    fn path_to_root(&self) -> String {
        let depth = self.current_interface.map_or(0, |(_, depth)| depth);
        "super::".repeat(depth)
    }
}

impl<'a> RustGenerator<'a> for InterfaceGenerator<'a> {
    fn resolve(&self) -> &'a Resolve {
        self.resolve
    }

    fn path_to_interface(&self, interface: InterfaceId) -> Option<String> {
        if let Some((cur, _)) = self.current_interface
            && cur == interface
        {
            return None;
        }
        let mut path_to_root = self.path_to_root();
        if let Some(parts) = self.generator.interface_names.get(&interface) {
            path_to_root.push_str(&parts.join("::"));
        }
        Some(path_to_root)
    }

    fn push_str(&mut self, s: &str) {
        self.src.push_str(s);
    }

    fn info(&self, ty: TypeId) -> TypeInfo {
        self.generator.types.get(ty)
    }
}

impl ImportFn {
    fn from_function(
        resolve: &Resolve,
        key: &WorldKey,
        func: &Function,
    ) -> anyhow::Result<Option<Self>> {
        let is_resource_method = matches!(
            func.kind,
            FunctionKind::Method(_) | FunctionKind::AsyncMethod(_)
        );
        let is_wit_async = matches!(
            func.kind,
            FunctionKind::AsyncFreestanding
                | FunctionKind::AsyncMethod(_)
                | FunctionKind::AsyncStatic(_)
        );
        let explicit_params: Vec<&wit_parser::Param> = if is_resource_method {
            func.params.iter().skip(1).collect()
        } else {
            func.params.iter().collect()
        };

        let mut explicit_param_idents = Vec::with_capacity(explicit_params.len());
        let mut explicit_param_types = Vec::with_capacity(explicit_params.len());
        for p in explicit_params {
            if let Some(kind) = unsupported_kind(resolve, &p.ty) {
                // Native WIT does not bridge Component Model `future`/`stream` handles yet;
                // omit them from the resolver so unused imports do not block host bindgen.
                if matches!(kind, "future" | "stream") {
                    return Ok(None);
                }
                anyhow::bail!(
                    "parameter `{}` of imported function `{}` uses unsupported WIT kind `{kind}`",
                    p.name,
                    func.name
                );
            }
            explicit_param_idents.push(to_rust_ident(&p.name));
            explicit_param_types.push(p.ty);
        }
        if let Some(ty) = func.result
            && let Some(kind) = unsupported_kind(resolve, &ty)
        {
            if matches!(kind, "future" | "stream") {
                return Ok(None);
            }
            anyhow::bail!(
                "result of imported function `{}` uses unsupported WIT kind `{kind}`",
                func.name
            );
        }

        let wasmtime_name = rust_function_name(func);
        let symbol = symbol_name::func_import_symbol(resolve, key, func);
        Ok(Some(Self {
            wasmtime_name,
            is_resource_method,
            is_wit_async,
            is_drop: false,
            explicit_param_idents,
            explicit_param_types,
            source: Some(func.clone()),
            result_type: func.result,
            symbol,
            module: resolve.name_world_key(key),
            name: func.name.clone(),
            config_name: func.name.clone(),
        }))
    }

    fn resource_drop(module: &str, res_name: &str) -> Self {
        let wire_name = format!("[resource-drop]{res_name}");
        let config_name = format!("[drop]{res_name}");
        Self {
            wasmtime_name: "drop".to_string(),
            is_resource_method: false,
            is_wit_async: false,
            is_drop: true,
            explicit_param_idents: Vec::new(),
            explicit_param_types: Vec::new(),
            source: None,
            result_type: None,
            symbol: symbol_name::import_symbol(module, &wire_name),
            module: module.to_string(),
            name: wire_name,
            config_name,
        }
    }
}

struct ExportPointer {
    canonical: String,
    field: String,
    fn_ty: String,
    symbol: String,
    post_return: Option<(String, String, String)>,
}

impl ExportPointer {
    fn new(resolve: &Resolve, module: Option<&str>, func: &Function) -> Self {
        let canonical = symbol_name::canonical_export_name(module, &func.name);
        let sig = resolve.wasm_signature(AbiVariant::GuestExport, func);
        let params = sig
            .params
            .iter()
            .copied()
            .map(abi_gen::wasm_type_rust)
            .collect::<Vec<_>>()
            .join(", ");
        let ret = sig
            .results
            .first()
            .map(|t| format!(" -> {}", abi_gen::wasm_type_rust(*t)))
            .unwrap_or_default();

        let post_return = wit_bindgen_core::abi::guest_export_needs_post_return(resolve, func)
            .then(|| {
                let post_params = sig
                    .results
                    .iter()
                    .copied()
                    .map(abi_gen::wasm_type_rust)
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    symbol_name::post_return_field(&canonical),
                    format!("unsafe extern \"C\" fn({post_params})"),
                    symbol_name::post_return_symbol(&canonical),
                )
            });

        Self {
            field: symbol_name::export_field(&canonical),
            fn_ty: format!("unsafe extern \"C\" fn({params}){ret}"),
            symbol: symbol_name::export_symbol(&canonical),
            post_return,
            canonical,
        }
    }

    fn emit_resolution(src: &mut String, field: &str, fn_ty: &str, symbol: &str, what: &str) {
        let _ = writeln!(
            src,
            "let {field}: {fn_ty} = unsafe {{ *lib.get(b\"{symbol}\\0\").map_err(|e| \
             ::pumpkin_native_wit::wasmtime::Error::msg(format!(\"plugin is missing {what} \
             (symbol `{symbol}`): {{e}}\")))? }};"
        );
    }
}

#[derive(Copy, Clone)]
struct WasmtimeTypes<'a> {
    names: &'a IndexMap<InterfaceId, Vec<String>>,
    export_only: &'a HashSet<InterfaceId>,
}

impl WasmtimeTypes<'_> {
    fn owner_path(&self, resolve: &Resolve, id: TypeId) -> String {
        let id = dealias_id(resolve, id);
        let TypeOwner::Interface(owner_id) = resolve.types[id].owner else {
            return String::new();
        };
        let path = self.names[&owner_id].join("::");
        if self.export_only.contains(&owner_id) {
            format!("exports::{path}")
        } else {
            path
        }
    }
}

fn wasmtime_type_name(resolve: &Resolve, id: TypeId) -> String {
    to_rust_upper_camel_case(resolve.types[id].name.as_deref().unwrap_or("anon"))
}

impl TypeNames for WasmtimeTypes<'_> {
    fn named(&self, resolve: &Resolve, id: TypeId) -> String {
        format!(
            "super::{}::{}",
            self.owner_path(resolve, id),
            wasmtime_type_name(resolve, id)
        )
    }

    fn flags(&self, resolve: &Resolve, id: TypeId, _flags: &Flags) -> String {
        self.named(resolve, id)
    }

    fn handle(&self, resolve: &Resolve, handle: &Handle) -> String {
        let (Handle::Own(id) | Handle::Borrow(id)) = handle;
        let id = dealias_id(resolve, *id);
        format!(
            "::pumpkin_native_wit::wasmtime::component::Resource<super::{}::{}>",
            self.owner_path(resolve, id),
            wasmtime_type_name(resolve, id)
        )
    }
}

fn rust_type_for_wasmtime(resolve: &Resolve, wt: WasmtimeTypes<'_>, ty: &Type) -> String {
    render_type(resolve, &wt, ty)
}

fn type_has_list(resolve: &Resolve, ty: &Type) -> bool {
    let any = |t: &Option<Type>| t.is_some_and(|t| type_has_list(resolve, &t));
    match ty {
        Type::String => true,
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::List(_) | TypeDefKind::Map(_, _) => true,
            TypeDefKind::Type(inner) | TypeDefKind::Option(inner) => type_has_list(resolve, inner),
            TypeDefKind::Record(r) => r.fields.iter().any(|f| type_has_list(resolve, &f.ty)),
            TypeDefKind::Variant(v) => v.cases.iter().any(|c| any(&c.ty)),
            TypeDefKind::Result(r) => any(&r.ok) || any(&r.err),
            TypeDefKind::Tuple(t) => t.types.iter().any(|t| type_has_list(resolve, t)),
            _ => false,
        },
        _ => false,
    }
}

fn wasmtime_param_type(resolve: &Resolve, wt: WasmtimeTypes<'_>, ty: &Type) -> String {
    let recurse = |t: &Type| wasmtime_param_type(resolve, wt, t);
    let optional = |t: &Option<Type>| t.map_or_else(|| "()".to_string(), |t| recurse(&t));
    if matches!(ty, Type::String) {
        return "&str".to_string();
    }
    let Type::Id(id) = ty else {
        return rust_type_for_wasmtime(resolve, wt, ty);
    };
    let def = &resolve.types[*id];
    if def.name.is_some() {
        return if type_has_list(resolve, ty) {
            format!("&{}", rust_type_for_wasmtime(resolve, wt, ty))
        } else {
            rust_type_for_wasmtime(resolve, wt, ty)
        };
    }
    match &def.kind {
        TypeDefKind::Type(inner) => recurse(inner),
        TypeDefKind::List(elem) => format!("&[{}]", rust_type_for_wasmtime(resolve, wt, elem)),
        TypeDefKind::Option(inner) => format!("::std::option::Option<{}>", recurse(inner)),
        TypeDefKind::Result(r) => format!(
            "::std::result::Result<{}, {}>",
            optional(&r.ok),
            optional(&r.err)
        ),
        TypeDefKind::Tuple(t) => {
            let parts: Vec<String> = t.types.iter().map(recurse).collect();
            if parts.len() == 1 {
                format!("({},)", parts[0])
            } else {
                format!("({})", parts.join(", "))
            }
        }
        _ => rust_type_for_wasmtime(resolve, wt, ty),
    }
}

fn find_world_owned_named_type(
    resolve: &Resolve,
    ty: Type,
    visited: &mut HashSet<TypeId>,
) -> Option<(String, WorldId)> {
    let Type::Id(id) = ty else {
        return None;
    };
    let target = dealias_id(resolve, id);
    if !visited.insert(target) {
        return None;
    }
    let def = &resolve.types[target];
    if let TypeOwner::World(world_id) = def.owner
        && let Some(name) = &def.name
    {
        return Some((name.clone(), world_id));
    }
    match &def.kind {
        TypeDefKind::Record(r) => r
            .fields
            .iter()
            .find_map(|f| find_world_owned_named_type(resolve, f.ty, visited)),
        TypeDefKind::Variant(v) => v.cases.iter().find_map(|c| {
            c.ty.and_then(|t| find_world_owned_named_type(resolve, t, visited))
        }),
        TypeDefKind::Option(inner)
        | TypeDefKind::List(inner)
        | TypeDefKind::FixedLengthList(inner, _)
        | TypeDefKind::Type(inner) => find_world_owned_named_type(resolve, *inner, visited),
        TypeDefKind::Result(r) => {
            r.ok.and_then(|t| find_world_owned_named_type(resolve, t, visited))
                .or_else(|| {
                    r.err
                        .and_then(|t| find_world_owned_named_type(resolve, t, visited))
                })
        }
        TypeDefKind::Tuple(t) => t
            .types
            .iter()
            .find_map(|elem| find_world_owned_named_type(resolve, *elem, visited)),
        TypeDefKind::Map(k, v) => find_world_owned_named_type(resolve, *k, visited)
            .or_else(|| find_world_owned_named_type(resolve, *v, visited)),
        TypeDefKind::Future(inner) | TypeDefKind::Stream(inner) => {
            inner.and_then(|t| find_world_owned_named_type(resolve, t, visited))
        }
        TypeDefKind::Handle(Handle::Own(res) | Handle::Borrow(res)) => {
            find_world_owned_named_type(resolve, Type::Id(*res), visited)
        }
        TypeDefKind::Enum(_)
        | TypeDefKind::Flags(_)
        | TypeDefKind::Resource
        | TypeDefKind::Unknown => None,
    }
}

fn resolve_alias(resolve: &Resolve, mut ty: Type) -> Type {
    while let Type::Id(id) = ty {
        match &resolve.types[id].kind {
            TypeDefKind::Type(inner) => ty = *inner,
            _ => break,
        }
    }
    ty
}

fn conv_fn_suffix(resolve: &Resolve, id: TypeId) -> String {
    abi_gen::type_name(resolve, id).to_snake_case()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Dir {
    ToWasmtime,
    ToNative,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Src {
    Owned,
    Borrowed,
    ThroughRef,
}

/// Converts `expr` between Wasmtime's generated types and `native`'s generated types,
/// delegating named types to their single shared `__to_wt_*` / `__to_native_*` helper.
fn convert_expr(
    resolve: &Resolve,
    ty: &Type,
    expr: &str,
    dir: Dir,
    src: Src,
    concurrent: bool,
) -> String {
    if src == Src::Borrowed
        && matches!(ty, Type::Id(id) if resolve.types[*id].name.is_some())
        && type_has_list(resolve, ty)
    {
        return convert_expr(resolve, ty, expr, dir, Src::ThroughRef, concurrent);
    }

    let resolved = resolve_alias(resolve, *ty);
    let Type::Id(id) = resolved else {
        return match (src, resolved) {
            (Src::Borrowed | Src::ThroughRef, Type::String) => format!("({expr}).to_owned()"),
            (Src::ThroughRef, _) => format!("(*{expr})"),
            _ => expr.to_string(),
        };
    };

    match &resolve.types[id].kind {
        TypeDefKind::Record(_)
        | TypeDefKind::Variant(_)
        | TypeDefKind::Enum(_)
        | TypeDefKind::Flags(_) => {
            let suffix = conv_fn_suffix(resolve, id);
            match (dir, src) {
                (Dir::ToWasmtime, _) => format!("__to_wt_{suffix}({expr})"),
                (Dir::ToNative, Src::ThroughRef) => format!("__to_native_ref_{suffix}({expr})"),
                (Dir::ToNative, Src::Owned | Src::Borrowed) => {
                    format!("__to_native_{suffix}({expr})")
                }
            }
        }
        TypeDefKind::Type(inner) => convert_expr(resolve, inner, expr, dir, src, concurrent),
        TypeDefKind::Handle(handle) => match dir {
            Dir::ToWasmtime => {
                let ctor = match handle {
                    Handle::Own(_) => "new_own",
                    Handle::Borrow(_) => "new_borrow",
                };
                let to_rep = if concurrent {
                    "tagged_handle_to_rep"
                } else {
                    "handle_to_rep"
                };
                format!(
                    "::pumpkin_native_wit::wasmtime::component::Resource::{ctor}(\
                     ::pumpkin_native_wit::{to_rep}(({expr}) as u32))"
                )
            }
            Dir::ToNative => {
                let to_handle = match (concurrent, handle) {
                    (true, Handle::Own(_)) => "concurrent_rep_to_handle",
                    (true, Handle::Borrow(_)) => "concurrent_rep_to_borrow_handle",
                    (false, Handle::Own(_)) => "rep_to_handle",
                    (false, Handle::Borrow(_)) => "rep_to_borrow_handle",
                };
                format!("::pumpkin_native_wit::{to_handle}(({expr}).rep())")
            }
        },
        TypeDefKind::List(elem) => {
            let (elem_src, iter) = if src == Src::Owned {
                (Src::Owned, "into_iter")
            } else {
                (Src::ThroughRef, "iter")
            };
            let inner = convert_expr(resolve, elem, "x", dir, elem_src, concurrent);
            format!("({expr}).{iter}().map(|x| {inner}).collect::<::std::vec::Vec<_>>()")
        }
        TypeDefKind::Option(elem) => {
            let inner = convert_expr(resolve, elem, "x", dir, src, concurrent);
            let as_ref = if src == Src::ThroughRef {
                ".as_ref()"
            } else {
                ""
            };
            format!("({expr}){as_ref}.map(|x| {inner})")
        }
        TypeDefKind::Result(r) => convert_result_expr(resolve, r, expr, dir, src, concurrent),
        TypeDefKind::Tuple(t) => convert_tuple_expr(resolve, t, expr, dir, src, concurrent),
        _ => expr.to_string(),
    }
}

fn convert_result_expr(
    resolve: &Resolve,
    r: &wit_parser::Result_,
    expr: &str,
    dir: Dir,
    src: Src,
    concurrent: bool,
) -> String {
    let ok = r.ok.as_ref().map_or_else(
        || "()".to_string(),
        |t| convert_expr(resolve, t, "x", dir, src, concurrent),
    );
    let err = r.err.as_ref().map_or_else(
        || "()".to_string(),
        |t| convert_expr(resolve, t, "x", dir, src, concurrent),
    );
    let as_ref = if src == Src::ThroughRef {
        ".as_ref()"
    } else {
        ""
    };
    let ok_bind = if r.ok.is_some() { "x" } else { "_" };
    let err_bind = if r.err.is_some() { "x" } else { "_" };
    format!(
        "match ({expr}){as_ref} {{ \
         ::core::result::Result::Ok({ok_bind}) => ::core::result::Result::Ok({ok}), \
         ::core::result::Result::Err({err_bind}) => ::core::result::Result::Err({err}), \
         }}"
    )
}

fn convert_tuple_expr(
    resolve: &Resolve,
    t: &Tuple,
    expr: &str,
    dir: Dir,
    src: Src,
    concurrent: bool,
) -> String {
    let parts: Vec<String> = t
        .types
        .iter()
        .enumerate()
        .map(|(i, elem)| {
            let sub = if src == Src::ThroughRef {
                format!("(&({expr}).{i})")
            } else {
                format!("({expr}).{i}")
            };
            convert_expr(resolve, elem, &sub, dir, src, concurrent)
        })
        .collect();
    if parts.len() == 1 {
        format!("({},)", parts[0])
    } else {
        format!("({})", parts.join(", "))
    }
}

fn convert_body(
    resolve: &Resolve,
    wt: WasmtimeTypes<'_>,
    id: TypeId,
    expr: &str,
    dir: Dir,
    src: Src,
    concurrent: bool,
) -> String {
    let project = |sub: String| {
        if src == Src::ThroughRef {
            format!("(&{sub})")
        } else {
            sub
        }
    };
    let inner_src = if src == Src::ThroughRef {
        Src::ThroughRef
    } else {
        Src::Owned
    };

    let native_camel = abi_gen::type_name(resolve, id);
    let wasmtime_camel = format!(
        "super::{}::{}",
        wt.owner_path(resolve, id),
        wasmtime_type_name(resolve, id)
    );
    let (from, to) = match dir {
        Dir::ToWasmtime => (native_camel, wasmtime_camel),
        Dir::ToNative => (wasmtime_camel, native_camel),
    };

    match &resolve.types[id].kind {
        TypeDefKind::Record(r) => {
            let inits: Vec<String> = r
                .fields
                .iter()
                .map(|f| {
                    let field = to_rust_ident(&f.name);
                    let sub = project(format!("{expr}.{field}"));
                    let converted = convert_expr(resolve, &f.ty, &sub, dir, inner_src, concurrent);
                    format!("{field}: {converted}")
                })
                .collect();
            format!("{to} {{ {} }}", inits.join(", "))
        }
        TypeDefKind::Variant(v) => {
            let arms: Vec<String> = v
                .cases
                .iter()
                .map(|c| {
                    let case = c.name.to_upper_camel_case();
                    c.ty.as_ref().map_or_else(
                        || format!("{from}::{case} => {to}::{case},"),
                        |t| {
                            let inner = convert_expr(resolve, t, "val", dir, inner_src, concurrent);
                            format!("{from}::{case}(val) => {to}::{case}({inner}),")
                        },
                    )
                })
                .collect();
            format!("match {expr} {{ {} }}", arms.join(" "))
        }
        TypeDefKind::Enum(e) => {
            let arms: Vec<String> = e
                .cases
                .iter()
                .map(|c| {
                    let case = c.name.to_upper_camel_case();
                    format!("{from}::{case} => {to}::{case},")
                })
                .collect();
            format!("match {expr} {{ {} }}", arms.join(" "))
        }
        TypeDefKind::Flags(f) => match dir {
            Dir::ToWasmtime => {
                let sets: Vec<String> = f
                    .flags
                    .iter()
                    .enumerate()
                    .map(|(i, flag)| {
                        let konst = flag.name.to_shouty_snake_case();
                        format!(
                            "if words[{}] & (1u32 << {}) != 0 {{ out |= {to}::{konst}; }}",
                            i / 32,
                            i % 32
                        )
                    })
                    .collect();
                format!(
                    "{{ let words = {expr}; let mut out = {to}::empty(); {} out }}",
                    sets.join(" ")
                )
            }
            Dir::ToNative => format!("({expr}).as_array()"),
        },
        _ => expr.to_string(),
    }
}

fn native_helper_signature(resolve: &Resolve, f: &ImportFn) -> (String, String) {
    let (params, ret) = f.source.as_ref().map_or_else(
        || ("rep: u32".to_string(), None),
        |func| {
            let params = func
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let name = if i == 0
                        && matches!(
                            func.kind,
                            FunctionKind::Method(_) | FunctionKind::AsyncMethod(_)
                        ) {
                        "rep".to_string()
                    } else {
                        to_rust_ident(&p.name)
                    };
                    format!("{name}: {}", abi_gen::rust_type_for_native(resolve, &p.ty))
                })
                .collect::<Vec<_>>()
                .join(", ");
            let ret = func
                .result
                .map(|ty| abi_gen::rust_type_for_native(resolve, &ty));
            (params, ret)
        },
    );
    let ret = match ret.as_deref() {
        None | Some("()") => String::new(),
        Some(ty) => format!(" -> {ty}"),
    };
    (params, ret)
}

fn unsupported_kind(resolve: &Resolve, ty: &Type) -> Option<&'static str> {
    let Type::Id(id) = resolve_alias(resolve, *ty) else {
        return None;
    };
    match &resolve.types[id].kind {
        TypeDefKind::Type(inner) => unsupported_kind(resolve, inner),
        TypeDefKind::Record(r) => r
            .fields
            .iter()
            .find_map(|f| unsupported_kind(resolve, &f.ty)),
        TypeDefKind::Variant(v) => v
            .cases
            .iter()
            .filter_map(|c| c.ty.as_ref())
            .find_map(|t| unsupported_kind(resolve, t)),
        TypeDefKind::Enum(_) | TypeDefKind::Handle(_) | TypeDefKind::Flags(_) => None,
        TypeDefKind::Tuple(t) => t.types.iter().find_map(|f| unsupported_kind(resolve, f)),
        TypeDefKind::List(elem) | TypeDefKind::Option(elem) => unsupported_kind(resolve, elem),
        TypeDefKind::Result(r) => {
            r.ok.iter()
                .chain(r.err.iter())
                .find_map(|t| unsupported_kind(resolve, t))
        }
        other => Some(other.as_str()),
    }
}

pub fn rust_function_name(func: &Function) -> String {
    match func.kind {
        FunctionKind::Constructor(_) => "new".to_string(),
        _ => to_rust_ident(func.item_name()),
    }
}
