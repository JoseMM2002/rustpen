use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    editor::{EditorBufferContext, EditorContext},
    editor_modes::EditorMode,
};

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ContextMessage {
    pub editor: EditorContext,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InputMessage {
    pub input: String,
    pub editor_mode: EditorMode,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BufferMessage {
    pub request_id: String,
    pub buffer_name: String,
    pub buffer: EditorBufferContext,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(tag = "message_type", content = "info", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessages {
    Context(ContextMessage),
    Input(InputMessage),
    Buffer(BufferMessage),
}
