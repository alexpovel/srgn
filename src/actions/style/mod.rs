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
        const NEWLINE: char = '\n';

        input
            // Split on lines: only that way, terminal coloring has a chance to *reset*
            // after each line and relaunch correctly on the next. Otherwise, escape
            // codes etc. are dragged across lines/contexts and might not work.
            //
            // This sadly encodes knowledge `Style` isn't supposed to have (the fact
            // that sometimes, we're operating line-based.)
            .split_inclusive(NEWLINE)
            .map(|s| {
                // *Only style the part without newline*, if any. Including the newline
                // gives bad results for... reasons. Put the suffix, if any, back later
                // manually. We might not get a suffix at all if there's none at all in
                // `input`.
                let (s, suffix) = s
                    .strip_suffix(NEWLINE)
                    .map_or((s, None), |without_suffix| (without_suffix, Some(NEWLINE)));

                // Debug-only: not mission-critical if this fires
                debug_assert!(!s.ends_with(NEWLINE));

                let mut s = ColoredString::from(s);

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

                let mut res = s.to_string();
                if let Some(suffix) = suffix {
                    // Put it back... it'll not contain styling. This properly resets if
                    // styling across lines.
                    res.push(suffix);
                }

                res
            })
            .collect()
    }
}
