use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    TextRun(TextRun),
    FillRect(FillRect),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextRun {
    pub x: u32,
    pub y: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FillRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: [u8; 4],
}
