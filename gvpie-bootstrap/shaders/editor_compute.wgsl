// GVPIE Editor Compute Kernel
// All text-editing logic runs here

struct EditorState {
    cursor_line: atomic<u32>,
    cursor_col: atomic<u32>,
    scroll_line: atomic<u32>,
    scroll_col: atomic<u32>,
    text_length: atomic<u32>,
    line_count: atomic<u32>,
    key_ring_head: atomic<u32>,
    key_ring_tail: atomic<u32>,
    running: atomic<u32>,
    dirty: atomic<u32>,
    frame_count: atomic<u32>,
    reserved: array<u32, 245>,
};

struct KeyEvent {
    scancode: u32,
    state: u32,
    modifiers: u32,
    _padding: u32,
};

struct HostEvent {
    event_type: u32,
    data0: u32,
    data1: u32,
    data2: u32,
};

struct HostEventsBuffer {
    version: u32,
    event_count: u32,
    frame_number: u32,
    _padding: u32,
    events: array<HostEvent, 256>,
};

@group(0) @binding(0) var<storage, read_write> state: EditorState;
@group(0) @binding(1) var<storage, read_write> text: array<u32>;
@group(0) @binding(2) var<storage, read_write> key_ring: array<KeyEvent, 1>;
@group(0) @binding(4) var<storage, read> host_events: HostEventsBuffer;
const MAX_TEXT_SIZE: u32 = 10000000u;
const TAB_SIZE: u32 = 4u;

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

const EVENT_TYPE_KEY_PRESS: u32 = 1u;
const EVENT_TYPE_KEY_RELEASE: u32 = 2u;

fn is_newline(value: u32) -> bool {
    return value == 10u;
}

fn is_printable(value: u32) -> bool {
    return value >= 32u && value <= 126u;
}

fn get_line_start(line_index: u32) -> u32 {
    if line_index == 0u {
        return 0u;
    }

    var count: u32 = 0u;
    var index: u32 = 0u;
    let len = atomicLoad(&state.text_length);

    loop {
        if index >= len || count == line_index {
            break;
        }

        if is_newline(text[index]) {
            count = count + 1u;
        }

        index = index + 1u;
    }

    return index;
}

fn get_line_length(line: u32) -> u32 {
    let start = get_line_start(line);
    let len = atomicLoad(&state.text_length);

    var cursor = start;

    loop {
        if cursor >= len {
            break;
        }
        if is_newline(text[cursor]) {
            break;
        }
        cursor = cursor + 1u;
    }

    return cursor - start;
}

fn cursor_to_offset() -> u32 {
    let line = atomicLoad(&state.cursor_line);
    let col = atomicLoad(&state.cursor_col);
    return get_line_start(line) + col;
}

fn clamp_cursor() {
    let line_count = max(atomicLoad(&state.line_count), 1u);
    let current_line = min(atomicLoad(&state.cursor_line), line_count - 1u);
    atomicStore(&state.cursor_line, current_line);

    let current_col = min(atomicLoad(&state.cursor_col), get_line_length(current_line));
    atomicStore(&state.cursor_col, current_col);
}

fn count_lines() -> u32 {
    let len = atomicLoad(&state.text_length);
    if len == 0u {
        return 1u;
    }

    var lines: u32 = 1u;
    var i: u32 = 0u;

    loop {
        if i >= len {
            break;
        }

        if is_newline(text[i]) {
            lines = lines + 1u;
        }

        i = i + 1u;
    }

    return lines;
}

fn insert_char(c: u32) {
    let len = atomicLoad(&state.text_length);
    if len >= MAX_TEXT_SIZE - 1u {
        return;
    }

    let offset = cursor_to_offset();

    var idx = len;
    loop {
        if idx <= offset {
            break;
        }

        text[idx] = text[idx - 1u];
        idx = idx - 1u;
    }

    text[offset] = c;
    atomicStore(&state.text_length, len + 1u);

    if is_newline(c) {
        let line = atomicLoad(&state.cursor_line) + 1u;
        atomicStore(&state.cursor_line, line);
        atomicStore(&state.cursor_col, 0u);
        atomicStore(&state.line_count, atomicLoad(&state.line_count) + 1u);
    } else {
        atomicStore(&state.cursor_col, atomicLoad(&state.cursor_col) + 1u);
    }

    atomicStore(&state.dirty, 1u);
}

fn delete_before_cursor() {
    let offset = cursor_to_offset();
    if offset == 0u {
        return;
    }

    let len = atomicLoad(&state.text_length);
    let removed = text[offset - 1u];

    var i = offset;
    loop {
        if i >= len {
            break;
        }
        text[i - 1u] = text[i];
        i = i + 1u;
    }

    atomicStore(&state.text_length, len - 1u);

    if is_newline(removed) {
        let line = atomicLoad(&state.cursor_line);
        if line > 0u {
            let new_line = line - 1u;
            atomicStore(&state.cursor_line, new_line);
            atomicStore(&state.cursor_col, get_line_length(new_line));
        }
        atomicStore(&state.line_count, max(atomicLoad(&state.line_count) - 1u, 1u));
    } else {
        let col = atomicLoad(&state.cursor_col);
        if col > 0u {
            atomicStore(&state.cursor_col, col - 1u);
        }
    }

    atomicStore(&state.dirty, 1u);
}

fn delete_at_cursor() {
    let offset = cursor_to_offset();
    let len = atomicLoad(&state.text_length);
    if offset >= len {
        return;
    }

    let removed = text[offset];

    var i = offset + 1u;
    loop {
        if i >= len {
            break;
        }
        text[i - 1u] = text[i];
        i = i + 1u;
    }

    atomicStore(&state.text_length, len - 1u);

    if is_newline(removed) {
        atomicStore(&state.line_count, max(atomicLoad(&state.line_count) - 1u, 1u));
    }

    atomicStore(&state.dirty, 1u);
}

fn move_cursor_left() {
    let col = atomicLoad(&state.cursor_col);
    if col > 0u {
        atomicStore(&state.cursor_col, col - 1u);
        return;
    }

    let line = atomicLoad(&state.cursor_line);
    if line == 0u {
        return;
    }

    let prev_line = line - 1u;
    atomicStore(&state.cursor_line, prev_line);
    atomicStore(&state.cursor_col, get_line_length(prev_line));
}

fn move_cursor_right() {
    let line = atomicLoad(&state.cursor_line);
    let col = atomicLoad(&state.cursor_col);
    let len = get_line_length(line);

    if col < len {
        atomicStore(&state.cursor_col, col + 1u);
        return;
    }

    let line_count = atomicLoad(&state.line_count);
    if line + 1u < line_count {
        atomicStore(&state.cursor_line, line + 1u);
        atomicStore(&state.cursor_col, 0u);
    }
}

fn move_cursor_up() {
    let line = atomicLoad(&state.cursor_line);
    if line == 0u {
        return;
    }

    atomicStore(&state.cursor_line, line - 1u);
    clamp_cursor();
}

fn move_cursor_down() {
    let line = atomicLoad(&state.cursor_line);
    let line_count = atomicLoad(&state.line_count);

    if line + 1u >= line_count {
        return;
    }

    atomicStore(&state.cursor_line, line + 1u);
    clamp_cursor();
}

fn move_cursor_home() {
    atomicStore(&state.cursor_col, 0u);
}

fn move_cursor_end() {
    let line = atomicLoad(&state.cursor_line);
    atomicStore(&state.cursor_col, get_line_length(line));
}

fn process_key(event: KeyEvent) {
    if event.state == 0u {
        return;
    }

    let code = event.scancode;

    if code == KEY_LEFT {
        move_cursor_left();
        return;
    }
    if code == KEY_RIGHT {
        move_cursor_right();
        return;
    }
    if code == KEY_UP {
        move_cursor_up();
        return;
    }
    if code == KEY_DOWN {
        move_cursor_down();
        return;
    }
    if code == KEY_HOME {
        move_cursor_home();
        return;
    }
    if code == KEY_END {
        move_cursor_end();
        return;
    }
    if code == KEY_BACKSPACE {
        delete_before_cursor();
        return;
    }
    if code == KEY_DELETE {
        delete_at_cursor();
        return;
    }
    if code == KEY_RETURN {
        insert_char(10u);
        return;
    }
    if code == KEY_TAB {
        var i: u32 = 0u;
        loop {
            if i >= TAB_SIZE {
                break;
            }
            insert_char(32u);
            i = i + 1u;
        }
        return;
    }

    var character = 0u;
    if code >= 65u && code <= 90u {
        character = code + 32u;
    } else if code >= 48u && code <= 57u {
        character = code;
    } else if code == 32u {
        character = 32u;
    }

    if is_printable(character) {
        insert_char(character);
    }
}

fn process_input_queue() {
    let has_event = atomicLoad(&state.key_ring_head);
    if has_event == 0u {
        return;
    }

    let event = key_ring[0];
    process_key(event);

    atomicStore(&state.key_ring_head, 0u);
}

fn process_events() {
    let count = min(host_events.event_count, 256u);
    var i: u32 = 0u;
    loop {
        if i >= count {
            break;
        }

        let event = host_events.events[i];
        if event.event_type == EVENT_TYPE_KEY_PRESS {
            let key_event = KeyEvent(event.data0, 1u, event.data2, 0u);
            process_key(key_event);
        } else if event.event_type == EVENT_TYPE_KEY_RELEASE {
            let key_event = KeyEvent(event.data0, 0u, event.data2, 0u);
            process_key(key_event);
        }

        i = i + 1u;
    }
}

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
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    if atomicLoad(&state.running) == 0u {
        initialize();
    }

    process_events();
    process_input_queue();

    if atomicLoad(&state.dirty) == 1u {
        atomicStore(&state.line_count, count_lines());
        atomicStore(&state.dirty, 0u);
    }

    atomicAdd(&state.frame_count, 1u);

    storageBarrier();
}
