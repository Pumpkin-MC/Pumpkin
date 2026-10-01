use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::forms::{
    CustomForm, CustomFormElement, Host, ImageType, ModalForm, SimpleForm,
};
use pumpkin_util::translation::Locale;
use serde_json::{Value, json};
use wasmtime::component::Resource;

impl Host for PluginHostState {}

impl PluginHostState {
    pub(crate) fn translate_res_v0_2(
        &mut self,
        res: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::text::TextComponent,
        >,
        locale: Locale,
    ) -> String {
        let component = self.take(res).expect("Couldn't get text component");
        component.0.get_text(locale)
    }

    pub(crate) fn serialize_simple_form_v0_2(&mut self, form: SimpleForm, locale: Locale) -> Value {
        let buttons: Vec<Value> = form
            .buttons
            .into_iter()
            .map(|b| {
                let mut obj = json!({ "text": self.translate_res_v0_2(b.text, locale) });
                if let Some(image) = b.image
                    && let Some(obj) = obj.as_object_mut()
                {
                    obj.insert(
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
            "title": self.translate_res_v0_2(form.title, locale),
            "content": self.translate_res_v0_2(form.content, locale),
            "buttons": buttons
        })
    }

    pub(crate) fn serialize_modal_form_v0_2(&mut self, form: ModalForm, locale: Locale) -> Value {
        json!({
            "type": "modal",
            "title": self.translate_res_v0_2(form.title, locale),
            "content": self.translate_res_v0_2(form.content, locale),
            "button1": self.translate_res_v0_2(form.button1, locale),
            "button2": self.translate_res_v0_2(form.button2, locale)
        })
    }

    pub(crate) fn serialize_custom_form_v0_2(&mut self, form: CustomForm, locale: Locale) -> Value {
        let elements: Vec<Value> = form.elements.into_iter().map(|e| {
            match e {
                CustomFormElement::Label(text) => json!({ "type": "label", "text": self.translate_res_v0_2(text, locale) }),
                CustomFormElement::Toggle((text, default)) => json!({ "type": "toggle", "text": self.translate_res_v0_2(text, locale), "default": default }),
                CustomFormElement::Slider((text, min, max, step, default)) => json!({
                    "type": "slider", "text": self.translate_res_v0_2(text, locale), "min": min, "max": max, "step": step, "default": default
                }),
                CustomFormElement::StepSlider((text, steps, default)) => json!({
                    "type": "step_slider", "text": self.translate_res_v0_2(text, locale), "steps": steps, "default": default
                }),
                CustomFormElement::Dropdown((text, options, default)) => json!({
                    "type": "dropdown", "text": self.translate_res_v0_2(text, locale), "options": options, "default": default
                }),
                CustomFormElement::Input((text, placeholder, default)) => json!({
                    "type": "input", "text": self.translate_res_v0_2(text, locale), "placeholder": placeholder, "default": default
                }),
            }
        }).collect();

        json!({
            "type": "custom_form",
            "title": self.translate_res_v0_2(form.title, locale),
            "content": elements
        })
    }
}
