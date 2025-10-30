#![allow(dead_code)]

//! GVPIE Canonical I/O Contract v1.0
//!
//! SINGLE SOURCE OF TRUTH for CPU ↔ GPU communication.
//! Any change here **must** be reflected in `shaders/contract.wgsl`.

use bytemuck::{Pod, Zeroable};

// ============================================================================
// BUFFER LAYOUTS (Must match WGSL exactly)
// ============================================================================

/// Editor state stored in GPU-accessible memory.
#[repr(C)]
pub struct EditorState {
    /// UTF-32 code points for the current document (1 MiB).
    pub text_data: [u32; 262_144],
    pub gap_start: u32,
    pub gap_end: u32,
    pub total_chars: u32,
    pub cursor_pos: u32,
    pub dirty: u32,

    /// Line index (256 KiB).
    pub line_offsets: [u32; 65_536],
    pub line_count: u32,
    pub lines_dirty: u32,

    /// High-level editor state.
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub scroll_offset: u32, // doubles as 3D mode flag when >= MODE_3D
    pub selection_start: u32,
    pub selection_end: u32,
}

/// Render uniforms shared between CPU and GPU.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderUniforms {
    pub time: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub _padding: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3D {
    pub position: [f32; 3],
    pub focus: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
}

impl Camera3D {
    pub fn default() -> Self {
        Self {
            position: [0.0, 5.0, 10.0],
            focus: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 60.0_f32.to_radians(),
            move_speed: 5.0,
            mouse_sensitivity: 0.002,
        }
    }
}

// ============================================================================
// EVENT SYSTEM (Must match WGSL exactly)
// ============================================================================

pub const EVENT_NONE: u32 = 0;
pub const EVENT_CHARACTER: u32 = 1;
pub const EVENT_SPECIAL_KEY: u32 = 2;
pub const EVENT_MOUSE_MOVE: u32 = 3;
pub const EVENT_MOUSE_BUTTON: u32 = 4;
pub const EVENT_SCROLL: u32 = 5;

// Special key codes (derived from USB HID usage IDs).
pub const KEY_BACKSPACE: u32 = 8;
pub const KEY_TAB: u32 = 9;
pub const KEY_ENTER: u32 = 13;
pub const KEY_SHIFT: u32 = 16;
pub const KEY_CTRL: u32 = 17;
pub const KEY_ALT: u32 = 18;
pub const KEY_ESCAPE: u32 = 27;
pub const KEY_SPACE: u32 = 32;
pub const KEY_LEFT: u32 = 37;
pub const KEY_UP: u32 = 38;
pub const KEY_RIGHT: u32 = 39;
pub const KEY_DOWN: u32 = 40;
pub const KEY_A: u32 = 0x0004;
pub const KEY_D: u32 = 0x0007;
pub const KEY_S: u32 = 0x0016;
pub const KEY_W: u32 = 0x001A;
pub const KEY_DELETE: u32 = 127;

// Modifier flags (bitfield).
pub const MOD_CTRL: u32 = 1;
pub const MOD_SHIFT: u32 = 2;
pub const MOD_ALT: u32 = 4;

// ============================================================================
// BINDING LAYOUT (Must match WGSL exactly)
// ============================================================================

pub const BINDING_GROUP: u32 = 0;
pub const BINDING_STATE: u32 = 0;
pub const BINDING_UNIFORMS: u32 = 1;
pub const BINDING_EVENTS: u32 = 2;
pub const BINDING_REQUESTS: u32 = 3;
pub const BINDING_FONT_TEXTURE: u32 = 2;
pub const BINDING_FONT_SAMPLER: u32 = 3;

// ============================================================================
// 3D MODE SYSTEM (Must match WGSL exactly)
// ============================================================================

pub const MODE_2D: u32 = 0;
pub const MODE_3D: u32 = 10_001;

pub const CAMERA_DATA_OFFSET: usize = 0;
pub const CAMERA_DATA_FLOATS: usize = 16;
const CAMERA_SENTINEL_INDEX: usize = CAMERA_DATA_OFFSET + CAMERA_DATA_FLOATS - 1;
const CAMERA_SENTINEL: u32 = 0xC0FF_EE00;

pub const CARDS_SENTINEL: u32 = 0xDEC0_DE00;
pub const CARDS_META_FLOATS: usize = 4;
pub const SPLAT_FLOATS: usize = 8;
pub const MAX_SPLATS: usize = 512;
pub const CARDS_DATA_OFFSET: usize = CAMERA_DATA_OFFSET + CAMERA_DATA_FLOATS;
const CARDS_SENTINEL_INDEX: usize = CARDS_DATA_OFFSET;
const CARDS_COUNT_INDEX: usize = CARDS_SENTINEL_INDEX + 1;
const CARDS_SELECTED_INDEX: usize = CARDS_SENTINEL_INDEX + 2;
const CARDS_HOVERED_INDEX: usize = CARDS_SENTINEL_INDEX + 3;
const CARDS_TIME_INDEX: usize = CARDS_SENTINEL_INDEX + 4;
const CARDS_DATA_START: usize = CARDS_SENTINEL_INDEX + 5;
const CARDS_TOTAL_FLOATS: usize = MAX_SPLATS * SPLAT_FLOATS;

#[inline]
pub const fn is_3d_mode(scroll_offset: u32) -> bool {
    scroll_offset >= MODE_3D
}

#[inline]
pub fn set_3d_mode(scroll_offset: &mut u32, enabled: bool) {
    *scroll_offset = if enabled { MODE_3D } else { MODE_2D };
}

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

pub fn validate_buffer_sizes() -> Result<(), String> {
    let state_size = std::mem::size_of::<EditorState>();
    let expected_state_size = (262_144 + 65_536) * 4 + 12 * 4;

    if state_size != expected_state_size {
        return Err(format!(
            "EditorState size mismatch: expected {} bytes, got {} bytes",
            expected_state_size, state_size
        ));
    }

    let uniforms_size = std::mem::size_of::<RenderUniforms>();
    if uniforms_size != 16 {
        return Err(format!(
            "RenderUniforms size mismatch: expected 16 bytes, got {} bytes",
            uniforms_size
        ));
    }

    Ok(())
}

#[inline]
fn read_f32(bits: u32) -> f32 {
    f32::from_bits(bits)
}

#[inline]
fn write_f32(value: f32) -> u32 {
    value.to_bits()
}

pub fn set_camera_to_state(state: &mut EditorState, camera: &Camera3D) {
    let base = CAMERA_DATA_OFFSET;
    state.text_data[base + 0] = write_f32(camera.position[0]);
    state.text_data[base + 1] = write_f32(camera.position[1]);
    state.text_data[base + 2] = write_f32(camera.position[2]);
    state.text_data[base + 3] = write_f32(camera.focus[0]);
    state.text_data[base + 4] = write_f32(camera.focus[1]);
    state.text_data[base + 5] = write_f32(camera.focus[2]);
    state.text_data[base + 6] = write_f32(camera.up[0]);
    state.text_data[base + 7] = write_f32(camera.up[1]);
    state.text_data[base + 8] = write_f32(camera.up[2]);
    state.text_data[base + 9] = write_f32(camera.fov);
    state.text_data[base + 10] = write_f32(camera.move_speed);
    state.text_data[base + 11] = write_f32(camera.mouse_sensitivity);
    state.text_data[base + 12] = 0;
    state.text_data[base + 13] = 0;
    state.text_data[base + 14] = 0;
    state.text_data[CAMERA_SENTINEL_INDEX] = CAMERA_SENTINEL;
}

pub fn camera_initialized(state: &EditorState) -> bool {
    state.text_data[CAMERA_SENTINEL_INDEX] == CAMERA_SENTINEL
}

pub fn get_camera_from_state(state: &EditorState) -> Camera3D {
    if !camera_initialized(state) {
        return Camera3D::default();
    }

    let base = CAMERA_DATA_OFFSET;
    Camera3D {
        position: [
            read_f32(state.text_data[base + 0]),
            read_f32(state.text_data[base + 1]),
            read_f32(state.text_data[base + 2]),
        ],
        focus: [
            read_f32(state.text_data[base + 3]),
            read_f32(state.text_data[base + 4]),
            read_f32(state.text_data[base + 5]),
        ],
        up: [
            read_f32(state.text_data[base + 6]),
            read_f32(state.text_data[base + 7]),
            read_f32(state.text_data[base + 8]),
        ],
        fov: read_f32(state.text_data[base + 9]),
        move_speed: read_f32(state.text_data[base + 10]),
        mouse_sensitivity: read_f32(state.text_data[base + 11]),
    }
}

pub fn ensure_camera_in_state(state: &mut EditorState) -> Camera3D {
    if camera_initialized(state) {
        get_camera_from_state(state)
    } else {
        let camera = Camera3D::default();
        set_camera_to_state(state, &camera);
        camera
    }
}

pub fn cards_initialized(state: &EditorState) -> bool {
    state.text_data[CARDS_SENTINEL_INDEX] == CARDS_SENTINEL
}

pub fn initialise_cards(state: &mut EditorState) {
    if cards_initialized(state) {
        return;
    }

    state.text_data[CARDS_SENTINEL_INDEX] = CARDS_SENTINEL;
    state.text_data[CARDS_COUNT_INDEX] = 0;
    state.text_data[CARDS_SELECTED_INDEX] = 0xFFFF_FFFF;
    state.text_data[CARDS_HOVERED_INDEX] = 0xFFFF_FFFF;
    state.text_data[CARDS_TIME_INDEX] = write_f32(0.0);

    for i in 0..CARDS_TOTAL_FLOATS {
        state.text_data[CARDS_DATA_START + i] = 0;
    }
}

pub fn set_cards_count(state: &mut EditorState, count: u32) {
    state.text_data[CARDS_COUNT_INDEX] = count;
}

pub fn cards_count(state: &EditorState) -> u32 {
    state.text_data[CARDS_COUNT_INDEX]
}
