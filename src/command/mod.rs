use std::collections::HashMap;

use lazy_static::lazy_static;
use termion::color::{self};

use crate::{
    editor::{Editor, EditorFunctions},
    editor_modes::EditorMode,
};

lazy_static! {
    pub static ref EDITOR_COMMANDS: HashMap<&'static str, EditorFunctions> = {
        let mut m: HashMap<&'static str, EditorFunctions> = HashMap::new();
        m.insert(
            "q",
            Box::new(|editor: &mut Editor| {
                editor.close_editor();
            }),
        );

        m.insert(
            "w",
            Box::new(|editor: &mut Editor| {
                if let Some(buffer) = editor.get_buffer(&editor.focus_buffer) {
                    match buffer.write_file() {
                        Ok(_) => editor.commands_hist.push(format!(
                            "File saved: {}",
                            buffer.file_name.as_ref().unwrap()
                        )),
                        Err(e) => editor
                            .commands_hist
                            .push(format!("Failed to save file: {}", e)),
                    }

                    editor.commands_hist.push(String::new());
                    editor.editor_mode = EditorMode::Normal;
                }
            }),
        );
        m
    };
}

pub fn execute_assignated_command(editor: &mut Editor, command: &str) {
    if let Some(command_fn) = EDITOR_COMMANDS.get(command) {
        command_fn(editor);
    } else {
        editor.commands_hist.push(format!(
            "{}Command \"{}\" not found.{}",
            color::Fg(color::Red),
            command,
            color::Fg(color::Reset)
        ));
        editor.commands_hist.push(String::new());
        editor.editor_mode = EditorMode::Normal;
    }
}

pub fn match_keys_command(editor: &mut Editor, key: &str) {
    if let Some(last_entry) = editor.commands_hist.last_mut() {
        match key {
            "<Enter>" => {
                let last_entry_clone = last_entry.clone();
                execute_assignated_command(editor, &last_entry_clone);
            }
            "<BS>" => {
                last_entry.pop();
            }
            "<C-h>" => {
                let mut splitted: Vec<&str> = last_entry.split('.').collect();
                splitted.pop();
                *last_entry = splitted.join(".");
            }
            _ if key.len() == 3 => {
                let ch = key.chars().nth(1).unwrap();
                let _ = &last_entry.push(ch);
            }
            "<Space>" => {
                let _ = &last_entry.push(' ');
            }
            _ => {
                *last_entry = "".to_string();
                editor.editor_mode = EditorMode::Normal;
            }
        }
    }
}
