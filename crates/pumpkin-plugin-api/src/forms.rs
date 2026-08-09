use crate::text::TextComponent;
use crate::wit::pumpkin::plugin::forms::{
    CustomForm, CustomFormElement, Form, FormImage, ImageType, ModalForm, SimpleForm,
    SimpleFormButton,
};

/// Builder for creating a Bedrock simple form.
pub struct SimpleFormBuilder {
    title: TextComponent,
    content: TextComponent,
    buttons: Vec<SimpleFormButton>,
}

impl SimpleFormBuilder {
    /// Creates a new simple form builder with a title and main content text.
    pub fn new(title: impl Into<TextComponent>, content: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            buttons: Vec::new(),
        }
    }

    /// Adds a button to the simple form.
    pub fn button(mut self, text: impl Into<TextComponent>, image: Option<FormImage>) -> Self {
        self.buttons.push(SimpleFormButton {
            text: text.into(),
            image,
        });
        self
    }

    /// Builds the final simple form instance.
    pub fn build(self) -> Form {
        Form::Simple(SimpleForm {
            title: self.title,
            content: self.content,
            buttons: self.buttons,
        })
    }
}

/// Builder for creating a Bedrock modal form (two-button dialog).
pub struct ModalFormBuilder {
    title: TextComponent,
    content: TextComponent,
    button1: TextComponent,
    button2: TextComponent,
}

impl ModalFormBuilder {
    /// Creates a new modal form builder with a title and main content text.
    pub fn new(title: impl Into<TextComponent>, content: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            button1: TextComponent::translate("gui.yes", vec![]),
            button2: TextComponent::translate("gui.no", vec![]),
        }
    }

    /// Sets the label for the first button (confirm).
    pub fn button1(mut self, text: impl Into<TextComponent>) -> Self {
        self.button1 = text.into();
        self
    }

    /// Sets the label for the second button (cancel).
    pub fn button2(mut self, text: impl Into<TextComponent>) -> Self {
        self.button2 = text.into();
        self
    }

    /// Builds the final modal form instance.
    pub fn build(self) -> Form {
        Form::Modal(ModalForm {
            title: self.title,
            content: self.content,
            button1: self.button1,
            button2: self.button2,
        })
    }
}

/// Builder for creating a Bedrock custom form with multiple input elements.
pub struct CustomFormBuilder {
    title: TextComponent,
    elements: Vec<CustomFormElement>,
}

impl CustomFormBuilder {
    /// Creates a new custom form builder with a title.
    pub fn new(title: impl Into<TextComponent>) -> Self {
        Self {
            title: title.into(),
            elements: Vec::new(),
        }
    }

    /// Adds a text label element to the form.
    pub fn label(mut self, text: impl Into<TextComponent>) -> Self {
        self.elements.push(CustomFormElement::Label(text.into()));
        self
    }

    /// Adds a toggle (switch) element to the form.
    pub fn toggle(mut self, text: impl Into<TextComponent>, default: bool) -> Self {
        self.elements
            .push(CustomFormElement::Toggle((text.into(), default)));
        self
    }

    /// Adds a numeric slider element to the form.
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

    /// Adds a step slider element to the form.
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

    /// Adds a dropdown selector element to the form.
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

    /// Adds a text input field element to the form.
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

    /// Builds the final custom form instance.
    pub fn build(self) -> Form {
        Form::Custom(CustomForm {
            title: self.title,
            elements: self.elements,
        })
    }
}

/// Creates a `FormImage` pointing to an HTTP(S) URL.
pub fn url_image(url: impl Into<String>) -> FormImage {
    FormImage {
        type_: ImageType::Url,
        data: url.into(),
    }
}

/// Creates a `FormImage` pointing to a local file path.
pub fn path_image(path: impl Into<String>) -> FormImage {
    FormImage {
        type_: ImageType::Path,
        data: path.into(),
    }
}

/// Represents the response received from a submitted UI form.
pub enum FormResponse {
    /// Response from a simple button form containing selected button index.
    Simple(u32),
    /// Response from a modal dialog containing boolean outcome.
    Modal(bool),
    /// Response from a custom form containing element values.
    Custom(Vec<serde_json::Value>),
    /// Form was closed by player without submission.
    Closed,
}

impl FormResponse {
    /// Parses a JSON response payload string into a `FormResponse`.
    #[must_use]
    pub fn parse(data: Option<String>) -> Self {
        match data {
            None => Self::Closed,
            Some(s) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(num) = val.as_u64() {
                        Self::Simple(num as u32)
                    } else if let Some(b) = val.as_bool() {
                        Self::Modal(b)
                    } else if let Some(arr) = val.as_array() {
                        Self::Custom(arr.clone())
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
