use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub end: bool,
}
