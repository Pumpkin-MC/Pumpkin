use bytes::Bytes;
use pumpkin_protocol::java::client::dialog::{
    ActionButton as ProtocolActionButton, Dialog as ProtocolDialog, DialogAction,
    DialogBody as ProtocolDialogBody, DialogInput as ProtocolDialogInput, DialogLink,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::plugin::api::events::dialog::{
    DialogClearEvent, DialogClickActionEvent, DialogShowEvent,
};
use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::{
        events::{ToFromWasmEvent, consume_player},
        player::text_component_from_resource,
        pumpkin::plugin::{
            event::{DialogClearEventData, DialogClickActionEventData, DialogShowEventData, Event},
            java_dialogs::{
                Action, ActionButton, AfterAction, CustomClickAction, Dialog, DialogBody,
                DialogInput, DialogInputBool, DialogInputNumberRange, DialogInputSingleOption,
                DialogInputText, DialogType, Link, LinkLabel, LinkType,
            },
        },
    },
};

#[allow(clippy::too_many_lines)]
pub(crate) fn protocol_dialog_from_wasm(
    state: &PluginHostState,
    dialog: &Dialog,
) -> ProtocolDialog {
    let title = text_component_from_resource(state, &dialog.title);

    let body: Vec<_> = dialog
        .body
        .iter()
        .map(|b| match b {
            DialogBody::PlainMessage(c) => ProtocolDialogBody::PlainMessage {
                contents: text_component_from_resource(state, c),
            },
            DialogBody::Item(_i) => ProtocolDialogBody::Item { item: 0 },
        })
        .collect();

    let inputs: Vec<_> = dialog
        .inputs
        .iter()
        .map(|i| match i {
            DialogInput::Bool(b) => ProtocolDialogInput::Boolean {
                label: text_component_from_resource(state, &b.label),
                default_value: b.default_value,
            },
            DialogInput::Text(t) => ProtocolDialogInput::Text {
                label: text_component_from_resource(state, &t.label),
                placeholder: text_component_from_resource(state, &t.placeholder),
                default_value: t.default_value.clone(),
            },
            DialogInput::NumberRange(n) => ProtocolDialogInput::NumberRange {
                label: text_component_from_resource(state, &n.label),
                min: n.min_value,
                max: n.max_value,
                initial: n.initial_value,
                step: n.step,
                label_format: n.label_format.clone(),
            },
            DialogInput::SingleOption(s) => ProtocolDialogInput::SingleOption {
                label: text_component_from_resource(state, &s.label),
                options: s
                    .options
                    .iter()
                    .map(|o| text_component_from_resource(state, o))
                    .collect(),
                initial_index: s.initial_index,
            },
        })
        .collect();

    let buttons: Vec<_> = dialog
        .buttons
        .iter()
        .map(|b| ProtocolActionButton {
            text: text_component_from_resource(state, &b.text),
            tooltip: b
                .tooltip
                .as_ref()
                .map(|t| text_component_from_resource(state, t)),
            width: b.width,
            action: match &b.action {
                Action::OpenUrl(u) => DialogAction::OpenUrl { url: u.clone() },
                Action::CustomClick(c) => DialogAction::Custom {
                    id: c.id.clone(),
                    payload: c.payload.clone(),
                },
            },
        })
        .collect();

    let links: Vec<_> = dialog
        .links
        .iter()
        .map(|l| {
            let label = match &l.label {
                LinkLabel::BuiltIn(t) => {
                    let link_type = match t {
                        LinkType::BugReport => pumpkin_protocol::LinkType::BugReport,
                        LinkType::CommunityGuidelines => {
                            pumpkin_protocol::LinkType::CommunityGuidelines
                        }
                        LinkType::Support => pumpkin_protocol::LinkType::Support,
                        LinkType::Status => pumpkin_protocol::LinkType::Status,
                        LinkType::Feedback => pumpkin_protocol::LinkType::Feedback,
                        LinkType::Community => pumpkin_protocol::LinkType::Community,
                        LinkType::Website => pumpkin_protocol::LinkType::Website,
                        LinkType::Forums => pumpkin_protocol::LinkType::Forums,
                        LinkType::News => pumpkin_protocol::LinkType::News,
                        LinkType::Announcements => pumpkin_protocol::LinkType::Announcements,
                    };
                    pumpkin_protocol::Label::BuiltIn(link_type)
                }
                LinkLabel::Custom(c) => pumpkin_protocol::Label::TextComponent(Box::new(
                    text_component_from_resource(state, c),
                )),
            };
            DialogLink {
                label,
                url: l.url.clone(),
            }
        })
        .collect();

    ProtocolDialog {
        r#type: match dialog.type_ {
            DialogType::Notice => "minecraft:notice".to_string(),
            DialogType::Confirmation => "minecraft:confirmation".to_string(),
            DialogType::MultiAction => "minecraft:multi_action".to_string(),
            DialogType::DialogList => "minecraft:dialog_list".to_string(),
            DialogType::ServerLinks => "minecraft:server_links".to_string(),
        },
        title,
        body,
        inputs,
        buttons,
        links,
        exit_action: None,
        after_action: dialog.after_action.map(|a| match a {
            AfterAction::Peek => "peek".to_string(),
            AfterAction::Pop => "pop".to_string(),
        }),
        can_close_with_escape: dialog.can_close_with_escape,
        external_title: dialog
            .external_title
            .as_ref()
            .map(|t| text_component_from_resource(state, t)),
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn protocol_dialog_to_wasm(
    state: &mut PluginHostState,
    dialog: &ProtocolDialog,
) -> Dialog {
    let title = state
        .add_text_component(dialog.title.clone())
        .expect("failed to add text component");

    let type_ = match dialog.r#type.as_str() {
        "minecraft:confirmation" => DialogType::Confirmation,
        "minecraft:multi_action" => DialogType::MultiAction,
        "minecraft:dialog_list" => DialogType::DialogList,
        "minecraft:server_links" => DialogType::ServerLinks,
        _ => DialogType::Notice,
    };

    let body = dialog
        .body
        .iter()
        .map(|b| match b {
            ProtocolDialogBody::PlainMessage { contents } => {
                let comp = state
                    .add_text_component(contents.clone())
                    .expect("failed to add text component");
                DialogBody::PlainMessage(comp)
            }
            ProtocolDialogBody::Item { item: _ } => {
                let item_res = state
                    .add_item_stack(Arc::new(Mutex::new(
                        pumpkin_data::item_stack::ItemStack::new(0, &pumpkin_data::item::Item::AIR),
                    )))
                    .expect("failed to add item stack resource");
                DialogBody::Item(item_res)
            }
        })
        .collect();

    let inputs = dialog
        .inputs
        .iter()
        .map(|i| match i {
            ProtocolDialogInput::Boolean {
                label,
                default_value,
            } => {
                let lbl = state
                    .add_text_component(label.clone())
                    .expect("failed to add text component");
                DialogInput::Bool(DialogInputBool {
                    label: lbl,
                    default_value: *default_value,
                })
            }
            ProtocolDialogInput::Text {
                label,
                placeholder,
                default_value,
            } => {
                let lbl = state
                    .add_text_component(label.clone())
                    .expect("failed to add text component");
                let ph = state
                    .add_text_component(placeholder.clone())
                    .expect("failed to add text component");
                DialogInput::Text(DialogInputText {
                    label: lbl,
                    placeholder: ph,
                    default_value: default_value.clone(),
                })
            }
            ProtocolDialogInput::NumberRange {
                label,
                min,
                max,
                initial,
                step,
                label_format,
            } => {
                let lbl = state
                    .add_text_component(label.clone())
                    .expect("failed to add text component");
                DialogInput::NumberRange(DialogInputNumberRange {
                    label: lbl,
                    min_value: *min,
                    max_value: *max,
                    initial_value: *initial,
                    step: *step,
                    label_format: label_format.clone(),
                })
            }
            ProtocolDialogInput::SingleOption {
                label,
                options,
                initial_index,
            } => {
                let lbl = state
                    .add_text_component(label.clone())
                    .expect("failed to add text component");
                let opts = options
                    .iter()
                    .map(|o| {
                        state
                            .add_text_component(o.clone())
                            .expect("failed to add text component")
                    })
                    .collect();
                DialogInput::SingleOption(DialogInputSingleOption {
                    label: lbl,
                    options: opts,
                    initial_index: *initial_index,
                })
            }
        })
        .collect();

    let buttons = dialog
        .buttons
        .iter()
        .map(|b| {
            let text = state
                .add_text_component(b.text.clone())
                .expect("failed to add text component");
            let tooltip = b.tooltip.as_ref().map(|t| {
                state
                    .add_text_component(t.clone())
                    .expect("failed to add text component")
            });
            let action = match &b.action {
                DialogAction::OpenUrl { url } => Action::OpenUrl(url.clone()),
                DialogAction::Custom { id, payload } => Action::CustomClick(CustomClickAction {
                    id: id.clone(),
                    payload: payload.clone(),
                }),
            };
            ActionButton {
                text,
                tooltip,
                width: b.width,
                action,
            }
        })
        .collect();

    let links = dialog
        .links
        .iter()
        .map(|l| {
            let label = match &l.label {
                pumpkin_protocol::Label::BuiltIn(t) => LinkLabel::BuiltIn(match t {
                    pumpkin_protocol::LinkType::BugReport => LinkType::BugReport,
                    pumpkin_protocol::LinkType::CommunityGuidelines => {
                        LinkType::CommunityGuidelines
                    }
                    pumpkin_protocol::LinkType::Support => LinkType::Support,
                    pumpkin_protocol::LinkType::Status => LinkType::Status,
                    pumpkin_protocol::LinkType::Feedback => LinkType::Feedback,
                    pumpkin_protocol::LinkType::Community => LinkType::Community,
                    pumpkin_protocol::LinkType::Website => LinkType::Website,
                    pumpkin_protocol::LinkType::Forums => LinkType::Forums,
                    pumpkin_protocol::LinkType::News => LinkType::News,
                    pumpkin_protocol::LinkType::Announcements => LinkType::Announcements,
                }),
                pumpkin_protocol::Label::TextComponent(c) => {
                    let comp = state
                        .add_text_component((**c).clone())
                        .expect("failed to add text component");
                    LinkLabel::Custom(comp)
                }
            };
            Link {
                label,
                url: l.url.clone(),
            }
        })
        .collect();

    let external_title = dialog.external_title.as_ref().map(|t| {
        state
            .add_text_component(t.clone())
            .expect("failed to add text component")
    });

    Dialog {
        title,
        type_,
        body,
        inputs,
        buttons,
        links,
        after_action: dialog.after_action.as_deref().map(|a| match a {
            "peek" => AfterAction::Peek,
            _ => AfterAction::Pop,
        }),
        can_close_with_escape: dialog.can_close_with_escape,
        external_title,
    }
}

impl ToFromWasmEvent for DialogClickActionEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        Event::DialogClickActionEvent(DialogClickActionEventData {
            player: state
                .add_player(self.player.clone())
                .expect("failed to add player resource"),
            id: self.id.clone(),
            payload: self.payload.as_ref().map(|p| p.to_vec()),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::DialogClickActionEvent(data) => Self {
                player: consume_player(state, &data.player),
                id: data.id,
                payload: data.payload.map(Bytes::from),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for DialogClearEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        Event::DialogClearEvent(DialogClearEventData {
            player: state
                .add_player(self.player.clone())
                .expect("failed to add player resource"),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::DialogClearEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for DialogShowEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let dialog = protocol_dialog_to_wasm(state, &self.dialog);
        Event::DialogShowEvent(DialogShowEventData {
            player: state
                .add_player(self.player.clone())
                .expect("failed to add player resource"),
            dialog,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::DialogShowEvent(data) => {
                let dialog = protocol_dialog_from_wasm(state, &data.dialog);
                Self {
                    player: consume_player(state, &data.player),
                    dialog,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
