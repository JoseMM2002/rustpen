use std::sync::Arc;

use crate::{
    editor::{Editor, EditorBuffer, EditorWindow},
    editor_modes::EditorMode,
    normal::{move_cursors, CursorDirections},
};

pub fn match_keys_normal(editor: &mut Editor, key: &str) {
    let buffer = editor.get_buffer_mut(&editor.focus_buffer.clone()).unwrap();
    match key {
        "<:>" => {
            editor.editor_mode = EditorMode::Command;
            editor.commands_hist.push(String::new());
        }
        "<h>" => move_cursors(buffer, CursorDirections::Left),
        "<j>" => move_cursors(buffer, CursorDirections::Down),
        "<k>" => move_cursors(buffer, CursorDirections::Up),
        "<l>" => move_cursors(buffer, CursorDirections::Right),
        "<Enter>" => {}
        _ => {}
    }
}

pub fn match_editor_mode(editor: &mut Editor, key: &str) {
    match editor.editor_mode {
        EditorMode::Normal => match_keys_normal(editor, key),
        _ => {}
    }
}

pub fn init_explorer_buffer(root: &str, terminal_size: (u16, u16)) -> EditorBuffer {
    let mut buffer = EditorBuffer::new(
        Arc::new(|editor: &mut Editor, key: &str| {
            match_editor_mode(editor, key);
        }),
        EditorWindow {
            start: (8, 1),
            end: (terminal_size.0, terminal_size.1 - 1),
        },
        4,
    );

    let mut paths = vec![
        "---------------------------------".to_string(),
        "".to_string(),
        root.to_string(),
        "".to_string(),
        "---------------------------------".to_string(),
        "../".to_string(),
    ];

    for entry in std::fs::read_dir(&root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let path = path.strip_prefix(root).unwrap();
        paths.push(path.to_str().unwrap().to_string());
    }

    buffer.content = paths;

    buffer
}
