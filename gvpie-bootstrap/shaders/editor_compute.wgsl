#import "contract.wgsl"

@group(BINDING_GROUP) @binding(BINDING_STATE)
var<storage, read_write> state: EditorState;

@group(BINDING_GROUP) @binding(BINDING_UNIFORMS)
var<uniform> uniforms: RenderUniforms;

@group(BINDING_GROUP) @binding(BINDING_EVENTS)
var<storage, read_write> events: array<u32>;

@group(BINDING_GROUP) @binding(BINDING_REQUESTS)
var<storage, read_write> requests: array<u32>;

fn store_camera_3d(camera: Camera3D) {
    let base = CAMERA_DATA_OFFSET;
    state.text_data[base + 0u] = bitcast_u32(camera.position.x);
    state.text_data[base + 1u] = bitcast_u32(camera.position.y);
    state.text_data[base + 2u] = bitcast_u32(camera.position.z);
    state.text_data[base + 3u] = bitcast_u32(camera.focus.x);
    state.text_data[base + 4u] = bitcast_u32(camera.focus.y);
    state.text_data[base + 5u] = bitcast_u32(camera.focus.z);
    state.text_data[base + 6u] = bitcast_u32(camera.up.x);
    state.text_data[base + 7u] = bitcast_u32(camera.up.y);
    state.text_data[base + 8u] = bitcast_u32(camera.up.z);
    state.text_data[base + 9u] = bitcast_u32(camera.fov);
    state.text_data[base + 10u] = bitcast_u32(camera.move_speed);
    state.text_data[base + 11u] = bitcast_u32(camera.mouse_sensitivity);
    state.text_data[base + 12u] = 0u;
    state.text_data[base + 13u] = 0u;
    state.text_data[base + 14u] = 0u;
    state.text_data[CAMERA_SENTINEL_INDEX] = CAMERA_SENTINEL;
}

fn init_card_system() {
    state.text_data[CARDS_SENTINEL_INDEX] = CARDS_SENTINEL;
    state.text_data[CARDS_COUNT_INDEX] = 0u;
    state.text_data[CARDS_SELECTED_INDEX] = 0xFFFFFFFFu;
    state.text_data[CARDS_HOVERED_INDEX] = 0xFFFFFFFFu;
    state.text_data[CARDS_TIME_INDEX] = bitcast_u32(0.0);

    for (var i: u32 = 0u; i < CARDS_TOTAL_FLOATS; i = i + 1u) {
        state.text_data[CARDS_DATA_START + i] = 0u;
    }
}

fn set_cards_count(count: u32) {
    state.text_data[CARDS_COUNT_INDEX] = count;
}

fn set_cards_time(time: f32) {
    state.text_data[CARDS_TIME_INDEX] = bitcast_u32(time);
}

fn write_splat(index: u32, position: vec3<f32>, scale: f32, color: vec3<f32>, alpha: f32) {
    if (index >= MAX_SPLATS) {
        return;
    }
    let base = CARDS_DATA_START + index * SPLAT_FLOATS;
    state.text_data[base + 0u] = bitcast_u32(position.x);
    state.text_data[base + 1u] = bitcast_u32(position.y);
    state.text_data[base + 2u] = bitcast_u32(position.z);
    state.text_data[base + 3u] = bitcast_u32(scale);
    state.text_data[base + 4u] = bitcast_u32(color.x);
    state.text_data[base + 5u] = bitcast_u32(color.y);
    state.text_data[base + 6u] = bitcast_u32(color.z);
    state.text_data[base + 7u] = bitcast_u32(alpha);
}

fn clear_event() {
    events[0] = EVENT_NONE;
}

fn toggle_3d_mode() {
    if (is_3d_mode(state.scroll_offset)) {
        state.scroll_offset = MODE_2D;
    } else {
        state.scroll_offset = MODE_3D;
        if (!camera_is_initialised()) {
            store_camera_3d(default_camera_3d());
        }
    }
}

fn handle_keyboard_event(key: u32, modifiers: u32) {
    if (key == KEY_SPACE && (modifiers & MOD_CTRL) != 0u) {
        toggle_3d_mode();
        return;
    }

    if (!is_3d_mode(state.scroll_offset)) {
        return;
    }

    var camera = default_camera_3d();
    if (camera_is_initialised()) {
        camera = load_camera_3d();
    } else {
        store_camera_3d(camera);
    }
    let forward = normalize(camera.focus - camera.position);
    let right = normalize(cross(forward, camera.up));
    let step = camera.move_speed * 0.2;
    var moved = false;

    switch (key) {
        case KEY_W: {
        camera.position = camera.position + forward * step;
            moved = true;
        }
        case KEY_S: {
        camera.position = camera.position - forward * step;
            moved = true;
        }
        case KEY_A: {
        camera.position = camera.position - right * step;
            moved = true;
        }
        case KEY_D: {
        camera.position = camera.position + right * step;
            moved = true;
        }
        case KEY_SPACE: {
        camera.position = camera.position + camera.up * step;
            moved = true;
        }
        case KEY_SHIFT: {
        camera.position = camera.position - camera.up * step;
            moved = true;
        }
        default: {}
    }

    if (moved) {
        camera.focus = camera.position + forward;
        store_camera_3d(camera);
    }
}

fn generate_3d_cards() {
    if (!cards_initialized()) {
        init_card_system();
    }

    let cards_per_row: u32 = 8u;
    let cards_per_col: u32 = 4u;
    let splats_per_side: u32 = 4u;
    let splats_per_card: u32 = splats_per_side * splats_per_side;

    var splat_index: u32 = 0u;
    let row_offset = f32(cards_per_row - 1u) * 0.5;
    let col_offset = f32(cards_per_col - 1u) * 0.5;

    for (var c: u32 = 0u; c < cards_per_col && splat_index < MAX_SPLATS; c = c + 1u) {
        for (var r: u32 = 0u; r < cards_per_row && splat_index < MAX_SPLATS; r = r + 1u) {
            let base_x = (f32(r) - row_offset) * 4.0;
            let base_z = (f32(c) - col_offset) * 3.5;
            let base_y = 2.5 + sin(uniforms.time * 0.8 + f32(r) * 0.6 + f32(c) * 0.4) * 0.4;

            for (var sy: u32 = 0u; sy < splats_per_side && splat_index < MAX_SPLATS; sy = sy + 1u) {
                for (var sx: u32 = 0u; sx < splats_per_side && splat_index < MAX_SPLATS; sx = sx + 1u) {
                    let offset_x = (f32(sx) - 1.5) * 0.7;
                    let offset_z = (f32(sy) - 1.5) * 0.6;
                    let wave = sin(uniforms.time * 1.5 + f32(sx) * 0.7 + f32(sy) * 0.5 + f32(r) * 0.3) * 0.1;

                    let position = vec3<f32>(
                        base_x + offset_x,
                        base_y + wave,
                        base_z + offset_z
                    );

                    let scale = 0.55;

                    let color = vec3<f32>(
                        clamp(0.35 + f32(r) * 0.05, 0.2, 0.9),
                        clamp(0.4 + f32(c) * 0.04, 0.2, 0.9),
                        clamp(0.8 - f32(r + c) * 0.03, 0.1, 0.9)
                    );

                    let alpha = 0.65;

                    write_splat(splat_index, position, scale, color, alpha);
                    splat_index = splat_index + 1u;
                }
            }

            if (splat_index >= MAX_SPLATS) {
                break;
            }
        }
    }

    set_cards_count(splat_index);
}

fn handle_scroll_event(delta_bits: u32) {
    let delta = bitcast<f32>(delta_bits);
    let base_scroll = f32(state.scroll_offset & 0xFFFFu);
    let updated = clamp(base_scroll + delta * 50.0, 0.0, 10000.0);
    state.scroll_offset = u32(updated);
}

fn process_events() {
    let event_type = events[0];
    switch (event_type) {
        case EVENT_SPECIAL_KEY: {
            let key = events[1];
            let modifiers = events[2];
            handle_keyboard_event(key, modifiers);
        }
        case EVENT_SCROLL: {
            handle_scroll_event(events[1]);
        }
        default: {}
    }
    clear_event();
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x != 0u) {
        return;
    }

    if (!cards_initialized()) {
        init_card_system();
    }

    set_cards_time(uniforms.time);
    generate_3d_cards();
    process_events();
    // Placeholder hook for future GPU-driven requests.
    requests[0] = 0u;
}
