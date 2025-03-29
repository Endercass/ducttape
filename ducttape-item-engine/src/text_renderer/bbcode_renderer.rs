use valence_text::{Text, TextContent};

pub trait BBCodeRenderer {
    fn to_bbcode_string(&self) -> String;
}

impl BBCodeRenderer for Text {
    fn to_bbcode_string(&self) -> String {
        let mut bbcode_string = String::new();

        fn render_text(text: &Text, bbcode_string: &mut String) {
            if let TextContent::Text { text: content } = &text.content {
                if text.bold.unwrap_or(false) {
                    bbcode_string.push_str("[b]");
                }

                if text.italic.unwrap_or(false) {
                    bbcode_string.push_str("[i]");
                }

                if text.underlined.unwrap_or(false) {
                    bbcode_string.push_str("[u]");
                }

                if text.strikethrough.unwrap_or(false) {
                    bbcode_string.push_str("[s]");
                }

                if text.color.is_some() {
                    bbcode_string.push_str("[color=");
                    bbcode_string.push_str(&text.color.as_ref().unwrap().to_string());
                    bbcode_string.push(']');
                }

                bbcode_string.push_str(content);

                if text.color.is_some() {
                    bbcode_string.push_str("[/color]");
                }

                if text.strikethrough.unwrap_or(false) {
                    bbcode_string.push_str("[/s]");
                }

                if text.underlined.unwrap_or(false) {
                    bbcode_string.push_str("[/u]");
                }

                if text.italic.unwrap_or(false) {
                    bbcode_string.push_str("[/i]");
                }

                if text.bold.unwrap_or(false) {
                    bbcode_string.push_str("[/b]");
                }
            }

            for child in &text.extra {
                render_text(child, bbcode_string);
            }
        }

        render_text(self, &mut bbcode_string);

        bbcode_string
    }
}
