use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI8, Ordering},
};

use super::BlockEntity;
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;

pub use pumpkin_data::dye_color::DyeColor;

/// Extra reach allowed when checking that a sign editor is still near the sign.
///
/// Added on top of the block interaction range. Vanilla passes the same value to
/// `Player#isWithinBlockInteractionRange` from `SignBlockEntity#playerIsTooFarAwayToEdit`.
pub const SIGN_EDIT_RANGE_BUFFER: f64 = 4.0;

pub struct SignBlockEntity {
    pub front_text: SignText,
    pub back_text: SignText,
    pub is_waxed: AtomicBool,
    pub allow_op_features: AtomicBool,
    position: BlockPos,
    pub currently_editing_player: Arc<Mutex<Option<uuid::Uuid>>>,
}

pub type Text = SignText;

pub struct SignText {
    pub has_glowing_text: AtomicBool,
    color: AtomicI8,
    pub messages: Arc<Mutex<[Box<str>; 4]>>,
    pub filtered_messages: Arc<Mutex<[Box<str>; 4]>>,
}

impl Clone for SignText {
    fn clone(&self) -> Self {
        Self {
            has_glowing_text: AtomicBool::new(self.has_glowing_text.load(Ordering::Relaxed)),
            color: AtomicI8::new(self.color.load(Ordering::Relaxed)),
            messages: self.messages.clone(),
            filtered_messages: self.filtered_messages.clone(),
        }
    }
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            has_glowing_text: AtomicBool::new(false),
            color: AtomicI8::new(DyeColor::Black as i8),
            messages: Arc::new(Mutex::new(Self::empty_messages())),
            filtered_messages: Arc::new(Mutex::new(Self::empty_messages())),
        }
    }
}

fn component_to_json(component: &TextComponent) -> Box<str> {
    Box::from(component.to_json_for_version(&CURRENT_MC_VERSION))
}

/// Converts a stored NBT message to JSON.
///
/// String tags are literal text unless the sign allows op features, where they may be
/// component JSON written by an older Pumpkin version.
fn nbt_message_to_json(tag: &NbtTag, allow_op_features: bool) -> Box<str> {
    match tag {
        NbtTag::String(value) => {
            if allow_op_features && let Ok(component) = serde_json::from_str::<TextComponent>(value)
            {
                return component_to_json(&component);
            }
            component_to_json(&TextComponent::text(value.to_string()))
        }
        _ => component_to_json(&TextComponent::from_nbt(tag)),
    }
}

/// Converts a stored JSON message to NBT, writing parsed components as compounds.
///
/// Writing a simple literal back as a string tag would let an enabled sign promote
/// saved client text into a component with click actions on the next load.
fn json_message_to_nbt(message: &str) -> NbtTag {
    serde_json::from_str::<TextComponent>(message).map_or_else(
        |_| NbtTag::String(Box::from(message)),
        |component| NbtTag::Compound(component.to_nbt_compound_for_version(&CURRENT_MC_VERSION)),
    )
}

/// Wraps a client-submitted line in a literal component, keeping the original line's style.
fn literal_message(raw: &str, original: &str) -> Box<str> {
    let style = serde_json::from_str::<TextComponent>(original)
        .map(|component| component.0.style)
        .unwrap_or_default();
    let mut component = TextComponent::text(raw.to_string());
    component.0.style = style;
    component_to_json(&component)
}

/// Returns whether a stored line has visible component text rather than only structure.
fn message_has_text(message: &str) -> bool {
    if message.is_empty() {
        return false;
    }
    serde_json::from_str::<TextComponent>(message).map_or(true, |component| {
        !component.0.to_bedrock_string().is_empty()
    })
}

#[allow(clippy::fallible_impl_from)]
impl From<SignText> for NbtTag {
    fn from(value: SignText) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_bool(
            "has_glowing_text",
            value.has_glowing_text.load(Ordering::Relaxed),
        );
        nbt.put_string("color", value.get_color().name().to_string());

        let messages = value
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nbt.put_list(
            "messages",
            messages.iter().map(|s| json_message_to_nbt(s)).collect(),
        );

        let filtered_messages = value
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *filtered_messages != *messages {
            nbt.put_list(
                "filtered_messages",
                filtered_messages
                    .iter()
                    .map(|s| json_message_to_nbt(s))
                    .collect(),
            );
        }

        Self::Compound(nbt)
    }
}

impl From<NbtTag> for SignText {
    fn from(tag: NbtTag) -> Self {
        Self::from_nbt_tag(&tag, false)
    }
}

impl SignText {
    /// Converts a sign text compound, preserving legacy component strings only for trusted signs.
    pub fn from_nbt_tag(tag: &NbtTag, allow_op_features: bool) -> Self {
        let Some(nbt) = tag.extract_compound() else {
            return Self::default();
        };
        let has_glowing_text = nbt.get_bool("has_glowing_text").unwrap_or(false);
        let color = nbt.get_string("color").unwrap_or("black");
        let messages: Vec<Box<str>> = nbt.get_list("messages").map_or_else(Vec::new, |list| {
            list.iter()
                .map(|tag| nbt_message_to_json(tag, allow_op_features))
                .collect()
        });
        let get_message =
            |i: usize| -> Box<str> { messages.get(i).cloned().unwrap_or_else(|| Box::from("")) };

        let parsed_messages = [
            get_message(0),
            get_message(1),
            get_message(2),
            get_message(3),
        ];

        let filtered_messages: Option<Vec<Box<str>>> =
            nbt.get_list("filtered_messages").map(|list| {
                list.iter()
                    .map(|tag| nbt_message_to_json(tag, allow_op_features))
                    .collect()
            });

        let parsed_filtered = filtered_messages.map_or_else(
            || parsed_messages.clone(),
            |filtered| {
                let get_filtered = |i: usize| -> Box<str> {
                    filtered.get(i).cloned().unwrap_or_else(|| Box::from(""))
                };
                [
                    get_filtered(0),
                    get_filtered(1),
                    get_filtered(2),
                    get_filtered(3),
                ]
            },
        );

        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(DyeColor::by_name(color).unwrap_or(DyeColor::Black).id() as i8),
            messages: Arc::new(Mutex::new(parsed_messages)),
            filtered_messages: Arc::new(Mutex::new(parsed_filtered)),
        }
    }
}

impl SignText {
    pub const LINES: usize = 4;

    #[must_use]
    pub fn empty_messages() -> [Box<str>; 4] {
        [Box::from(""), Box::from(""), Box::from(""), Box::from("")]
    }

    #[must_use]
    pub fn new(
        messages: [Box<str>; 4],
        filtered_messages: Option<[Box<str>; 4]>,
        color: DyeColor,
        has_glowing_text: bool,
    ) -> Self {
        let filtered = filtered_messages.unwrap_or_else(|| messages.clone());
        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(color.id() as i8),
            messages: Arc::new(Mutex::new(messages)),
            filtered_messages: Arc::new(Mutex::new(filtered)),
        }
    }

    #[must_use]
    pub fn from_messages(messages: [Box<str>; 4]) -> Self {
        Self::new(messages, None, DyeColor::Black, false)
    }

    #[must_use]
    pub fn has_glowing_text(&self) -> bool {
        self.has_glowing_text.load(Ordering::Relaxed)
    }

    pub fn set_has_glowing_text(&self, has_glowing_text: bool) {
        self.has_glowing_text
            .store(has_glowing_text, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_color(&self) -> DyeColor {
        let c = self.color.load(Ordering::Relaxed);
        if c >= 0 {
            DyeColor::by_id(c as u8).unwrap_or(DyeColor::Black)
        } else {
            DyeColor::Black
        }
    }

    pub fn set_color(&self, color: DyeColor) {
        self.color.store(color.id() as i8, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_message(&self, index: usize, should_filter: bool) -> Box<str> {
        if index >= Self::LINES {
            return Box::from("");
        }
        let lock = if should_filter {
            self.filtered_messages.lock()
        } else {
            self.messages.lock()
        };
        lock.unwrap_or_else(std::sync::PoisonError::into_inner)[index].clone()
    }

    pub fn set_message(&self, index: usize, message: Box<str>, filtered_message: Option<Box<str>>) {
        if index >= Self::LINES {
            return;
        }
        let filtered = filtered_message.unwrap_or_else(|| message.clone());
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = message;
        self.filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = filtered;
    }

    #[must_use]
    pub fn get_messages(&self, should_filter: bool) -> [Box<str>; 4] {
        let lock = if should_filter {
            self.filtered_messages.lock()
        } else {
            self.messages.lock()
        };
        lock.unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Returns whether any line has visible text content.
    #[must_use]
    pub fn has_message(&self, should_filter: bool) -> bool {
        let messages = self.get_messages(should_filter);
        messages.iter().any(|msg| message_has_text(msg))
    }

    #[must_use]
    pub fn has_any_click_commands(&self, should_filter: bool) -> bool {
        let messages = self.get_messages(should_filter);
        for msg in &messages {
            if msg.is_empty() || !msg.starts_with('{') {
                continue;
            }
            if let Ok(component) = serde_json::from_str::<TextComponent>(msg)
                && let Some(ClickEvent::RunCommand { .. }) = &component.0.style.click_event
            {
                return true;
            }
        }
        false
    }

    /// Replaces the raw and filtered messages with lines submitted by a client.
    ///
    /// Each line is stored as a literal component with the previous line's style,
    /// matching `SignBlockEntity#updateMessages`. A submitted line must never keep
    /// the component syntax the client typed, because that could carry a click event.
    pub fn set_client_messages(&self, messages: &[Box<str>; 4]) {
        let mut raw = self
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let new_messages = std::array::from_fn(|i| literal_message(&messages[i], &raw[i]));
        raw.clone_from(&new_messages);
        drop(raw);

        let mut filtered = self
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *filtered = new_messages;
    }
}

impl BlockEntity for SignBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let allow_op_features = nbt.get_bool("allow_op_features").unwrap_or(false);
        let front_text = nbt
            .get("front_text")
            .map(|tag| SignText::from_nbt_tag(tag, allow_op_features))
            .unwrap_or_default();
        let back_text = nbt
            .get("back_text")
            .map(|tag| SignText::from_nbt_tag(tag, allow_op_features))
            .unwrap_or_default();
        let is_waxed = nbt.get_bool("is_waxed").unwrap_or(false);
        Self {
            position,
            front_text,
            back_text,
            is_waxed: AtomicBool::new(is_waxed),
            allow_op_features: AtomicBool::new(allow_op_features),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        write_allow_op_features(nbt, &self.allow_op_features);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        write_allow_op_features(&mut nbt, &self.allow_op_features);
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        Some(build_sign_actor(
            self.position,
            false,
            &self.front_text,
            &self.back_text,
            self.is_waxed.load(Ordering::Relaxed),
        ))
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        collect_sign_item_components(&self.front_text, &self.back_text, &self.is_waxed, stack);
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        apply_sign_item_components(&self.front_text, &self.back_text, &self.is_waxed, stack);
        // Item data must not be able to enable op-level click commands.
        self.allow_op_features.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) fn apply_sign_item_components(
    front_text: &SignText,
    back_text: &SignText,
    is_waxed: &AtomicBool,
    stack: &pumpkin_data::item_stack::ItemStack,
) {
    use pumpkin_data::data_component_impl::{SignTextBackImpl, SignTextFrontImpl, WaxedImpl};

    if let Some(front) = stack.get_data_component::<SignTextFrontImpl>() {
        front_text.set_color(front.color);
        front_text.set_has_glowing_text(front.has_glowing_text);
        for (i, msg) in front.messages.iter().enumerate() {
            let filtered = front
                .filtered_messages
                .as_ref()
                .map(|f| Box::<str>::from(f[i].as_ref()));
            front_text.set_message(i, Box::<str>::from(msg.as_ref()), filtered);
        }
    }
    if let Some(back) = stack.get_data_component::<SignTextBackImpl>() {
        back_text.set_color(back.color);
        back_text.set_has_glowing_text(back.has_glowing_text);
        for (i, msg) in back.messages.iter().enumerate() {
            let filtered = back
                .filtered_messages
                .as_ref()
                .map(|f| Box::<str>::from(f[i].as_ref()));
            back_text.set_message(i, Box::<str>::from(msg.as_ref()), filtered);
        }
    }
    is_waxed.store(
        stack.get_data_component::<WaxedImpl>().is_some(),
        Ordering::Relaxed,
    );
}

pub(crate) fn collect_sign_item_components(
    front_text: &SignText,
    back_text: &SignText,
    is_waxed: &AtomicBool,
    stack: &mut pumpkin_data::item_stack::ItemStack,
) {
    use pumpkin_data::data_component_impl::{SignTextBackImpl, SignTextFrontImpl, WaxedImpl};
    use std::borrow::Cow;

    let to_cow_lines = |lines: [Box<str>; 4]| -> [Cow<'static, str>; 4] {
        lines.map(|line| Cow::Owned(line.into_string()))
    };

    let front_msgs = front_text.get_messages(false);
    let front_filtered = front_text.get_messages(true);
    stack.set_data_component(SignTextFrontImpl {
        filtered_messages: (front_filtered != front_msgs).then(|| to_cow_lines(front_filtered)),
        messages: to_cow_lines(front_msgs),
        color: front_text.get_color(),
        has_glowing_text: front_text.has_glowing_text(),
    });

    let back_msgs = back_text.get_messages(false);
    let back_filtered = back_text.get_messages(true);
    stack.set_data_component(SignTextBackImpl {
        filtered_messages: (back_filtered != back_msgs).then(|| to_cow_lines(back_filtered)),
        messages: to_cow_lines(back_msgs),
        color: back_text.get_color(),
        has_glowing_text: back_text.has_glowing_text(),
    });

    if is_waxed.load(Ordering::Relaxed) {
        stack.set_data_component(WaxedImpl);
    }
}

impl SignBlockEntity {
    pub const ID: &'static str = "minecraft:sign";

    #[must_use]
    pub fn new(position: BlockPos, is_front: bool, messages: [Box<str>; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            allow_op_features: AtomicBool::new(false),
            front_text: if is_front {
                SignText::from_messages(messages.clone())
            } else {
                SignText::default()
            },
            back_text: if is_front {
                SignText::default()
            } else {
                SignText::from_messages(messages)
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn empty(position: BlockPos) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            allow_op_features: AtomicBool::new(false),
            front_text: SignText::default(),
            back_text: SignText::default(),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
}

pub enum SignEntityRef<'a> {
    Sign(&'a SignBlockEntity),
    Hanging(&'a super::hanging_sign::HangingSignBlockEntity),
}

impl<'a> SignEntityRef<'a> {
    pub fn from_block_entity(entity: &'a dyn BlockEntity) -> Option<Self> {
        entity
            .as_any()
            .downcast_ref::<SignBlockEntity>()
            .map(Self::Sign)
            .or_else(|| {
                entity
                    .as_any()
                    .downcast_ref::<super::hanging_sign::HangingSignBlockEntity>()
                    .map(Self::Hanging)
            })
    }

    #[must_use]
    pub const fn front_text(&self) -> &'a SignText {
        match self {
            Self::Sign(s) => &s.front_text,
            Self::Hanging(s) => &s.front_text,
        }
    }

    #[must_use]
    pub const fn back_text(&self) -> &'a SignText {
        match self {
            Self::Sign(s) => &s.back_text,
            Self::Hanging(s) => &s.back_text,
        }
    }

    #[must_use]
    pub const fn get_text(&self, is_front: bool) -> &'a SignText {
        if is_front {
            self.front_text()
        } else {
            self.back_text()
        }
    }

    #[must_use]
    pub fn is_waxed(&self) -> bool {
        match self {
            Self::Sign(s) => s.is_waxed.load(Ordering::Relaxed),
            Self::Hanging(s) => s.is_waxed.load(Ordering::Relaxed),
        }
    }

    pub fn set_waxed(&self, waxed: bool) {
        match self {
            Self::Sign(s) => s.is_waxed.store(waxed, Ordering::Relaxed),
            Self::Hanging(s) => s.is_waxed.store(waxed, Ordering::Relaxed),
        }
    }

    #[must_use]
    pub const fn currently_editing_player(&self) -> &'a Arc<Mutex<Option<uuid::Uuid>>> {
        match self {
            Self::Sign(s) => &s.currently_editing_player,
            Self::Hanging(s) => &s.currently_editing_player,
        }
    }

    /// Returns whether this sign is allowed to run op-level click commands.
    #[must_use]
    pub fn allow_op_features(&self) -> bool {
        match self {
            Self::Sign(s) => s.allow_op_features.load(Ordering::Relaxed),
            Self::Hanging(s) => s.allow_op_features.load(Ordering::Relaxed),
        }
    }

    /// Returns the sign's block position.
    #[must_use]
    pub fn block_pos(&self) -> BlockPos {
        match self {
            Self::Sign(s) => s.get_position(),
            Self::Hanging(s) => s.get_position(),
        }
    }

    /// Mirrors the gate in `SignBlockEntity#updateSignText`: only the player the editor was
    /// opened for may write text, and only while they can still reach the sign. Vanilla keeps
    /// that registration from going stale in `SignBlockEntity#tick`; Pumpkin has no sign
    /// ticker, so `SignBlockEntity#playerIsTooFarAwayToEdit` is checked here instead.
    #[must_use]
    pub fn may_update_text(&self, world: &World, player: &Player) -> bool {
        if self.is_waxed() {
            return false;
        }

        let mut editor = self
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *editor != Some(player.gameprofile.id) {
            return false;
        }

        let pos = self.block_pos();
        if !world.level.is_chunk_loaded(&pos.chunk_position()) {
            return false;
        }
        if !player.can_interact_with_block_at(&pos, SIGN_EDIT_RANGE_BUFFER) {
            // Vanilla clears this from `SignBlockEntity#tick`; without a sign ticker,
            // clear it when the registered editor is out of range.
            *editor = None;
            return false;
        }
        true
    }
}

/// Writes the sign's operator-feature flag, matching vanilla's write-only-when-true NBT.
pub(crate) fn write_allow_op_features(nbt: &mut NbtCompound, allow_op_features: &AtomicBool) {
    if allow_op_features.load(Ordering::Relaxed) {
        nbt.put_bool("allow_op_features", true);
    }
}

/// Converts a [`DyeColor`] into the signed 32-bit ARGB integer used by Bedrock signs.
#[must_use]
pub const fn dye_color_to_bedrock_sign_text_color(color: DyeColor) -> i32 {
    match color {
        DyeColor::White => -986896,
        DyeColor::Orange => -425955,
        DyeColor::Magenta => -3715395,
        DyeColor::LightBlue => -12930086,
        DyeColor::Yellow => -75715,
        DyeColor::Lime => -8337633,
        DyeColor::Pink => -816214,
        DyeColor::Gray => -12103854,
        DyeColor::LightGray => -6447721,
        DyeColor::Cyan => -15295332,
        DyeColor::Purple => -7785800,
        DyeColor::Blue => -12827478,
        DyeColor::Brown => -8170446,
        DyeColor::Green => -10585066,
        DyeColor::Red => -5231066,
        DyeColor::Black => -16777216,
    }
}

/// Maps a Bedrock sign text color ARGB integer back to a [`DyeColor`].
#[must_use]
pub const fn bedrock_sign_text_color_to_dye_color(color: i32) -> DyeColor {
    match color {
        -986896 => DyeColor::White,
        -425955 => DyeColor::Orange,
        -3715395 => DyeColor::Magenta,
        -12930086 => DyeColor::LightBlue,
        -75715 => DyeColor::Yellow,
        -8337633 => DyeColor::Lime,
        -816214 => DyeColor::Pink,
        -12103854 => DyeColor::Gray,
        -6447721 => DyeColor::LightGray,
        -15295332 => DyeColor::Cyan,
        -7785800 => DyeColor::Purple,
        -12827478 => DyeColor::Blue,
        -8170446 => DyeColor::Brown,
        -10585066 => DyeColor::Green,
        -5231066 => DyeColor::Red,
        _ => DyeColor::Black,
    }
}

/// Builds the Bedrock sub-compound for sign text (`FrontText` or `BackText`).
#[must_use]
pub fn build_sign_text_compound(sign_text: &SignText) -> NbtCompound {
    let mut compound = NbtCompound::new();
    let messages = sign_text.get_messages(false);
    let text = messages
        .iter()
        .map(|message| {
            serde_json::from_str::<TextComponent>(message).map_or_else(
                |_| message.to_string(),
                |component| component.0.to_bedrock_string(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    compound.put_string("Text", text);
    compound.put_int(
        "SignTextColor",
        dye_color_to_bedrock_sign_text_color(sign_text.get_color()),
    );
    compound.put_bool("IgnoreLighting", sign_text.has_glowing_text());
    compound.put_bool("PersistFormatting", true);
    compound
}

/// Builds the complete Bedrock block-actor NBT for standard or hanging signs.
#[must_use]
pub fn build_sign_actor(
    position: BlockPos,
    is_hanging: bool,
    front_text: &SignText,
    back_text: &SignText,
    is_waxed: bool,
) -> NbtCompound {
    let id = if is_hanging { "HangingSign" } else { "Sign" };
    super::bedrock::BedrockBlockActorBuilder::new(id, position)
        .put_compound("FrontText", build_sign_text_compound(front_text))
        .put_compound("BackText", build_sign_text_compound(back_text))
        .put_bool("IsWaxed", is_waxed)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_actor_builder() {
        let pos = BlockPos::new(1, 64, -2);
        let front = SignText::from_messages([
            Box::from("Welcome"),
            Box::from("To"),
            Box::from("Pumpkin"),
            Box::from("Server"),
        ]);
        front.set_color(DyeColor::Red);
        front.set_has_glowing_text(true);

        let back = SignText::from_messages([
            Box::from("Back"),
            Box::from("Side"),
            Box::from(""),
            Box::from(""),
        ]);
        back.set_color(DyeColor::Blue);

        let nbt = build_sign_actor(pos, false, &front, &back, false);
        assert_eq!(nbt.get_string("id"), Some("Sign"));
        assert_eq!(nbt.get_int("x"), Some(1));
        assert_eq!(nbt.get_int("y"), Some(64));
        assert_eq!(nbt.get_int("z"), Some(-2));
        assert_eq!(nbt.get_bool("isMovable"), Some(true));
        assert_eq!(nbt.get_bool("IsWaxed"), Some(false));

        let front_nbt = nbt.get_compound("FrontText").expect("FrontText missing");
        assert_eq!(
            front_nbt.get_string("Text"),
            Some("Welcome\nTo\nPumpkin\nServer")
        );
        assert_eq!(
            front_nbt.get_int("SignTextColor"),
            Some(dye_color_to_bedrock_sign_text_color(DyeColor::Red))
        );
        assert_eq!(front_nbt.get_bool("IgnoreLighting"), Some(true));

        let back_nbt = nbt.get_compound("BackText").expect("BackText missing");
        assert_eq!(back_nbt.get_string("Text"), Some("Back\nSide\n\n"));
        assert_eq!(
            back_nbt.get_int("SignTextColor"),
            Some(dye_color_to_bedrock_sign_text_color(DyeColor::Blue))
        );
        assert_eq!(back_nbt.get_bool("IgnoreLighting"), Some(false));
    }

    #[test]
    fn hanging_sign_actor_id() {
        let pos = BlockPos::new(0, 70, 0);
        let front = SignText::default();
        let back = SignText::default();
        let nbt = build_sign_actor(pos, true, &front, &back, true);
        assert_eq!(nbt.get_string("id"), Some("HangingSign"));
        assert_eq!(nbt.get_bool("IsWaxed"), Some(true));
    }

    #[test]
    fn allow_op_features_round_trips_through_nbt() {
        let pos = BlockPos::new(1, 64, 2);
        let sign = SignBlockEntity::empty(pos);

        let mut nbt = NbtCompound::new();
        sign.write_nbt(&mut nbt);
        assert_eq!(nbt.get_bool("allow_op_features"), None);

        sign.allow_op_features.store(true, Ordering::Relaxed);
        let mut nbt = NbtCompound::new();
        sign.write_nbt(&mut nbt);
        assert_eq!(nbt.get_bool("allow_op_features"), Some(true));

        let loaded = SignBlockEntity::from_nbt(&nbt, pos);
        assert!(loaded.allow_op_features.load(Ordering::Relaxed));
    }

    fn plain_text(message: &str) -> String {
        serde_json::from_str::<TextComponent>(message).map_or_else(
            |_| message.to_string(),
            |component| component.0.to_bedrock_string(),
        )
    }

    #[test]
    fn literal_client_messages_do_not_carry_click_commands() {
        let text = SignText::default();
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        text.set_client_messages(&[
            Box::from(crafted),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ]);

        assert!(!text.has_any_click_commands(false));
        assert_eq!(plain_text(&text.get_message(0, false)), crafted);
    }

    #[test]
    fn client_messages_keep_previous_style_and_filtered_text() {
        use pumpkin_util::text::color::{Color, NamedColor};

        let text = SignText::default();
        let styled = r#"{"text":"old","color":"red"}"#;
        *text
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = [
            Box::from(styled),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ];
        *text
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = [
            Box::from(styled),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ];

        text.set_client_messages(&[
            Box::from("new"),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ]);

        let messages = text.get_messages(false);
        assert_eq!(messages, text.get_messages(true));
        let component = serde_json::from_str::<TextComponent>(&messages[0])
            .expect("stored client message should be valid JSON");
        assert_eq!(component.0.style.color, Some(Color::Named(NamedColor::Red)));
        assert!(component.0.style.click_event.is_none());
    }

    #[test]
    fn bedrock_sign_text_keeps_client_lines_literal() {
        let text = SignText::default();
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        text.set_client_messages(&[
            Box::from(crafted),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ]);

        let nbt = build_sign_text_compound(&text);
        let expected = format!("{crafted}\n\n\n");
        assert_eq!(nbt.get_string("Text"), Some(expected.as_str()));
    }

    #[test]
    fn component_messages_round_trip_through_nbt() {
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        let text = SignText::from_messages([
            Box::from("hello"),
            Box::from(crafted),
            Box::from(""),
            Box::from(""),
        ]);

        let loaded = SignText::from(NbtTag::from(text));

        assert_eq!(plain_text(&loaded.get_message(0, false)), "hello");
        assert!(loaded.has_any_click_commands(false));
    }

    #[test]
    fn client_text_stays_literal_after_enabled_sign_reload() {
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        let pos = BlockPos::new(1, 64, 2);
        let sign = SignBlockEntity::empty(pos);
        sign.allow_op_features.store(true, Ordering::Relaxed);
        sign.front_text.set_client_messages(&[
            Box::from(crafted),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ]);

        let mut nbt = NbtCompound::new();
        sign.write_nbt(&mut nbt);
        let reloaded = SignBlockEntity::from_nbt(&nbt, pos);

        assert!(!reloaded.front_text.has_any_click_commands(false));
        assert_eq!(
            plain_text(&reloaded.front_text.get_message(0, false)),
            crafted
        );
    }

    #[test]
    fn old_string_messages_load_as_literal_text() {
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        let mut nbt = NbtCompound::new();
        nbt.put_list(
            "messages",
            vec![
                NbtTag::String(Box::from(crafted)),
                NbtTag::String(Box::from("")),
                NbtTag::String(Box::from("")),
                NbtTag::String(Box::from("")),
            ],
        );

        let text = SignText::from(NbtTag::Compound(nbt));

        assert!(!text.has_any_click_commands(false));
        assert_eq!(plain_text(&text.get_message(0, false)), crafted);
    }

    #[test]
    fn trusted_legacy_string_components_load_as_components() {
        let crafted =
            r#"{"text":"click","click_event":{"action":"run_command","command":"/say hi"}}"#;
        let mut nbt = NbtCompound::new();
        nbt.put_bool("allow_op_features", true);
        let mut front = NbtCompound::new();
        front.put_list(
            "messages",
            vec![
                NbtTag::String(Box::from(crafted)),
                NbtTag::String(Box::from("")),
                NbtTag::String(Box::from("")),
                NbtTag::String(Box::from("")),
            ],
        );
        nbt.put_compound("front_text", front);

        let trusted = SignBlockEntity::from_nbt(&nbt, BlockPos::new(1, 64, 2));
        assert!(trusted.front_text.has_any_click_commands(false));

        nbt.put_bool("allow_op_features", false);
        let untrusted = SignBlockEntity::from_nbt(&nbt, BlockPos::new(1, 64, 2));
        assert!(!untrusted.front_text.has_any_click_commands(false));
    }

    #[test]
    fn empty_literal_messages_are_not_content() {
        let text = SignText::default();
        text.set_client_messages(&[Box::from(""), Box::from(""), Box::from(""), Box::from("")]);
        assert!(!text.has_message(false));

        text.set_client_messages(&[
            Box::from("visible"),
            Box::from(""),
            Box::from(""),
            Box::from(""),
        ]);
        assert!(text.has_message(false));
    }

    #[test]
    fn item_components_do_not_enable_op_features() {
        let sign = SignBlockEntity::empty(BlockPos::new(1, 64, 2));
        sign.allow_op_features.store(true, Ordering::Relaxed);

        sign.apply_item_components(&pumpkin_data::item_stack::ItemStack::new(
            1,
            &pumpkin_data::item::Item::OAK_SIGN,
        ));

        assert!(!sign.allow_op_features.load(Ordering::Relaxed));
    }
}
