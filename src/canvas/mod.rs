//! Canvas interaction — port of canvas.cpp / canvas_mouse behaviour.
//!
//! State machine mirrors ASSDrawCanvas:
//! - hover highlight of points (controls first, then main points win on overlap)
//! - left down/up/drag: select, move, place M/L/B/S, delete, transform handles
//! - right drag: pan drawing (and/or background)
//! - double-click in Color mode: set contour color
//! - marquee selection
//! - status: logical coords + command under cursor

pub mod fill;
pub mod mouse;
pub mod render;

pub use fill::*;
pub use mouse::*;
pub use render::*;
