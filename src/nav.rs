// ─────────────────────────────────────────────────────────────────────────────
// nav.rs
//
// Arrow-key navigation over the visible subset of the apps.
//
// Navigation is purely 1D and based on the *latest rendered visual positions*
// (after jitter and AABB inflation/separation).
//
// DOWN → goes to the next visual item in reading order (top-to-bottom,
//        left-to-right).
// UP   → goes to the previous visual item in reading order.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
}

/// Move the selection from `current` in `direction` among the `items` slice.
///
/// `items` — indices (into the global app list) that are currently navigatable.
/// `current` — the currently selected app index (must be in `items`).
/// `app_positions` — the exact rendered `(cx, cy)` positions of all apps.
/// `row_height` — proxy for row bucketing (usually base_font_size).
///
/// Returns the new selected index.
pub fn navigate(
    items: &[usize],
    current: usize,
    app_positions: &[(f64, f64)],
    row_height: f64,
    dir: Direction,
) -> usize {
    if items.is_empty() {
        return current;
    }

    // Sort items by their visual position to establish a reading order.
    // Because of scatter jitter and AABB pushes, `cy` isn't perfectly aligned
    // into grid rows.  We bucket `cy` into rows using `row_height * 0.5` so
    // items visually on the "same line" are sorted by `cx`.
    let bucket_size = (row_height * 0.5).max(10.0);

    let mut sorted_items = items.to_vec();
    sorted_items.sort_by(|&a, &b| {
        let (ax, ay) = app_positions.get(a).copied().unwrap_or((0.0, 0.0));
        let (bx, by) = app_positions.get(b).copied().unwrap_or((0.0, 0.0));

        let row_a = (ay / bucket_size).round() as i32;
        let row_b = (by / bucket_size).round() as i32;

        row_a.cmp(&row_b).then(ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal))
    });

    let current_idx = sorted_items.iter().position(|&x| x == current).unwrap_or(0);

    match dir {
        Direction::Down => {
            // Next item, wrap to first
            sorted_items[(current_idx + 1) % sorted_items.len()]
        }
        Direction::Up => {
            // Previous item, wrap to last
            if current_idx == 0 {
                sorted_items[sorted_items.len() - 1]
            } else {
                sorted_items[current_idx - 1]
            }
        }
    }
}
