use super::Action;
pub use colored::{Color, ColoredString, Colorize, Styles};

/// Renders in the given style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    /// Foreground color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Styles to apply.
    pub styles: Vec<Styles>,
}

impl Action for Style {
    fn act(&self, input: &str) -> String {
        let mut s = ColoredString::from(input);

        if let Some(c) = self.fg {
            s = s.color(c);
        }

        if let Some(c) = self.bg {
            s = s.on_color(c);
        }

        for style in &self.styles {
            s = match style {
                Styles::Clear => s.clear(),
                Styles::Bold => s.bold(),
                Styles::Dimmed => s.dimmed(),
                Styles::Underline => s.underline(),
                Styles::Reversed => s.reversed(),
                Styles::Italic => s.italic(),
                Styles::Blink => s.blink(),
                Styles::Hidden => s.hidden(),
                Styles::Strikethrough => s.strikethrough(),
            }
        }

        s.to_string()
    }
}
