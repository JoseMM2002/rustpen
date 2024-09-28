use rustpen::buffers::editor_buffer::match_editor_mode;
use rustpen::editor::{ColorRange, Editor, EditorBuffer, EditorWindow, Rgb};
use rustpen::server::UnixServer;
use rustpen::{key_to_string, EditorMessage};
use signal_hook::consts::SIGWINCH;
use signal_hook::iterator::Signals;
use std::io::{stdin, stdout};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;
use std::{env, thread};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::terminal_size;

fn init_editor() -> Result<Editor, ()> {
    let stdout = stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    let mut editor = Editor::new(stdout);
    let args: Vec<String> = env::args().collect();
    let terminal_size = editor.terminal_size.clone();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        return Err(());
    }

    let filename = &args[1];

    let main_buffer = EditorBuffer::from_file(
        filename,
        Arc::new(|editor: &mut Editor, key: &str| match_editor_mode(editor, key)),
        EditorWindow {
            start: (9, 1),
            end: (terminal_size.0, terminal_size.1 - 1),
        },
        4,
    );

    editor.add_buffer("main".to_string(), main_buffer.clone());
    let mut numerate_lines_content: Vec<String> = Vec::new();
    let mut numerate_lines_colors: Vec<Vec<ColorRange>> = Vec::new();

    for i in 1..=main_buffer.content.len() {
        numerate_lines_content.push(format!("{:>6} ", i));
        numerate_lines_colors.push(vec![ColorRange {
            range: (0, 7),
            bg_color: Some(Rgb(42, 42, 55)),
            fg_color: Some(Rgb(84, 83, 108)),
        }]);
    }

    editor.add_buffer(
        "numerate_lines".to_string(),
        EditorBuffer {
            cursors: vec![],
            content: numerate_lines_content,
            colors: numerate_lines_colors,
            file_name: None,
            is_modified: false,
            memory: vec![],
            last_input: Instant::now(),
            buffer_window: EditorWindow {
                start: (1, 1),
                end: (8, terminal_size.1 - 1),
            },
            pivot: (0, 0),
            handle_keys: Arc::new(|_, _| {}),
            tab_width: 4,
        },
    );

    editor.buffers_to_show = vec!["numerate_lines".to_string(), "main".to_string()];
    editor.focus_buffer = "main".to_string();

    Ok(editor)
}

fn main() {
    let mut editor = init_editor().unwrap();
    editor.render("Init".to_string());

    let editor_ref = Arc::new(Mutex::new(editor));

    let (tx, rx) = mpsc::channel::<EditorMessage>();

    // let (tx_server, rx_server) = mpsc::channel();

    let editor_keys = Arc::clone(&editor_ref);
    let editor_resize = Arc::clone(&editor_ref);
    let editor_server = Arc::clone(&editor_ref);

    let tx_key = tx.clone();
    thread::spawn(move || {
        let stdin = stdin();
        for key in stdin.keys() {
            let key = key.unwrap();
            let key_str = key_to_string(key);
            let mut editor = editor_keys.lock().unwrap();

            editor.execute_key(&key_str);

            if editor.close {
                tx_key.send(EditorMessage::Close).unwrap();
                break;
            }

            tx_key.send(EditorMessage::Render(key_str)).unwrap();
        }
    });

    let tx_resize = tx.clone();

    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGWINCH]).unwrap();
        for _ in signals.forever() {
            let mut editor = editor_resize.lock().unwrap();
            editor.redraw(terminal_size().unwrap());
            tx_resize
                .send(EditorMessage::Render("Redraw".to_string()))
                .unwrap();
        }
    });

    let tx_server = tx.clone();

    thread::spawn(move || {
        let server = UnixServer::new("/tmp/rustpen_unix_socket/ts");
        server.start(tx_server, editor_server);
    });

    loop {
        match rx.recv() {
            Ok(EditorMessage::Close) => {
                break;
            }
            Ok(EditorMessage::Render(ch)) => {
                let mut editor = editor_ref.lock().unwrap();
                editor.render(ch);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}
