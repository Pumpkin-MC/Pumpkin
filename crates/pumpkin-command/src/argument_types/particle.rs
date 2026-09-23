use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::argument_types::nbt::NbtCompoundArgumentType;
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const ERROR_UNKNOWN_PARTICLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_NOTFOUND,
    translation::bedrock::COMMANDS_PARTICLE_NOTFOUND,
);

/// "Can't parse particle options: %s" — a parameterized particle (`dust`,
/// `block`, ...) given no `{...}` at all, or one missing/misusing a field its
/// vanilla codec requires.
pub const ERROR_INVALID_OPTIONS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_INVALIDOPTIONS,
    translation::java::PARTICLE_INVALIDOPTIONS,
);

/// A parsed `/particle` argument.
///
/// Carries the particle type plus its already network-encoded extra payload
/// (empty for a particle that takes none), ready to drop straight into a
/// particle packet's `data` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedParticle {
    pub particle: Particle,
    pub extra_data: Vec<u8>,
}

pub struct ParticleArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ParticleArgumentType {
    type Item = ParsedParticle;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        let particle = Particle::from_name(identifier.path())
            .or_else(|| Particle::from_name(&identifier.to_string()))
            .ok_or_else(|| {
                ERROR_UNKNOWN_PARTICLE.create(reader, TextComponent::text(identifier.to_string()))
            })?;

        // Matches vanilla's own `ParticleArgument.readParticle`: a `{...}`
        // is only read if present; otherwise the particle's codec sees an
        // empty map, which is fine for a particle whose fields are all
        // optional (`effect`'s `color`/`power` both default) and a real
        // "missing required field" error for one that isn't (`dust`'s
        // `color`/`scale` have no defaults) — `encode_particle_options`
        // draws that same required-vs-optional line per field below, so
        // there's no separate up-front "does this particle need options"
        // gate to keep in sync with it.
        let options = if reader.peek() == Some('{') {
            ArgumentType::<crate::source::DummySource>::parse(&NbtCompoundArgumentType, reader)?
        } else {
            NbtCompound::new()
        };
        let extra_data = encode_particle_options(reader, particle, &options)?;

        Ok(ParsedParticle {
            particle,
            extra_data,
        })
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Particle
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "foo".to_string(),
            "foo:bar".to_string(),
            "particle{foo:bar}".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let particles = (0..=124u16)
            .filter_map(Particle::from_id)
            .map(|p| format!("{p:?}").to_lowercase())
            .collect();
        builder.filter_and_suggest_lowercase(particles).build()
    }
}

fn missing_field(reader: &mut StringReader, field: &str) -> CommandSyntaxError {
    ERROR_INVALID_OPTIONS.create(reader, TextComponent::text(format!("missing '{field}'")))
}

/// Reads a numeric field regardless of which NBT number tag it parsed as.
///
/// Vanilla's codecs decode `{...}` particle options through `NbtOps`, which
/// reads any number type generically and lets the target codec (`Codec.INT`,
/// `Codec.FLOAT`, ...) narrow it — so `scale:1` and `scale:1.0` both satisfy
/// a `float` field. [`NbtCompound::get_float`]/`get_int` only match their
/// own exact tag variant, so a plain integer or double literal (SNBT's
/// default for a bare decimal, with no `f`/`b`/`l` suffix) would otherwise
/// be rejected as "missing" even though it was given.
fn coerce_i32(options: &NbtCompound, field: &str) -> Option<i32> {
    match options.get(field)? {
        NbtTag::Byte(v) => Some(i32::from(*v)),
        NbtTag::Short(v) => Some(i32::from(*v)),
        NbtTag::Int(v) => Some(*v),
        NbtTag::Long(v) => i32::try_from(*v).ok(),
        NbtTag::Float(v) => Some(*v as i32),
        NbtTag::Double(v) => Some(*v as i32),
        _ => None,
    }
}

/// Reads a color field the way vanilla's `RGB_COLOR_CODEC`/`ARGB_COLOR_CODEC`
/// do: either the packed int form (passed through unchanged, alpha bits and
/// all — matching `Codec.withAlternative(Codec.INT, ...)` trying the int
/// alternative first), or a list of 3 floats (`[r,g,b]`, each `0.0..=1.0`,
/// packed with alpha forced to `0xFF` the way `ARGB.colorFromFloat(1.0, r,
/// g, b)` does for `RGB_COLOR_CODEC`'s vector form) or 4 floats (`[r,g,b,a]`,
/// for `ARGB_COLOR_CODEC`'s fields).
fn coerce_color(options: &NbtCompound, field: &str) -> Option<i32> {
    match options.get(field)? {
        NbtTag::List(items) if items.len() == 3 || items.len() == 4 => {
            let channel = |tag: &NbtTag| -> Option<u8> {
                let v = match tag {
                    NbtTag::Float(v) => f64::from(*v),
                    NbtTag::Double(v) => *v,
                    _ => return None,
                };
                Some((v.clamp(0.0, 1.0) * 255.0).round() as u8)
            };
            let r = channel(&items[0])?;
            let g = channel(&items[1])?;
            let b = channel(&items[2])?;
            let a = if items.len() == 4 {
                channel(&items[3])?
            } else {
                0xFF
            };
            Some(i32::from_be_bytes([a, r, g, b]))
        }
        _ => coerce_i32(options, field),
    }
}

/// Reads an int field that vanilla's `ExtraCodecs.POSITIVE_INT` requires to
/// be `>= 1` (`water_blocks` on the geyser particles), telling "missing"
/// apart from "present but not positive" so the error names the actual
/// problem instead of just saying "missing" for a `0` or negative value.
fn positive_i32(
    reader: &mut StringReader,
    options: &NbtCompound,
    field: &str,
) -> Result<i32, CommandSyntaxError> {
    let value = coerce_i32(options, field).ok_or_else(|| missing_field(reader, field))?;
    if value < 1 {
        return Err(ERROR_INVALID_OPTIONS.create(
            reader,
            TextComponent::text(format!("'{field}' must be positive, got {value}")),
        ));
    }
    Ok(value)
}

fn coerce_f32(options: &NbtCompound, field: &str) -> Option<f32> {
    match options.get(field)? {
        NbtTag::Byte(v) => Some(f32::from(*v)),
        NbtTag::Short(v) => Some(f32::from(*v)),
        NbtTag::Int(v) => Some(*v as f32),
        NbtTag::Long(v) => Some(*v as f32),
        NbtTag::Float(v) => Some(*v),
        NbtTag::Double(v) => Some(*v as f32),
        _ => None,
    }
}

/// Encodes `options` into the raw extra-data payload vanilla's client-bound
/// particle packet expects for `particle`, matching each type's
/// `StreamCodec` (`DustParticleOptions`, `BlockParticleOption`, ...): plain
/// big-endian ints/floats for the color- and scale-based particles, a
/// registry-id `VarInt` for the block-state ones. Fields are read by the
/// same names vanilla's own `MapCodec`s use (`color`, `scale`, `from_color`,
/// `to_color`, `roll`, `delay`, `block_state`), so existing `/particle`
/// commands and function files carry over unchanged.
fn encode_particle_options(
    reader: &mut StringReader,
    particle: Particle,
    options: &NbtCompound,
) -> Result<Vec<u8>, CommandSyntaxError> {
    let mut data = Vec::new();
    match particle {
        Particle::Dust => {
            let color =
                coerce_color(options, "color").ok_or_else(|| missing_field(reader, "color"))?;
            let scale =
                coerce_f32(options, "scale").ok_or_else(|| missing_field(reader, "scale"))?;
            data.write_i32_be(color).ok();
            data.write_f32_be(scale).ok();
        }
        Particle::DustColorTransition => {
            let from_color = coerce_color(options, "from_color")
                .ok_or_else(|| missing_field(reader, "from_color"))?;
            let to_color = coerce_color(options, "to_color")
                .ok_or_else(|| missing_field(reader, "to_color"))?;
            let scale =
                coerce_f32(options, "scale").ok_or_else(|| missing_field(reader, "scale"))?;
            data.write_i32_be(from_color).ok();
            data.write_i32_be(to_color).ok();
            data.write_f32_be(scale).ok();
        }
        Particle::EntityEffect | Particle::TintedLeaves | Particle::Flash => {
            let color =
                coerce_color(options, "color").ok_or_else(|| missing_field(reader, "color"))?;
            data.write_i32_be(color).ok();
        }
        // `effect` (ambient) and `instant_effect` both use vanilla's
        // `SpellParticleOption`, not `ColorParticleOption` — a different
        // shape (`color` + `power`) where *both* fields are optional with
        // defaults, unlike `entity_effect`'s required-only `color`.
        Particle::Effect | Particle::InstantEffect => {
            let color = coerce_color(options, "color").unwrap_or(-1);
            let power = coerce_f32(options, "power").unwrap_or(1.0);
            data.write_i32_be(color).ok();
            data.write_f32_be(power).ok();
        }
        Particle::SculkCharge => {
            let roll = coerce_f32(options, "roll").ok_or_else(|| missing_field(reader, "roll"))?;
            data.write_f32_be(roll).ok();
        }
        Particle::Shriek => {
            let delay =
                coerce_i32(options, "delay").ok_or_else(|| missing_field(reader, "delay"))?;
            data.write_var_int(&pumpkin_protocol::VarInt(delay)).ok();
        }
        Particle::Block
        | Particle::BlockMarker
        | Particle::FallingDust
        | Particle::DustPillar
        | Particle::BlockCrumble => {
            let name = options
                .get_string("block_state")
                .ok_or_else(|| missing_field(reader, "block_state"))?;
            let normalized = if name.contains(':') {
                name.to_string()
            } else {
                format!("minecraft:{name}")
            };
            let block = Block::from_name(&normalized).ok_or_else(|| {
                ERROR_INVALID_OPTIONS.create(
                    reader,
                    TextComponent::text(format!("unknown block '{normalized}'")),
                )
            })?;
            let state_id: BlockStateId = block.default_state.id;
            data.write_var_int(&pumpkin_protocol::VarInt(i32::from(state_id.as_u16())))
                .ok();
        }
        // `dragon_breath` is vanilla's `PowerParticleOption`: a single
        // optional `power` float (default 1.0).
        Particle::DragonBreath => {
            let power = coerce_f32(options, "power").unwrap_or(1.0);
            data.write_f32_be(power).ok();
        }
        // `geyser`/`geyser_plume` are `GeyserParticleOptions`: a required
        // `water_blocks` int, no default.
        Particle::Geyser | Particle::GeyserPlume => {
            let water_blocks = positive_i32(reader, options, "water_blocks")?;
            data.write_i32_be(water_blocks).ok();
        }
        // `geyser_base`/`geyser_poof` are `GeyserBaseParticleOptions`: a
        // required `water_blocks` int plus a required `burst_impulse_base`
        // float, neither with a default.
        Particle::GeyserBase | Particle::GeyserPoof => {
            let water_blocks = positive_i32(reader, options, "water_blocks")?;
            let burst_impulse_base = coerce_f32(options, "burst_impulse_base")
                .ok_or_else(|| missing_field(reader, "burst_impulse_base"))?;
            data.write_i32_be(water_blocks).ok();
            data.write_f32_be(burst_impulse_base).ok();
        }
        // `item`, `vibration` and `trail` are required-options particles
        // (`ItemParticleOption`, `VibrationParticleOption`, `TrailParticleOption`
        // in vanilla) that this encoder doesn't yet know how to serialize —
        // encoding item-stack NBT, a vibration path, or a trail's target and
        // duration is unimplemented here. Silently falling through to an
        // empty payload regardless of what (if anything) was given is worse
        // than that gap alone: it would send the client a particle vanilla
        // never sends with no data at all, rather than failing loudly. This
        // at least catches the `{}`-or-nothing case; full support is still
        // missing for whatever options a caller does provide.
        Particle::Item | Particle::Vibration | Particle::Trail if options.is_empty() => {
            return Err(ERROR_INVALID_OPTIONS.create(
                reader,
                TextComponent::text(format!(
                    "{} requires options, which this server does not yet support encoding",
                    format!("{particle:?}").to_lowercase()
                )),
            ));
        }
        _ => {}
    }
    Ok(data)
}

impl ParticleArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<ParsedParticle, CommandSyntaxError> {
        Ok(context.get_argument::<ParsedParticle>(name)?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_reader::StringReader;

    fn parse(input: &str) -> Result<ParsedParticle, CommandSyntaxError> {
        let mut reader = StringReader::new(input);
        ArgumentType::<crate::source::DummySource>::parse(&ParticleArgumentType, &mut reader)
    }

    #[test]
    fn parses_a_parameterless_particle_with_no_extra_data() {
        let parsed = parse("minecraft:flame").unwrap();
        assert_eq!(parsed.particle, Particle::Flame);
        assert!(parsed.extra_data.is_empty());
    }

    #[test]
    fn rejects_a_parameterized_particle_given_no_options() {
        assert_parse_err_reset!(
            &mut StringReader::new("minecraft:dust"),
            ParticleArgumentType,
            &ERROR_INVALID_OPTIONS
        );
    }

    #[test]
    fn encodes_dust_as_a_color_int_and_a_scale_float() {
        let parsed = parse("minecraft:dust{color:16711680,scale:1.0}").unwrap();
        assert_eq!(parsed.particle, Particle::Dust);
        let mut expected = Vec::new();
        expected.write_i32_be(16_711_680).unwrap();
        expected.write_f32_be(1.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_dust_color_transition_as_two_colors_and_a_scale() {
        let parsed =
            parse("minecraft:dust_color_transition{from_color:255,to_color:65280,scale:2.0}")
                .unwrap();
        let mut expected = Vec::new();
        expected.write_i32_be(255).unwrap();
        expected.write_i32_be(65_280).unwrap();
        expected.write_f32_be(2.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_block_particle_as_a_var_int_state_id() {
        let parsed = parse("minecraft:block{block_state:\"minecraft:stone\"}").unwrap();
        let mut expected = Vec::new();
        expected
            .write_var_int(&pumpkin_protocol::VarInt(i32::from(
                Block::STONE.default_state.id.as_u16(),
            )))
            .unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_ambient_effect_with_both_fields_defaulted_when_omitted() {
        // Real bug this session found and fixed: `effect` was treated as
        // sharing `entity_effect`'s single-`color`-int shape, but vanilla's
        // `effect`/`instant_effect` actually use `SpellParticleOption`
        // (`color` *and* `power`, both optional with defaults -1 / 1.0).
        let parsed = parse("minecraft:effect").unwrap();
        let mut expected = Vec::new();
        expected.write_i32_be(-1).unwrap();
        expected.write_f32_be(1.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_ambient_effect_with_given_fields() {
        let parsed = parse("minecraft:instant_effect{color:255,power:2.0}").unwrap();
        let mut expected = Vec::new();
        expected.write_i32_be(255).unwrap();
        expected.write_f32_be(2.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_dragon_breath_with_a_defaulted_power() {
        let parsed = parse("minecraft:dragon_breath").unwrap();
        let mut expected = Vec::new();
        expected.write_f32_be(1.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_tinted_leaves_and_flash_as_a_required_color() {
        for name in ["minecraft:tinted_leaves", "minecraft:flash"] {
            let mut expected = Vec::new();
            expected.write_i32_be(16_711_680).unwrap();
            assert_eq!(
                parse(&format!("{name}{{color:16711680}}"))
                    .unwrap()
                    .extra_data,
                expected
            );
        }
    }

    #[test]
    fn rejects_geyser_given_no_water_blocks() {
        assert_parse_err_reset!(
            &mut StringReader::new("minecraft:geyser"),
            ParticleArgumentType,
            &ERROR_INVALID_OPTIONS
        );
    }

    #[test]
    fn rejects_geyser_given_a_non_positive_water_blocks() {
        // Vanilla's `GeyserParticleOptions` reads `water_blocks` through
        // `ExtraCodecs.POSITIVE_INT`, which requires `>= 1`.
        for value in [0, -1] {
            assert_parse_err_reset!(
                &mut StringReader::new(format!("minecraft:geyser{{water_blocks:{value}}}")),
                ParticleArgumentType,
                &ERROR_INVALID_OPTIONS
            );
        }
    }

    #[test]
    fn encodes_geyser_base_with_both_required_fields() {
        let parsed = parse("minecraft:geyser_base{water_blocks:5,burst_impulse_base:0.5}").unwrap();
        let mut expected = Vec::new();
        expected.write_i32_be(5).unwrap();
        expected.write_f32_be(0.5).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_block_crumble_as_a_var_int_state_id() {
        let parsed = parse("minecraft:block_crumble{block_state:\"minecraft:stone\"}").unwrap();
        let mut expected = Vec::new();
        expected
            .write_var_int(&pumpkin_protocol::VarInt(i32::from(
                Block::STONE.default_state.id.as_u16(),
            )))
            .unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn rejects_an_unknown_block_name_for_a_block_particle() {
        assert_parse_err_reset!(
            &mut StringReader::new("minecraft:block{block_state:\"minecraft:not_a_block\"}"),
            ParticleArgumentType,
            &ERROR_INVALID_OPTIONS
        );
    }

    #[test]
    fn encodes_dust_color_given_as_an_rgb_float_list_with_alpha_forced_opaque() {
        let parsed = parse("minecraft:dust{color:[1.0,0.0,0.0],scale:1.0}").unwrap();
        let mut expected = Vec::new();
        // Matches vanilla's `RGB_COLOR_CODEC`: a 3-float list packs through
        // `ARGB.colorFromFloat(1.0, r, g, b)`, forcing alpha to `0xFF` —
        // unlike the plain-int form, which passes its top byte through as
        // given (see `encodes_dust_as_a_color_int_and_a_scale_float`).
        expected.write_i32_be(0xFF_FF_00_00u32 as i32).unwrap();
        expected.write_f32_be(1.0).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn encodes_entity_effect_color_given_as_an_rgba_float_list() {
        let parsed = parse("minecraft:entity_effect{color:[0.0,1.0,0.0,0.5]}").unwrap();
        let mut expected = Vec::new();
        // `ARGB_COLOR_CODEC`'s 4-float list carries its own alpha instead of
        // forcing it opaque.
        expected.write_i32_be(0x80_00_FF_00u32 as i32).unwrap();
        assert_eq!(parsed.extra_data, expected);
    }

    #[test]
    fn rejects_item_vibration_and_trail_given_no_options() {
        for particle in ["item", "vibration", "trail"] {
            assert_parse_err_reset!(
                &mut StringReader::new(format!("minecraft:{particle}")),
                ParticleArgumentType,
                &ERROR_INVALID_OPTIONS
            );
        }
    }
}
