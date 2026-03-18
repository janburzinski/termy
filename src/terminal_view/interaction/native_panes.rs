use super::*;

impl TerminalView {
    pub(in super::super) fn native_pane_min_extent_for_axis(axis: PaneResizeAxis) -> u16 {
        match axis {
            PaneResizeAxis::Horizontal => NATIVE_PANE_MIN_COLS,
            PaneResizeAxis::Vertical => NATIVE_PANE_MIN_ROWS,
        }
    }

    pub(in super::super) fn native_min_extent_allowed(
        total_extent: u16,
        pane_count: usize,
        min_extent: u16,
    ) -> u16 {
        let pane_count =
            u16::try_from(pane_count).expect("native pane count must fit into u16");
        assert!(pane_count > 0, "native pane count must be non-zero");
        let required = min_extent.saturating_mul(pane_count);
        if total_extent >= required {
            min_extent
        } else {
            (total_extent / pane_count).max(1)
        }
    }

    pub(in super::super) fn compute_terminal_cols(
        terminal_width: f32,
        cell_width: f32,
        edge_to_edge_grid: bool,
    ) -> u16 {
        let cols = if edge_to_edge_grid {
            (terminal_width / cell_width).ceil()
        } else {
            (terminal_width / cell_width).floor()
        };
        cols.max(2.0) as u16
    }

    pub(in super::super) fn compute_terminal_rows(terminal_height: f32, cell_height: f32) -> u16 {
        (terminal_height / cell_height).floor().max(1.0) as u16
    }

    fn scale_native_pane_edge(edge: u16, old_extent: u16, new_extent: u16) -> u16 {
        if old_extent == 0 || new_extent == 0 {
            return 0;
        }

        let scaled = (u32::from(edge) * u32::from(new_extent)) / u32::from(old_extent);
        scaled.min(u32::from(new_extent)) as u16
    }

    pub(in super::super) fn sync_native_tab_pane_geometry(tab: &mut TerminalTab, cols: u16, rows: u16) {
        if tab.panes.is_empty() {
            return;
        }
        assert!(
            tab.panes.iter().any(|pane| pane.id == tab.active_pane_id),
            "native tab resize requires a valid active pane id"
        );

        let cols = cols.max(1);
        let rows = rows.max(1);

        if tab.panes.len() == 1 {
            if let Some(only) = tab.panes.first_mut() {
                only.left = 0;
                only.top = 0;
                only.width = cols;
                only.height = rows;
            }
            return;
        }

        let old_cols = tab
            .panes
            .iter()
            .map(|pane| pane.left.saturating_add(pane.width))
            .max()
            .unwrap_or(cols)
            .max(1);
        let old_rows = tab
            .panes
            .iter()
            .map(|pane| pane.top.saturating_add(pane.height))
            .max()
            .unwrap_or(rows)
            .max(1);
        for pane in &mut tab.panes {
            let old_left = pane.left;
            let old_top = pane.top;
            let old_right = pane.left.saturating_add(pane.width);
            let old_bottom = pane.top.saturating_add(pane.height);

            let mut new_left =
                Self::scale_native_pane_edge(old_left, old_cols, cols).min(cols.saturating_sub(1));
            let mut new_top =
                Self::scale_native_pane_edge(old_top, old_rows, rows).min(rows.saturating_sub(1));
            let mut new_right = Self::scale_native_pane_edge(old_right, old_cols, cols).min(cols);
            let mut new_bottom = Self::scale_native_pane_edge(old_bottom, old_rows, rows).min(rows);

            if new_right <= new_left {
                new_right = (new_left + 1).min(cols);
                new_left = new_right.saturating_sub(1);
            }
            if new_bottom <= new_top {
                new_bottom = (new_top + 1).min(rows);
                new_top = new_bottom.saturating_sub(1);
            }

            pane.left = new_left;
            pane.top = new_top;
            pane.width = new_right.saturating_sub(new_left).max(1);
            pane.height = new_bottom.saturating_sub(new_top).max(1);
        }
    }

    pub(in super::super) fn should_emit_tmux_resize_error_toast(&mut self, now: Instant) -> bool {
        let debounce_window = Duration::from_millis(TMUX_RESIZE_ERROR_TOAST_DEBOUNCE_MS);
        match self.last_tmux_resize_error_at {
            Some(previous) if now.saturating_duration_since(previous) < debounce_window => false,
            _ => {
                self.last_tmux_resize_error_at = Some(now);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_terminal_rows_floors_fractional_row_count() {
        assert_eq!(TerminalView::compute_terminal_rows(24.1, 12.0), 2);
        assert_eq!(TerminalView::compute_terminal_rows(23.9, 12.0), 1);
    }

    #[test]
    fn compute_terminal_rows_enforces_minimum_one_row() {
        assert_eq!(TerminalView::compute_terminal_rows(0.5, 12.0), 1);
    }

    #[test]
    fn compute_terminal_cols_preserves_edge_to_edge_ceil_behavior() {
        assert_eq!(TerminalView::compute_terminal_cols(24.1, 12.0, false), 2);
        assert_eq!(TerminalView::compute_terminal_cols(24.1, 12.0, true), 3);
    }

    fn test_terminal() -> Terminal {
        Terminal::new_tmux(TerminalSize::default(), TerminalOptions::default())
    }

    fn test_pane(id: &str, left: u16, top: u16, width: u16, height: u16) -> TerminalPane {
        TerminalPane {
            id: id.to_string(),
            left,
            top,
            width,
            height,
            degraded: false,
            terminal: test_terminal(),
            render_cache: RefCell::new(TerminalPaneRenderCache::default()),
            last_alternate_screen: Cell::new(false),
        }
    }

    #[test]
    #[should_panic(expected = "native pane count must be non-zero")]
    fn native_min_extent_allowed_rejects_zero_panes() {
        let _ = TerminalView::native_min_extent_allowed(10, 0, 2);
    }

    #[test]
    fn sync_native_tab_pane_geometry_keeps_existing_active_pane_id() {
        let mut tab = TerminalTab {
            id: 1,
            window_id: "@native-1".to_string(),
            window_index: 0,
            panes: vec![test_pane("%native-1", 0, 0, 40, 20)],
            active_pane_id: "%native-1".to_string(),
            manual_title: None,
            explicit_title: None,
            shell_title: None,
            current_command: None,
            pending_command_title: None,
            pending_command_token: 0,
            last_prompt_cwd: None,
            title: DEFAULT_TAB_TITLE.to_string(),
            title_text_width: 0.0,
            sticky_title_width: 0.0,
            display_width: TAB_MIN_WIDTH,
            running_process: false,
        };

        TerminalView::sync_native_tab_pane_geometry(&mut tab, 120, 42);

        assert_eq!(tab.active_pane_id, "%native-1");
        let pane = &tab.panes[0];
        assert_eq!(pane.width, 120);
        assert_eq!(pane.height, 42);
    }
}
