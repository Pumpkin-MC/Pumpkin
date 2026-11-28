//! Canonical ABI lifting and lowering codegen for native shared-library plugins.

use std::fmt::Write as _;

use heck::ToUpperCamelCase;
use wit_bindgen_core::abi::{AbiVariant, Bindgen, Bitcast, Instruction, LiftLower, WasmType};
use wit_parser::{
    Alignment, ArchitectureSize, Function, Resolve, SizeAlign, Type, TypeDefKind, TypeId, TypeOwner,
};

use crate::rust::{TypeNames, render_type, to_rust_ident, to_rust_upper_camel_case};
use crate::symbol_name::{canonical_export_name, export_field, post_return_field};

const PTR_SIZE_EXPR: &str = "::core::mem::size_of::<*const u8>()";

fn fmt_size(size: &ArchitectureSize) -> String {
    size.format_term(PTR_SIZE_EXPR, false)
}

fn fmt_align(align: Alignment) -> String {
    align.format(PTR_SIZE_EXPR)
}

pub struct NativeBindgen<'a> {
    sizes: &'a SizeAlign,
    params: Vec<String>,
    src: String,
    tmp: usize,
    block_storage: Vec<String>,
    blocks: Vec<String>,
    wraps_result: bool,
    export_module: Option<String>,
    import_target: Option<String>,
    needs_post_return: bool,
    post_return: Option<(String, String)>,
}

impl<'a> NativeBindgen<'a> {
    const fn new(
        sizes: &'a SizeAlign,
        params: Vec<String>,
        wraps_result: bool,
        export_module: Option<String>,
        import_target: Option<String>,
        needs_post_return: bool,
    ) -> Self {
        Self {
            sizes,
            params,
            src: String::new(),
            tmp: 0,
            block_storage: Vec::new(),
            blocks: Vec::new(),
            wraps_result,
            export_module,
            import_target,
            needs_post_return,
            post_return: None,
        }
    }

    const fn tmp(&mut self) -> usize {
        let ret = self.tmp;
        self.tmp += 1;
        ret
    }

    fn push_str(&mut self, s: &str) {
        self.src.push_str(s);
    }

    fn let_results(&mut self, amt: usize, results: &mut Vec<String>) {
        match amt {
            0 => {}
            1 => {
                let tmp = self.tmp();
                let res = format!("result{tmp}");
                let _ = write!(self.src, "let {res} = ");
                results.push(res);
            }
            n => {
                let tmp = self.tmp();
                self.push_str("let (");
                for i in 0..n {
                    let arg = format!("result{tmp}_{i}");
                    let _ = write!(self.src, "{arg},");
                    results.push(arg);
                }
                self.push_str(") = ");
            }
        }
    }

    fn load(
        offset: &ArchitectureSize,
        ty: &str,
        widen_to: Option<&str>,
        operands: &[String],
        results: &mut Vec<String>,
    ) {
        let off = fmt_size(offset);
        let read = format!("(*(({}).add({off}) as *const {ty}))", operands[0]);
        results.push(widen_to.map_or_else(|| read.clone(), |wide| format!("({read} as {wide})")));
    }

    fn store(&mut self, offset: &ArchitectureSize, ty: &str, narrow: bool, operands: &[String]) {
        let off = fmt_size(offset);
        let value = if narrow {
            format!("({}) as {ty}", operands[0])
        } else {
            operands[0].clone()
        };
        let _ = writeln!(
            self.src,
            "unsafe {{ *(({}).add({off}) as *mut {ty}) = {value}; }}",
            operands[1],
        );
    }
}

pub fn type_name(resolve: &Resolve, id: TypeId) -> String {
    let type_camel = to_rust_upper_camel_case(resolve.types[id].name.as_deref().unwrap_or("anon"));
    let TypeOwner::Interface(owner_id) = resolve.types[id].owner else {
        return type_camel;
    };
    let iface_camel =
        to_rust_upper_camel_case(resolve.interfaces[owner_id].name.as_deref().unwrap_or(""));
    format!("{iface_camel}{type_camel}")
}

struct NativeNames;

impl TypeNames for NativeNames {
    fn named(&self, resolve: &Resolve, id: TypeId) -> String {
        type_name(resolve, id)
    }

    fn flags(&self, _resolve: &Resolve, _id: TypeId, flags: &wit_parser::Flags) -> String {
        format!("[u32; {}]", flags.repr().count())
    }

    fn handle(&self, _resolve: &Resolve, _handle: &wit_parser::Handle) -> String {
        "u32".to_string()
    }
}

pub fn rust_type_for_native(resolve: &Resolve, ty: &Type) -> String {
    render_type(resolve, &NativeNames, ty)
}

fn perform_cast(operand: &str, cast: &Bitcast) -> String {
    match cast {
        Bitcast::None => operand.to_owned(),
        Bitcast::I32ToI64 => format!("i64::from({operand})"),
        Bitcast::F32ToI32 => format!("({operand}).to_bits() as i32"),
        Bitcast::F64ToI64 => format!("({operand}).to_bits() as i64"),
        Bitcast::I64ToI32 | Bitcast::PToI32 | Bitcast::LToI32 => format!("{operand} as i32"),
        Bitcast::I32ToF32 | Bitcast::I64ToF32 => format!("f32::from_bits({operand} as u32)"),
        Bitcast::I64ToF64 => format!("f64::from_bits({operand} as u64)"),
        Bitcast::F32ToI64 => format!("i64::from(({operand}).to_bits())"),
        Bitcast::I64ToP64 => format!("::core::mem::MaybeUninit::new({operand} as u64)"),
        Bitcast::P64ToI64 => format!("{operand}.assume_init() as i64"),
        Bitcast::PToP64 => format!(
            "{{ let mut t = ::core::mem::MaybeUninit::<u64>::uninit(); \
               t.as_mut_ptr().cast::<*mut u8>().write({operand}); t }}"
        ),
        Bitcast::P64ToP => format!("{operand}.as_ptr().cast::<*mut u8>().read()"),
        Bitcast::I32ToP | Bitcast::LToP => format!("{operand} as *mut u8"),
        Bitcast::I32ToL | Bitcast::I64ToL | Bitcast::PToL => format!("{operand} as usize"),
        Bitcast::LToI64 => format!("{operand} as i64"),
        Bitcast::Sequence(seq) => {
            let [first, second] = &**seq;
            perform_cast(&perform_cast(operand, first), second)
        }
    }
}

fn primitive_rust_ty(resolve: &Resolve, ty: &Type) -> &'static str {
    let mut ty = *ty;
    while let Type::Id(id) = ty {
        if let TypeDefKind::Type(inner) = resolve.types[id].kind {
            ty = inner;
        } else {
            break;
        }
    }
    match ty {
        Type::Bool => "bool",
        Type::U16 => "u16",
        Type::U32 => "u32",
        Type::U64 => "u64",
        Type::S8 => "i8",
        Type::S16 => "i16",
        Type::S32 => "i32",
        Type::S64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
        Type::Char => "char",
        // `Type::U8` and fallback for non-primitives.
        _ => "u8",
    }
}

pub const fn wasm_type_rust(ty: WasmType) -> &'static str {
    match ty {
        WasmType::I32 => "i32",
        WasmType::I64 => "i64",
        WasmType::F32 => "f32",
        WasmType::F64 => "f64",
        WasmType::Pointer => "*mut u8",
        WasmType::Length => "usize",
        WasmType::PointerOrI64 => "::core::mem::MaybeUninit<u64>",
    }
}

#[expect(
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreachable,
    reason = "Canonical ABI instruction dispatch covers all wit-bindgen-core opcodes in one match"
)]
impl Bindgen for NativeBindgen<'_> {
    type Operand = String;

    fn emit(
        &mut self,
        resolve: &Resolve,
        inst: &Instruction<'_>,
        operands: &mut Vec<Self::Operand>,
        results: &mut Vec<Self::Operand>,
    ) {
        match inst {
            Instruction::GetArg { nth } => results.push(self.params[*nth].clone()),
            Instruction::I32Const { val } => results.push(format!("{val}i32")),
            Instruction::ConstZero { tys } => {
                for ty in *tys {
                    results.push(
                        match ty {
                            WasmType::I32 => "0i32",
                            WasmType::I64 => "0i64",
                            WasmType::F32 => "0.0f32",
                            WasmType::F64 => "0.0f64",
                            WasmType::Pointer => "::core::ptr::null_mut::<u8>()",
                            WasmType::PointerOrI64 => "::core::mem::MaybeUninit::<u64>::zeroed()",
                            WasmType::Length => "0usize",
                        }
                        .to_string(),
                    );
                }
            }
            Instruction::Bitcasts { casts } => {
                for (cast, op) in casts.iter().zip(operands.iter()) {
                    results.push(perform_cast(op, cast));
                }
            }

            Instruction::I32Load { offset } => Self::load(offset, "i32", None, operands, results),
            Instruction::I64Load { offset } => Self::load(offset, "i64", None, operands, results),
            Instruction::F32Load { offset } => Self::load(offset, "f32", None, operands, results),
            Instruction::F64Load { offset } => Self::load(offset, "f64", None, operands, results),
            Instruction::PointerLoad { offset } => {
                Self::load(offset, "*mut u8", None, operands, results);
            }
            Instruction::LengthLoad { offset } => {
                Self::load(offset, "usize", None, operands, results);
            }
            Instruction::I32Load8U { offset } => {
                Self::load(offset, "u8", Some("i32"), operands, results);
            }
            Instruction::I32Load8S { offset } => {
                Self::load(offset, "i8", Some("i32"), operands, results);
            }
            Instruction::I32Load16U { offset } => {
                Self::load(offset, "u16", Some("i32"), operands, results);
            }
            Instruction::I32Load16S { offset } => {
                Self::load(offset, "i16", Some("i32"), operands, results);
            }

            Instruction::I32Store { offset } => self.store(offset, "i32", false, operands),
            Instruction::I64Store { offset } => self.store(offset, "i64", false, operands),
            Instruction::F32Store { offset } => self.store(offset, "f32", false, operands),
            Instruction::F64Store { offset } => self.store(offset, "f64", false, operands),
            Instruction::PointerStore { offset } => self.store(offset, "*mut u8", false, operands),
            Instruction::LengthStore { offset } => self.store(offset, "usize", false, operands),
            Instruction::I32Store8 { offset } => self.store(offset, "u8", true, operands),
            Instruction::I32Store16 { offset } => self.store(offset, "u16", true, operands),

            Instruction::I32FromChar
            | Instruction::I32FromU8
            | Instruction::I32FromS8
            | Instruction::I32FromU16
            | Instruction::I32FromS16
            | Instruction::I32FromU32
            | Instruction::I32FromS32
            | Instruction::S32FromI32
            | Instruction::EnumLower { .. }
            | Instruction::HandleLower { .. } => results.push(format!("({}) as i32", operands[0])),
            Instruction::I64FromU64 | Instruction::I64FromS64 | Instruction::S64FromI64 => {
                results.push(format!("({}) as i64", operands[0]));
            }
            Instruction::CoreF32FromF32 | Instruction::F32FromCoreF32 => {
                results.push(format!("({}) as f32", operands[0]));
            }
            Instruction::CoreF64FromF64 | Instruction::F64FromCoreF64 => {
                results.push(format!("({}) as f64", operands[0]));
            }
            Instruction::S8FromI32 => results.push(format!("({}) as i8", operands[0])),
            Instruction::U8FromI32 => results.push(format!("({}) as u8", operands[0])),
            Instruction::S16FromI32 => results.push(format!("({}) as i16", operands[0])),
            Instruction::U16FromI32 => results.push(format!("({}) as u16", operands[0])),
            Instruction::U32FromI32 | Instruction::HandleLift { .. } => {
                results.push(format!("({}) as u32", operands[0]));
            }
            Instruction::U64FromI64 => results.push(format!("({}) as u64", operands[0])),
            Instruction::CharFromI32 => results.push(format!(
                "char::from_u32(({}) as u32).expect(\"invalid char from plugin\")",
                operands[0]
            )),
            Instruction::BoolFromI32 => results.push(format!("(({}) != 0)", operands[0])),
            Instruction::I32FromBool => {
                results.push(format!("(if {} {{ 1i32 }} else {{ 0i32 }})", operands[0]));
            }
            Instruction::Flush { amt } => {
                for operand in &operands[..*amt] {
                    let tmp = self.tmp();
                    let result = format!("result{tmp}");
                    let _ = writeln!(self.src, "let {result} = {operand};");
                    results.push(result);
                }
            }

            // Always allocate lowered strings in the guest's allocator (`cabi_realloc`)
            // so the guest frees them with the same allocator that allocated them.
            Instruction::StringLower { .. } => {
                let tmp = self.tmp();
                let val = format!("s{tmp}");
                let ptr = format!("ptr{tmp}");
                let len = format!("len{tmp}");
                let _ = writeln!(
                    self.src,
                    "let {val}: ::std::string::String = {};",
                    operands[0]
                );
                let _ = writeln!(self.src, "let {len} = {val}.len();");
                let _ = writeln!(
                    self.src,
                    "let {ptr} = unsafe {{ ::pumpkin_native_wit::abi_alloc_copy({val}.as_bytes(), 1) }};"
                );
                results.push(ptr);
                results.push(len);
            }
            Instruction::StringLift => {
                let tmp = self.tmp();
                let ptr = format!("ptr{tmp}");
                let len = format!("len{tmp}");
                let _ = writeln!(self.src, "let {ptr} = {};", operands[0]);
                let _ = writeln!(self.src, "let {len} = {};", operands[1]);
                results.push(format!(
                    "if {len} == 0 {{ ::std::string::String::new() }} else {{ \
                     ::std::string::String::from_utf8(unsafe {{ ::core::slice::from_raw_parts({ptr} as *const u8, {len}) }}.to_vec()) \
                     .expect(\"native WIT plugin passed invalid UTF-8 string\") }}"
                ));
            }

            // Always allocate lowered canonical lists in the guest's allocator (`cabi_realloc`).
            Instruction::ListCanonLower { element, .. } => {
                let tmp = self.tmp();
                let val = format!("v{tmp}");
                let ptr = format!("ptr{tmp}");
                let len = format!("len{tmp}");
                let elem_ty = primitive_rust_ty(resolve, element);
                let _ = writeln!(
                    self.src,
                    "let {val}: ::std::vec::Vec<{elem_ty}> = {};",
                    operands[0]
                );
                let _ = writeln!(self.src, "let {len} = {val}.len();");
                let _ = writeln!(
                    self.src,
                    "let {ptr} = if {len} == 0 {{ \
                     ::core::ptr::without_provenance_mut(::core::mem::align_of::<{elem_ty}>()) \
                     }} else {{ \
                     let bytes = unsafe {{ ::core::slice::from_raw_parts({val}.as_ptr().cast::<u8>(), {len} * ::core::mem::size_of::<{elem_ty}>()) }}; \
                     unsafe {{ ::pumpkin_native_wit::abi_alloc_copy(bytes, ::core::mem::align_of::<{elem_ty}>()) }} \
                     }};"
                );
                results.push(ptr);
                results.push(len);
            }
            Instruction::ListCanonLift { element, .. } => {
                let tmp = self.tmp();
                let ptr = format!("ptr{tmp}");
                let len = format!("len{tmp}");
                let elem_ty = primitive_rust_ty(resolve, element);
                let _ = writeln!(self.src, "let {ptr} = {};", operands[0]);
                let _ = writeln!(self.src, "let {len} = {};", operands[1]);
                results.push(format!(
                    "if {len} == 0 {{ ::std::vec::Vec::new() }} else {{ \
                     unsafe {{ ::core::slice::from_raw_parts({ptr} as *const {elem_ty}, {len}) }}.to_vec() }}"
                ));
            }
            Instruction::ListLower { element, .. } => {
                let body = self.blocks.pop().unwrap();
                let tmp = self.tmp();
                let vec = format!("vec{tmp}");
                let result = format!("result{tmp}");
                let len = format!("len{tmp}");
                let size = fmt_size(&self.sizes.size(element));
                let align = fmt_align(self.sizes.align(element));
                let _ = writeln!(self.src, "let {vec} = {};", operands[0]);
                let _ = writeln!(self.src, "let {len} = {vec}.len();");
                let _ = writeln!(
                    self.src,
                    "let {result} = unsafe {{ ::pumpkin_native_wit::abi_alloc({len} * {size}, {align}) }};"
                );
                let _ = writeln!(self.src, "for (i, e) in {vec}.into_iter().enumerate() {{");
                let _ = writeln!(
                    self.src,
                    "let base = unsafe {{ {result}.add(i * {size}) }};"
                );
                self.push_str(&body);
                self.push_str("\n}\n");
                results.push(result);
                results.push(len);
            }
            Instruction::ListLift { element, .. } => {
                let body = self.blocks.pop().unwrap();
                let tmp = self.tmp();
                let size = fmt_size(&self.sizes.size(element));
                let len = format!("len{tmp}");
                let base = format!("base{tmp}");
                let result = format!("result{tmp}");
                let _ = writeln!(self.src, "let {base} = {};", operands[0]);
                let _ = writeln!(self.src, "let {len} = {};", operands[1]);
                let _ = writeln!(
                    self.src,
                    "let mut {result} = ::std::vec::Vec::with_capacity({len});"
                );
                let _ = writeln!(self.src, "for i in 0..{len} {{");
                let _ = writeln!(self.src, "let base = unsafe {{ {base}.add(i * {size}) }};");
                let _ = writeln!(self.src, "let e{tmp} = {body};");
                let _ = writeln!(self.src, "{result}.push(e{tmp});");
                self.push_str("}\n");
                results.push(result);
            }
            Instruction::IterElem { .. } | Instruction::VariantPayloadName => {
                results.push("e".to_string());
            }
            Instruction::IterBasePointer => results.push("base".to_string()),
            Instruction::Malloc { size, align, .. } => {
                let tmp = self.tmp();
                let ptr = format!("ptr{tmp}");
                let sz = fmt_size(size);
                let al = fmt_align(*align);
                let _ = writeln!(
                    self.src,
                    "let {ptr} = unsafe {{ ::pumpkin_native_wit::abi_alloc({sz}, {al}) }};"
                );
                results.push(ptr);
            }

            Instruction::RecordLower { record, ty, .. } => {
                let tmp = self.tmp();
                let name = type_name(resolve, *ty);
                let mut pat = Vec::new();
                for field in &record.fields {
                    let field_ident = to_rust_ident(&field.name);
                    let arg = format!("{field_ident}{tmp}");
                    pat.push(format!("{field_ident}: {arg}"));
                    results.push(arg);
                }
                let _ = writeln!(
                    self.src,
                    "let {name} {{ {} }} = {};",
                    pat.join(", "),
                    operands[0]
                );
            }
            Instruction::RecordLift { record, ty, .. } => {
                let name = type_name(resolve, *ty);
                let fields: Vec<String> = record
                    .fields
                    .iter()
                    .zip(operands.iter())
                    .map(|(field, val)| format!("{}: {val}", to_rust_ident(&field.name)))
                    .collect();
                results.push(format!("{name} {{ {} }}", fields.join(", ")));
            }

            Instruction::TupleLower { tuple, .. } => {
                let tmp = self.tmp();
                let mut pat = Vec::new();
                for i in 0..tuple.types.len() {
                    let arg = format!("t{tmp}_{i}");
                    pat.push(arg.clone());
                    results.push(arg);
                }
                let _ = writeln!(self.src, "let ({},) = {};", pat.join(", "), operands[0]);
            }
            Instruction::TupleLift { .. } => {
                if operands.len() == 1 {
                    results.push(format!("({},)", operands[0]));
                } else {
                    results.push(format!("({})", operands.join(", ")));
                }
            }

            Instruction::EnumLift { enum_, ty, .. } => {
                let name = type_name(resolve, *ty);
                let mut arms = String::new();
                for (idx, case) in enum_.cases.iter().enumerate() {
                    let _ = write!(
                        arms,
                        "{idx} => {name}::{}, ",
                        case.name.to_upper_camel_case()
                    );
                }
                results.push(format!(
                    "match ({}) as u32 {{ {arms}_ => panic!(\"invalid {name} discriminant\") }}",
                    operands[0]
                ));
            }

            Instruction::VariantLower {
                variant,
                results: result_types,
                ty,
                ..
            } => {
                let blocks: Vec<String> = self
                    .blocks
                    .drain(self.blocks.len() - variant.cases.len()..)
                    .collect();
                let name = type_name(resolve, *ty);
                self.let_results(result_types.len(), results);
                let _ = writeln!(self.src, "match {} {{", operands[0]);
                for (case, block) in variant.cases.iter().zip(blocks) {
                    let case_name = case.name.to_upper_camel_case();
                    if case.ty.is_some() {
                        let _ = writeln!(self.src, "{name}::{case_name}(e) => {block},");
                    } else {
                        let _ = writeln!(self.src, "{name}::{case_name} => {{\n{block}\n}}");
                    }
                }
                self.push_str(if results.is_empty() { "}\n" } else { "};\n" });
            }
            Instruction::VariantLift { variant, ty, .. } => {
                let blocks: Vec<String> = self
                    .blocks
                    .drain(self.blocks.len() - variant.cases.len()..)
                    .collect();
                let tmp = self.tmp();
                let name = type_name(resolve, *ty);
                let _ = writeln!(self.src, "let v{tmp} = match ({}) as u32 {{", operands[0]);
                for (i, (case, block)) in variant.cases.iter().zip(blocks).enumerate() {
                    let _ = writeln!(self.src, "{i} => {{");
                    let case_name = case.name.to_upper_camel_case();
                    if case.ty.is_none() {
                        let _ = writeln!(self.src, "{name}::{case_name}");
                    } else {
                        let _ = writeln!(
                            self.src,
                            "let e{tmp} = {block};\n{name}::{case_name}(e{tmp})"
                        );
                    }
                    self.push_str("}\n");
                }
                let _ = writeln!(
                    self.src,
                    "n => panic!(\"invalid {name} discriminant {{n}}\"),"
                );
                self.push_str("};\n");
                results.push(format!("v{tmp}"));
            }

            Instruction::OptionLower {
                results: result_types,
                ..
            } => {
                let some = self.blocks.pop().unwrap();
                let none = self.blocks.pop().unwrap();
                self.let_results(result_types.len(), results);
                let _ = writeln!(
                    self.src,
                    "match {} {{ ::core::option::Option::Some(e) => {some}, ::core::option::Option::None => {{\n{none}\n}}, }};",
                    operands[0]
                );
            }
            Instruction::OptionLift { .. } => {
                let some = self.blocks.pop().unwrap();
                let none = self.blocks.pop().unwrap();
                assert_eq!(none, "()");
                results.push(format!(
                    "match ({}) as u32 {{ \
                     0 => ::core::option::Option::None, \
                     1 => {{ let e = {some}; ::core::option::Option::Some(e) }} \
                     n => panic!(\"invalid option discriminant {{n}}\"), }}",
                    operands[0]
                ));
            }

            Instruction::ResultLower {
                results: result_types,
                result,
                ..
            } => {
                let err = self.blocks.pop().unwrap();
                let ok = self.blocks.pop().unwrap();
                self.let_results(result_types.len(), results);
                let ok_binding = if result.ok.is_some() { "e" } else { "_" };
                let err_binding = if result.err.is_some() { "e" } else { "_" };
                let _ = writeln!(
                    self.src,
                    "match {} {{ \
                     ::core::result::Result::Ok({ok_binding}) => {{ {ok} }}, \
                     ::core::result::Result::Err({err_binding}) => {{ {err} }}, \
                     }};",
                    operands[0]
                );
            }
            Instruction::ResultLift { .. } => {
                let err = self.blocks.pop().unwrap();
                let ok = self.blocks.pop().unwrap();
                results.push(format!(
                    "match ({}) as u32 {{ \
                     0 => {{ let e = {ok}; ::core::result::Result::Ok(e) }} \
                     1 => {{ let e = {err}; ::core::result::Result::Err(e) }} \
                     n => panic!(\"invalid result discriminant {{n}}\"), }}",
                    operands[0]
                ));
            }

            Instruction::FlagsLower { flags, .. } => {
                let tmp = self.tmp();
                let _ = writeln!(self.src, "let flags{tmp} = {};", operands[0]);
                for i in 0..flags.repr().count() {
                    results.push(format!("flags{tmp}[{i}] as i32"));
                }
            }
            Instruction::FlagsLift { flags, .. } => {
                let words = (0..flags.repr().count())
                    .map(|i| format!("({}) as u32", operands[i]))
                    .collect::<Vec<_>>()
                    .join(", ");
                results.push(format!("[{words}]"));
            }

            Instruction::CallWasm { name, sig } => {
                let tmp = self.tmp();
                let canonical = canonical_export_name(self.export_module.as_deref(), name);
                let field = export_field(&canonical);
                if !sig.results.is_empty() {
                    let _ = write!(self.src, "let ret{tmp} = ");
                    results.push(format!("ret{tmp}"));
                }

                let _ = writeln!(
                    self.src,
                    "unsafe {{ (self.{field})({}) }};",
                    operands.join(", ")
                );

                if self.needs_post_return {
                    let retptr = results
                        .last()
                        .expect("an export needing a post-return returns its result area")
                        .clone();
                    self.post_return = Some((retptr, post_return_field(&canonical)));
                }
            }
            Instruction::CallInterface { func, .. } => {
                let target = self
                    .import_target
                    .clone()
                    .expect("import_target must be set for GuestImport");
                self.let_results(usize::from(func.result.is_some()), results);
                let _ = writeln!(self.src, "{target}({});", operands.join(", "));
            }
            Instruction::Return { amt, .. } => {
                let inner = match amt {
                    0 => "()".to_string(),
                    1 => operands[0].clone(),
                    _ => format!("({})", operands.join(", ")),
                };
                let post_return = self.post_return.take();

                if *amt == 0 && !self.wraps_result && post_return.is_none() {
                    return;
                }

                let Some((retptr, field)) = post_return else {
                    if self.wraps_result {
                        let _ = writeln!(self.src, "::core::result::Result::Ok({inner})");
                    } else {
                        let _ = writeln!(self.src, "{inner}");
                    }
                    return;
                };

                let tmp = self.tmp();
                let _ = writeln!(self.src, "let ret{tmp} = {inner};");
                let _ = writeln!(self.src, "unsafe {{ (self.{field})({retptr}) }};");
                if self.wraps_result {
                    let _ = writeln!(self.src, "::core::result::Result::Ok(ret{tmp})");
                } else {
                    let _ = writeln!(self.src, "ret{tmp}");
                }
            }

            _ => {
                unreachable!("{inst:?} is not supported by pumpkin-native-wit");
            }
        }
    }

    fn return_pointer(&mut self, _size: ArchitectureSize, _align: Alignment) -> Self::Operand {
        unreachable!("return_pointer is not used by native host codegen")
    }

    fn push_block(&mut self) {
        let prev_src = std::mem::take(&mut self.src);
        self.block_storage.push(prev_src);
    }

    fn finish_block(&mut self, operands: &mut Vec<Self::Operand>) {
        let prev_src = self.block_storage.pop().unwrap();
        let src = std::mem::replace(&mut self.src, prev_src);
        let expr = match operands.len() {
            0 => "()".to_string(),
            1 => operands[0].clone(),
            _ => format!("({})", operands.join(", ")),
        };
        if src.is_empty() {
            self.blocks.push(expr);
        } else if operands.is_empty() {
            self.blocks.push(format!("{{\n{src}}}"));
        } else {
            self.blocks.push(format!("{{\n{src}\n{expr}\n}}"));
        }
    }

    fn sizes(&self) -> &SizeAlign {
        self.sizes
    }

    fn is_list_canonical(&self, resolve: &Resolve, element: &Type) -> bool {
        let mut ty = *element;
        while let Type::Id(id) = ty {
            match &resolve.types[id].kind {
                TypeDefKind::Type(inner) => ty = *inner,
                _ => break,
            }
        }
        // Exclude `bool` and `char`: constructing `&[bool]` or `&[char]` directly from guest
        // bytes would be UB if a plugin returned an invalid discriminant or Unicode scalar value.
        matches!(
            ty,
            Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::F32
                | Type::F64
        )
    }
}

pub fn generate_export_body(
    resolve: &Resolve,
    sizes: &SizeAlign,
    func: &Function,
    params: Vec<String>,
    export_module: Option<&str>,
) -> String {
    let needs_post_return = wit_bindgen_core::abi::guest_export_needs_post_return(resolve, func);
    let mut bindgen = NativeBindgen::new(
        sizes,
        params,
        true,
        export_module.map(str::to_string),
        None,
        needs_post_return,
    );
    wit_bindgen_core::abi::call(
        resolve,
        AbiVariant::GuestExport,
        LiftLower::LowerArgsLiftResults,
        func,
        &mut bindgen,
        false,
    );
    bindgen.src
}

pub fn generate_import_body(
    resolve: &Resolve,
    sizes: &SizeAlign,
    func: &Function,
    params: Vec<String>,
    import_target: String,
) -> String {
    let mut bindgen = NativeBindgen::new(sizes, params, false, None, Some(import_target), false);
    wit_bindgen_core::abi::call(
        resolve,
        AbiVariant::GuestImport,
        LiftLower::LiftArgsLowerResults,
        func,
        &mut bindgen,
        false,
    );
    bindgen.src
}
