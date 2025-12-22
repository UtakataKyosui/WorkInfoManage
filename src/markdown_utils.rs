use regex::Regex;

pub fn handle_markdown_enter(textarea: &mut tui_textarea::TextArea) {
    let cursor_pos = textarea.cursor();
    let row = cursor_pos.0;

    enum Action {
        Clear,
        Continue(String),
        Default,
    }
    
    let action = {
        let lines = textarea.lines();
        if row < lines.len() {
            let current_line = &lines[row];
            
            // Check for bullet points: -, *, +
            let bullet_regex = Regex::new(r"^(\s*)([-*+])\s+(.*)$").unwrap();
            if let Some(caps) = bullet_regex.captures(current_line) {
                let indent = &caps[1];
                let marker = &caps[2];
                let content = &caps[3];
                
                if content.trim().is_empty() {
                    Action::Clear
                } else {
                    Action::Continue(format!("{}{} ", indent, marker))
                }
            } else {
                // Check for numbered lists: 1.
                let numbered_regex = Regex::new(r"^(\s*)(\d+)\.\s+(.*)$").unwrap();
                if let Some(caps) = numbered_regex.captures(current_line) {
                     let indent = &caps[1];
                     let number: usize = caps[2].parse().unwrap_or(1);
                     let content = &caps[3];
                     
                     if content.trim().is_empty() {
                         Action::Clear
                     } else {
                         Action::Continue(format!("{}{}. ", indent, number + 1))
                     }
                } else {
                    Action::Default
                }
            }
        } else {
            Action::Default
        }
    };
    
    match action {
        Action::Clear => {
            textarea.delete_line_by_head();
        }
        Action::Continue(text) => {
            textarea.insert_newline();
            textarea.insert_str(text);
        }
        Action::Default => {
            textarea.insert_newline();
        }
    }
}
