use std::io::Read;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::{env, fs, thread};

use crate::editor::Editor;
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

    pub fn start(&self, tx: mpsc::Sender<EditorMessage>, editor_ref: Arc<Mutex<Editor>>) {
        if Path::new(&self.socket_path).exists() {
            fs::remove_file(&self.socket_path).expect("Failed to remove existing socket.");
        }

        let listener = UnixListener::bind(&self.socket_path).expect("Failed to bind to socket.");

        tx.send(EditorMessage::Render(format!(
            "Listening on socket: {}",
            self.socket_path
        )))
        .unwrap();

        thread::spawn(move || {
            let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

            let home_dir = env::var("HOME").expect("HOME directory not found");
            let config_dir = format!("{}/.config/rustpen/ts-plugins", home_dir);

            Command::new(&shell_path)
                .current_dir(&config_dir)
                .arg("-c")
                .arg("npm start")
                .stdout(Stdio::piped())
                .status()
                .unwrap();
        });

        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buffer = [0; 512];
                loop {
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                tx.send(EditorMessage::Render(
                                    "Plugin connection closed".to_string(),
                                ))
                                .unwrap();
                                break;
                            } else {
                                let received = String::from_utf8(buffer[0..n].to_vec()).unwrap();
                                tx.send(EditorMessage::Render(received)).unwrap();
                            }
                        }
                        Err(e) => {
                            tx.send(EditorMessage::Render(format!(
                                "Error reading plugins: {}",
                                e
                            )))
                            .unwrap();
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                tx.send(EditorMessage::Render(format!("Connection failed: {}", e)))
                    .unwrap();
            }
        }
    }
}
