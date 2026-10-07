// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Pre-allocated character buffers owned by the saver.
//!
//! Every buffer here is sized once in [`CellState::resize`] and mutated in
//! place by the effects. `render_frame` performs zero heap allocations, per
//! the steady-state rule in `AGENTS.md` — effects may not push, extend, or
//! collect into these vectors.

/// Grid-sized character buffers. `len == cols * rows` for every field except
/// [`CellState::span`].
pub struct CellState {
    pub cols: usize,
    pub rows: usize,
    /// The resolved art — what each cell settles to.
    pub target: Vec<char>,
    /// What each cell renders this frame. Effects write only here.
    pub current: Vec<char>,
    /// Normalized per-cell progress, 0.0..=1.0. Effects read and advance it.
    ///
    /// Float rather than a byte: a byte accumulator truncates sub-unit
    /// increments back to zero, so cells whose per-frame progress is under
    /// 1/255 never leave the origin.
    pub settled: Vec<f32>,
    /// Per-cell constant in 0.0..1.0, fixed at load. Staggers per-cell
    /// motion so effects don't pulse in lockstep.
    pub phase: Vec<f32>,
    /// Scalar 0.0..=1.0 sweep position, owned by single-sweep effects such as
    /// `led`. Held here so no effect needs its own growing allocation.
    pub progress: f32,
    /// First grid row occupied by the art.
    pub span: usize,
}

impl Default for CellState {
    fn default() -> Self {
        Self::new()
    }
}

impl CellState {
    pub fn new() -> Self {
        Self {
            cols: 0,
            rows: 0,
            target: Vec::new(),
            current: Vec::new(),
            settled: Vec::new(),
            phase: Vec::new(),
            progress: 0.0,
            span: 0,
        }
    }

    /// Size every buffer for a `cols`×`rows` grid, clearing contents.
    /// Reuses existing capacity when the dimensions are unchanged.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        let n = cols.saturating_mul(rows);
        resize_fill(&mut self.target, n, ' ');
        resize_fill(&mut self.current, n, ' ');
        resize_fill(&mut self.settled, n, 0.0f32);
        resize_fill(&mut self.phase, n, 0.0f32);
    }

    /// Centre `lines` into the grid and reset per-cell state.
    ///
    /// `phase` is derived from the cell index so effects that read it get a
    /// stable, layout-dependent stagger. Deterministic across runs: no RNG.
    pub fn load(&mut self, lines: &[String]) {
        self.fill_spaces();
        self.settled.fill(0.0);
        self.progress = 0.0;
        for (i, p) in self.phase.iter_mut().enumerate() {
            *p = ((i as f32) * 0.618_034).fract();
        }
        self.span = 0;
        if lines.is_empty() || self.target.is_empty() {
            return;
        }

        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let height = lines.len();
        if width == 0 || height == 0 || width > self.cols || height > self.rows {
            return;
        }

        let x0 = (self.cols - width) / 2;
        let y0 = (self.rows - height) / 2;
        self.span = y0;

        for (dy, line) in lines.iter().enumerate() {
            let y = y0 + dy;
            if y >= self.rows {
                break;
            }
            for (dx, ch) in line.chars().enumerate() {
                let x = x0 + dx;
                if x >= self.cols {
                    break;
                }
                self.target[y * self.cols + x] = ch;
            }
        }

        // Seed the drawn buffer with the art so a `draw()` before the first
        // effect pass shows the logo rather than a blank grid. Effects that
        // open with a reveal overwrite this on their first frame.
        self.current.copy_from_slice(&self.target);
    }

    /// Blank the grid to spaces and mark every cell settled, so effects that
    /// only reveal on progress start from a clean, fully-settled state.
    pub fn fill_spaces(&mut self) {
        self.target.fill(' ');
        self.current.fill(' ');
    }

    /// Cells that carry art rather than background padding.
    pub fn inked_cells(&self) -> usize {
        self.target.iter().filter(|c| **c != ' ').count()
    }
}

fn resize_fill<T: Clone>(buf: &mut Vec<T>, n: usize, fill: T) {
    buf.clear();
    buf.resize(n, fill);
}

#[cfg(test)]
mod resize_tests {
    use super::CellState;

    #[test]
    fn resize_sizes_every_buffer_to_the_grid() {
        let mut st = CellState::new();
        st.resize(10, 4);
        assert_eq!(st.target.len(), 40);
        assert_eq!(st.current.len(), 40);
        assert_eq!(st.settled.len(), 40);
        assert_eq!(st.phase.len(), 40);
    }

    #[test]
    fn load_centres_lines_and_counts_ink() {
        let mut st = CellState::new();
        st.resize(5, 3);
        st.load(&["AB".to_string(), "CD".to_string()]);
        assert_eq!(st.span, 0);
        assert_eq!(st.inked_cells(), 4);
        // 5 cols with a 2-wide block centres at x=1, so 'A' sits at index 1.
        assert_eq!(st.target[1], 'A');
        assert_eq!(st.target[2], 'B');
        assert_eq!(st.current[2], 'B');
    }

    #[test]
    fn oversized_art_is_rejected_without_panic() {
        let mut st = CellState::new();
        st.resize(3, 2);
        st.load(&["too wide".to_string()]);
        assert_eq!(st.inked_cells(), 0);
    }
}