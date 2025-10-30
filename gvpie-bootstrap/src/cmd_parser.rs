use wgpu::{Buffer, Queue};

/// Result of pushing ASCII commands into the GPU command queue.
#[derive(Debug, Clone, Default)]
pub struct PushResult {
    pub bytes_consumed: usize,
    pub commands_written: u32,
    pub new_head: u32,
    pub queue_was_full: bool,
}

/// Parse ASCII hex into 16-byte commands and stream them into the queue.
/// The `head`/`tail`/`capacity` arguments operate in command units.
#[allow(dead_code)]
pub fn push_ascii_commands(
    queue: &Queue,
    regs_buf: &Buffer,
    cmdq_buf: &Buffer,
    ascii_input: &str,
    head: u32,
    tail: u32,
    capacity: u32,
) -> PushResult {
    let mut result = PushResult::default();

    if capacity == 0 {
        return result;
    }

    let used = head.wrapping_sub(tail);
    if used >= capacity {
        result.queue_was_full = true;
        return result;
    }

    let mut remaining_slots = capacity - used;
    if remaining_slots == 0 {
        result.queue_was_full = true;
        return result;
    }

    let mut bytes: Vec<u8> = Vec::new();
    let mut have_high = false;
    let mut high: u8 = 0;
    let mut last_complete_idx: usize = 0;
    let mut queue_full = false;

    for (idx, ch) in ascii_input.bytes().enumerate() {
        let v = match ch {
            b'0'..=b'9' => ch - b'0',
            b'a'..=b'f' => ch - b'a' + 10,
            b'A'..=b'F' => ch - b'A' + 10,
            b';' => {
                last_complete_idx = idx + 1;
                break;
            }
            b' ' | b'\n' | b'\t' | b'\r' => {
                last_complete_idx = idx + 1;
                continue;
            }
            _ => continue,
        };

        if !have_high {
            high = v;
            have_high = true;
        } else {
            bytes.push((high << 4) | v);
            have_high = false;

            if bytes.len() % 16 == 0 {
                if remaining_slots == 0 {
                    bytes.truncate(bytes.len().saturating_sub(16));
                    queue_full = true;
                    break;
                }
                remaining_slots -= 1;
                last_complete_idx = idx + 1;
            }
        }
    }

    let full_len = (bytes.len() / 16) * 16;
    bytes.truncate(full_len);

    let commands_to_push = (full_len / 16) as u32;
    result.bytes_consumed = last_complete_idx;
    result.commands_written = commands_to_push;
    result.queue_was_full = queue_full;

    if commands_to_push == 0 {
        return result;
    }

    let cap_mask = capacity - 1;
    let start_idx = head & cap_mask;
    let first_commands = (capacity - start_idx).min(commands_to_push);
    let first_bytes = (first_commands as usize) * 16;
    let offset_bytes = (start_idx as u64) * 16;

    if first_bytes > 0 {
        queue.write_buffer(cmdq_buf, offset_bytes, &bytes[..first_bytes]);
    }

    if first_commands < commands_to_push {
        queue.write_buffer(cmdq_buf, 0, &bytes[first_bytes..]);
    }

    let new_head = head.wrapping_add(commands_to_push);
    queue.write_buffer(regs_buf, 0, &new_head.to_le_bytes());

    result.new_head = new_head;
    result
}

#[allow(unused_imports)]
pub use crate::machine_abi::pack_command;
