//! WIT type-property analysis (`has_list`, `has_handle`, `error`, etc.).

use std::collections::HashMap;
use std::ops::{BitOr, BitOrAssign};

use wit_parser::{Function, Resolve, Result_, Type, TypeDefKind, TypeId, WorldId, WorldItem};

#[derive(Default)]
pub struct Types {
    type_info: HashMap<TypeId, TypeInfo>,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct TypeInfo {
    pub has_list: bool,
    pub has_handle: bool,
    pub has_future_or_stream: bool,
    pub error: bool,
}

impl BitOrAssign for TypeInfo {
    fn bitor_assign(&mut self, rhs: Self) {
        self.has_list |= rhs.has_list;
        self.has_handle |= rhs.has_handle;
        self.has_future_or_stream |= rhs.has_future_or_stream;
        self.error |= rhs.error;
    }
}

impl BitOr for TypeInfo {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl Types {
    pub fn analyze(&mut self, resolve: &Resolve, world: WorldId) {
        for (t, _) in &resolve.types {
            self.type_id_info(resolve, t);
        }

        let world = &resolve.worlds[world];
        for item in world.imports.values().chain(world.exports.values()) {
            match item {
                WorldItem::Function(f) => self.type_info_func(resolve, f),
                WorldItem::Interface { id, .. } => {
                    let iface = &resolve.interfaces[*id];
                    for &t in iface.types.values() {
                        self.type_id_info(resolve, t);
                    }
                    for f in iface.functions.values() {
                        self.type_info_func(resolve, f);
                    }
                }
                WorldItem::Type { id, .. } => {
                    self.type_id_info(resolve, *id);
                }
            }
        }
    }

    fn type_info_func(&mut self, resolve: &Resolve, func: &Function) {
        for param in &func.params {
            self.type_info(resolve, &param.ty);
        }
        if let Some(ty) = &func.result {
            self.type_info(resolve, ty);
        }

        if let Some(Type::Id(id)) = func.result
            && let TypeDefKind::Result(Result_ {
                err: Some(Type::Id(err_id)),
                ..
            }) = &resolve.types[id].kind
        {
            let resolved = dealias_id(resolve, *err_id);
            if let Some(info) = self.type_info.get_mut(&resolved) {
                info.error = true;
            }
        }
    }

    pub fn get(&self, id: TypeId) -> TypeInfo {
        self.type_info.get(&id).copied().unwrap_or_default()
    }

    fn type_id_info(&mut self, resolve: &Resolve, ty: TypeId) -> TypeInfo {
        if let Some(info) = self.type_info.get(&ty) {
            return *info;
        }
        let mut info = TypeInfo::default();
        match &resolve.types[ty].kind {
            TypeDefKind::Record(r) => {
                for field in &r.fields {
                    info |= self.type_info(resolve, &field.ty);
                }
            }
            TypeDefKind::Tuple(t) => {
                for ty in &t.types {
                    info |= self.type_info(resolve, ty);
                }
            }
            TypeDefKind::Flags(_)
            | TypeDefKind::Enum(_)
            | TypeDefKind::Resource
            | TypeDefKind::Unknown => {}
            TypeDefKind::Variant(v) => {
                for case in &v.cases {
                    info |= self.optional_type_info(resolve, case.ty.as_ref());
                }
            }
            TypeDefKind::List(elem) => {
                info = self.type_info(resolve, elem);
                info.has_list = true;
            }
            TypeDefKind::FixedLengthList(elem, _) => {
                info = self.type_info(resolve, elem);
            }
            TypeDefKind::Map(k, v) => {
                info = self.type_info(resolve, k);
                info |= self.type_info(resolve, v);
                info.has_list = true;
            }
            TypeDefKind::Type(inner) | TypeDefKind::Option(inner) => {
                info = self.type_info(resolve, inner);
            }
            TypeDefKind::Result(r) => {
                info = self.optional_type_info(resolve, r.ok.as_ref());
                info |= self.optional_type_info(resolve, r.err.as_ref());
            }
            TypeDefKind::Future(inner) | TypeDefKind::Stream(inner) => {
                info = self.optional_type_info(resolve, inner.as_ref());
                info.has_future_or_stream = true;
            }
            TypeDefKind::Handle(_) => info.has_handle = true,
        }
        self.type_info.insert(ty, info);
        info
    }

    fn type_info(&mut self, resolve: &Resolve, ty: &Type) -> TypeInfo {
        match ty {
            Type::String => TypeInfo {
                has_list: true,
                ..TypeInfo::default()
            },
            Type::Id(id) => self.type_id_info(resolve, *id),
            _ => TypeInfo::default(),
        }
    }

    fn optional_type_info(&mut self, resolve: &Resolve, ty: Option<&Type>) -> TypeInfo {
        ty.map_or_else(TypeInfo::default, |t| self.type_info(resolve, t))
    }
}

impl TypeInfo {
    pub const fn is_copy(self) -> bool {
        !self.has_list && !self.has_handle && !self.has_future_or_stream
    }

    pub const fn is_clone(self) -> bool {
        !self.has_handle && !self.has_future_or_stream
    }
}

pub fn dealias_id(resolve: &Resolve, mut id: TypeId) -> TypeId {
    while let TypeDefKind::Type(Type::Id(inner)) = resolve.types[id].kind {
        id = inner;
    }
    id
}
