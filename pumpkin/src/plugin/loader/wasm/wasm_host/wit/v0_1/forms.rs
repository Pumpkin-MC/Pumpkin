use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::player::text_component_from_resource;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::forms::{
    CustomForm, CustomFormElement, Host, ImageType, ModalForm, SimpleForm,
};
use pumpkin_util::translation::Locale;
use serde_json::{Value, json};
use wasmtime::component::Resource;

impl Host for PluginHostState {}

impl PluginHostState {
    pub(crate) fn translate_res(
        &self,
        res: &Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
        locale: Locale,
    ) -> String {
        let component = text_component_from_resource(self, res);
        component.0.get_text(locale)
    }

    pub(crate) fn serialize_simple_form(&self, form: SimpleForm, locale: Locale) -> Value {
        let buttons: Vec<Value> = form
            .buttons
            .into_iter()
            .map(|b| {
                let mut obj = json!({ "text": self.translate_res(&b.text, locale) });
                if let Some(image) = b.image {
                    obj.as_object_mut().unwrap().insert(
                        "image".to_string(),
                        json!({
                            "type": match image.type_ {
                                ImageType::Url => "url",
                                ImageType::Path => "path",
                            },
                            "data": image.data
                        }),
                    );
                }
                obj
            })
            .collect();

        json!({
            "type": "form",
            "title": self.translate_res(&form.title, locale),
            "content": self.translate_res(&form.content, locale),
            "buttons": buttons
        })
    }

    pub(crate) fn serialize_modal_form(&self, form: &ModalForm, locale: Locale) -> Value {
        json!({
            "type": "modal",
            "title": self.translate_res(&form.title, locale),
            "content": self.translate_res(&form.content, locale),
            "button1": self.translate_res(&form.button1, locale),
            "button2": self.translate_res(&form.button2, locale)
        })
    }

    pub(crate) fn serialize_custom_form(&self, form: CustomForm, locale: Locale) -> Value {
        let elements: Vec<Value> = form.elements.into_iter().map(|e| {
            match e {
                CustomFormElement::Label(text) => json!({ "type": "label", "text": self.translate_res(&text, locale) }),
                CustomFormElement::Toggle((text, default)) => json!({ "type": "toggle", "text": self.translate_res(&text, locale), "default": default }),
                CustomFormElement::Slider((text, min, max, step, default)) => json!({
                    "type": "slider", "text": self.translate_res(&text, locale), "min": min, "max": max, "step": step, "default": default
                }),
                CustomFormElement::StepSlider((text, steps, default)) => json!({
                    "type": "step_slider", "text": self.translate_res(&text, locale), "steps": steps, "default": default
                }),
                CustomFormElement::Dropdown((text, options, default)) => json!({
                    "type": "dropdown", "text": self.translate_res(&text, locale), "options": options, "default": default
                }),
                CustomFormElement::Input((text, placeholder, default)) => json!({
                    "type": "input", "text": self.translate_res(&text, locale), "placeholder": placeholder, "default": default
                }),
            }
        }).collect();

        json!({
            "type": "custom_form",
            "title": self.translate_res(&form.title, locale),
            "content": elements
        })
    }
}
