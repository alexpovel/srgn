use std::ops::Deref;

use colored::{ColoredString, Colorize};

use super::Action;

/// Renders in the given style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<colored::Color>,
    pub bg: Option<colored::Color>,
    pub styles: Vec<colored::Styles>,
}

impl Action for Style {
    fn act(&self, input: &str) -> String {
        // return String::from("what");
        let mut s = ColoredString::from(input);

        if let Some(c) = self.fg {
            s = s.color(c);
        }

        if let Some(c) = self.bg {
            s = s.on_color(c);
        }

        for style in &self.styles {
            s = match style {
                colored::Styles::Clear => s.clear(),
                colored::Styles::Bold => s.bold(),
                colored::Styles::Dimmed => s.dimmed(),
                colored::Styles::Underline => s.underline(),
                colored::Styles::Reversed => s.reversed(),
                colored::Styles::Italic => s.italic(),
                colored::Styles::Blink => s.blink(),
                colored::Styles::Hidden => s.hidden(),
                colored::Styles::Strikethrough => s.strikethrough(),
            }
        }

        s.to_string()
        // s.deref().to_owned()
    }
}
