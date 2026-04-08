use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use pumpkin_data::{translation, world::SAY_COMMAND};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    random::{RandomImpl, xoroshiro128::Xoroshiro},
    text::TextComponent,
};
use pumpkin_world::world_info::RandomSequence;
use rand::RngExt;
use tokio::sync::Mutex;

#[cfg(test)]
use crate::command::dispatcher::CommandError;
use crate::command::{
    argument_builder::{ArgumentBuilder, argument, command, literal},
    argument_types::{
        argument_type::{ArgumentType, JavaClientArgumentType},
        core::{bool::BoolArgumentType, long::LongArgumentType, string::StringArgumentType},
    },
    context::{command_context::CommandContext, command_source::CommandSource},
    errors::{
        command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext},
        error_types,
    },
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
    string_reader::StringReader,
    tree::RawArg,
};
use pumpkin_protocol::java::client::play::StringProtoArgBehavior;

const DESCRIPTION: &str = "Generates a random integer, or controls random sequences.";
const PERMISSION: &str = "minecraft:command.random";

const ARG_RANGE: &str = "range";
const ARG_SEQUENCE: &str = "sequence";
const ARG_SEED: &str = "seed";
const ARG_INCLUDE_WORLD_SEED: &str = "includeWorldSeed";
const ARG_INCLUDE_SEQUENCE_ID: &str = "includeSequenceId";

const fn is_valid_namespaced_id_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.' | '/' | ':')
}

fn syntax_expected_separator(raw_arg: RawArg<'_>, local_cursor: usize) -> CommandSyntaxError {
    let mut clamped_local_cursor = local_cursor.min(raw_arg.value.len());
    while clamped_local_cursor > 0 && !raw_arg.value.is_char_boundary(clamped_local_cursor) {
        clamped_local_cursor -= 1;
    }

    let context = CommandSyntaxErrorContext {
        input: raw_arg.input.to_string(),
        cursor: raw_arg.start + clamped_local_cursor,
    };

    error_types::DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR.create(&context)
}

fn validate_sequence_name(raw_arg: RawArg<'_>) -> Result<&str, CommandSyntaxError> {
    let value = raw_arg.value;
    if value.is_empty() {
        return Err(syntax_expected_separator(raw_arg, 0));
    }

    let mut namespace_separator_at = None;
    for (index, c) in value.char_indices() {
        if !is_valid_namespaced_id_char(c) {
            return Err(syntax_expected_separator(raw_arg, index));
        }
        if c == ':' {
            if namespace_separator_at.is_some() {
                return Err(syntax_expected_separator(raw_arg, index));
            }
            namespace_separator_at = Some(index);
        }
    }

    if let Some(separator_index) = namespace_separator_at {
        if separator_index == 0 {
            return Err(syntax_expected_separator(raw_arg, 0));
        }
        if separator_index + 1 == value.len() {
            return Err(syntax_expected_separator(raw_arg, value.len()));
        }
    }

    Ok(value)
}

#[derive(Clone, Copy)]
enum DrawMode {
    Value,
    Roll,
}

#[derive(Clone, Copy)]
struct DrawExecutor {
    mode: DrawMode,
    uses_sequence: bool,
}

#[derive(Clone, Copy)]
enum ResetTarget {
    All,
    Sequence,
}

#[derive(Clone, Copy)]
struct ResetExecutor {
    target: ResetTarget,
}

#[derive(Clone, Copy, Debug, Default)]
struct InclusiveRange {
    min: i32,
    max: i32,
}

struct RangeArgumentType;

impl ArgumentType for RangeArgumentType {
    type Item = (i32, i32);

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();

        let min = if reader.peek() == Some('.') {
            None
        } else {
            Some(reader.read_int()?)
        };

        let has_separator = reader.peek() == Some('.') && reader.peek_with_offset(1) == Some('.');

        let (min, max) = if has_separator {
            reader.expect('.')?;
            reader.expect('.')?;

            let max = match reader.peek() {
                None => None,
                Some(c) if c.is_whitespace() => None,
                _ => Some(reader.read_int()?),
            };

            (min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX))
        } else {
            let Some(value) = min else {
                reader.set_cursor(start);
                return Err(error_types::READER_EXPECTED_INT.create(reader));
            };

            (value, value)
        };

        let range_size = i64::from(max) - i64::from(min) + 1;
        if range_size < 2 {
            return Err(
                error_types::DISPATCHER_PARSE_EXCEPTION.create_without_context(
                    TextComponent::translate(
                        translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
                        [],
                    ),
                ),
            );
        }
        if range_size > 2_147_483_646 {
            return Err(
                error_types::DISPATCHER_PARSE_EXCEPTION.create_without_context(
                    TextComponent::translate(
                        translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
                        [],
                    ),
                ),
            );
        }

        Ok((min, max))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::String(StringProtoArgBehavior::SingleWord)
    }
}

impl InclusiveRange {
    #[cfg(test)]
    fn parse(range: &str) -> Result<Self, CommandError> {
        let (min, max) = if let Some((min_raw, max_raw)) = range.split_once("..") {
            let min = if min_raw.is_empty() {
                i32::MIN
            } else {
                parse_i32_range_bound(min_raw)?
            };
            let max = if max_raw.is_empty() {
                i32::MAX
            } else {
                parse_i32_range_bound(max_raw)?
            };

            (min, max)
        } else {
            let value = parse_i32_range_bound(range)?;
            (value, value)
        };

        let range_size = i64::from(max) - i64::from(min) + 1;
        if range_size < 2 {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
                [],
            )));
        }
        if range_size > 2_147_483_646 {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
                [],
            )));
        }

        Ok(Self { min, max })
    }
}

#[cfg(test)]
fn parse_i32_range_bound(raw: &str) -> Result<i32, CommandError> {
    raw.parse::<i32>().map_err(|_| {
        CommandError::CommandFailed(TextComponent::translate(
            translation::PARSING_INT_INVALID,
            [TextComponent::text(raw.to_string())],
        ))
    })
}

#[derive(Clone, Copy, Debug)]
struct SequenceParameters {
    seed: i64,
    include_world_seed: bool,
    include_sequence_id: bool,
}

impl Default for SequenceParameters {
    fn default() -> Self {
        // Snapshot 23w31a default parameters for random sequences.
        Self {
            seed: 0,
            include_world_seed: true,
            include_sequence_id: true,
        }
    }
}

impl From<RandomSequence> for SequenceParameters {
    fn from(value: RandomSequence) -> Self {
        Self {
            seed: value.seed,
            include_world_seed: value.include_world_seed,
            include_sequence_id: value.include_sequence_id,
        }
    }
}

impl From<SequenceParameters> for RandomSequence {
    fn from(value: SequenceParameters) -> Self {
        Self {
            seed: value.seed,
            include_world_seed: value.include_world_seed,
            include_sequence_id: value.include_sequence_id,
        }
    }
}

struct SequenceState {
    rng: Xoroshiro,
}

impl SequenceState {
    fn new(parameters: SequenceParameters, world_seed: i64, sequence_id: &str) -> Self {
        let effective_seed = derive_sequence_seed(parameters, world_seed, sequence_id);
        Self {
            rng: Xoroshiro::from_seed(effective_seed),
        }
    }

    fn sample(&mut self, range: InclusiveRange) -> i32 {
        self.rng.next_inbetween_i32(range.min, range.max)
    }
}

#[derive(Default)]
struct WorldSequenceState {
    defaults: SequenceParameters,
    sequences: HashMap<String, SequenceState>,
}

#[derive(Default)]
struct SequenceStore {
    worlds: HashMap<String, WorldSequenceState>,
}

static RANDOM_SEQUENCES: LazyLock<Mutex<SequenceStore>> =
    LazyLock::new(|| Mutex::new(SequenceStore::default()));

fn world_key(server: &crate::server::Server) -> String {
    server
        .basic_config
        .get_world_path()
        .to_string_lossy()
        .into_owned()
}

fn world_seed(server: &crate::server::Server) -> i64 {
    server.level_info.load().world_gen_settings.seed
}

const fn stafford_mix_13(value: u64) -> u64 {
    let value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn sequence_id_hash(sequence_id: &str) -> u64 {
    const OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const PRIME: u64 = 1_099_511_628_211;

    let mut hash = OFFSET_BASIS;
    for byte in sequence_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn derive_sequence_seed(parameters: SequenceParameters, world_seed: i64, sequence_id: &str) -> u64 {
    let mut seed = parameters.seed as u64;
    if parameters.include_world_seed {
        seed ^= world_seed as u64;
    }
    if parameters.include_sequence_id {
        seed ^= sequence_id_hash(sequence_id);
    }
    stafford_mix_13(seed)
}

fn load_sequence_parameters(
    server: &crate::server::Server,
    sequence_id: &str,
) -> Option<SequenceParameters> {
    let info = server.level_info.load();
    info.random_sequences
        .get(sequence_id)
        .cloned()
        .map(SequenceParameters::from)
}

fn persist_sequence_parameters(
    server: &crate::server::Server,
    sequence_id: &str,
    parameters: SequenceParameters,
) {
    let current_info = server.level_info.load();
    let mut new_info = (**current_info).clone();
    new_info
        .random_sequences
        .insert(sequence_id.to_string(), parameters.into());
    server.level_info.store(Arc::new(new_info));
}

fn remove_sequence_parameters(server: &crate::server::Server, sequence_id: &str) {
    let current_info = server.level_info.load();
    let mut new_info = (**current_info).clone();
    new_info.random_sequences.remove(sequence_id);
    server.level_info.store(Arc::new(new_info));
}

async fn sample_sequence_value(
    server: &crate::server::Server,
    sequence_id: &str,
    range: InclusiveRange,
) -> i32 {
    let world_seed = world_seed(server);
    let key = world_key(server);
    let persisted_parameters = load_sequence_parameters(server, sequence_id);

    let mut store = RANDOM_SEQUENCES.lock().await;
    let world_state = store.worlds.entry(key).or_default();
    let parameters = persisted_parameters.unwrap_or(world_state.defaults);

    let sequence = world_state
        .sequences
        .entry(sequence_id.to_string())
        .or_insert_with(|| SequenceState::new(parameters, world_seed, sequence_id));

    let result = sequence.sample(range);
    drop(store);

    if persisted_parameters.is_none() {
        persist_sequence_parameters(server, sequence_id, parameters);
    }

    result
}

async fn reset_all_sequences(
    server: &crate::server::Server,
    parameter_override: Option<SequenceParameters>,
) -> (i32, SequenceParameters) {
    let key = world_key(server);
    let mut store = RANDOM_SEQUENCES.lock().await;
    let world_state = store.worlds.entry(key).or_default();

    let new_defaults = parameter_override.unwrap_or_default();
    world_state.defaults = new_defaults;
    world_state.sequences.clear();
    drop(store);

    let current_info = server.level_info.load();
    let reset_count = usize_to_i32_saturating(current_info.random_sequences.len());
    let mut new_info = (**current_info).clone();
    new_info.random_sequences.clear();
    server.level_info.store(Arc::new(new_info));

    (reset_count, new_defaults)
}

async fn reset_sequence(
    server: &crate::server::Server,
    sequence_id: &str,
    parameter_override: Option<SequenceParameters>,
) -> SequenceParameters {
    let world_seed = world_seed(server);
    let key = world_key(server);

    let mut store = RANDOM_SEQUENCES.lock().await;
    let world_state = store.worlds.entry(key).or_default();

    let parameters = parameter_override.unwrap_or_default();
    world_state.sequences.insert(
        sequence_id.to_string(),
        SequenceState::new(parameters, world_seed, sequence_id),
    );
    drop(store);

    if parameter_override.is_some() {
        persist_sequence_parameters(server, sequence_id, parameters);
    } else {
        remove_sequence_parameters(server, sequence_id);
    }
    parameters
}

fn usize_to_i32_saturating(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn get_optional_argument<'a, T: 'static>(
    context: &'a CommandContext,
    name: &str,
) -> Result<Option<&'a T>, CommandSyntaxError> {
    if context.arguments.contains_key(name) {
        context.get_argument(name).map(Some)
    } else {
        Ok(None)
    }
}

fn parse_range_arg(context: &CommandContext) -> Result<InclusiveRange, CommandSyntaxError> {
    let &(min, max) = context.get_argument::<(i32, i32)>(ARG_RANGE)?;
    Ok(InclusiveRange { min, max })
}

fn parse_sequence_arg<'a>(context: &'a CommandContext) -> Result<&'a str, CommandSyntaxError> {
    let sequence = context.get_argument::<String>(ARG_SEQUENCE)?;
    let raw_arg = RawArg {
        value: sequence,
        start: 0,
        end: sequence.len(),
        input: sequence,
    };

    validate_sequence_name(raw_arg)
}

fn parse_reset_parameters(
    context: &CommandContext,
) -> Result<Option<SequenceParameters>, CommandSyntaxError> {
    let Some(seed) = get_optional_argument::<i64>(context, ARG_SEED)? else {
        return Ok(None);
    };

    let include_world_seed =
        *get_optional_argument::<bool>(context, ARG_INCLUDE_WORLD_SEED)?.unwrap_or(&true);
    let include_sequence_id =
        *get_optional_argument::<bool>(context, ARG_INCLUDE_SEQUENCE_ID)?.unwrap_or(&true);

    Ok(Some(SequenceParameters {
        seed: *seed,
        include_world_seed,
        include_sequence_id,
    }))
}

const fn seed_argument_type() -> LongArgumentType {
    LongArgumentType::any()
}

fn level_two_requirement(
    source: &CommandSource,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
    Box::pin(async move { source.output.has_permission_lvl(PermissionLvl::Two) })
}

impl CommandExecutor for DrawExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let range = parse_range_arg(context)?;

            let sequence = if self.uses_sequence {
                Some(parse_sequence_arg(context)?)
            } else {
                None
            };

            let result = if let Some(sequence_id) = sequence {
                sample_sequence_value(context.server(), sequence_id, range).await
            } else {
                rand::rng().random_range(range.min..=range.max)
            };

            match self.mode {
                DrawMode::Value => {
                    context
                        .source
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                            [TextComponent::text(result.to_string())],
                        ))
                        .await;
                }
                DrawMode::Roll => {
                    let message = TextComponent::translate(
                        translation::COMMANDS_RANDOM_ROLL,
                        [
                            TextComponent::text(context.source.output.to_string()),
                            TextComponent::text(result.to_string()),
                            TextComponent::text(range.min.to_string()),
                            TextComponent::text(range.max.to_string()),
                        ],
                    );

                    context
                        .server()
                        .broadcast_message(
                            &message,
                            &TextComponent::text(context.source.output.to_string()),
                            SAY_COMMAND,
                            None,
                        )
                        .await;
                }
            }

            Ok(result)
        })
    }
}

impl CommandExecutor for ResetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let parameter_override = parse_reset_parameters(context)?;

            match self.target {
                ResetTarget::All => {
                    let (reset_count, _) =
                        reset_all_sequences(context.server(), parameter_override).await;
                    context
                        .source
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_RANDOM_RESET_ALL_SUCCESS,
                            [TextComponent::text(reset_count.to_string())],
                        ))
                        .await;
                    Ok(reset_count)
                }
                ResetTarget::Sequence => {
                    let sequence_id = parse_sequence_arg(context)?;

                    let _ = reset_sequence(context.server(), sequence_id, parameter_override).await;
                    context
                        .source
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_RANDOM_RESET_SUCCESS,
                            [TextComponent::text(sequence_id.to_string())],
                        ))
                        .await;

                    Ok(1)
                }
            }
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Allow,
    ));

    dispatcher.register(
        command("random", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("value").then(
                    argument(ARG_RANGE, RangeArgumentType)
                        .executes(DrawExecutor {
                            mode: DrawMode::Value,
                            uses_sequence: false,
                        })
                        .then(
                            argument(ARG_SEQUENCE, StringArgumentType::SingleWord)
                                .requires(level_two_requirement)
                                .executes(DrawExecutor {
                                    mode: DrawMode::Value,
                                    uses_sequence: true,
                                }),
                        ),
                ),
            )
            .then(
                literal("roll").then(
                    argument(ARG_RANGE, RangeArgumentType)
                        .executes(DrawExecutor {
                            mode: DrawMode::Roll,
                            uses_sequence: false,
                        })
                        .then(
                            argument(ARG_SEQUENCE, StringArgumentType::SingleWord)
                                .requires(level_two_requirement)
                                .executes(DrawExecutor {
                                    mode: DrawMode::Roll,
                                    uses_sequence: true,
                                }),
                        ),
                ),
            )
            .then(
                literal("reset")
                    .requires(level_two_requirement)
                    .then(
                        literal("*")
                            .executes(ResetExecutor {
                                target: ResetTarget::All,
                            })
                            .then(
                                argument(ARG_SEED, seed_argument_type())
                                    .executes(ResetExecutor {
                                        target: ResetTarget::All,
                                    })
                                    .then(
                                        argument(ARG_INCLUDE_WORLD_SEED, BoolArgumentType)
                                            .executes(ResetExecutor {
                                                target: ResetTarget::All,
                                            })
                                            .then(
                                                argument(ARG_INCLUDE_SEQUENCE_ID, BoolArgumentType)
                                                    .executes(ResetExecutor {
                                                        target: ResetTarget::All,
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                    .then(
                        argument(ARG_SEQUENCE, StringArgumentType::SingleWord)
                            .executes(ResetExecutor {
                                target: ResetTarget::Sequence,
                            })
                            .then(
                                argument(ARG_SEED, seed_argument_type())
                                    .executes(ResetExecutor {
                                        target: ResetTarget::Sequence,
                                    })
                                    .then(
                                        argument(ARG_INCLUDE_WORLD_SEED, BoolArgumentType)
                                            .executes(ResetExecutor {
                                                target: ResetTarget::Sequence,
                                            })
                                            .then(
                                                argument(ARG_INCLUDE_SEQUENCE_ID, BoolArgumentType)
                                                    .executes(ResetExecutor {
                                                        target: ResetTarget::Sequence,
                                                    }),
                                            ),
                                    ),
                            ),
                    ),
            ),
    );
}

#[cfg(test)]
mod test {
    use super::{
        CommandError, InclusiveRange, SequenceParameters, derive_sequence_seed,
        validate_sequence_name,
    };
    use crate::command::{errors::error_types, tree::RawArg};

    #[test]
    fn parse_valid_closed_range() {
        let range = InclusiveRange::parse("1..10").expect("range should parse");
        assert_eq!(range.min, 1);
        assert_eq!(range.max, 10);
    }

    #[test]
    fn parse_valid_open_lower_bound_range() {
        let range = InclusiveRange::parse("..-2147483647").expect("range should parse");
        assert_eq!(range.min, i32::MIN);
        assert_eq!(range.max, -2_147_483_647);
    }

    #[test]
    fn parse_valid_open_upper_bound_range() {
        let range = InclusiveRange::parse("2147483646..").expect("range should parse");
        assert_eq!(range.min, 2_147_483_646);
        assert_eq!(range.max, i32::MAX);
    }

    #[test]
    fn reject_single_value_range() {
        assert!(matches!(
            InclusiveRange::parse("5"),
            Err(CommandError::CommandFailed(_))
        ));
    }

    #[test]
    fn reject_reversed_range() {
        assert!(matches!(
            InclusiveRange::parse("10..1"),
            Err(CommandError::CommandFailed(_))
        ));
    }

    #[test]
    fn reject_too_large_range_size() {
        assert!(matches!(
            InclusiveRange::parse("-2147483648..2147483647"),
            Err(CommandError::CommandFailed(_))
        ));
    }

    #[test]
    fn derived_seed_depends_on_sequence_id_when_enabled() {
        let params = SequenceParameters {
            seed: 123,
            include_world_seed: false,
            include_sequence_id: true,
        };
        assert_ne!(
            derive_sequence_seed(params, 0, "pumpkin:first"),
            derive_sequence_seed(params, 0, "pumpkin:second")
        );
    }

    #[test]
    fn derived_seed_ignores_sequence_id_when_disabled() {
        let params = SequenceParameters {
            seed: 123,
            include_world_seed: false,
            include_sequence_id: false,
        };
        assert_eq!(
            derive_sequence_seed(params, 0, "pumpkin:first"),
            derive_sequence_seed(params, 0, "pumpkin:second")
        );
    }

    #[test]
    fn sequence_name_allows_lowercase_namespaced_ids() {
        let input = "random reset pumpkin:test/path_1";
        let raw_arg = RawArg {
            value: "pumpkin:test/path_1",
            start: 13,
            end: input.len(),
            input,
        };

        assert_eq!(
            validate_sequence_name(raw_arg).expect("name should be valid"),
            "pumpkin:test/path_1"
        );
    }

    #[test]
    fn sequence_name_rejects_uppercase_with_precise_cursor() {
        let input = "random reset seqA 111 true true";
        let raw_arg = RawArg {
            value: "seqA",
            start: 13,
            end: 17,
            input,
        };

        let error = validate_sequence_name(raw_arg).expect_err("name should be rejected");
        assert!(error.is(&error_types::DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR));
        assert_eq!(
            error
                .context
                .expect("syntax error should have context")
                .cursor,
            16
        );
    }
}
