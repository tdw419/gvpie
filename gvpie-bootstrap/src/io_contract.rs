//! GVPIE I/O Contract v1.0
//!
//! This module defines the immutable interface between the frozen CPU bootstrap
//! and the evolving GPU system. The structures here are FROZEN forever.
#![allow(dead_code)]

pub const IO_CONTRACT_VERSION: u32 = 1;
pub const MAX_EVENTS_PER_FRAME: usize = 256;
pub const MAX_PENDING_REQUESTS: usize = 16;
pub const FILE_IO_BUFFER_SIZE: usize = 1_048_576; // 1 MiB

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None = 0,
    KeyPress = 1,
    KeyRelease = 2,
    MouseMove = 3,
    MousePress = 4,
    MouseRelease = 5,
    WindowResize = 6,
    Scroll = 7,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub event_type: u32,
    pub data: [u32; 3],
}

impl Event {
    pub const NONE: Self = Self {
        event_type: EventType::None as u32,
        data: [0; 3],
    };
}

#[repr(C)]
pub struct EventsBuffer {
    pub version: u32,
    pub event_count: u32,
    pub frame_number: u32,
    pub _padding: u32,
    pub events: [Event; MAX_EVENTS_PER_FRAME],
}

impl EventsBuffer {
    pub fn new() -> Self {
        Self {
            version: IO_CONTRACT_VERSION,
            event_count: 0,
            frame_number: 0,
            _padding: 0,
            events: [Event::NONE; MAX_EVENTS_PER_FRAME],
        }
    }

    pub fn clear(&mut self) {
        self.event_count = 0;
    }

    pub fn push_event(&mut self, event: Event) -> bool {
        if (self.event_count as usize) < MAX_EVENTS_PER_FRAME {
            self.events[self.event_count as usize] = event;
            self.event_count += 1;
            true
        } else {
            false
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    None = 0,
    FileRead = 1,
    FileWrite = 2,
    DirList = 3,
    ShaderReload = 4,
    Exit = 5,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    Pending = 0,
    Success = 1,
    Error = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Request {
    pub request_type: u32,
    pub status: u32,
    pub params: [u32; 4],
    pub path: [u8; 48],
}

impl Request {
    pub const NONE: Self = Self {
        request_type: RequestType::None as u32,
        status: RequestStatus::Pending as u32,
        params: [0; 4],
        path: [0; 48],
    };

    pub fn set_path(&mut self, path: &str) {
        let bytes = path.as_bytes();
        let len = bytes.len().min(self.path.len().saturating_sub(1));
        self.path[..len].copy_from_slice(&bytes[..len]);
        if len < self.path.len() {
            self.path[len] = 0;
        }
    }

    pub fn get_path(&self) -> Option<&str> {
        let len = self
            .path
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.path.len());
        std::str::from_utf8(&self.path[..len]).ok()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RequestBuffer {
    pub version: u32,
    pub request_count: u32,
    pub request_id_counter: u32,
    pub _padding: u32,
    pub requests: [Request; MAX_PENDING_REQUESTS],
}

impl RequestBuffer {
    pub fn new() -> Self {
        Self {
            version: IO_CONTRACT_VERSION,
            request_count: 0,
            request_id_counter: 0,
            _padding: 0,
            requests: [Request::NONE; MAX_PENDING_REQUESTS],
        }
    }
}

#[repr(C)]
pub struct FileIOBuffer {
    pub data: [u8; FILE_IO_BUFFER_SIZE],
}

impl FileIOBuffer {
    pub fn new() -> Self {
        Self {
            data: [0; FILE_IO_BUFFER_SIZE],
        }
    }
}

pub mod buffer_sizes {
    use super::*;

    pub const EVENTS_BUFFER: u64 = std::mem::size_of::<EventsBuffer>() as u64;
    pub const REQUEST_BUFFER: u64 = std::mem::size_of::<RequestBuffer>() as u64;
    pub const FILE_IO_BUFFER: u64 = std::mem::size_of::<FileIOBuffer>() as u64;
    pub const TOTAL: u64 = EVENTS_BUFFER + REQUEST_BUFFER + FILE_IO_BUFFER;
}

pub fn serialize_events(events: &EventsBuffer) -> Vec<u8> {
    unsafe {
        let ptr = events as *const EventsBuffer as *const u8;
        let size = std::mem::size_of::<EventsBuffer>();
        std::slice::from_raw_parts(ptr, size).to_vec()
    }
}

pub fn deserialize_requests(bytes: &[u8]) -> Option<RequestBuffer> {
    if bytes.len() < std::mem::size_of::<RequestBuffer>() {
        return None;
    }
    unsafe {
        let ptr = bytes.as_ptr() as *const RequestBuffer;
        Some(std::ptr::read(ptr))
    }
}
