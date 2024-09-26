// unix_server.rs

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc;

use crate::EditorMessage;

pub struct UnixServer {
    socket_path: String,
}

impl UnixServer {
    pub fn new(socket_path: &str) -> UnixServer {
        UnixServer {
            socket_path: socket_path.to_string(),
        }
    }

    pub fn start(&self, tx: mpsc::Sender<EditorMessage>) {
        if Path::new(&self.socket_path).exists() {
            fs::remove_file(&self.socket_path).expect("Failed to remove existing socket.");
        }

        let listener = UnixListener::bind(&self.socket_path).expect("Failed to bind to socket.");

        tx.send(EditorMessage::Render(format!(
            "Listening on socket: {}",
            self.socket_path
        )))
        .unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let message = UnixServer::process_message(&mut stream);

                    tx.send(EditorMessage::Render(message)).unwrap();

                    UnixServer::send_message(&mut stream, "Message processed!");
                }
                Err(e) => {
                    tx.send(EditorMessage::Render(format!("Connection failed: {}", e)))
                        .unwrap();
                }
            }
        }
    }

    fn process_message(stream: &mut UnixStream) -> String {
        let mut buffer = [0u8; 1024]; // Buffer de 1024 bytes
        let bytes_read = stream
            .read(&mut buffer)
            .expect("Failed to read from stream");
        String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
    }

    fn send_message(stream: &mut UnixStream, message: &str) {
        stream
            .write_all(message.as_bytes())
            .expect("Failed to write to stream");
        stream.flush().expect("Failed to flush stream");
    }
}
