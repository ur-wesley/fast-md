mod component;

pub use component::VirtualList;

/// Visible index range `[start, end)` with top/bottom spacer heights (px).
#[must_use]
pub fn visible_range(
    scroll_top: f64,
    viewport_height: f64,
    item_count: usize,
    row_height: u32,
    overscan: usize,
) -> (usize, usize, u32, u32) {
    if item_count == 0 || row_height == 0 {
        return (0, 0, 0, 0);
    }

    let row_h = f64::from(row_height);
    let first_visible = (scroll_top / row_h).floor() as usize;
    let visible_count = (viewport_height / row_h).ceil() as usize + 1;
    let window = visible_count + overscan * 2 + 1;

    let mut start = first_visible.saturating_sub(overscan);
    let mut end = (first_visible + visible_count + overscan + 1).min(item_count);

    if start >= item_count {
        start = item_count.saturating_sub(window);
        end = item_count;
    }

    start = start.min(end);

    let top_pad = u32::try_from(start.saturating_mul(row_height as usize)).unwrap_or(u32::MAX);
    let bottom_pad = u32::try_from((item_count - end).saturating_mul(row_height as usize))
        .unwrap_or(u32::MAX);

    (start, end, top_pad, bottom_pad)
}

#[must_use]
pub fn total_list_height(item_count: usize, row_height: u32) -> u32 {
    u32::try_from(item_count.saturating_mul(row_height as usize)).unwrap_or(u32::MAX)
}

/// How far the list has been scrolled past the ancestor viewport top.
///
/// Uses bounding rects (scroll is already baked in). Negative (list still below
/// the viewport) clamps to 0.
#[must_use]
pub fn relative_scroll(parent_rect_top: f64, list_rect_top: f64) -> f64 {
    (parent_rect_top - list_rect_top).max(0.0)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_returns_zeros() {
        assert_eq!(visible_range(0.0, 400.0, 0, 28, 8), (0, 0, 0, 0));
    }

    #[test]
    fn zero_row_height_returns_zeros() {
        assert_eq!(visible_range(0.0, 400.0, 10, 0, 8), (0, 0, 0, 0));
    }

    #[test]
    fn start_includes_overscan() {
        let (start, end, top_pad, _) = visible_range(280.0, 400.0, 100, 28, 8);
        assert_eq!(start, 2);
        assert!(end > start);
        assert_eq!(top_pad, 56);
    }

    #[test]
    fn end_clamps_to_item_count() {
        let (start, end, _, bottom_pad) = visible_range(0.0, 400.0, 5, 28, 8);
        assert_eq!(start, 0);
        assert_eq!(end, 5);
        assert_eq!(bottom_pad, 0);
    }

    #[test]
    fn scroll_past_end_clamps_start() {
        let (start, end, _, bottom_pad) = visible_range(10_000.0, 200.0, 20, 28, 4);
        assert!(start < 20);
        assert_eq!(end, 20);
        assert_eq!(bottom_pad, 0);
    }

    #[test]
    fn total_list_height_multiplies() {
        assert_eq!(total_list_height(10, 28), 280);
    }

    #[test]
    fn relative_scroll_clamps_when_list_below_viewport() {
        assert_eq!(relative_scroll(100.0, 200.0), 0.0);
    }

    #[test]
    fn relative_scroll_zero_when_aligned() {
        assert_eq!(relative_scroll(80.0, 80.0), 0.0);
    }

    #[test]
    fn relative_scroll_is_distance_scrolled_past() {
        assert_eq!(relative_scroll(100.0, 40.0), 60.0);
    }
}
