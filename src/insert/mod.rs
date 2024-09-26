use std::usize;

use crate::editor::EditorBuffer;

pub fn insert_chars_to_buffer(buffer: &mut EditorBuffer, chs: Vec<char>) {
    for (idx, cursor) in buffer.cursors.iter_mut().enumerate() {
        if let Some(line) = buffer.content.get_mut(cursor.position.1 as usize) {
            let mut chars: Vec<char> = line.chars().collect();
            let remaining_chars = chars.split_off(cursor.position.0 as usize);

            chars.extend(chs.clone());
            chars.extend(remaining_chars);

            *line = chars.into_iter().collect();

            cursor.position.0 += chs.len() as u16;
        }

        let window_width = buffer.buffer_window.end.0 - buffer.buffer_window.start.0;

        if idx == 0 && cursor.position.0 >= window_width + 1 as u16 {
            buffer.pivot.0 = cursor.position.0 - window_width - 1;
        }
    }
}
