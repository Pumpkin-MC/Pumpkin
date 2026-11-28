//! Rust identifier formatting and WIT-to-Rust type rendering.

use heck::{ToSnakeCase, ToUpperCamelCase};
use wit_parser::{Flags, Handle, InterfaceId, Resolve, Type, TypeDefKind, TypeId, TypeOwner};

use crate::types::TypeInfo;

pub trait RustGenerator<'a> {
    fn resolve(&self) -> &'a Resolve;
    fn push_str(&mut self, s: &str);
    fn info(&self, ty: TypeId) -> TypeInfo;
    fn path_to_interface(&self, interface: InterfaceId) -> Option<String>;

    fn print_ty(&mut self, ty: &Type) {
        let rendered = self.ty(ty);
        self.push_str(&rendered);
    }

    fn ty(&self, ty: &Type) -> String {
        match ty {
            Type::Id(t) => self.tyid(*t),
            Type::Bool => "bool".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::S8 => "i8".to_string(),
            Type::S16 => "i16".to_string(),
            Type::S32 => "i32".to_string(),
            Type::S64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "::std::string::String".to_string(),
            Type::ErrorContext => "()".to_string(),
        }
    }

    fn optional_ty(&self, ty: Option<&Type>) -> String {
        ty.map_or_else(|| "()".to_string(), |t| self.ty(t))
    }

    fn tyid(&self, id: TypeId) -> String {
        let ty = &self.resolve().types[id];
        if let TypeDefKind::Flags(f) = &ty.kind {
            return format!("[u32; {}]", f.repr().count());
        }
        if ty.name.is_some() {
            return self.type_name_in_interface(ty.owner, &self.type_name(id));
        }

        match &ty.kind {
            TypeDefKind::List(t) => self.list(t),
            TypeDefKind::Option(t) => format!("::std::option::Option<{}>", self.ty(t)),
            TypeDefKind::Result(r) => {
                let ok = self.optional_ty(r.ok.as_ref());
                let err = self.optional_ty(r.err.as_ref());
                format!("::std::result::Result<{ok}, {err}>")
            }
            TypeDefKind::Tuple(t) => {
                let mut out = "(".to_string();
                for elem in &t.types {
                    out.push_str(&self.ty(elem));
                    out.push(',');
                }
                out.push(')');
                out
            }
            TypeDefKind::Handle(_) => "u32".to_string(),
            TypeDefKind::Type(t) => self.ty(t),
            TypeDefKind::Map(k, v) => {
                format!(
                    "::std::collections::HashMap<{}, {}>",
                    self.ty(k),
                    self.ty(v)
                )
            }
            _ => "()".to_string(),
        }
    }

    fn list(&self, ty: &Type) -> String {
        format!("::std::vec::Vec<{}>", self.ty(ty))
    }

    fn print_list(&mut self, ty: &Type) {
        let s = self.list(ty);
        self.push_str(&s);
    }

    fn type_name(&self, id: TypeId) -> String {
        let ty = &self.resolve().types[id];
        to_rust_upper_camel_case(ty.name.as_deref().unwrap_or("unnamed"))
    }

    fn type_name_in_interface(&self, owner: TypeOwner, name: &str) -> String {
        let mut out = String::new();
        if let TypeOwner::Interface(id) = owner
            && let Some(path) = self.path_to_interface(id)
        {
            out.push_str(&path);
            out.push_str("::");
        }
        out.push_str(name);
        out
    }

    fn print_rust_enum_debug<'b>(
        &mut self,
        name: &str,
        cases: impl IntoIterator<Item = (String, Option<&'b Type>)>,
    ) {
        self.push_str("impl ::core::fmt::Debug for ");
        self.push_str(name);
        self.push_str(" {\n");
        self.push_str(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {\n",
        );
        self.push_str("match self {\n");
        for (case_name, payload) in cases {
            self.push_str(name);
            self.push_str("::");
            self.push_str(&case_name);
            if payload.is_some() {
                self.push_str("(e)");
            }
            self.push_str(" => f.debug_tuple(\"");
            self.push_str(name);
            self.push_str("::");
            self.push_str(&case_name);
            self.push_str("\")");
            if payload.is_some() {
                self.push_str(".field(e)");
            }
            self.push_str(".finish(),\n");
        }
        self.push_str("}\n}\n}\n");
    }
}

/// Strategy for rendering named types, flags, and resource handles.
pub trait TypeNames {
    fn named(&self, resolve: &Resolve, id: TypeId) -> String;
    fn flags(&self, resolve: &Resolve, id: TypeId, flags: &Flags) -> String;
    fn handle(&self, resolve: &Resolve, handle: &Handle) -> String;
}

pub fn render_type(resolve: &Resolve, names: &dyn TypeNames, ty: &Type) -> String {
    match ty {
        Type::Bool => "bool".to_string(),
        Type::U8 => "u8".to_string(),
        Type::U16 => "u16".to_string(),
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::S8 => "i8".to_string(),
        Type::S16 => "i16".to_string(),
        Type::S32 => "i32".to_string(),
        Type::S64 => "i64".to_string(),
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Char => "char".to_string(),
        Type::String => "::std::string::String".to_string(),
        Type::ErrorContext => "()".to_string(),
        Type::Id(id) => match &resolve.types[*id].kind {
            TypeDefKind::Record(_) | TypeDefKind::Variant(_) | TypeDefKind::Enum(_) => {
                names.named(resolve, *id)
            }
            TypeDefKind::Flags(flags) => names.flags(resolve, *id, flags),
            TypeDefKind::Type(inner) => render_type(resolve, names, inner),
            TypeDefKind::Option(inner) => {
                format!(
                    "::std::option::Option<{}>",
                    render_type(resolve, names, inner)
                )
            }
            TypeDefKind::Result(r) => {
                let ok =
                    r.ok.as_ref()
                        .map_or_else(|| "()".to_string(), |t| render_type(resolve, names, t));
                let err = r
                    .err
                    .as_ref()
                    .map_or_else(|| "()".to_string(), |t| render_type(resolve, names, t));
                format!("::std::result::Result<{ok}, {err}>")
            }
            TypeDefKind::List(elem) => {
                format!("::std::vec::Vec<{}>", render_type(resolve, names, elem))
            }
            TypeDefKind::Tuple(tuple) => {
                let mut out = "(".to_string();
                for elem in &tuple.types {
                    out.push_str(&render_type(resolve, names, elem));
                    out.push(',');
                }
                out.push(')');
                out
            }
            TypeDefKind::Handle(handle) => names.handle(resolve, handle),
            TypeDefKind::Map(k, v) => format!(
                "::std::collections::HashMap<{}, {}>",
                render_type(resolve, names, k),
                render_type(resolve, names, v),
            ),
            _ => "()".to_string(),
        },
    }
}

pub fn to_rust_ident(name: &str) -> String {
    match name {
        "as" => "as_".into(),
        "break" => "break_".into(),
        "const" => "const_".into(),
        "continue" => "continue_".into(),
        "crate" => "crate_".into(),
        "else" => "else_".into(),
        "enum" => "enum_".into(),
        "extern" => "extern_".into(),
        "false" => "false_".into(),
        "fn" => "fn_".into(),
        "for" => "for_".into(),
        "if" => "if_".into(),
        "impl" => "impl_".into(),
        "in" => "in_".into(),
        "let" => "let_".into(),
        "loop" => "loop_".into(),
        "match" => "match_".into(),
        "mod" => "mod_".into(),
        "move" => "move_".into(),
        "mut" => "mut_".into(),
        "pub" => "pub_".into(),
        "ref" => "ref_".into(),
        "return" => "return_".into(),
        "self" => "self_".into(),
        "static" => "static_".into(),
        "struct" => "struct_".into(),
        "super" => "super_".into(),
        "trait" => "trait_".into(),
        "true" => "true_".into(),
        "type" => "type_".into(),
        "unsafe" => "unsafe_".into(),
        "use" => "use_".into(),
        "where" => "where_".into(),
        "while" => "while_".into(),
        "async" => "async_".into(),
        "await" => "await_".into(),
        "dyn" => "dyn_".into(),
        "abstract" => "abstract_".into(),
        "become" => "become_".into(),
        "box" => "box_".into(),
        "do" => "do_".into(),
        "final" => "final_".into(),
        "macro" => "macro_".into(),
        "override" => "override_".into(),
        "priv" => "priv_".into(),
        "typeof" => "typeof_".into(),
        "unsized" => "unsized_".into(),
        "virtual" => "virtual_".into(),
        "yield" => "yield_".into(),
        "try" => "try_".into(),
        "gen" => "gen_".into(),
        s => s.to_snake_case(),
    }
}

pub fn to_rust_upper_camel_case(name: &str) -> String {
    match name {
        "Self" => "Self_".into(),
        s => s.to_upper_camel_case(),
    }
}
