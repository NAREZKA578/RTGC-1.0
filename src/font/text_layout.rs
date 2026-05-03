pub struct TextLayout {
    pub lines: Vec<LineInfo>,
    pub total_height: f32,
}

pub struct LineInfo {
    pub text: String,
    pub y_offset: f32,
    pub width: f32,
}

impl TextLayout {
    pub fn layout(
        text: &str,
        max_width: f32,
        line_height: f32,
    ) -> Self {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut y_offset = 0.0;

        for word in text.split_whitespace() {
            let word_width = word.len() as f32 * 8.0;
            if current_width + word_width > max_width && !current_line.is_empty() {
                lines.push(LineInfo {
                    text: current_line.clone(),
                    y_offset,
                    width: current_width,
                });
                current_line.clear();
                current_width = 0.0;
                y_offset += line_height;
            }
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 4.0;
            }
            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() {
            lines.push(LineInfo {
                text: current_line,
                y_offset,
                width: current_width,
            });
        }

        let total_height = if lines.is_empty() {
            0.0
        } else {
            lines.last().unwrap().y_offset + line_height
        };

        Self {
            lines,
            total_height,
        }
    }
}
