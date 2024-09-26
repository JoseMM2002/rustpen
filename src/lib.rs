use termion::event::Key;
pub mod editor;
pub mod editor_modes;

pub mod buffers;
pub mod command;
pub mod insert;
pub mod normal;
pub mod visual;

pub mod server;

pub enum EditorMessage {
    Close,
    Render(String),
}

pub fn match_char_with_special_keys(c: &str, prefix: &str, suffix: &str) -> String {
    match c {
        "\t" => format!("{}Tab{}", prefix, suffix),
        "\n" | "\r" => format!("{}Enter{}", prefix, suffix),
        " " => format!("{}Space{}", prefix, suffix),
        "\u{7f}" => format!("{}BS{}", prefix, suffix),
        _ => format!("{}{}{}", prefix, c, suffix),
    }
}

pub fn key_to_string(key: Key) -> String {
    match key {
        Key::Char(c) => match_char_with_special_keys(c.to_string().as_str(), "<", ">"),
        Key::Ctrl(c) => match_char_with_special_keys(c.to_string().as_str(), "<C-", ">"), // Combinación Ctrl
        Key::Alt(c) => match_char_with_special_keys(c.to_string().as_str(), "<A-", ">"), // Combinación Alt
        Key::ShiftLeft => "<S-Left>".to_string(), // Shift + Left
        Key::ShiftRight => "<S-Right>".to_string(), // Shift + Right
        Key::ShiftUp => "<S-Up>".to_string(),     // Shift + Up
        Key::ShiftDown => "<S-Down>".to_string(), // Shift + Down
        Key::CtrlLeft => "<C-Left>".to_string(),  // Ctrl + Left
        Key::CtrlRight => "<C-Right>".to_string(), // Ctrl + Right
        Key::CtrlUp => "<C-Up>".to_string(),      // Ctrl + Up
        Key::CtrlDown => "<C-Down>".to_string(),  // Ctrl + Down
        Key::AltLeft => "<A-Left>".to_string(),   // Alt + Left
        Key::AltRight => "<A-Right>".to_string(), // Alt + Right
        Key::AltUp => "<A-Up>".to_string(),       // Alt + Up
        Key::AltDown => "<A-Down>".to_string(),   // Alt + Down
        Key::BackTab => "<S-Tab>".to_string(),    // Shift + Tab (BackTab)
        Key::Backspace => "<BS>".to_string(),     // Backspace
        Key::Insert => "<Ins>".to_string(),       // Insert
        Key::Delete => "<Del>".to_string(),       // Delete
        Key::Left => "<Left>".to_string(),        // Left
        Key::Right => "<Right>".to_string(),      // Right
        Key::Up => "<Up>".to_string(),            // Up
        Key::Down => "<Down>".to_string(),        // Down
        Key::Home => "<Home>".to_string(),        // Home
        Key::End => "<End>".to_string(),          // End
        Key::PageUp => "<PageUp>".to_string(),    // Page Up
        Key::PageDown => "<PageDown>".to_string(), // Page Down
        Key::Esc => "<Esc>".to_string(),          // Escape
        Key::F(n) => format!("<F{}>", n),         // Teclas de función (F1, F2, ...)
        _ => "<Unknown>".to_string(),             // Teclas no manejadas
    }
}
