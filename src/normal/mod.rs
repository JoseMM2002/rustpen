use crate::editor::EditorBuffer;

pub enum CursorDirections {
    Left,
    Right,
    Up,
    Down,
}

pub fn move_cursors(buffer: &mut EditorBuffer, direction: CursorDirections) {
    for cursor in buffer.cursors.iter_mut() {
        match direction {
            CursorDirections::Left => {
                // Mover el cursor a la izquierda si no está en el borde
                if cursor.position.0 > 0 {
                    cursor.position.0 -= 1;
                }
            }
            CursorDirections::Right => {
                // Mover el cursor a la derecha si no está al final de la línea
                if cursor.position.0 < buffer.content[cursor.position.1 as usize].len() as u16 {
                    cursor.position.0 += 1;
                }
            }
            CursorDirections::Up => {
                // Mover el cursor hacia arriba si no está en la primera línea
                if cursor.position.1 > 0 {
                    cursor.position.1 -= 1;
                    // Asegurar que el cursor no esté más allá del final de la línea
                    cursor.position.0 = cursor
                        .position
                        .0
                        .min(buffer.content[cursor.position.1 as usize].len() as u16);
                }
            }
            CursorDirections::Down => {
                // Mover el cursor hacia abajo si no está en la última línea
                if cursor.position.1 < buffer.content.len() as u16 - 1 {
                    cursor.position.1 += 1;
                    // Asegurar que el cursor no esté más allá del final de la línea
                    cursor.position.0 = cursor
                        .position
                        .0
                        .min(buffer.content[cursor.position.1 as usize].len() as u16);
                }
            }
        }
    }
}
