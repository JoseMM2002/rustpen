use termion::color;

use crate::{
    editor::{ColorRange, Editor},
    editor_modes::EditorMode,
    insert::insert_chars_to_buffer,
    normal::{move_cursors, CursorDirections},
};
use std::{char, usize};

use super::adapt_pivot_from_cursor;

pub fn match_keys_insert(editor: &mut Editor, key: &str) {
    let mut binding = editor.buffers.clone();
    let buffer = binding.get_mut(&editor.focus_buffer).unwrap();

    let mut binding = editor.buffers.clone();
    let numeration_buffer = binding.get_mut("numerate_lines").unwrap();

    match key {
        "<C-c>" | "<Esc>" => editor.editor_mode = EditorMode::Normal,
        "<Enter>" => {
            for cursor in buffer.cursors.iter_mut() {
                let mut string_to_insert = String::new();

                if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
                    let mut chars: Vec<char> = line.chars().collect();
                    let remaining_chars = chars.split_off(cursor.position.0 as usize);

                    string_to_insert = remaining_chars.iter().collect();

                    *line = chars.iter().collect();
                }

                if cursor.position.1 as usize + 1 <= buffer.content.len() {
                    buffer
                        .content
                        .insert(cursor.position.1 as usize + 1, string_to_insert);
                } else {
                    buffer.content.push(string_to_insert);
                }

                cursor.position.0 = 0;
                cursor.position.1 += 1;
            }
        }
        "<Space>" => {
            insert_chars_to_buffer(buffer, " ".chars().collect());
        }
        "<BS>" => {
            for cursor in buffer.cursors.iter_mut() {
                if cursor.position.0 > 0 {
                    if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
                        let mut chars: Vec<char> = line.chars().collect();

                        if cursor.position.0 as usize <= chars.len() {
                            chars.remove((cursor.position.0 - 1) as usize);
                        } else {
                            chars.pop();
                        }
                        *line = chars.into_iter().collect();
                        cursor.position.0 -= 1;
                    }
                } else if cursor.position.1 > 0 {
                    let current_line = buffer.content.remove(cursor.position.1 as usize);

                    cursor.position.1 -= 1;
                    cursor.position.0 = buffer.content[cursor.position.1 as usize].len() as u16;

                    if let Some(prev_line) = buffer.content.get_mut(cursor.position.1 as usize) {
                        prev_line.push_str(&current_line);
                    }
                }
            }
        }
        "<C-h>" => {}
        _ if key.len() == 3 => {
            insert_chars_to_buffer(buffer, vec![key.chars().nth(1).unwrap()]);
        }
        "<A-BS>" => {
            for cursor in buffer.cursors.iter_mut() {
                if cursor.position.0 > 0 {
                    if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
                        let mut chars: Vec<char> = line.chars().collect();

                        let remaining_chars = chars.split_off(cursor.position.0 as usize);

                        chars.pop();

                        if let Some(cut_idx) = chars.iter().rposition(|&c| !c.is_alphanumeric()) {
                            let _ = chars.split_off(cut_idx + 1);
                            chars.extend(remaining_chars);
                            cursor.position.0 = (cut_idx + 1) as u16;
                            *line = chars.into_iter().collect();
                        } else {
                            *line = remaining_chars.into_iter().collect();
                            cursor.position.0 = 0;
                        }
                    }
                } else if cursor.position.1 > 0 {
                    let current_line = buffer.content.remove(cursor.position.1 as usize);

                    cursor.position.1 -= 1;
                    cursor.position.0 = buffer.content[cursor.position.1 as usize].len() as u16;

                    if let Some(prev_line) = buffer.content.get_mut(cursor.position.1 as usize) {
                        prev_line.push_str(&current_line);
                    }
                }
            }
        }
        "<Tab>" => {
            insert_chars_to_buffer(
                buffer,
                " ".repeat(buffer.tab_width as usize)
                    .chars()
                    .into_iter()
                    .collect(),
            );
        }
        _ => {}
    }

    let current_lines = numeration_buffer.content.len();
    let new_lines = buffer.content.len();

    if current_lines < new_lines {
        for i in current_lines..new_lines {
            numeration_buffer.content.push(format!("{:>6} ", i + 1));
            numeration_buffer.colors.push(vec![ColorRange {
                range: (0, 7),
                bg_color: Some(color::Rgb(42, 42, 55)),
                fg_color: Some(color::Rgb(84, 83, 108)),
            }]);
        }
    } else if current_lines > new_lines {
        let _ = numeration_buffer.content.truncate(new_lines);
    }

    adapt_pivot_from_cursor(&buffer.clone().cursors[0], buffer);

    numeration_buffer.pivot.1 = buffer.pivot.1;

    let _ = editor
        .buffers
        .insert("main".to_string(), buffer.clone())
        .unwrap();
    let _ = editor
        .buffers
        .insert("numerate_lines".to_string(), numeration_buffer.clone())
        .unwrap();
}

pub fn match_keys_normal(editor: &mut Editor, key: &str) {
    let mut binding = editor.buffers.clone();
    let buffer = binding.get_mut(&editor.focus_buffer).unwrap();

    let mut binding = editor.buffers.clone();
    let numeration_buffer = binding.get_mut("numerate_lines").unwrap();
    match key {
        "<i>" => editor.editor_mode = EditorMode::Insert,
        "<:>" => {
            editor.editor_mode = EditorMode::Command;
            editor.commands_hist.push(String::new());
        }
        "<h>" => move_cursors(buffer, CursorDirections::Left),
        "<j>" => move_cursors(buffer, CursorDirections::Down),
        "<k>" => move_cursors(buffer, CursorDirections::Up),
        "<l>" => move_cursors(buffer, CursorDirections::Right),
        "<w>" => {
            for cursor in buffer.cursors.iter_mut() {
                if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
                    if cursor.position.0 + 1 < line.len() as u16 {
                        let chars: Vec<char> = line.chars().collect();

                        let flag = chars[cursor.position.0 as usize].is_alphanumeric();

                        if chars[cursor.position.0 as usize] == ' ' {
                            while cursor.position.0 + 1 < chars.len() as u16
                                && chars[cursor.position.0 as usize] == ' '
                            {
                                cursor.position.0 += 1;
                            }
                        }

                        while cursor.position.0 + 1 < chars.len() as u16
                            && flag == chars[cursor.position.0 as usize].is_alphanumeric()
                        {
                            cursor.position.0 += 1;
                        }

                        while cursor.position.0 + 1 < chars.len() as u16
                            && chars[cursor.position.0 as usize] == ' '
                        {
                            cursor.position.0 += 1;
                        }
                    } else if cursor.position.1 + 1 < buffer.content.len() as u16 {
                        cursor.position.0 = 0;
                        cursor.position.1 += 1;
                    }
                }
            }
        }
        "<b>" => {
            for cursor in buffer.cursors.iter_mut() {
                if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
                    if cursor.position.0 > 0 {
                        let chars: Vec<char> = line.chars().collect();

                        if cursor.position.0 == chars.len() as u16 {
                            cursor.position.0 -= 1;
                        }

                        let flag = chars[cursor.position.0 as usize].is_alphanumeric();

                        if chars[cursor.position.0 as usize] == ' ' {
                            while cursor.position.0 > 0 && chars[cursor.position.0 as usize] == ' '
                            {
                                cursor.position.0 -= 1;
                            }
                        }

                        while cursor.position.0 > 0
                            && flag == chars[cursor.position.0 as usize].is_alphanumeric()
                        {
                            cursor.position.0 -= 1;
                        }

                        while cursor.position.0 > 0 && chars[cursor.position.0 as usize] == ' ' {
                            cursor.position.0 -= 1;
                        }
                    } else if cursor.position.1 > 0 {
                        cursor.position.1 -= 1;
                        if let Some(prev_line) = buffer.content.get(cursor.position.1 as usize) {
                            cursor.position.0 = if prev_line.len() > 0 {
                                (prev_line.len() - 1) as u16
                            } else {
                                0
                            };
                        }
                    }
                }
            }
        }
        _ => {}
    }

    adapt_pivot_from_cursor(&buffer.clone().cursors[0], buffer);

    numeration_buffer.pivot.1 = buffer.pivot.1;

    let _ = editor
        .buffers
        .insert("main".to_string(), buffer.clone())
        .unwrap();
    let _ = editor
        .buffers
        .insert("numerate_lines".to_string(), numeration_buffer.clone())
        .unwrap();
}

pub fn match_editor_mode(editor: &mut Editor, key: &str) {
    match editor.editor_mode {
        EditorMode::Insert => match_keys_insert(editor, key),
        EditorMode::Normal => match_keys_normal(editor, key),
        _ => {}
    }
}
