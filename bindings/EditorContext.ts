// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { EditorBufferContext } from "./EditorBufferContext";
import type { EditorMode } from "./EditorMode";

export type EditorContext = { buffers: { [key in string]?: EditorBufferContext }, buffersToShow: Array<string>, focusBuffer: string, editorMode: EditorMode, terminalSize: [number, number], commandsHist: Array<string>, };
