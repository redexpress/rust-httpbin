// Compile-time embedded image fixtures served by the /image/* endpoints.
//
// Source files live in `src/assets/`. They are bundled into the binary
// at build time via `include_bytes!` — no runtime filesystem access.

pub const PNG_BYTES: &[u8] = include_bytes!("../assets/image.png");
pub const JPEG_BYTES: &[u8] = include_bytes!("../assets/image.jpg");
pub const WEBP_BYTES: &[u8] = include_bytes!("../assets/imag.webp"); // sic: filename is "imag"
pub const SVG_BYTES: &[u8] = include_bytes!("../assets/image.svg");
