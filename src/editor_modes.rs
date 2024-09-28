use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{command::match_keys_command, editor::Editor};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EditorMode {
    Visual,
    Insert,
    Command,
    Normal,
}

// Implementamos Display para convertir cada variante en su representación como String
impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode_str = match self {
            EditorMode::Visual => "Visual",
            EditorMode::Insert => "Insert",
            EditorMode::Command => "Command",
            EditorMode::Normal => "Normal",
        };
        write!(f, "{}", mode_str)
    }
}

pub trait ExecuteKey {
    fn execute_key(self, key: &str, editor: &mut Editor);
}

impl ExecuteKey for EditorMode {
    fn execute_key(self, key: &str, editor: &mut Editor) {
        match self {
            EditorMode::Command => {
                match_keys_command(editor, key);
            }
            _ => {
                editor.invoke_buffer_handler(key);
            }
        }
    }
}
