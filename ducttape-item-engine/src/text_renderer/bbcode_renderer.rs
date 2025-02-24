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

                bbcode_string.push_str(content);

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
        }

        render_text(self, &mut bbcode_string);

        bbcode_string
    }
}
