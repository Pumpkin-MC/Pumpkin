use std::{fs, path::Path};

use heck::ToPascalCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use wit_parser::{Resolve, Type, TypeDefKind, TypeId, TypeOwner};

use super::{WIT_OUT_DIR, format_code};

const CORE_EVENTS_OUT: &str = "../../crates/pumpkin-core/src/plugin/api/events/generated.rs";
const HOST_EVENTS_OUT: &str = "../../crates/pumpkin-wasm-host-v0_2/src/events/generated.rs";

const MANUAL_PACKET_EVENTS: &[&str] = &[
    "packet-received-event",
    "packet-sent-event",
    "java-connection-packet-received-event",
    "java-connection-packet-sent-event",
    "bedrock-connection-packet-received-event",
    "bedrock-connection-packet-sent-event",
];

const SKIPPED_LOCAL_ENUMS: &[&str] = &[
    "event-priority",
    "event-type",
    "java-connection-state",
    "entity-interaction-action",
];

const SKIPPED_HELPER_RECORDS: &[&str] = &["raw-packet"];

#[derive(Clone)]
enum ResolvedType {
    CopyPrimitive(TokenStream),
    String,
    BlockPos,
    Position,
    BlockName,
    BlockStateId,
    EntityTypeName,
    SocketAddress,
    RawBytes,
    Hand,
    GameMode,
    ClickType,
    EntityInteractionAction,
    Uuid,
    Screen,
    DamageType,
    Enchantment,
    BlockDirection,
    DyeColor,
    LocalEnum(String),
    HelperRecord {
        wit_name: String,
        fields: Vec<(String, ResolvedType)>,
    },
    PlayerRes,
    WorldRes,
    EntityRes,
    TextComponentRes,
    ItemStackRes,
    Dialog,
    PacketOnlyNoCleanup,
    Option(Box<ResolvedType>),
    List(Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
}

impl ResolvedType {
    fn is_copy(&self) -> bool {
        match self {
            Self::CopyPrimitive(_)
            | Self::BlockPos
            | Self::Position
            | Self::BlockName
            | Self::BlockStateId
            | Self::EntityTypeName
            | Self::SocketAddress
            | Self::Hand
            | Self::GameMode
            | Self::ClickType
            | Self::EntityInteractionAction
            | Self::Uuid
            | Self::Screen
            | Self::DamageType
            | Self::Enchantment
            | Self::BlockDirection
            | Self::DyeColor
            | Self::LocalEnum(_) => true,
            Self::HelperRecord { fields, .. } => fields.iter().all(|(_, fty)| fty.is_copy()),
            Self::Option(inner) => inner.is_copy(),
            Self::Tuple(elems) => elems.iter().all(Self::is_copy),
            Self::String
            | Self::RawBytes
            | Self::PlayerRes
            | Self::WorldRes
            | Self::EntityRes
            | Self::TextComponentRes
            | Self::ItemStackRes
            | Self::Dialog
            | Self::PacketOnlyNoCleanup
            | Self::List(_) => false,
        }
    }

    fn is_identity_primitive(&self) -> bool {
        matches!(self, Self::CopyPrimitive(_))
    }

    fn is_identity_clone(&self) -> bool {
        matches!(self, Self::CopyPrimitive(_) | Self::String)
    }

    fn supports_eq_debug(&self) -> bool {
        match self {
            Self::Enchantment => false,
            Self::HelperRecord { fields, .. } => {
                fields.iter().all(|(_, fty)| fty.supports_eq_debug())
            }
            Self::Option(inner) | Self::List(inner) => inner.supports_eq_debug(),
            Self::Tuple(elems) => elems.iter().all(Self::supports_eq_debug),
            _ => true,
        }
    }

    fn needs_state(&self) -> bool {
        match self {
            Self::PlayerRes
            | Self::WorldRes
            | Self::EntityRes
            | Self::TextComponentRes
            | Self::ItemStackRes
            | Self::Dialog => true,
            Self::HelperRecord { fields, .. } => fields.iter().any(|(_, fty)| fty.needs_state()),
            Self::Option(inner) | Self::List(inner) => inner.needs_state(),
            Self::Tuple(elems) => elems.iter().any(Self::needs_state),
            _ => false,
        }
    }

    fn needs_cleanup(&self) -> bool {
        self.needs_state()
    }

    fn core_type(&self) -> TokenStream {
        match self {
            Self::CopyPrimitive(ts) => ts.clone(),
            Self::String => quote! { String },
            Self::BlockPos => quote! { pumpkin_util::math::position::BlockPos },
            Self::Position => quote! { pumpkin_util::math::vector3::Vector3<f64> },
            Self::BlockName => quote! { &'static pumpkin_data::Block },
            Self::BlockStateId => quote! { pumpkin_data::BlockStateId },
            Self::EntityTypeName => quote! { &'static pumpkin_data::entity::EntityType },
            Self::SocketAddress => quote! { std::net::SocketAddr },
            Self::RawBytes => quote! { bytes::Bytes },
            Self::Hand => quote! { pumpkin_util::Hand },
            Self::GameMode => quote! { pumpkin_util::GameMode },
            Self::ClickType => quote! { pumpkin_inventory::screen_handler::ClickType },
            Self::EntityInteractionAction => {
                quote! { pumpkin_protocol::java::server::play::ActionType }
            }
            Self::Uuid => quote! { uuid::Uuid },
            Self::Screen => quote! { pumpkin_data::screen::WindowType },
            Self::DamageType => quote! { pumpkin_data::damage::DamageType },
            Self::Enchantment => quote! { &'static pumpkin_data::Enchantment },
            Self::BlockDirection => quote! { pumpkin_data::BlockDirection },
            Self::DyeColor => quote! { pumpkin_data::dye_color::DyeColor },
            Self::LocalEnum(wit_name) => {
                let ident = pascal_ident(wit_name);
                quote! { #ident }
            }
            Self::HelperRecord { wit_name, .. } => {
                let ident = pascal_ident(wit_name);
                quote! { #ident }
            }
            Self::PlayerRes => quote! { std::sync::Arc<crate::entity::player::Player> },
            Self::WorldRes => quote! { std::sync::Arc<crate::world::World> },
            Self::EntityRes => quote! { std::sync::Arc<dyn crate::entity::EntityBase> },
            Self::TextComponentRes => quote! { pumpkin_util::text::TextComponent },
            Self::ItemStackRes => quote! { pumpkin_data::item_stack::ItemStack },
            Self::Dialog => quote! { pumpkin_protocol::java::client::dialog::Dialog },
            Self::PacketOnlyNoCleanup => {
                panic!("PacketOnlyNoCleanup cannot be emitted in core_type")
            }
            Self::Option(inner) => {
                let inner_ts = inner.core_type();
                quote! { Option<#inner_ts> }
            }
            Self::List(inner) => {
                let inner_ts = inner.core_type();
                quote! { Vec<#inner_ts> }
            }
            Self::Tuple(elems) => {
                let elem_ts = elems.iter().map(Self::core_type);
                quote! { (#(#elem_ts),*) }
            }
        }
    }

    fn to_wasm(&self, expr: &TokenStream, is_ref: bool) -> TokenStream {
        let val = if is_ref {
            quote! { *#expr }
        } else {
            expr.clone()
        };
        let r = if is_ref {
            expr.clone()
        } else {
            quote! { &#expr }
        };

        match self {
            Self::CopyPrimitive(_) => val,
            Self::String => {
                if is_ref {
                    quote! { (#expr).clone() }
                } else {
                    expr.clone()
                }
            }
            Self::BlockPos => quote! { to_wasm_block_position(#val) },
            Self::Position => quote! { to_wasm_position(#val) },
            Self::BlockName => quote! { to_wasm_block_name(#val) },
            Self::BlockStateId => quote! { (#val).as_u16() },
            Self::EntityTypeName => quote! { to_wasm_entity_type(#val) },
            Self::SocketAddress => quote! { (#r).to_string() },
            Self::RawBytes => quote! { (#r).to_vec() },
            Self::Hand => quote! { to_wasm_hand(#val) },
            Self::GameMode => quote! { to_wasm_game_mode(#val) },
            Self::ClickType => quote! { to_wasm_click_type(#val) },
            Self::EntityInteractionAction => quote! { to_wasm_entity_interaction_action(#val) },
            Self::Uuid => {
                quote! { <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(#r) }
            }
            Self::Screen => quote! { crate::gui::to_wit_screen(#val) },
            Self::DamageType => quote! { crate::living_entity::to_wit_damage_type(#r) },
            Self::Enchantment => quote! { crate::item_stack::to_wit_enchantment(#val) },
            Self::BlockDirection => quote! { crate::mob::to_wit_block_direction(#val) },
            Self::DyeColor => quote! { to_wit_event_dye_color(#val) },
            Self::LocalEnum(wit_name) => {
                let fn_ident = format_ident!("to_wit_{}", snake_str(wit_name));
                quote! { #fn_ident(#val) }
            }
            Self::HelperRecord { wit_name, fields } => {
                let rec_ident = pascal_ident(wit_name);
                let field_inits = fields.iter().map(|(fname, fty)| {
                    let f_ident = snake_ident(fname);
                    let f_access = quote! { &rec.#f_ident };
                    let f_conv = fty.to_wasm(&f_access, true);
                    quote! { #f_ident: #f_conv }
                });
                quote! {
                    {
                        let rec = #r;
                        crate::pumpkin::plugin::event::#rec_ident {
                            #(#field_inits),*
                        }
                    }
                }
            }
            Self::PlayerRes => {
                quote! { state.add(std::sync::Arc::clone(#r)).expect("failed to add player resource") }
            }
            Self::WorldRes => {
                quote! { state.add(std::sync::Arc::clone(#r)).expect("failed to add world resource") }
            }
            Self::EntityRes => {
                quote! { state.add(std::sync::Arc::clone(#r)).expect("failed to add entity resource") }
            }
            Self::TextComponentRes => {
                quote! { state.add((#r).clone()).expect("failed to add text-component resource") }
            }
            Self::ItemStackRes => {
                quote! {
                    state
                        .add(std::sync::Arc::new(tokio::sync::Mutex::new((#r).clone())))
                        .expect("failed to add item-stack resource")
                }
            }
            Self::Dialog => {
                quote! { crate::events::dialog::protocol_dialog_to_wasm(state, #r) }
            }
            Self::PacketOnlyNoCleanup => {
                panic!("PacketOnlyNoCleanup cannot be emitted in to_wasm")
            }
            Self::Option(inner) => {
                if inner.is_identity_primitive() {
                    val
                } else if inner.is_copy() {
                    let v = quote! { v };
                    let inner_conv = inner.to_wasm(&v, false);
                    quote! { (#val).map(|v| #inner_conv) }
                } else {
                    let v = quote! { v };
                    let inner_conv = inner.to_wasm(&v, true);
                    quote! { (#r).as_ref().map(|v| #inner_conv) }
                }
            }
            Self::List(inner) => {
                if inner.is_identity_clone() {
                    quote! { (#r).clone() }
                } else if inner.is_copy() {
                    let v = quote! { v };
                    let inner_conv = inner.to_wasm(&v, false);
                    quote! { (#r).iter().copied().map(|v| #inner_conv).collect() }
                } else {
                    let v = quote! { v };
                    let inner_conv = inner.to_wasm(&v, true);
                    quote! { (#r).iter().map(|v| #inner_conv).collect() }
                }
            }
            Self::Tuple(elems) => {
                let vars: Vec<Ident> = (0..elems.len()).map(|i| format_ident!("t{i}")).collect();
                let convs = elems.iter().zip(&vars).map(|(ety, var)| {
                    let var_ts = quote! { #var };
                    ety.to_wasm(&var_ts, is_ref && !ety.is_copy())
                });
                if self.is_copy() {
                    quote! {
                        {
                            let (#(#vars),*) = #val;
                            (#(#convs),*)
                        }
                    }
                } else {
                    quote! {
                        {
                            let (#(#vars),*) = #r;
                            (#(#convs),*)
                        }
                    }
                }
            }
        }
    }

    fn from_wasm(&self, expr: &TokenStream) -> TokenStream {
        match self {
            Self::CopyPrimitive(_) | Self::String => expr.clone(),
            Self::BlockPos => quote! { from_wasm_block_position(#expr) },
            Self::Position => quote! { from_wasm_position(#expr) },
            Self::BlockName => quote! { from_wasm_block_name(&#expr) },
            Self::BlockStateId => quote! { pumpkin_data::BlockStateId::new_or_air(#expr) },
            Self::EntityTypeName => quote! { from_wasm_entity_type(&#expr) },
            Self::SocketAddress => {
                quote! {
                    (#expr)
                        .parse()
                        .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0)))
                }
            }
            Self::RawBytes => quote! { bytes::Bytes::from(#expr) },
            Self::Hand => quote! { from_wasm_hand(#expr) },
            Self::GameMode => quote! { from_wasm_game_mode(#expr) },
            Self::ClickType => quote! { from_wasm_click_type(#expr) },
            Self::EntityInteractionAction => quote! { from_wasm_entity_interaction_action(#expr) },
            Self::Uuid => {
                quote! { <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(&#expr) }
            }
            Self::Screen => quote! { crate::gui::from_wit_screen(#expr) },
            Self::DamageType => quote! { crate::living_entity::from_wit_damage_type(#expr) },
            Self::Enchantment => quote! { crate::item_stack::from_wit_enchantment(#expr) },
            Self::BlockDirection => quote! { crate::mob::from_wit_block_direction(#expr) },
            Self::DyeColor => quote! { from_wit_event_dye_color(#expr) },
            Self::LocalEnum(wit_name) => {
                let fn_ident = format_ident!("from_wit_{}", snake_str(wit_name));
                quote! { #fn_ident(#expr) }
            }
            Self::HelperRecord { wit_name, fields } => {
                let rec_ident = pascal_ident(wit_name);
                let field_inits = fields.iter().map(|(fname, fty)| {
                    let f_ident = snake_ident(fname);
                    let f_expr = quote! { rec.#f_ident };
                    let f_conv = fty.from_wasm(&f_expr);
                    quote! { #f_ident: #f_conv }
                });
                quote! {
                    {
                        let rec = #expr;
                        pumpkin_core::plugin::#rec_ident {
                            #(#field_inits),*
                        }
                    }
                }
            }
            Self::PlayerRes => quote! { consume_player(state, &#expr) },
            Self::WorldRes => quote! { consume_world(state, &#expr) },
            Self::EntityRes => quote! { consume_entity(state, &#expr) },
            Self::TextComponentRes => quote! { consume_text_component(state, &#expr) },
            Self::ItemStackRes => quote! { consume_item_stack(state, &#expr) },
            Self::Dialog => {
                quote! {
                    {
                        let dialog_ref = &#expr;
                        let converted = crate::events::dialog::protocol_dialog_from_wasm(state, dialog_ref);
                        crate::events::dialog::cleanup_dialog(state, dialog_ref);
                        converted
                    }
                }
            }
            Self::PacketOnlyNoCleanup => {
                panic!("PacketOnlyNoCleanup cannot be emitted in from_wasm")
            }
            Self::Option(inner) => {
                if inner.is_identity_clone() {
                    expr.clone()
                } else {
                    let v = quote! { v };
                    let inner_conv = inner.from_wasm(&v);
                    quote! { (#expr).map(|v| #inner_conv) }
                }
            }
            Self::List(inner) => {
                if inner.is_identity_clone() {
                    expr.clone()
                } else {
                    let v = quote! { v };
                    let inner_conv = inner.from_wasm(&v);
                    quote! { (#expr).into_iter().map(|v| #inner_conv).collect() }
                }
            }
            Self::Tuple(elems) => {
                let vars: Vec<Ident> = (0..elems.len()).map(|i| format_ident!("t{i}")).collect();
                let convs = elems.iter().zip(&vars).map(|(ety, var)| {
                    let var_ts = quote! { #var };
                    ety.from_wasm(&var_ts)
                });
                quote! {
                    {
                        let (#(#vars),*) = #expr;
                        (#(#convs),*)
                    }
                }
            }
        }
    }

    fn cleanup(&self, ref_expr: &TokenStream) -> Option<TokenStream> {
        if !self.needs_cleanup() {
            return None;
        }
        let ts = match self {
            Self::PlayerRes => quote! { cleanup_player(state, #ref_expr); },
            Self::WorldRes => quote! { cleanup_world(state, #ref_expr); },
            Self::EntityRes => quote! { cleanup_entity(state, #ref_expr); },
            Self::TextComponentRes => quote! { cleanup_text_component(state, #ref_expr); },
            Self::ItemStackRes => quote! { cleanup_item_stack(state, #ref_expr); },
            Self::Dialog => quote! { crate::events::dialog::cleanup_dialog(state, #ref_expr); },
            Self::HelperRecord { fields, .. } => {
                let stmts: Vec<TokenStream> = fields
                    .iter()
                    .filter_map(|(fname, fty)| {
                        let f_ident = snake_ident(fname);
                        let sub = quote! { &(#ref_expr).#f_ident };
                        fty.cleanup(&sub)
                    })
                    .collect();
                quote! { #(#stmts)* }
            }
            Self::Option(inner) => {
                let res_ident = quote! { res };
                let inner_stmt = inner.cleanup(&res_ident)?;
                quote! {
                    if let Some(res) = #ref_expr {
                        #inner_stmt
                    }
                }
            }
            Self::List(inner) => {
                let res_ident = quote! { res };
                let inner_stmt = inner.cleanup(&res_ident)?;
                quote! {
                    for res in #ref_expr {
                        #inner_stmt
                    }
                }
            }
            Self::Tuple(elems) => {
                let vars: Vec<Ident> = (0..elems.len()).map(|i| format_ident!("t{i}")).collect();
                let stmts: Vec<TokenStream> = elems
                    .iter()
                    .zip(&vars)
                    .filter_map(|(ety, var)| {
                        let var_ts = quote! { #var };
                        ety.cleanup(&var_ts)
                    })
                    .collect();
                quote! {
                    {
                        let (#(#vars),*) = #ref_expr;
                        #(#stmts)*
                    }
                }
            }
            _ => return None,
        };
        Some(ts)
    }
}

fn pascal_ident(wit_name: &str) -> Ident {
    format_ident!("{}", wit_name.to_pascal_case())
}

fn snake_str(wit_name: &str) -> String {
    wit_name.replace('-', "_")
}

fn snake_ident(wit_name: &str) -> Ident {
    let s = snake_str(wit_name);
    match s.as_str() {
        "type" | "move" | "ref" | "match" | "loop" | "in" | "where" | "async" | "await" | "dyn"
        | "impl" | "mod" | "pub" | "return" | "self" | "super" | "trait" | "unsafe" | "use"
        | "static" | "const" | "fn" | "let" | "mut" | "struct" | "enum" | "for" | "if" | "else"
        | "while" | "break" | "continue" | "crate" | "extern" | "false" | "true" | "as" => {
            format_ident!("r#{s}")
        }
        _ => format_ident!("{s}"),
    }
}

fn peel_named_import(resolve: &Resolve, mut id: TypeId) -> TypeId {
    while let TypeDefKind::Type(Type::Id(next_id)) = resolve.types[id].kind {
        if resolve.types[next_id].name.is_some() {
            id = next_id;
        } else {
            break;
        }
    }
    id
}

fn owner_iface_name<'a>(resolve: &'a Resolve, id: TypeId) -> &'a str {
    match resolve.types[id].owner {
        TypeOwner::Interface(iface_id) => {
            resolve.interfaces[iface_id].name.as_deref().unwrap_or("")
        }
        _ => "",
    }
}

fn resolve_wit_type(resolve: &Resolve, ty: &Type) -> ResolvedType {
    match ty {
        Type::Bool => ResolvedType::CopyPrimitive(quote! { bool }),
        Type::U8 => ResolvedType::CopyPrimitive(quote! { u8 }),
        Type::U16 => ResolvedType::CopyPrimitive(quote! { u16 }),
        Type::U32 => ResolvedType::CopyPrimitive(quote! { u32 }),
        Type::U64 => ResolvedType::CopyPrimitive(quote! { u64 }),
        Type::S8 => ResolvedType::CopyPrimitive(quote! { i8 }),
        Type::S16 => ResolvedType::CopyPrimitive(quote! { i16 }),
        Type::S32 => ResolvedType::CopyPrimitive(quote! { i32 }),
        Type::S64 => ResolvedType::CopyPrimitive(quote! { i64 }),
        Type::F32 => ResolvedType::CopyPrimitive(quote! { f32 }),
        Type::F64 => ResolvedType::CopyPrimitive(quote! { f64 }),
        Type::Char => ResolvedType::CopyPrimitive(quote! { char }),
        Type::String => ResolvedType::String,
        Type::ErrorContext => panic!("Unsupported WIT error-context in event.wit"),
        Type::Id(raw_id) => {
            let id = peel_named_import(resolve, *raw_id);
            let tdef = &resolve.types[id];
            if let Some(name) = tdef.name.as_deref() {
                let iface = owner_iface_name(resolve, id);
                match (iface, name) {
                    ("common", "block-pos") => ResolvedType::BlockPos,
                    ("common", "position") => ResolvedType::Position,
                    ("common", "block-name") => ResolvedType::BlockName,
                    ("common", "block-state-id") => ResolvedType::BlockStateId,
                    ("common", "entity-type-name") => ResolvedType::EntityTypeName,
                    ("common", "socket-address") => ResolvedType::SocketAddress,
                    ("common", "raw-bytes") => ResolvedType::RawBytes,
                    ("common", "hand") => ResolvedType::Hand,
                    ("common", "game-mode") => ResolvedType::GameMode,
                    ("common", "click-type") => ResolvedType::ClickType,
                    ("player", "player") => ResolvedType::PlayerRes,
                    ("world", "world") => ResolvedType::WorldRes,
                    ("world", "entity") => ResolvedType::EntityRes,
                    ("text", "text-component") => ResolvedType::TextComponentRes,
                    ("item-stack", "item-stack") => ResolvedType::ItemStackRes,
                    ("java-dialogs", "dialog") => ResolvedType::Dialog,
                    ("uuid", "uuid") => ResolvedType::Uuid,
                    ("screens", "screen") => ResolvedType::Screen,
                    ("damage-types", "damage-type") => ResolvedType::DamageType,
                    ("enchantments", "enchantment") => ResolvedType::Enchantment,
                    ("world", "block-direction") => ResolvedType::BlockDirection,
                    ("world", "dye-color") => ResolvedType::DyeColor,
                    ("event", "entity-interaction-action") => ResolvedType::EntityInteractionAction,
                    (
                        "event",
                        "serverbound-packet"
                        | "clientbound-packet"
                        | "java-connection-state"
                        | "raw-packet",
                    )
                    | ("player", "java-minecraft-version" | "bedrock-minecraft-version") => {
                        ResolvedType::PacketOnlyNoCleanup
                    }
                    ("event", local_name) => match &tdef.kind {
                        TypeDefKind::Enum(_) => ResolvedType::LocalEnum(local_name.to_string()),
                        TypeDefKind::Record(rec) => {
                            let fields = rec
                                .fields
                                .iter()
                                .map(|f| (f.name.clone(), resolve_wit_type(resolve, &f.ty)))
                                .collect();
                            ResolvedType::HelperRecord {
                                wit_name: local_name.to_string(),
                                fields,
                            }
                        }
                        other => panic!(
                            "Unsupported named local type `{local_name}` in event.wit: {other:?}"
                        ),
                    },
                    (other_iface, other_name) => {
                        panic!("Unmapped WIT type `{other_iface}.{other_name}` in event.wit")
                    }
                }
            } else {
                match &tdef.kind {
                    TypeDefKind::Option(inner) => {
                        ResolvedType::Option(Box::new(resolve_wit_type(resolve, inner)))
                    }
                    TypeDefKind::List(inner) => {
                        ResolvedType::List(Box::new(resolve_wit_type(resolve, inner)))
                    }
                    TypeDefKind::Tuple(tup) => ResolvedType::Tuple(
                        tup.types
                            .iter()
                            .map(|t| resolve_wit_type(resolve, t))
                            .collect(),
                    ),
                    TypeDefKind::Handle(
                        wit_parser::Handle::Own(res_id) | wit_parser::Handle::Borrow(res_id),
                    ) => resolve_wit_type(resolve, &Type::Id(*res_id)),
                    other => panic!("Unsupported anonymous WIT type in event.wit: {other:?}"),
                }
            }
        }
    }
}

pub fn generate() {
    let mut resolve = Resolve::new();
    let (pkg_id, _) = resolve
        .push_dir(Path::new(WIT_OUT_DIR))
        .expect("Failed to parse WIT v0.2 directory");
    let pkg = &resolve.packages[pkg_id];
    let event_iface_id = *pkg
        .interfaces
        .get("event")
        .expect("Missing `event` interface in WIT package");
    let world_iface_id = *pkg
        .interfaces
        .get("world")
        .expect("Missing `world` interface in WIT package");

    let event_iface = &resolve.interfaces[event_iface_id];
    let world_iface = &resolve.interfaces[world_iface_id];

    // Collect variant `event` cases in declaration order.
    let event_variant_id = *event_iface
        .types
        .get("event")
        .expect("Missing `variant event` in event.wit");
    let TypeDefKind::Variant(event_variant) = &resolve.types[event_variant_id].kind else {
        panic!("`event` in event.wit must be a variant");
    };

    let mut event_data_record_ids = std::collections::HashSet::new();
    let mut all_events = Vec::with_capacity(event_variant.cases.len());

    for case in &event_variant.cases {
        let Some(Type::Id(raw_rec_id)) = case.ty else {
            panic!(
                "Event variant case `{}` must carry a record payload",
                case.name
            );
        };
        let rec_id = peel_named_import(&resolve, raw_rec_id);
        event_data_record_ids.insert(rec_id);
        let TypeDefKind::Record(rec) = &resolve.types[rec_id].kind else {
            panic!("Payload of event case `{}` must be a record", case.name);
        };
        let fields: Vec<(String, ResolvedType)> = rec
            .fields
            .iter()
            .map(|f| (f.name.clone(), resolve_wit_type(&resolve, &f.ty)))
            .collect();
        let is_manual_packet = MANUAL_PACKET_EVENTS.contains(&case.name.as_str());
        all_events.push((case.name.clone(), rec_id, fields, is_manual_packet));
    }

    // Collect local enums and helper records owned directly by `interface event`.
    let mut local_enums: Vec<(String, Vec<String>)> = Vec::new();
    let mut helper_records: Vec<(String, Vec<(String, ResolvedType)>)> = Vec::new();

    for (tname, &tid) in &event_iface.types {
        if resolve.types[tid].owner != TypeOwner::Interface(event_iface_id) {
            continue;
        }
        // Skip `use` aliases whose kind is `TypeDefKind::Type(Type::Id(_))`
        if matches!(resolve.types[tid].kind, TypeDefKind::Type(Type::Id(_))) {
            continue;
        }
        match &resolve.types[tid].kind {
            TypeDefKind::Enum(en) => {
                if !SKIPPED_LOCAL_ENUMS.contains(&tname.as_str()) {
                    let cases = en.cases.iter().map(|c| c.name.clone()).collect();
                    local_enums.push((tname.clone(), cases));
                }
            }
            TypeDefKind::Record(rec) => {
                if !event_data_record_ids.contains(&tid)
                    && !SKIPPED_HELPER_RECORDS.contains(&tname.as_str())
                {
                    let fields = rec
                        .fields
                        .iter()
                        .map(|f| (f.name.clone(), resolve_wit_type(&resolve, &f.ty)))
                        .collect();
                    helper_records.push((tname.clone(), fields));
                }
            }
            _ => {}
        }
    }

    // Also inspect `world.dye-color` enum cases for host conversion.
    let dye_color_tid = *world_iface
        .types
        .get("dye-color")
        .expect("Missing `dye-color` in world.wit");
    let TypeDefKind::Enum(dye_color_enum) = &resolve.types[dye_color_tid].kind else {
        panic!("`world.dye-color` must be an enum");
    };
    let dye_color_cases: Vec<String> = dye_color_enum
        .cases
        .iter()
        .map(|c| c.name.clone())
        .collect();

    let core_tokens = build_core_events(&local_enums, &helper_records, &all_events);
    let host_tokens = build_host_events(&local_enums, &dye_color_cases, &all_events);

    write_formatted_rust(CORE_EVENTS_OUT, &core_tokens);
    write_formatted_rust(HOST_EVENTS_OUT, &host_tokens);
}

fn build_core_events(
    local_enums: &[(String, Vec<String>)],
    helper_records: &[(String, Vec<(String, ResolvedType)>)],
    all_events: &[(String, TypeId, Vec<(String, ResolvedType)>, bool)],
) -> TokenStream {
    let enum_defs = local_enums.iter().map(|(wit_name, cases)| {
        let enum_ident = pascal_ident(wit_name);
        let variants = cases.iter().map(|c| pascal_ident(c));
        quote! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                #(#variants),*
            }
        }
    });

    let helper_defs = helper_records.iter().map(|(wit_name, fields)| {
        let struct_ident = pascal_ident(wit_name);
        let is_copy = fields.iter().all(|(_, fty)| fty.is_copy());
        let eq_debug = fields.iter().all(|(_, fty)| fty.supports_eq_debug());

        let derives = match (is_copy, eq_debug) {
            (true, true) => quote! { #[derive(Clone, Copy, Debug, PartialEq, Eq)] },
            (true, false) => quote! { #[derive(Clone, Copy)] },
            (false, true) => quote! { #[derive(Clone, Debug, PartialEq, Eq)] },
            (false, false) => quote! { #[derive(Clone)] },
        };

        let field_defs = fields.iter().map(|(fname, fty)| {
            let f_ident = snake_ident(fname);
            let f_ty = fty.core_type();
            quote! { pub #f_ident: #f_ty }
        });

        let ctor_params = fields.iter().map(|(fname, fty)| {
            let f_ident = snake_ident(fname);
            let f_ty = fty.core_type();
            quote! { #f_ident: #f_ty }
        });

        let ctor_inits = fields.iter().map(|(fname, _)| {
            let f_ident = snake_ident(fname);
            quote! { #f_ident }
        });

        quote! {
            #derives
            pub struct #struct_ident {
                #(#field_defs),*
            }

            impl #struct_ident {
                #[must_use]
                pub const fn new(#(#ctor_params),*) -> Self {
                    Self {
                        #(#ctor_inits),*
                    }
                }
            }
        }
    });

    let event_defs = all_events
        .iter()
        .filter(|(_, _, _, is_manual)| !*is_manual)
        .map(|(case_name, _, fields, _)| {
            let struct_ident = pascal_ident(case_name);
            let is_cancellable = fields.last().is_some_and(|(fname, fty)| {
                fname == "cancelled" && matches!(fty, ResolvedType::CopyPrimitive(_))
            });

            let field_defs = fields.iter().map(|(fname, fty)| {
                let f_ident = snake_ident(fname);
                let f_ty = fty.core_type();
                quote! { pub #f_ident: #f_ty }
            });

            let ctor_fields = if is_cancellable {
                &fields[..fields.len() - 1]
            } else {
                &fields[..]
            };

            let ctor_params = ctor_fields.iter().map(|(fname, fty)| {
                let f_ident = snake_ident(fname);
                let f_ty = fty.core_type();
                quote! { #f_ident: #f_ty }
            });

            let mut ctor_inits: Vec<TokenStream> = ctor_fields
                .iter()
                .map(|(fname, _)| {
                    let f_ident = snake_ident(fname);
                    quote! { #f_ident }
                })
                .collect();
            if is_cancellable {
                ctor_inits.push(quote! { cancelled: false });
            }

            let cancellable_impl = if is_cancellable {
                quote! {
                    impl super::Cancellable for #struct_ident {
                        fn cancelled(&self) -> bool {
                            self.cancelled
                        }

                        fn set_cancelled(&mut self, cancelled: bool) {
                            self.cancelled = cancelled;
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #[derive(Clone)]
                pub struct #struct_ident {
                    #(#field_defs),*
                }

                impl #struct_ident {
                    #[must_use]
                    pub const fn new(#(#ctor_params),*) -> Self {
                        Self {
                            #(#ctor_inits),*
                        }
                    }
                }

                impl super::Payload for #struct_ident {
                    fn get_name_static() -> &'static str {
                        stringify!(#struct_ident)
                    }

                    fn get_name(&self) -> &'static str {
                        stringify!(#struct_ident)
                    }

                    fn as_any(&self) -> &dyn std::any::Any {
                        self
                    }

                    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                        self
                    }
                }

                #cancellable_impl
            }
        });

    quote! {
        #![allow(
            clippy::all,
            clippy::pedantic,
            clippy::nursery,
            clippy::too_many_arguments,
        )]

        #(#enum_defs)*

        #(#helper_defs)*

        #(#event_defs)*
    }
}

fn build_host_events(
    local_enums: &[(String, Vec<String>)],
    dye_color_cases: &[String],
    all_events: &[(String, TypeId, Vec<(String, ResolvedType)>, bool)],
) -> TokenStream {
    let enum_convs = local_enums.iter().map(|(wit_name, cases)| {
        let enum_ident = pascal_ident(wit_name);
        let to_fn = format_ident!("to_wit_{}", snake_str(wit_name));
        let from_fn = format_ident!("from_wit_{}", snake_str(wit_name));

        let to_arms = cases.iter().map(|c| {
            let v = pascal_ident(c);
            quote! {
                pumpkin_core::plugin::#enum_ident::#v => crate::pumpkin::plugin::event::#enum_ident::#v
            }
        });
        let from_arms = cases.iter().map(|c| {
            let v = pascal_ident(c);
            quote! {
                crate::pumpkin::plugin::event::#enum_ident::#v => pumpkin_core::plugin::#enum_ident::#v
            }
        });

        quote! {
            const fn #to_fn(
                val: pumpkin_core::plugin::#enum_ident,
            ) -> crate::pumpkin::plugin::event::#enum_ident {
                match val {
                    #(#to_arms),*
                }
            }

            const fn #from_fn(
                val: crate::pumpkin::plugin::event::#enum_ident,
            ) -> pumpkin_core::plugin::#enum_ident {
                match val {
                    #(#from_arms),*
                }
            }
        }
    });

    let dye_to_arms = dye_color_cases.iter().map(|c| {
        let v = pascal_ident(c);
        quote! {
            pumpkin_data::dye_color::DyeColor::#v => crate::pumpkin::plugin::world::DyeColor::#v
        }
    });
    let dye_from_arms = dye_color_cases.iter().map(|c| {
        let v = pascal_ident(c);
        quote! {
            crate::pumpkin::plugin::world::DyeColor::#v => pumpkin_data::dye_color::DyeColor::#v
        }
    });
    let dye_color_convs = quote! {
        const fn to_wit_event_dye_color(
            val: pumpkin_data::dye_color::DyeColor,
        ) -> crate::pumpkin::plugin::world::DyeColor {
            match val {
                #(#dye_to_arms),*
            }
        }

        const fn from_wit_event_dye_color(
            val: crate::pumpkin::plugin::world::DyeColor,
        ) -> pumpkin_data::dye_color::DyeColor {
            match val {
                #(#dye_from_arms),*
            }
        }
    };

    let event_impls = all_events
        .iter()
        .filter(|(_, _, _, is_manual)| !*is_manual)
        .map(|(case_name, _, fields, _)| {
            let event_ident = pascal_ident(case_name);
            let data_ident = format_ident!("{event_ident}Data");
            let needs_state = fields.iter().any(|(_, fty)| fty.needs_state());
            let state_param = if needs_state {
                format_ident!("state")
            } else {
                format_ident!("_state")
            };

            let to_wasm_fields = fields.iter().map(|(fname, fty)| {
                let f_ident = snake_ident(fname);
                let expr = if fty.is_copy() {
                    quote! { self.#f_ident }
                } else {
                    quote! { &self.#f_ident }
                };
                let conv = fty.to_wasm(&expr, !fty.is_copy());
                quote! { #f_ident: #conv }
            });

            let data_binding = if fields.is_empty() {
                quote! { _data }
            } else {
                quote! { data }
            };

            let from_wasm_fields = fields.iter().map(|(fname, fty)| {
                let f_ident = snake_ident(fname);
                let expr = quote! { data.#f_ident };
                let conv = fty.from_wasm(&expr);
                quote! { #f_ident: #conv }
            });

            let custom_apply = if case_name == "server-list-ping-event" {
                quote! {
                    fn apply_wasm_event(
                        &mut self,
                        event: crate::pumpkin::plugin::event::Event,
                        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
                    ) {
                        if !matches!(
                            &event,
                            crate::pumpkin::plugin::event::Event::ServerListPingEvent(_)
                        ) {
                            cleanup_event(&event, state);
                            panic!("unexpected event type");
                        }

                        let returned = Self::from_wasm_event(event, state);
                        self.motd = returned.motd;
                        self.max_players = returned.max_players;
                        self.num_players = returned.num_players;
                        self.favicon = returned.favicon;
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                impl ToFromWasmEvent for pumpkin_core::plugin::#event_ident {
                    fn to_wasm_event(
                        &self,
                        #state_param: &mut pumpkin_wasm_host_common::state::PluginHostState,
                    ) -> crate::pumpkin::plugin::event::Event {
                        crate::pumpkin::plugin::event::Event::#event_ident(
                            crate::pumpkin::plugin::event::#data_ident {
                                #(#to_wasm_fields),*
                            },
                        )
                    }

                    fn from_wasm_event(
                        event: crate::pumpkin::plugin::event::Event,
                        #state_param: &mut pumpkin_wasm_host_common::state::PluginHostState,
                    ) -> Self {
                        match event {
                            crate::pumpkin::plugin::event::Event::#event_ident(#data_binding) => Self {
                                #(#from_wasm_fields),*
                            },
                            _ => panic!("unexpected event type"),
                        }
                    }

                    #custom_apply
                }
            }
        });

    let cleanup_arms = all_events.iter().map(|(case_name, _, fields, _)| {
        let event_ident = pascal_ident(case_name);
        let stmts: Vec<TokenStream> = fields
            .iter()
            .filter_map(|(fname, fty)| {
                let f_ident = snake_ident(fname);
                let ref_expr = quote! { &data.#f_ident };
                fty.cleanup(&ref_expr)
            })
            .collect();
        if stmts.is_empty() {
            quote! {
                crate::pumpkin::plugin::event::Event::#event_ident(_) => {}
            }
        } else {
            quote! {
                crate::pumpkin::plugin::event::Event::#event_ident(data) => {
                    #(#stmts)*
                }
            }
        }
    });

    let register_arms = all_events.iter().map(|(case_name, _, _, _)| {
        let event_ident = pascal_ident(case_name);
        quote! {
            crate::pumpkin::plugin::event::EventType::#event_ident => {
                ctx.register_event::<pumpkin_core::plugin::#event_ident, _>(
                    std::sync::Arc::clone(handler),
                    priority,
                    blocking,
                );
            }
        }
    });

    quote! {
        #![allow(
            clippy::all,
            clippy::pedantic,
            clippy::nursery,
            clippy::too_many_lines,
            clippy::match_same_arms,
        )]

        use super::{
            ToFromWasmEvent, WasmPluginEventHandler, cleanup_entity, cleanup_item_stack,
            cleanup_player, cleanup_text_component, cleanup_world, consume_entity,
            consume_item_stack, consume_player, consume_text_component, consume_world,
            from_wasm_block_name, from_wasm_block_position, from_wasm_click_type,
            from_wasm_entity_interaction_action, from_wasm_entity_type, from_wasm_game_mode,
            from_wasm_hand, from_wasm_position, to_wasm_block_name, to_wasm_block_position,
            to_wasm_click_type, to_wasm_entity_interaction_action, to_wasm_entity_type,
            to_wasm_game_mode, to_wasm_hand, to_wasm_position,
        };

        #(#enum_convs)*

        #dye_color_convs

        #(#event_impls)*

        pub fn cleanup_event(
            event: &crate::pumpkin::plugin::event::Event,
            state: &mut pumpkin_wasm_host_common::state::PluginHostState,
        ) {
            match event {
                #(#cleanup_arms)*
            }
        }

        pub fn register_generated_event(
            ctx: &pumpkin_core::plugin::Context,
            handler: &std::sync::Arc<WasmPluginEventHandler>,
            priority: pumpkin_core::plugin::EventPriority,
            blocking: bool,
            event_type: crate::pumpkin::plugin::event::EventType,
        ) {
            match event_type {
                #(#register_arms)*
            }
        }
    }
}

fn write_formatted_rust(out_path: &str, tokens: &TokenStream) {
    let header = "/* This file is generated from WIT. Do not edit manually. */\n";
    let raw = tokens.to_string();
    let formatted = format_code(&raw).unwrap_or(raw);
    let content = format!("{header}{formatted}");

    let path = Path::new(out_path);
    if path.exists()
        && let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return;
    }

    fs::write(path, content).unwrap_or_else(|e| panic!("Failed to write {}: {e}", path.display()));
}
