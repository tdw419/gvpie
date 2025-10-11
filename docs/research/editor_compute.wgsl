// GVPIE Editor Compute Kernel
// This is where ALL editor logic lives - runs entirely on GPU
// Non-Stop Kernel: dispatched once, runs forever

// ============================================================================
// MEMORY LAYOUT - Matches Rust exactly
// ============================================================================

struct EditorState {
    // Core state
    cursor_line: atomic<u32>,
    cursor_col: atomic<u32>,
    scroll_line: atomic<u32>,
    scroll_col: atomic<u32>,
    
    // Text buffer metadata
    text_length: atomic<u32>,
    line_count: atomic<u32>,
    
    // Input ring buffer
    key_ring_head: atomic<u32>,
    key_ring_tail: atomic<u32>,
    
    // System flags
    running: atomic<u32>,
    dirty: atomic<u32>,
    frame_count: atomic<u32>,
    
    // Reserved for expansion
    reserved: array<u32, 245>,
}

struct KeyEvent {
    scancode: u32,
    state: u32,
    modifiers: u32,
    _padding: u32,
}

// ============================================================================
// GPU MEMORY BINDINGS
// ============================================================================

@group(0) @binding(0) var<storage, read_write> state: EditorState;
@group(0) @binding(1) var<storage, read_write> text: array<u32>; // UTF-32
@group(0) @binding(2) var<storage, read_write> key_ring: array<KeyEvent>;

// ============================================================================
// CONSTANTS
// ============================================================================

const RING_SIZE: u32 = 64u;
const MAX_TEXT_SIZE: u32 = 10000000u;
const TAB_SIZE: u32 = 4u;

// Virtual key codes (matching Rust winit)
const KEY_BACKSPACE: u32 = 8u;
const KEY_TAB: u32 = 9u;
const KEY_RETURN: u32 = 13u;
const KEY_LEFT: u32 = 37u;
const KEY_UP: u32 = 38u;
const KEY_RIGHT: u32 = 39u;
const KEY_DOWN: u32 = 40u;
const KEY_DELETE: u32 = 46u;
const KEY_HOME: u32 = 36u;
const KEY_END: u32 = 35u;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn is_newline(c: u32) -> bool {
    return c == 10u; // '\n'
}

fn is_printable(c: u32) -> bool {
    return c >= 32u && c <= 126u;
}

fn get_line_start(line: u32) -> u32 {
    if line == 0u { return 0u; }
    
    var count: u32 = 0u;
    var pos: u32 = 0u;
    let len = atomicLoad(&state.text_length);
    
    while pos < len && count < line {
        if is_newline(text[pos]) {
            count += 1u;
        }
        pos += 1u;
    }
    
    return pos;
}

fn get_line_length(line: u32) -> u32 {
    let start = get_line_start(line);
    let len = atomicLoad(&state.text_length);
    var pos = start;
    
    while pos < len && !is_newline(text[pos]) {
        pos += 1u;
    }
    
    return pos - start;
}

fn cursor_to_offset() -> u32 {
    let line = atomicLoad(&state.cursor_line);
    let col = atomicLoad(&state.cursor_col);
    return get_line_start(line) + col;
}

fn clamp_cursor() {
    let line = atomicLoad(&state.cursor_line);
    let col = atomicLoad(&state.cursor_col);
    let line_count = atomicLoad(&state.line_count);
    
    // Clamp line
    if line >= line_count {
        atomicStore(&state.cursor_line, max(line_count, 1u) - 1u);
    }
    
    // Clamp column
    let line_len = get_line_length(atomicLoad(&state.cursor_line));
    if col > line_len {
        atomicStore(&state.cursor_col, line_len);
    }
}

fn count_lines() -> u32 {
    var count: u32 = 1u; // At least one line
    let len = atomicLoad(&state.text_length);
    
    for (var i: u32 = 0u; i < len; i += 1u) {
        if is_newline(text[i]) {
            count += 1u;
        }
    }
    
    return count;
}

// ============================================================================
// TEXT EDITING OPERATIONS
// ============================================================================

fn insert_char(c: u32) {
    let offset = cursor_to_offset();
    let len = atomicLoad(&state.text_length);
    
    if len >= MAX_TEXT_SIZE - 1u {
        return; // Buffer full
    }
    
    // Shift text right
    for (var i = len; i > offset; i -= 1u) {
        text[i] = text[i - 1u];
    }
    
    // Insert character
    text[offset] = c;
    atomicAdd(&state.text_length, 1u);
    
    // Update line count if newline
    if is_newline(c) {
        atomicAdd(&state.line_count, 1u);
        atomicStore(&state.cursor_line, atomicLoad(&state.cursor_line) + 1u);
        atomicStore(&state.cursor_col, 0u);
    } else {
        atomicAdd(&state.cursor_col, 1u);
    }
    
    atomicStore(&state.dirty, 1u);
    storageBarrier(); // Ensure visibility
}

fn delete_char() {
    let offset = cursor_to_offset();
    
    if offset == 0u {
        return; // At start of buffer
    }
    
    let len = atomicLoad(&state.text_length);
    let deleted_char = text[offset - 1u];
    
    // Shift text left
    for (var i = offset; i < len; i += 1u) {
        text[i - 1u] = text[i];
    }
    
    atomicSub(&state.text_length, 1u);
    
    // Update cursor
    if is_newline(deleted_char) {
        atomicSub(&state.line_count, 1u);
        let line = atomicLoad(&state.cursor_line);
        if line > 0u {
            atomicStore(&state.cursor_line, line - 1u);
            atomicStore(&state.cursor_col, get_line_length(line - 1u));
        }
    } else {
        let col = atomicLoad(&state.cursor_col);
        if col > 0u {
            atomicStore(&state.cursor_col, col - 1u);
        }
    }
    
    atomicStore(&state.dirty, 1u);
    storageBarrier();
}

fn delete_forward() {
    let offset = cursor_to_offset();
    let len = atomicLoad(&state.text_length);
    
    if offset >= len {
        return; // At end of buffer
    }
    
    let deleted_char = text[offset];
    
    // Shift text left
    for (var i = offset + 1u; i < len; i += 1u) {
        text[i - 1u] = text[i];
    }
    
    atomicSub(&state.text_length, 1u);
    
    if is_newline(deleted_char) {
        atomicSub(&state.line_count, 1u);
    }
    
    atomicStore(&state.dirty, 1u);
    storageBarrier();
}

fn move_cursor_left() {
    let col = atomicLoad(&state.cursor_col);
    
    if col > 0u {
        atomicStore(&state.cursor_col, col - 1u);
    } else {
        // Move to end of previous line
        let line = atomicLoad(&state.cursor_line);
        if line > 0u {
            atomicStore(&state.cursor_line, line - 1u);
            atomicStore(&state.cursor_col, get_line_length(line - 1u));
        }
    }
}

fn move_cursor_right() {
    let line = atomicLoad(&state.cursor_line);
    let col = atomicLoad(&state.cursor_col);
    let line_len = get_line_length(line);
    
    if col < line_len {
        atomicStore(&state.cursor_col, col + 1u);
    } else {
        // Move to start of next line
        let line_count = atomicLoad(&state.line_count);
        if line + 1u < line_count {
            atomicStore(&state.cursor_line, line + 1u);
            atomicStore(&state.cursor_col, 0u);
        }
    }
}

fn move_cursor_up() {
    let line = atomicLoad(&state.cursor_line);
    if line > 0u {
        atomicStore(&state.cursor_line, line - 1u);
        clamp_cursor();
    }
}

fn move_cursor_down() {
    let line = atomicLoad(&state.cursor_line);
    let line_count = atomicLoad(&state.line_count);
    if line + 1u < line_count {
        atomicStore(&state.cursor_line, line + 1u);
        clamp_cursor();
    }
}

fn move_cursor_home() {
    atomicStore(&state.cursor_col, 0u);
}

fn move_cursor_end() {
    let line = atomicLoad(&state.cursor_line);
    atomicStore(&state.cursor_col, get_line_length(line));
}

// ============================================================================
// INPUT PROCESSING
// ============================================================================

fn process_key(event: KeyEvent) {
    if event.state == 0u {
        return; // Key released, ignore
    }
    
    let scancode = event.scancode;
    
    // Navigation keys
    if scancode == KEY_LEFT {
        move_cursor_left();
        return;
    }
    if scancode == KEY_RIGHT {
        move_cursor_right();
        return;
    }
    if scancode == KEY_UP {
        move_cursor_up();
        return;
    }
    if scancode == KEY_DOWN {
        move_cursor_down();
        return;
    }
    if scancode == KEY_HOME {
        move_cursor_home();
        return;
    }
    if scancode == KEY_END {
        move_cursor_end();
        return;
    }
    
    // Editing keys
    if scancode == KEY_BACKSPACE {
        delete_char();
        return;
    }
    if scancode == KEY_DELETE {
        delete_forward();
        return;
    }
    if scancode == KEY_RETURN {
        insert_char(10u); // '\n'
        return;
    }
    if scancode == KEY_TAB {
        // Insert spaces for tab
        for (var i = 0u; i < TAB_SIZE; i += 1u) {
            insert_char(32u); // ' '
        }
        return;
    }
    
    // Printable characters
    // Map virtual key codes to ASCII (simplified)
    var c: u32 = 0u;
    
    // Letters A-Z (65-90)
    if scancode >= 65u && scancode <= 90u {
        c = scancode + 32u; // Convert to lowercase
    }
    // Numbers 0-9 (48-57)
    else if scancode >= 48u && scancode <= 57u {
        c = scancode;
    }
    // Space (32)
    else if scancode == 32u {
        c = 32u;
    }
    
    if is_printable(c) {
        insert_char(c);
    }
}

fn process_input_queue() {
    let head = atomicLoad(&state.key_ring_head);
    let tail = atomicLoad(&state.key_ring_tail);
    
    // Process all pending events
    while head != tail {
        let idx = tail % RING_SIZE;
        let event = key_ring[idx];
        process_key(event);
        atomicAdd(&state.key_ring_tail, 1u);
    }
}

// ============================================================================
// INITIALIZATION
// ============================================================================

fn initialize() {
    atomicStore(&state.running, 1u);
    atomicStore(&state.cursor_line, 0u);
    atomicStore(&state.cursor_col, 0u);
    atomicStore(&state.scroll_line, 0u);
    atomicStore(&state.scroll_col, 0u);
    atomicStore(&state.text_length, 0u);
    atomicStore(&state.line_count, 1u);
    atomicStore(&state.key_ring_head, 0u);
    atomicStore(&state.key_ring_tail, 0u);
    atomicStore(&state.dirty, 0u);
    atomicStore(&state.frame_count, 0u);
    
    // Initialize with welcome message
    let welcome = array<u32, 49>(
        71u, 86u, 80u, 73u, 69u, 32u, 69u, 100u, 105u, 116u, 111u, 114u, 10u, // "GVPIE Editor\n"
        71u, 80u, 85u, 45u, 78u, 97u, 116u, 105u, 118u, 101u, 32u, 68u, 101u, 118u, 101u, 108u, 111u, 112u, 109u, 101u, 110u, 116u, 10u, // "GPU-Native Development\n"
        10u, // "\n"
        84u, 121u, 112u, 101u, 32u, 104u, 101u, 114u, 101u, 46u, 46u, 46u // "Type here..."
    );
    
    for (var i = 0u; i < 49u; i += 1u) {
        text[i] = welcome[i];
    }
    atomicStore(&state.text_length, 49u);
    atomicStore(&state.line_count, 4u);
    
    storageBarrier();
}

// ============================================================================
// MAIN COMPUTE KERNEL (Non-Stop Kernel)
// ============================================================================

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Initialize on first run
    if atomicLoad(&state.running) == 0u {
        initialize();
    }
    
    // Process input events
    process_input_queue();
    
    // Update line count if dirty
    if atomicLoad(&state.dirty) == 1u {
        atomicStore(&state.line_count, count_lines());
        atomicStore(&state.dirty, 0u);
    }
    
    // Increment frame counter
    atomicAdd(&state.frame_count, 1u);
    
    // Ensure all writes are visible to render shader
    storageBarrier();
}
