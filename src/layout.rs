// ─────────────────────────────────────────────────────────────────────────────
// layout.rs
// ─────────────────────────────────────────────────────────────────────────────

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};

// ── Grid ─────────────────────────────────────────────────────────────────────

pub fn calculate_grid(n: usize, screen_w: u32, screen_h: u32) -> (usize, usize) {
    if n <= 1 {
        return (1, 1);
    }
    let aspect = screen_w as f64 / screen_h as f64;
    let cols_f  = ((n as f64) * aspect).sqrt();
    let mut best_cols = 1usize;
    let mut best_rows = n;
    let mut best_waste = usize::MAX;
    for cols in [cols_f as usize, cols_f as usize + 1] {
        let cols = cols.max(1);
        let rows = (n + cols - 1) / cols;
        let waste = cols * rows - n;
        if waste < best_waste {
            best_waste = waste;
            best_cols  = cols;
            best_rows  = rows;
        }
    }
    (best_cols.max(1), best_rows.max(1))
}

// ── Scatter positions ─────────────────────────────────────────────────────────

/// Raw jittered grid positions — call `settle_positions` afterwards to
/// resolve any overlaps at the base font size.
pub fn scatter_positions(n: usize, screen_w: u32, screen_h: u32) -> Vec<(f64, f64)> {
    if n == 0 { return vec![]; }
    let (cols, _rows) = calculate_grid(n, screen_w, screen_h);
    let cell_w = screen_w as f64 / cols as f64;
    let (_, rows) = calculate_grid(n, screen_w, screen_h);
    let cell_h = screen_h as f64 / rows as f64;
    const JITTER: f64 = 0.20;
    (0..n).map(|i| {
        let col = i % cols;
        let row = i / cols;
        let cx = col as f64 * cell_w + cell_w / 2.0;
        let cy = row as f64 * cell_h + cell_h / 2.0;
        let jx = pseudo_rand(i.wrapping_mul(2)) * 2.0 - 1.0;
        let jy = pseudo_rand(i.wrapping_mul(2).wrapping_add(1)) * 2.0 - 1.0;
        let x = (cx + jx * cell_w * JITTER).clamp(cell_w * 0.15, screen_w as f64 - cell_w * 0.15);
        let y = (cy + jy * cell_h * JITTER).clamp(cell_h * 0.15, screen_h as f64 - cell_h * 0.15);
        (x, y)
    }).collect()
}

fn pseudo_rand(seed: usize) -> f64 {
    let x = seed.wrapping_mul(2_654_435_761).wrapping_add(0x9e37_79b9);
    let x = x ^ (x >> 16);
    let x = x.wrapping_mul(0x45d9_f3b7);
    let x = x ^ (x >> 16);
    (x & 0xFFFF) as f64 / 65_535.0
}

/// Run AABB separation on `initial` positions so that at `font_size` no two
/// names overlap.  Called once at startup in `compute_layout` — result is
/// stored in `App::app_positions` so the per-frame render loop starts from
/// a clean, non-overlapping state.
pub fn settle_positions(
    names:      &[&str],
    initial:    &[(f64, f64)],
    font_size:  f64,
    font_family: &str,
    screen_w:   u32,
    screen_h:   u32,
) -> Vec<(f64, f64)> {
    const MARGIN: f64 = 8.0;
    const PASSES: usize = 40;

    let dummy = ImageSurface::create(Format::ARgb32, 1, 1).expect("dummy");
    let cr    = Context::new(&dummy).expect("ctx");
    cr.select_font_face(font_family, FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(font_size);

    let half: Vec<(f64, f64)> = names.iter().map(|name| {
        let ext = cr.text_extents(name).unwrap();
        (ext.width() / 2.0, ext.height() / 2.0)
    }).collect();

    let mut pos = initial.to_vec();
    let n = pos.len();

    for _ in 0..PASSES {
        for i in 0..n {
            for j in (i + 1)..n {
                let (hw_i, hh_i) = half[i];
                let (hw_j, hh_j) = half[j];
                let (cx_i, cy_i) = pos[i];
                let (cx_j, cy_j) = pos[j];
                let dx = cx_j - cx_i;
                let dy = cy_j - cy_i;
                let ov_x = (hw_i + hw_j + MARGIN) - dx.abs();
                let ov_y = (hh_i + hh_j + MARGIN) - dy.abs();
                if ov_x > 0.0 && ov_y > 0.0 {
                    let (px, py) = if ov_x <= ov_y {
                        let s = if dx >= 0.0 { 1.0 } else { -1.0 };
                        (ov_x / 2.0 * s, 0.0)
                    } else {
                        let s = if dy >= 0.0 { 1.0 } else { -1.0 };
                        (0.0, ov_y / 2.0 * s)
                    };
                    pos[i] = (cx_i - px, cy_i - py);
                    pos[j] = (cx_j + px, cy_j + py);
                }
            }
        }
        for i in 0..n {
            let (hw, hh) = half[i];
            let (cx, cy) = pos[i];
            pos[i] = (
                cx.clamp(hw + MARGIN, screen_w as f64 - hw - MARGIN),
                cy.clamp(hh + MARGIN, screen_h as f64 - hh - MARGIN),
            );
        }
    }
    pos
}

// ── Font sizes ────────────────────────────────────────────────────────────────

/// Base font size at startup (all apps visible).
///
/// Uses a **screen-fill-ratio** formula instead of cell-based fitting:
///
///   font = sqrt(FILL × W × H / (N × avg_len × CHAR_W × LINE_H))
///
/// This means fewer apps on screen → bigger font, regardless of grid cell
/// size.  The resulting names may overlap their neighbours (especially long
/// ones) — `settle_positions` is called once in `compute_layout` to resolve
/// those overlaps so the startup view is always clean.
pub fn compute_base_font_size(
    names:    &[&str],
    screen_w: u32,
    screen_h: u32,
    _font_family: &str,   // analytical formula; no Cairo needed
    min_size: f64,
    max_size: f64,
) -> f64 {
    let n = names.len();
    if n == 0 { return min_size; }

    let avg_len = names.iter().map(|s| s.chars().count()).sum::<usize>() as f64 / n as f64;
    let avg_len = avg_len.max(1.0);

    let screen_area = screen_w as f64 * screen_h as f64;

    // Target: ~55 % of screen area covered by name bounding boxes.
    // At size s: avg text area ≈ avg_len × 0.55s × 1.20s = 0.66 × avg_len × s²
    // N × 0.66 × avg_len × s² = FILL × W × H  →  s = sqrt(FILL×W×H / (N×0.66×avg_len))
    const FILL:      f64 = 0.55;
    const CHAR_AREA: f64 = 0.55 * 1.20; // width_ratio × height_ratio ≈ 0.66

    let s = (FILL * screen_area / (n as f64 * CHAR_AREA * avg_len)).sqrt();
    s.clamp(min_size, max_size)
}

/// Target size for apps matching the current query.
/// Grows as fewer names match (smaller conceptual grid → bigger cells → bigger font).
pub fn compute_match_font_size(
    matching_names: &[&str],
    screen_w:       u32,
    screen_h:       u32,
    font_family:    &str,
    base_size:      f64,
    max_size:       f64,
) -> f64 {
    let n = matching_names.len();
    if n == 0 { return base_size; }

    let (cols, rows) = calculate_grid(n, screen_w, screen_h);
    let cell_w = screen_w as f64 / cols as f64;
    let cell_h = screen_h as f64 / rows as f64;

    let longest = matching_names
        .iter()
        .max_by_key(|s| s.chars().count())
        .copied()
        .unwrap_or("");

    if longest.is_empty() { return base_size; }

    fit_text_in_box(longest, cell_w * 0.85, cell_h * 0.85, font_family, base_size, max_size)
}

// ── Internal helper ───────────────────────────────────────────────────────────

fn fit_text_in_box(
    text:        &str,
    max_w:       f64,
    max_h:       f64,
    font_family: &str,
    lo_bound:    f64,
    hi_bound:    f64,
) -> f64 {
    let dummy = ImageSurface::create(Format::ARgb32, 1, 1).expect("dummy surface");
    let cr    = Context::new(&dummy).expect("dummy context");
    cr.select_font_face(font_family, FontSlant::Normal, FontWeight::Normal);
    let mut lo = lo_bound; let mut hi = hi_bound; let mut best = lo_bound;
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        cr.set_font_size(mid);
        let ext = cr.text_extents(text).unwrap();
        if ext.width() <= max_w && ext.height() <= max_h { best = mid; lo = mid; }
        else { hi = mid; }
    }
    best.clamp(lo_bound, hi_bound)
}
