use crate::color::Rgb;
use crate::pipeline::framebuffer::Framebuffer;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

/// How the framebuffer pixels are mapped to terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Half-block characters: 2 vertical pixels per cell using ▀ with fg=upper, bg=lower.
    #[default]
    HalfBlock,
    /// Braille characters: 2×4 dots per cell for high-resolution monochrome.
    Braille,
    /// ASCII shading ramp with colored characters.
    Ascii,
}

impl RenderMode {
    /// Compute the pixel resolution for a given terminal area.
    pub fn pixel_size(self, area: Rect) -> (u32, u32) {
        match self {
            RenderMode::HalfBlock => (area.width as u32, area.height as u32 * 2),
            RenderMode::Braille => (area.width as u32 * 2, area.height as u32 * 4),
            RenderMode::Ascii => (area.width as u32, area.height as u32),
        }
    }

    /// Blit the framebuffer into the ratatui buffer.
    pub fn blit(self, fb: &Framebuffer, area: Rect, buf: &mut Buffer) {
        match self {
            RenderMode::HalfBlock => blit_half_block(fb, area, buf),
            RenderMode::Braille => blit_braille(fb, area, buf),
            RenderMode::Ascii => blit_ascii(fb, area, buf),
        }
    }
}

/// Half-block blit: use ▀ (upper half block) with fg = upper pixel, bg = lower pixel.
fn blit_half_block(fb: &Framebuffer, area: Rect, buf: &mut Buffer) {
    for row in 0..area.height {
        for col in 0..area.width {
            let px = col as u32;
            let py_upper = row as u32 * 2;
            let py_lower = py_upper + 1;

            let upper = if px < fb.width && py_upper < fb.height {
                fb.get_pixel(px, py_upper)
            } else {
                Rgb::BLACK
            };

            let lower = if px < fb.width && py_lower < fb.height {
                fb.get_pixel(px, py_lower)
            } else {
                Rgb::BLACK
            };

            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_char('▀');
            cell.set_style(
                Style::default()
                    .fg(Color::Rgb(upper.0, upper.1, upper.2))
                    .bg(Color::Rgb(lower.0, lower.1, lower.2)),
            );
        }
    }
}

/// Braille blit: each cell maps to a 2×4 dot grid. Dots are set based on luminance threshold.
fn blit_braille(fb: &Framebuffer, area: Rect, buf: &mut Buffer) {
    // Braille dot offsets within Unicode block U+2800..U+28FF
    // Dot numbering:  0 3
    //                 1 4
    //                 2 5
    //                 6 7
    const DOT_BITS: [[u8; 4]; 2] = [
        [0x01, 0x02, 0x04, 0x40], // left column
        [0x08, 0x10, 0x20, 0x80], // right column
    ];

    for row in 0..area.height {
        for col in 0..area.width {
            let base_x = col as u32 * 2;
            let base_y = row as u32 * 4;

            let mut pattern: u8 = 0;
            let mut total_r: u32 = 0;
            let mut total_g: u32 = 0;
            let mut total_b: u32 = 0;
            let mut lit_count: u32 = 0;

            for dx in 0..2u32 {
                for dy in 0..4u32 {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    if px < fb.width && py < fb.height {
                        let color = fb.get_pixel(px, py);
                        if color.luminance() > 0.15 {
                            pattern |= DOT_BITS[dx as usize][dy as usize];
                            total_r += color.0 as u32;
                            total_g += color.1 as u32;
                            total_b += color.2 as u32;
                            lit_count += 1;
                        }
                    }
                }
            }

            let ch = char::from_u32(0x2800 + pattern as u32).unwrap_or(' ');
            let fg = if lit_count > 0 {
                Color::Rgb(
                    (total_r / lit_count) as u8,
                    (total_g / lit_count) as u8,
                    (total_b / lit_count) as u8,
                )
            } else {
                Color::Rgb(0, 0, 0)
            };

            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(fg).bg(Color::Rgb(0, 0, 0)));
        }
    }
}

const ASCII_RAMP: &[u8] = b" .:-=+*#%@";

/// ASCII blit: map luminance to a character ramp, colored with the pixel color.
fn blit_ascii(fb: &Framebuffer, area: Rect, buf: &mut Buffer) {
    for row in 0..area.height {
        for col in 0..area.width {
            let px = col as u32;
            let py = row as u32;

            let color = if px < fb.width && py < fb.height {
                fb.get_pixel(px, py)
            } else {
                Rgb::BLACK
            };

            let lum = color.luminance();
            let idx = (lum * (ASCII_RAMP.len() - 1) as f32).round() as usize;
            let ch = ASCII_RAMP[idx.min(ASCII_RAMP.len() - 1)] as char;

            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_char(ch);
            cell.set_style(
                Style::default()
                    .fg(Color::Rgb(color.0, color.1, color.2))
                    .bg(Color::Rgb(0, 0, 0)),
            );
        }
    }
}
