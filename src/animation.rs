use ratatui::prelude::*;
use ratatui::Frame;
use std::time::Duration;

/// Performs a manual reveal animation (Left to Right) by clearing the buffer
/// for the area that hasn't been revealed yet.
///
/// # Arguments
///
/// * `f` - The current frame to render on
/// * `elapsed` - The time elapsed since the start of the animation
/// * `duration` - The total duration of the animation
pub fn perform_manual_reveal(f: &mut Frame, elapsed: Duration, duration: Duration) {
    if elapsed >= duration {
        return;
    }

    let area = f.area();
    let buf = f.buffer_mut();

    let t: f32 = elapsed.as_secs_f32() / duration.as_secs_f32();
    let t = t.min(1.0);

    // Ease Out Cubic for smooth deceleration
    let progress = 1.0 - (1.0 - t).powi(3);

    let visible_width = (area.width as f32 * progress) as u16;
    let reveal_x = area.left() + visible_width;

    // Hide (clear) everything to the right of the reveal line
    for y in area.top()..area.bottom() {
        for x in reveal_x..area.right() {
            let cell = buf.get_mut(x, y);
            cell.set_char(' ');
            cell.reset(); // Reset style to default (usually terminal background)
        }
    }
}

/// A value that transitions smoothly between targets using exponential smoothing.
pub struct SmoothValue {
    pub current: f32,
    pub target: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
}

impl SmoothValue {
    /// Creates a new SmoothValue with default spring parameters optimized for UI.
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
            // Configuration for Cushion Effect (Exponential Smoothing)
            // Using a high speed factor for quick response but smooth arrival.
            // This method is unconditionally stable and never oscillates.
            stiffness: 15.0, // Represents "Speed" in this context
            damping: 0.0,    // Unused in exponential smoothing
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Updates the value state based on delta time (seconds).
    pub fn update(&mut self, dt: f32) {
        // Robust Exponential Smoothing (Frame-rate independent)
        // Formula: current += (target - current) * (1 - e^(-speed * dt))
        // This provides a "Cushion" effect: Fast approach, smooth deceleration, no bounce.

        let speed = self.stiffness; // Use stiffness field as speed

        // Clamp dt to avoid huge jumps if frame hangs
        let dt = dt.min(0.1);

        let diff = self.target - self.current;
        let alpha = 1.0 - (-speed * dt).exp();

        self.current += diff * alpha;

        // Snap to target if very close
        if (self.target - self.current).abs() < 0.001 {
            self.current = self.target;
        }

        // Velocity property is approximated for compatibility
        self.velocity = diff * speed;
    }

    /// Returns the current value
    pub fn value(&self) -> f32 {
        self.current
    }
}

/// Draws an animated border segment that travels around the perimeter of the given area.
/// The segment color is fixed to Cyan.
///
/// # Arguments
/// * `f` - The frame to render to
/// * `area` - The rectangular area of the block
/// * `progress` - A value between 0.0 and 1.0 representing the position of the segment
pub fn draw_traveling_border(f: &mut Frame, area: Rect, progress: f32) {
    let buf = f.buffer_mut();
    let width = area.width as usize;
    let height = area.height as usize;

    // Border requires at least 2x2
    if width < 2 || height < 2 {
        return;
    }

    // Calculate total perimeter length (excluding duplicated corners)
    // Top: w, Right: h-1, Bottom: w-1, Left: h-2
    // Sum = w + h - 1 + w - 1 + h - 2 = 2w + 2h - 4
    let perimeter_len = 2 * (width + height) - 4;

    // Length of the colored segment (e.g., 25% of the border)
    let segment_len = (perimeter_len as f32 * 0.25) as usize;
    let total_cells = perimeter_len;

    // Calculate starting index based on progress
    // Ensure loop wrapping
    let start_idx = (progress * total_cells as f32) as usize;

    // Function to map linear index (0..perimeter_len) to (x, y) coords
    let get_coords = |idx: usize| -> (u16, u16) {
        let idx = idx % total_cells;

        if idx < width {
            // Top edge (Left to Right)
            (area.left() + idx as u16, area.top())
        } else if idx < width + height - 1 {
            // Right edge (Top to Bottom)
            let offset = idx - width;
            (area.right() - 1, area.top() + 1 + offset as u16)
        } else if idx < width + height - 1 + width - 1 {
            // Bottom edge (Right to Left)
            let offset = idx - (width + height - 1);
            (area.right() - 2 - offset as u16, area.bottom() - 1)
        } else {
            // Left edge (Bottom to Top)
            let offset = idx - (width + height - 1 + width - 1);
            (area.left(), area.bottom() - 2 - offset as u16)
        }
    };

    // Draw the Cyan segment
    for i in 0..segment_len {
        let idx = start_idx + i;
        let (x, y) = get_coords(idx);

        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_fg(Color::Cyan);
            // Optionally make it bold
            cell.set_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }
}
