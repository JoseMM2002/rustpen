use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::sync::Arc;
use std::time::Instant;
use std::usize;
use std::{collections::HashMap, io::Stdout};
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use termion::{
    clear,
    color::{self, Rgb as TermionRgb},
    cursor,
};
use ts_rs::TS;

use crate::editor_modes::{EditorMode, ExecuteKey};

#[derive(Deserialize, TS, Clone, Copy)]
#[ts(export)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn to_termion_rgb(self) -> TermionRgb {
        TermionRgb(self.0, self.1, self.2)
    }
}

#[derive(Clone)]
pub enum CursorForm {
    SteadyBar,
    SteadyBlock,
    SteadyUnderline,
}

impl CursorForm {
    pub fn to_termion_cursor(&self) -> Box<dyn std::fmt::Display> {
        match self {
            CursorForm::SteadyBar => Box::new(cursor::SteadyBar),
            CursorForm::SteadyBlock => Box::new(cursor::SteadyBlock),
            CursorForm::SteadyUnderline => Box::new(cursor::SteadyUnderline),
        }
    }
    pub fn to_char(&self) -> char {
        match self {
            CursorForm::SteadyBar => '|',       // Representación de barra
            CursorForm::SteadyBlock => '█',     // Representación de bloque
            CursorForm::SteadyUnderline => '_', // Representación de subrayado
        }
    }
}

#[derive(Clone)]
pub struct EditorCursor {
    pub position: (u16, u16),
    pub form: CursorForm,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EditorCursorContext {
    pub position: (u16, u16),
}

impl EditorCursor {
    pub fn to_cursor_context(self) -> EditorCursorContext {
        EditorCursorContext {
            position: self.position,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EditorWindow {
    pub start: (u16, u16),
    pub end: (u16, u16),
}

pub type HandleKeysFn = Arc<dyn Fn(&mut Editor, &str) + Send + Sync>;

#[derive(Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ColorRange {
    pub range: (u16, u16),
    pub bg_color: Option<Rgb>,
    pub fg_color: Option<Rgb>,
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let rgb = [self.0, self.1, self.2];
        rgb.serialize(serializer)
    }
}

#[derive(Clone)]
pub struct EditorBuffer {
    pub cursors: Vec<EditorCursor>,
    pub content: Vec<String>,
    pub colors: Vec<Vec<ColorRange>>,
    pub file_name: Option<String>,
    pub is_modified: bool,
    pub memory: Vec<String>,
    pub last_input: Instant,
    pub buffer_window: EditorWindow,
    pub handle_keys: HandleKeysFn,
    pub pivot: (u16, u16),
    pub tab_width: u16,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EditorBufferContext {
    pub cursors: Vec<EditorCursorContext>,
    pub content: Vec<String>,
    pub colors: Vec<Vec<ColorRange>>,
    pub memory: Vec<String>,
    pub buffer_window: EditorWindow,
    pub tab_width: u16,
}

impl EditorBuffer {
    pub fn new(handle_keys: HandleKeysFn, buffer_window: EditorWindow, tab_width: u16) -> Self {
        EditorBuffer {
            cursors: vec![EditorCursor {
                position: (0, 0),
                form: CursorForm::SteadyBlock,
            }],
            content: vec![String::new()],
            colors: vec![],
            file_name: None,
            is_modified: false,
            memory: vec!["".to_string()],
            last_input: Instant::now(),
            pivot: (0, 0),
            buffer_window,
            handle_keys,
            tab_width,
        }
    }

    pub fn from_file(
        file_path: &str,
        handle_keys: HandleKeysFn,
        buffer_window: EditorWindow,
        tab_width: u16,
    ) -> Self {
        let file = File::open(file_path);

        let mut content: Vec<String> = Vec::new();

        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let processed_line =
                            line.clone().replace('\t', &" ".repeat(tab_width as usize));
                        content.push(processed_line);
                    }
                }
            }
            Err(_) => {
                content.push(String::new());
            }
        }

        EditorBuffer {
            cursors: vec![EditorCursor {
                position: (0, 0),
                form: CursorForm::SteadyBlock,
            }],
            content,
            colors: vec![],
            file_name: Some(file_path.to_string()),
            is_modified: false,
            memory: vec!["".to_string()],
            last_input: Instant::now(),
            pivot: (0, 0),
            buffer_window,
            handle_keys,
            tab_width,
        }
    }

    pub fn write_file(&self) -> io::Result<()> {
        if let Some(file_name) = &self.file_name {
            let mut file = File::create(file_name)?;

            for line in &self.content {
                writeln!(file, "{}", line)?;
            }
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No file name specified",
            ))
        }
    }

    pub fn to_buffer_context(self) -> EditorBufferContext {
        EditorBufferContext {
            cursors: self
                .cursors
                .iter()
                .map(|cursor| cursor.clone().to_cursor_context())
                .collect(),
            content: self.content.clone(),
            colors: self.colors.clone(),
            memory: self.memory.clone(),
            buffer_window: self.buffer_window.clone(),
            tab_width: self.tab_width,
        }
    }
}

pub struct Editor {
    pub buffers: HashMap<String, EditorBuffer>,

    pub buffers_to_show: Vec<String>,
    pub focus_buffer: String,
    pub editor_mode: EditorMode,
    pub close: bool,
    pub terminal_size: (u16, u16),
    pub commands_hist: Vec<String>,

    stdout: AlternateScreen<RawTerminal<Stdout>>,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EditorContext {
    pub buffers: HashMap<String, EditorBufferContext>,
    pub buffers_to_show: Vec<String>,
    pub focus_buffer: String,
    pub editor_mode: EditorMode,
    pub terminal_size: (u16, u16),
    pub commands_hist: Vec<String>,
}

pub type EditorFunctions = Box<dyn Fn(&mut Editor) + Sync + Send + 'static>;

impl Editor {
    pub fn new(stdout: AlternateScreen<RawTerminal<Stdout>>) -> Self {
        Editor {
            buffers_to_show: vec![],
            buffers: HashMap::new(),
            editor_mode: EditorMode::Normal,
            close: false,
            terminal_size: termion::terminal_size().unwrap(),
            focus_buffer: String::new(),
            commands_hist: vec![],
            stdout,
        }
    }

    pub fn add_buffer(&mut self, key: String, buffer: EditorBuffer) {
        self.buffers.insert(key.clone(), buffer);
    }

    pub fn get_buffer(&self, key: &str) -> Option<&EditorBuffer> {
        self.buffers.get(key)
    }

    pub fn get_buffer_mut(&mut self, key: &str) -> Option<&mut EditorBuffer> {
        self.buffers.get_mut(key)
    }

    pub fn invoke_buffer_handler(&mut self, key: &str) {
        let buffers = self.buffers.clone();
        if let Some(buffer) = buffers.get(&self.focus_buffer.clone()) {
            (buffer.handle_keys)(self, key);
        }
    }

    pub fn redraw(&mut self, terminal_size: (u16, u16)) {
        self.terminal_size = terminal_size;
    }

    pub fn execute_key(&mut self, key: &str) {
        let editor_mode = self.editor_mode;
        editor_mode.execute_key(key, self);
    }

    pub fn close_editor(&mut self) {
        self.close = true;
        write!(self.stdout, "{}{}", clear::All, cursor::Show).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn to_editor_context(&self) -> EditorContext {
        let buffers = self.buffers.clone();
        let mut buffers_context: HashMap<String, EditorBufferContext> = HashMap::new();

        buffers.into_iter().for_each(|(key, buffer)| {
            buffers_context.insert(key, buffer.to_buffer_context());
        });

        EditorContext {
            buffers: buffers_context,
            buffers_to_show: self.buffers_to_show.clone(),
            focus_buffer: self.focus_buffer.clone(),
            editor_mode: self.editor_mode.clone(),
            terminal_size: self.terminal_size.clone(),
            commands_hist: self.commands_hist.clone(),
        }
    }

    pub fn render(&mut self, info: String) {
        let editor = self;
        let terminal_size = editor.terminal_size;
        let editor_mode = editor.editor_mode;
        let buffers = editor.buffers.clone();

        let mut render_buffer = String::new();

        write!(
            render_buffer,
            "{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Hide
        )
        .unwrap();

        for buff_name in editor.buffers_to_show.clone().iter() {
            if let Some(buffer) = buffers.get(buff_name) {
                let window_width =
                    (buffer.buffer_window.end.0 - buffer.buffer_window.start.0) as usize;
                let window_height =
                    (buffer.buffer_window.end.1 - buffer.buffer_window.start.1) as usize;

                for (i, line) in buffer
                    .content
                    .iter()
                    .enumerate()
                    .skip(buffer.pivot.1 as usize)
                    .take(window_height + 1)
                {
                    let cursor_y = buffer.buffer_window.start.1 + i as u16 - buffer.pivot.1;
                    write!(
                        render_buffer,
                        "{}",
                        cursor::Goto(buffer.buffer_window.start.0, cursor_y)
                    )
                    .unwrap();

                    let mut color_ranges: Vec<ColorRange> = vec![];

                    if let Some(current_colors) = buffer.colors.get(i) {
                        color_ranges = current_colors.to_vec();
                    }

                    let mut current_colors = color_ranges.iter();
                    let mut current_color = current_colors.next();

                    let cursor_in_line = buffer.cursors.iter().find(|&cursor| {
                        cursor.position.1 as usize == i && editor_mode != EditorMode::Command
                    });

                    let mut line_plus_cursor = line.clone();
                    line_plus_cursor.push(' ');

                    for (j, ch) in line_plus_cursor
                        .chars()
                        .enumerate()
                        .skip(buffer.pivot.0 as usize)
                        .take(window_width + 1 as usize)
                    {
                        if let Some(current) = current_color {
                            if j == current.range.0 as usize {
                                let bg_color =
                                    current.bg_color.unwrap_or(Rgb(0, 0, 0)).to_termion_rgb();
                                let fg_color = current
                                    .fg_color
                                    .unwrap_or(Rgb(255, 255, 255))
                                    .to_termion_rgb();

                                if cursor_in_line
                                    .map_or(false, |cursor| cursor.position.0 == j as u16)
                                {
                                    write!(
                                        render_buffer,
                                        "{}{}{}{}{}",
                                        color::Fg(bg_color),
                                        color::Bg(fg_color),
                                        ch,
                                        color::Fg(fg_color),
                                        color::Bg(bg_color)
                                    )
                                    .unwrap();
                                } else {
                                    write!(
                                        render_buffer,
                                        "{}{}{}",
                                        color::Fg(bg_color),
                                        color::Bg(fg_color),
                                        ch
                                    )
                                    .unwrap();
                                }
                            } else if j == current.range.1 as usize {
                                write!(
                                    render_buffer,
                                    "{}{}{}",
                                    ch,
                                    color::Bg(color::Reset),
                                    color::Fg(color::Reset)
                                )
                                .unwrap();
                                current_color = current_colors.next();
                            } else {
                                write!(render_buffer, "{}", ch).unwrap();
                            }
                        } else {
                            if cursor_in_line.map_or(false, |cursor| cursor.position.0 == j as u16)
                            {
                                write!(
                                    render_buffer,
                                    "{}{}{}{}{}",
                                    color::Fg(TermionRgb(0, 0, 0)),
                                    color::Bg(TermionRgb(255, 255, 255)),
                                    ch,
                                    color::Bg(color::Reset),
                                    color::Fg(color::Reset)
                                )
                                .unwrap()
                            } else {
                                write!(render_buffer, "{}", ch).unwrap();
                            }
                        }
                    }

                    write!(
                        render_buffer,
                        "{}{}",
                        color::Bg(color::Reset),
                        color::Fg(color::Reset)
                    )
                    .unwrap();
                }
            }
        }

        write!(render_buffer, "{}", cursor::Goto(1, terminal_size.1)).unwrap();

        match editor_mode {
            EditorMode::Command => {
                let mut memory_text = String::new();
                if let Some(last_memory) = editor.commands_hist.last() {
                    memory_text = last_memory.to_string();
                }
                write!(render_buffer, "Command: {}{}", memory_text, cursor::Show).unwrap();
            }
            EditorMode::Visual => {
                write!(
                    render_buffer,
                    "{}--VISUAL--{}",
                    color::Fg(color::Cyan),
                    color::Fg(color::Reset)
                )
                .unwrap();
            }
            EditorMode::Insert => {
                write!(
                    render_buffer,
                    "{}--INSERT--{}",
                    color::Fg(color::Yellow),
                    color::Fg(color::Reset)
                )
                .unwrap();
            }
            EditorMode::Normal => {
                let mut memory_text = String::new();
                if editor.commands_hist.len() >= 2 {
                    if let Some(penultimate_memory) =
                        editor.commands_hist.get(editor.commands_hist.len() - 2)
                    {
                        memory_text = penultimate_memory.to_string();
                    }
                }
                write!(render_buffer, "{}", memory_text).unwrap();
            }
        }

        let padding = terminal_size.0.saturating_sub(info.len() as u16);

        write!(
            render_buffer,
            "{}{}",
            cursor::Goto(padding, terminal_size.1),
            info
        )
        .unwrap();

        let stdout = &mut editor.stdout;
        write!(stdout, "{}", render_buffer).unwrap();

        stdout.flush().unwrap();
    }
}
