use crate::editor::{EditorBuffer, EditorCursor};

pub mod editor_buffer;
pub mod explorer_buffer;

pub fn adapt_pivot_from_cursor(cursor: &EditorCursor, buffer: &mut EditorBuffer) {
    let window_height = buffer.buffer_window.end.1 - buffer.buffer_window.start.1;
    let window_width = buffer.buffer_window.end.0 - buffer.buffer_window.start.0;

    if cursor.position.1 > window_height + buffer.pivot.1 {
        buffer.pivot.1 = cursor.position.1 - window_height;
    } else if cursor.position.1 < buffer.pivot.1 {
        buffer.pivot.1 = cursor.position.1;
    }

    if cursor.position.0 > window_width + buffer.pivot.0 {
        buffer.pivot.0 = cursor.position.0 - window_width;
    } else if cursor.position.0 < buffer.pivot.0 {
        buffer.pivot.0 = cursor.position.0;
    }
}
