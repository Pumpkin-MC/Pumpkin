use crate::text::TextComponent;
use crate::wit::pumpkin::plugin::forms::{
    CustomForm, CustomFormElement, Form, FormImage, ImageType, ModalForm, SimpleForm,
    SimpleFormButton,
};

pub struct SimpleFormBuilder {
    title: TextComponent,
    content: TextComponent,
    buttons: Vec<SimpleFormButton>,
}

impl SimpleFormBuilder {
    pub fn new(title: impl Into<TextComponent>, content: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            buttons: Vec::new(),
        }
    }

    pub fn button(mut self, text: impl Into<TextComponent>, image: Option<FormImage>) -> Self {
        self.buttons.push(SimpleFormButton {
            text: text.into(),
            image,
        });
        self
    }

    pub fn build(self) -> Form {
        Form::Simple(SimpleForm {
            title: self.title,
            content: self.content,
            buttons: self.buttons,
        })
    }
}

pub struct ModalFormBuilder {
    title: TextComponent,
    content: TextComponent,
    button1: TextComponent,
    button2: TextComponent,
}

impl ModalFormBuilder {
    pub fn new(title: impl Into<TextComponent>, content: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            button1: TextComponent::text("Yes"),
            button2: TextComponent::text("No"),
        }
    }

    pub fn button1(mut self, text: impl Into<TextComponent>) -> Self {
        self.button1 = text.into();
        self
    }

    pub fn button2(mut self, text: impl Into<TextComponent>) -> Self {
        self.button2 = text.into();
        self
    }

    pub fn build(self) -> Form {
        Form::Modal(ModalForm {
            title: self.title,
            content: self.content,
            button1: self.button1,
            button2: self.button2,
        })
    }
}

pub struct CustomFormBuilder {
    title: TextComponent,
    elements: Vec<CustomFormElement>,
}

impl CustomFormBuilder {
    pub fn new(title: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            elements: Vec::new(),
        }
    }

    pub fn label(mut self, text: impl Into<TextComponent>) -> Self {
        self.elements.push(CustomFormElement::Label(text.into()));
        self
    }

    pub fn toggle(mut self, text: impl Into<TextComponent>, default: bool) -> Self {
        self.elements
            .push(CustomFormElement::Toggle((text.into(), default)));
        self
    }

    pub fn slider(
        mut self,
        text: impl Into<TextComponent>,
        min: f32,
        max: f32,
        step: f32,
        default: f32,
    ) -> Self {
        self.elements.push(CustomFormElement::Slider((
            text.into(),
            min,
            max,
            step,
            default,
        )));
        self
    }

    pub fn step_slider(
        mut self,
        text: impl Into<TextComponent>,
        steps: Vec<String>,
        default: u32,
    ) -> Self {
        self.elements
            .push(CustomFormElement::StepSlider((text.into(), steps, default)));
        self
    }

    pub fn dropdown(
        mut self,
        text: impl Into<TextComponent>,
        options: Vec<String>,
        default: u32,
    ) -> Self {
        self.elements
            .push(CustomFormElement::Dropdown((text.into(), options, default)));
        self
    }

    pub fn input(
        mut self,
        text: impl Into<TextComponent>,
        placeholder: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        self.elements.push(CustomFormElement::Input((
            text.into(),
            placeholder.into(),
            default.into(),
        )));
        self
    }

    pub fn build(self) -> Form {
        Form::Custom(CustomForm {
            title: self.title,
            elements: self.elements,
        })
    }
}

pub fn url_image(url: impl Into<String>) -> FormImage {
    FormImage {
        type_: ImageType::Url,
        data: url.into(),
    }
}

pub fn path_image(path: impl Into<String>) -> FormImage {
    FormImage {
        type_: ImageType::Path,
        data: path.into(),
    }
}

pub enum FormResponse {
    Simple(u32),
    Modal(bool),
    Custom(Vec<serde_json::Value>),
    Closed,
}

impl FormResponse {
    pub fn parse(data: Option<String>) -> Self {
        match data {
            None => Self::Closed,
            Some(s) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                    if val.is_u64() {
                        Self::Simple(val.as_u64().unwrap() as u32)
                    } else if val.is_boolean() {
                        Self::Modal(val.as_bool().unwrap())
                    } else if val.is_array() {
                        Self::Custom(val.as_array().unwrap().clone())
                    } else {
                        Self::Closed // Or some error state
                    }
                } else {
                    // Fallback for some clients that might send raw strings for simple/modal
                    if s == "true" {
                        Self::Modal(true)
                    } else if s == "false" {
                        Self::Modal(false)
                    } else if let Ok(idx) = s.parse::<u32>() {
                        Self::Simple(idx)
                    } else {
                        Self::Closed
                    }
                }
            }
        }
    }
}
