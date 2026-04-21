use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

// ── Deserialized input types (unchanged from original) ────────────────────────

#[derive(Deserialize)]
pub struct BlockStateCodecStruct {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

#[derive(Deserialize)]
pub struct GenerationSettingsStruct {
    #[serde(default)]
    pub aquifers_enabled: bool,
    #[serde(default)]
    pub ore_veins_enabled: bool,
    #[serde(default)]
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: BlockStateCodecStruct,
    pub default_block: BlockStateCodecStruct,
    #[serde(rename = "noise")]
    pub shape: GenerationShapeConfigStruct,
    pub surface_rule: MaterialRuleStruct,
}

#[derive(Deserialize)]
pub struct GenerationShapeConfigStruct {
    pub min_y: i8,
    pub height: u16,
    pub size_horizontal: u8,
    pub size_vertical: u8,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialRuleStruct {
    #[serde(rename = "minecraft:block")]
    Block { result_state: BlockStateCodecStruct },
    #[serde(rename = "minecraft:sequence")]
    Sequence { sequence: Vec<Self> },
    #[serde(rename = "minecraft:condition")]
    Condition {
        if_true: MaterialConditionStruct,
        then_run: Box<Self>,
    },
    #[serde(rename = "minecraft:bandlands")]
    Badlands,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialConditionStruct {
    #[serde(rename = "minecraft:biome")]
    Biome { biome_is: Vec<String> },
    #[serde(rename = "minecraft:noise_threshold")]
    NoiseThreshold {
        noise: String,
        min_threshold: f64,
        max_threshold: f64,
    },
    #[serde(rename = "minecraft:vertical_gradient")]
    VerticalGradient {
        random_name: String,
        true_at_and_below: YOffsetStruct,
        false_at_and_above: YOffsetStruct,
    },
    #[serde(rename = "minecraft:y_above")]
    YAbove {
        anchor: YOffsetStruct,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:water")]
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:temperature")]
    Temperature,
    #[serde(rename = "minecraft:steep")]
    Steep,
    #[serde(rename = "minecraft:not")]
    Not { invert: Box<Self> },
    #[serde(rename = "minecraft:hole")]
    Hole,
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface,
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface_type: String,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffsetStruct {
    Absolute { absolute: i16 },
    AboveBottom { above_bottom: i8 },
    BelowTop { below_top: i8 },
}

// ── Flat bytecode compiler ────────────────────────────────────────────────────

/// Compiles a `MaterialRuleStruct` tree into a flat Vec of `TokenStream`s,
/// each representing one `SurfaceInstruction` literal.
///
/// Layout rules:
///   - `Sequence`  → children in order, no instruction of its own
///   - `Condition` → `Test*` instruction (with a `skip` field) followed by the
///                   body instructions; skip jumps past the body on failure
///   - `Block`     → `PlaceBlock` terminal
///   - `Badlands`  → `PlaceBadlands` terminal
struct Compiler {
    out: Vec<TokenStream>,
}

impl Compiler {
    fn new() -> Self {
        Self { out: Vec::new() }
    }

    /// Appends instructions for `rule` and returns the number of instructions emitted.
    fn compile_rule(&mut self, rule: &MaterialRuleStruct) -> usize {
        let before = self.out.len();
        match rule {
            MaterialRuleStruct::Block { result_state } => {
                let ts = block_state_tokens(result_state);
                self.out
                    .push(quote!(SurfaceInstruction::PlaceBlock { state: #ts }));
            }
            MaterialRuleStruct::Badlands => {
                self.out.push(quote!(SurfaceInstruction::PlaceBadlands));
            }
            MaterialRuleStruct::Sequence { sequence } => {
                // Sequences have no instruction of their own; children are inline
                for child in sequence {
                    self.compile_rule(child);
                }
            }
            MaterialRuleStruct::Condition { if_true, then_run } => {
                // Reserve a slot for the Test instruction; we need body size first
                let test_slot = self.out.len();
                self.out.push(TokenStream::new()); // placeholder

                let body_len = self.compile_rule(then_run);
                let skip = body_len as u16;

                // Patch placeholder with the real instruction now that skip is known
                self.out[test_slot] = compile_condition(if_true, skip);
            }
        }
        self.out.len() - before
    }
}

/// Emits a `SurfaceInstruction::Test*` token stream for the given condition.
/// `skip` is the number of instructions to jump past on failure.
fn compile_condition(cond: &MaterialConditionStruct, skip: u16) -> TokenStream {
    match cond {
        MaterialConditionStruct::Biome { biome_is } => {
            let refs: Vec<TokenStream> = biome_is
                .iter()
                .map(|b| {
                    let ident = format_ident!(
                        "{}",
                        b.strip_prefix("minecraft:").unwrap_or(b).to_uppercase()
                    );
                    quote!(&crate::biome::Biome::#ident)
                })
                .collect();
            quote!(SurfaceInstruction::TestBiome {
                biome_is: &[#(#refs),*],
                skip: #skip,
            })
        }

        MaterialConditionStruct::NoiseThreshold {
            noise,
            min_threshold,
            max_threshold,
        } => {
            let noise_id = format_ident!(
                "{}",
                noise
                    .strip_prefix("minecraft:")
                    .unwrap()
                    .to_shouty_snake_case()
            );
            // f64::MAX (or near it) means "no upper bound" — emit the cheaper variant
            if *max_threshold >= f64::MAX / 2.0 {
                quote!(SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::#noise_id,
                    min: #min_threshold,
                    skip: #skip,
                })
            } else {
                quote!(SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::#noise_id,
                    min: #min_threshold,
                    max: #max_threshold,
                    skip: #skip,
                })
            }
        }

        MaterialConditionStruct::VerticalGradient {
            random_name,
            true_at_and_below,
            false_at_and_above,
        } => {
            let bytes = md5::compute(random_name.as_bytes());
            let lo = u64::from_be_bytes(bytes[0..8].try_into().expect("md5 slice"));
            let hi = u64::from_be_bytes(bytes[8..16].try_into().expect("md5 slice"));
            let below = y_offset_tokens(true_at_and_below);
            let above = y_offset_tokens(false_at_and_above);
            quote!(SurfaceInstruction::TestVerticalGradient {
                random_lo: #lo,
                random_hi: #hi,
                true_at_and_below: #below,
                false_at_and_above: #above,
                skip: #skip,
            })
        }

        MaterialConditionStruct::YAbove {
            anchor,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let anchor_ts = y_offset_tokens(anchor);
            quote!(SurfaceInstruction::TestYAbove {
                anchor: #anchor_ts,
                surface_depth_multiplier: #surface_depth_multiplier,
                add_stone_depth: #add_stone_depth,
                skip: #skip,
            })
        }

        MaterialConditionStruct::Water {
            offset,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            quote!(SurfaceInstruction::TestWater {
                offset: #offset,
                surface_depth_multiplier: #surface_depth_multiplier,
                add_stone_depth: #add_stone_depth,
                skip: #skip,
            })
        }

        MaterialConditionStruct::StoneDepth {
            offset,
            add_surface_depth,
            secondary_depth_range,
            surface_type,
        } => {
            let st = match surface_type.as_str() {
                "ceiling" => quote!(VerticalSurfaceType::Ceiling),
                _ => quote!(VerticalSurfaceType::Floor),
            };
            quote!(SurfaceInstruction::TestStoneDepth {
                offset: #offset,
                add_surface_depth: #add_surface_depth,
                secondary_depth_range: #secondary_depth_range,
                surface_type: #st,
                skip: #skip,
            })
        }

        MaterialConditionStruct::Not { invert } => {
            // Compile the inner condition with skip=0; the Not wrapper flips the result
            let inner = compile_condition(invert, 0);
            quote!(SurfaceInstruction::TestNot {
                inner: &#inner,
                skip: #skip,
            })
        }

        MaterialConditionStruct::AbovePreliminarySurface => {
            quote!(SurfaceInstruction::TestAbovePreliminarySurface { skip: #skip })
        }
        MaterialConditionStruct::Hole => {
            quote!(SurfaceInstruction::TestHole { skip: #skip })
        }
        MaterialConditionStruct::Steep => {
            quote!(SurfaceInstruction::TestSteep { skip: #skip })
        }
        MaterialConditionStruct::Temperature => {
            quote!(SurfaceInstruction::TestTemperature { skip: #skip })
        }
    }
}

// ── Small token helpers ───────────────────────────────────────────────────────

fn block_state_tokens(b: &BlockStateCodecStruct) -> TokenStream {
    let name = b.name.strip_prefix("minecraft:").unwrap_or(&b.name);
    let ident = format_ident!("{}", name.to_uppercase().replace([':', '-'], "_"));
    quote!(crate::Block::#ident.default_state)
}

fn y_offset_tokens(y: &YOffsetStruct) -> TokenStream {
    match y {
        YOffsetStruct::Absolute { absolute } => {
            quote!(YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: #absolute }))
        }
        YOffsetStruct::AboveBottom { above_bottom } => {
            quote!(YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom { above_bottom: #above_bottom }))
        }
        YOffsetStruct::BelowTop { below_top } => {
            quote!(YOffset::BelowTop(pumpkin_util::y_offset::BelowTop { below_top: #below_top }))
        }
    }
}

fn shape_tokens(s: &GenerationShapeConfigStruct) -> TokenStream {
    let min_y = s.min_y;
    let height = s.height;
    let hor = s.size_horizontal;
    let ver = s.size_vertical;
    quote!(GenerationShapeConfig {
        min_y: #min_y,
        height: #height,
        size_horizontal: #hor,
        size_vertical: #ver,
    })
}

fn settings_tokens(s: &GenerationSettingsStruct) -> TokenStream {
    let aquifers = s.aquifers_enabled;
    let ores = s.ore_veins_enabled;
    let legacy = s.legacy_random_source;
    let sea_level = s.sea_level;
    let fluid = block_state_tokens(&s.default_fluid);
    let block = block_state_tokens(&s.default_block);
    let shape = shape_tokens(&s.shape);

    let mut compiler = Compiler::new();
    compiler.compile_rule(&s.surface_rule);
    let instructions = &compiler.out;

    quote!(GenerationSettings {
        aquifers_enabled: #aquifers,
        ore_veins_enabled: #ores,
        legacy_random_source: #legacy,
        sea_level: #sea_level,
        default_fluid: #fluid,
        default_block: #block,
        shape: #shape,
        surface_rule: CompiledSurfaceRule {
            instructions: &[#(#instructions),*],
        },
    })
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn build() -> TokenStream {
    let json: BTreeMap<String, GenerationSettingsStruct> =
        serde_json::from_str(&fs::read_to_string("../assets/chunk_gen_settings.json").unwrap())
            .expect("Failed to parse chunk_gen_settings.json");

    let const_defs: TokenStream = json
        .iter()
        .map(|(name, settings)| {
            let const_name = format_ident!("{}", name.to_uppercase());
            let body = settings_tokens(settings);
            quote!(pub const #const_name: GenerationSettings = #body;)
        })
        .collect();

    quote!(
        use crate::dimension::Dimension;
        use crate::chunk::DoublePerlinNoiseParameters;
        use crate::BlockState;
        use pumpkin_util::y_offset::YOffset;
        use pumpkin_util::math::vertical_surface_type::VerticalSurfaceType;
        use crate::biome::Biome;

        // ── Core settings struct ──────────────────────────────────────────────

        pub struct GenerationSettings {
            pub aquifers_enabled: bool,
            pub ore_veins_enabled: bool,
            pub legacy_random_source: bool,
            pub sea_level: i32,
            pub default_fluid: &'static BlockState,
            pub default_block: &'static BlockState,
            pub shape: GenerationShapeConfig,
            /// Flat compiled surface rule — no recursion at runtime.
            pub surface_rule: CompiledSurfaceRule,
        }

        pub struct GenerationShapeConfig {
            pub min_y: i8,
            pub height: u16,
            pub size_horizontal: u8,
            pub size_vertical: u8,
        }

        impl GenerationShapeConfig {
            #[inline] #[must_use]
            pub const fn vertical_cell_block_count(&self) -> u8 { self.size_vertical << 2 }
            #[inline] #[must_use]
            pub const fn horizontal_cell_block_count(&self) -> u8 { self.size_horizontal << 2 }

            #[must_use]
            pub const fn max_y(&self) -> u16 {
                if self.min_y >= 0 {
                    self.height + self.min_y as u16
                } else {
                    (self.height as i32 + self.min_y as i32) as u16
                }
            }

            pub fn trim_height(&self, bottom_y: i8, top_y: u16) -> Self {
                let new_min = self.min_y.max(bottom_y);
                let this_top = if self.min_y >= 0 {
                    self.height + self.min_y as u16
                } else {
                    self.height - self.min_y.unsigned_abs() as u16
                };
                let new_top = this_top.min(top_y);
                let new_height = if new_min >= 0 {
                    new_top - new_min as u16
                } else {
                    new_top + new_min.unsigned_abs() as u16
                };
                Self { min_y: new_min, height: new_height,
                       size_horizontal: self.size_horizontal, size_vertical: self.size_vertical }
            }
        }

        // ── Flat bytecode types ───────────────────────────────────────────────

        /// A compiled, flat representation of a surface rule tree.
        /// Evaluated by a simple index-advancing loop — no recursion.
        pub struct CompiledSurfaceRule {
            pub instructions: &'static [SurfaceInstruction],
        }

        /// One instruction in the flat surface rule bytecode.
        ///
        /// Test instructions carry a `skip: u16` field. On failure the evaluator
        /// advances the program counter by `skip + 1` (past the body). On success
        /// it advances by 1 (into the body). Terminals end evaluation immediately.
        pub enum SurfaceInstruction {
            // ── Terminals ────────────────────────────────────────────────────
            PlaceBlock    { state: &'static BlockState },
            PlaceBadlands,

            // ── Conditions ───────────────────────────────────────────────────
            TestBiome {
                biome_is: &'static [&'static Biome],
                skip: u16,
            },
            /// Noise >= min  (the f64::MAX upper-bound fast path)
            TestNoiseAbove {
                noise: DoublePerlinNoiseParameters,
                min: f64,
                skip: u16,
            },
            /// min <= noise <= max
            TestNoiseRange {
                noise: DoublePerlinNoiseParameters,
                min: f64,
                max: f64,
                skip: u16,
            },
            TestVerticalGradient {
                random_lo: u64,
                random_hi: u64,
                true_at_and_below: YOffset,
                false_at_and_above: YOffset,
                skip: u16,
            },
            TestYAbove {
                anchor: YOffset,
                surface_depth_multiplier: i32,
                add_stone_depth: bool,
                skip: u16,
            },
            TestWater {
                offset: i32,
                surface_depth_multiplier: i32,
                add_stone_depth: bool,
                skip: u16,
            },
            TestStoneDepth {
                offset: i32,
                add_surface_depth: bool,
                secondary_depth_range: i32,
                surface_type: VerticalSurfaceType,
                skip: u16,
            },
            TestAbovePreliminarySurface { skip: u16 },
            TestHole                    { skip: u16 },
            TestSteep                   { skip: u16 },
            TestTemperature             { skip: u16 },
            /// Inverts a single inner condition.
            /// The inner condition is stored inline — it must not itself contain
            /// a body (i.e. it always has inner skip = 0).
            TestNot {
                inner: &'static SurfaceInstruction,
                skip: u16,
            },
        }

        // ── Per-column noise cache ────────────────────────────────────────────

        /// Caches noise samples for one (x, z) column.
        /// Call `invalidate(x, z)` once per column; `get` then returns the cached
        /// value on subsequent calls for the same noise parameter.
        pub struct ColumnNoiseCache {
            values: [f64; DoublePerlinNoiseParameters::COUNT],
            valid:  [bool; DoublePerlinNoiseParameters::COUNT],
            col_x: i32,
            col_z: i32,
        }

        impl ColumnNoiseCache {
            pub const fn new() -> Self {
                Self {
                    values: [0.0; DoublePerlinNoiseParameters::COUNT],
                    valid:  [false; DoublePerlinNoiseParameters::COUNT],
                    col_x: i32::MIN,
                    col_z: i32::MIN,
                }
            }

            /// Must be called at the start of each new (x, z) column.
            #[inline]
            pub fn invalidate(&mut self, x: i32, z: i32) {
                if self.col_x != x || self.col_z != z {
                    self.valid = [false; DoublePerlinNoiseParameters::COUNT];
                    self.col_x = x;
                    self.col_z = z;
                }
            }

            /// Returns the cached noise value, sampling it on first access.
            #[inline]
            pub fn get(
                &mut self,
                noise: &DoublePerlinNoiseParameters,
                sample: impl FnOnce() -> f64,
            ) -> f64 {
                let idx = noise.id;
                if !self.valid[idx] {
                    self.values[idx] = sample();
                    self.valid[idx] = true;
                }
                self.values[idx]
            }
        }

        impl Default for ColumnNoiseCache {
            fn default() -> Self { Self::new() }
        }

        // ── Generated constants ───────────────────────────────────────────────

        impl GenerationSettings {
            #const_defs

            pub fn from_dimension(dimension: &Dimension) -> &'static Self {
                match dimension {
                    d if d == &Dimension::OVERWORLD   => &Self::OVERWORLD,
                    d if d == &Dimension::THE_NETHER  => &Self::NETHER,
                    _                                  => &Self::END,
                }
            }
        }
    )
}
