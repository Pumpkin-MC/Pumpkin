use pumpkin_util::text::TextComponent;

/// Represents a Java Edition tab list configuration containing header and footer components.
#[derive(Debug, Clone)]
pub struct TabList {
    /// The tab list header component.
    pub header: TextComponent,
    /// The tab list footer component.
    pub footer: TextComponent,
}

impl Default for TabList {
    fn default() -> Self {
        Self {
            header: TextComponent::text(""),
            footer: TextComponent::text(""),
        }
    }
}

impl TabList {
    /// Creates a new `TabListBuilder`.
    #[must_use]
    pub fn builder() -> TabListBuilder {
        TabListBuilder::default()
    }
}

/// A fluent builder for `TabList`.
#[derive(Debug, Clone, Default)]
pub struct TabListBuilder {
    header: Option<TextComponent>,
    footer: Option<TextComponent>,
}

impl TabListBuilder {
    /// Creates a new `TabListBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tab list header component.
    #[must_use]
    pub fn header(mut self, header: impl Into<TextComponent>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Sets the tab list header from plain text.
    #[must_use]
    pub fn header_text(mut self, text: impl Into<String>) -> Self {
        self.header = Some(TextComponent::text(text.into()));
        self
    }

    /// Sets the tab list footer component.
    #[must_use]
    pub fn footer(mut self, footer: impl Into<TextComponent>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Sets the tab list footer from plain text.
    #[must_use]
    pub fn footer_text(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(TextComponent::text(text.into()));
        self
    }

    /// Builds the `TabList`.
    #[must_use]
    pub fn build(self) -> TabList {
        TabList {
            header: self.header.unwrap_or_else(|| TextComponent::text("")),
            footer: self.footer.unwrap_or_else(|| TextComponent::text("")),
        }
    }
}

impl From<TabListBuilder> for TabList {
    fn from(builder: TabListBuilder) -> Self {
        builder.build()
    }
}
