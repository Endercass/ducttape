use valence_text::{color::RgbColor, Color, Text, TextContent};

pub trait AnsiRenderer {
    fn to_ansi_string(&self) -> String;
}

fn color_to_rgb(color: Color) -> RgbColor {
    match color {
        Color::Rgb(rgb) => rgb,
        Color::Named(named) => match named {
            valence_text::color::NamedColor::Black => RgbColor::new(0, 0, 0),
            valence_text::color::NamedColor::DarkBlue => RgbColor::new(0, 0, 170),
            valence_text::color::NamedColor::DarkGreen => RgbColor::new(0, 170, 0),
            valence_text::color::NamedColor::DarkAqua => RgbColor::new(0, 170, 170),
            valence_text::color::NamedColor::DarkRed => RgbColor::new(170, 0, 0),
            valence_text::color::NamedColor::DarkPurple => RgbColor::new(170, 0, 170),
            valence_text::color::NamedColor::Gold => RgbColor::new(255, 170, 0),
            valence_text::color::NamedColor::Gray => RgbColor::new(170, 170, 170),
            valence_text::color::NamedColor::DarkGray => RgbColor::new(85, 85, 85),
            valence_text::color::NamedColor::Blue => RgbColor::new(85, 85, 255),
            valence_text::color::NamedColor::Green => RgbColor::new(85, 255, 85),
            valence_text::color::NamedColor::Aqua => RgbColor::new(85, 255, 255),
            valence_text::color::NamedColor::Red => RgbColor::new(255, 85, 85),
            valence_text::color::NamedColor::LightPurple => RgbColor::new(255, 85, 255),
            valence_text::color::NamedColor::Yellow => RgbColor::new(255, 255, 85),
            valence_text::color::NamedColor::White => RgbColor::new(255, 255, 255),
        },
        Color::Reset => RgbColor::new(255, 255, 255),
    }
}

impl AnsiRenderer for Text {
    fn to_ansi_string(&self) -> String {
        let mut ansi_string = String::new();

        fn render_text(text: &Text, ansi_string: &mut String) {
            if let TextContent::Text { text: content } = &text.content {
                let color = color_to_rgb(text.color.unwrap_or(Color::Reset));

                ansi_string.push_str(&format!("\x1b[38;2;{};{};{}m", color.r, color.g, color.b));

                if text.bold.unwrap_or(false) {
                    ansi_string.push_str("\x1b[1m");
                }
                if text.italic.unwrap_or(false) {
                    ansi_string.push_str("\x1b[3m");
                }
                if text.underlined.unwrap_or(false) {
                    ansi_string.push_str("\x1b[4m");
                }
                if text.strikethrough.unwrap_or(false) {
                    ansi_string.push_str("\x1b[9m");
                }
                if text.obfuscated.unwrap_or(false) {
                    ansi_string.push_str("\x1b[8m");
                }

                ansi_string.push_str(content);

                ansi_string.push_str("\x1b[0m"); // Reset formatting
            }

            for extra in &text.extra {
                render_text(extra, ansi_string);
            }
        }

        render_text(self, &mut ansi_string);

        ansi_string
    }
}
