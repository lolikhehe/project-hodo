//! Software rasterizer producing RGBA8 frames.

use hodo_layout::DisplayList;
use hodo_types::Size;

/// Rasterize to RGBA8 (non-premultiplied), row-major, `w * h * 4` bytes.
pub fn render(list: &DisplayList, size: Size) -> Vec<u8> {
    todo!("render({:?}, {:?})", list, size)
}
