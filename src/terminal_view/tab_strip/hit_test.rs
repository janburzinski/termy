use super::super::*;
use super::layout::TabStripGeometry;
use super::render::{VerticalBottomShelfLayout, VerticalNewTabShelfLayout};

impl TerminalView {
    fn point_hits_rect(
        x: f32,
        y: f32,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
    ) -> bool {
        x >= left && x < left + width && y >= top && y < top + height
    }

    pub(crate) fn unified_titlebar_tab_shell_hit_test(
        pointer_x: f32,
        pointer_y: f32,
        tab_widths: impl IntoIterator<Item = f32>,
        scroll_offset_x: f32,
    ) -> bool {
        let tab_top = TOP_STRIP_CONTENT_OFFSET_Y + (TABBAR_HEIGHT - TAB_ITEM_HEIGHT);
        let tab_bottom = TOP_STRIP_CONTENT_OFFSET_Y + TABBAR_HEIGHT;
        if pointer_y < tab_top || pointer_y >= tab_bottom {
            return false;
        }

        let mut left = TAB_HORIZONTAL_PADDING + scroll_offset_x;
        for width in tab_widths {
            let right = left + width;
            if pointer_x >= left && pointer_x < right {
                return true;
            }
            left = right + TAB_ITEM_GAP;
        }

        false
    }

    pub(crate) fn unified_titlebar_tab_interactive_hit_test(
        &self,
        x: f32,
        y: f32,
        window: &Window,
    ) -> bool {
        let geometry = self.tab_strip_geometry(window);
        let scroll_offset_x: f32 = self.tab_strip.horizontal_scroll_handle.offset().x.into();
        Self::unified_titlebar_tab_interactive_hit_test_for_geometry(
            x,
            y,
            geometry,
            self.tabs.iter().map(|tab| tab.display_width),
            scroll_offset_x,
        )
    }

    pub(crate) fn unified_titlebar_tab_interactive_hit_test_for_geometry(
        x: f32,
        y: f32,
        geometry: TabStripGeometry,
        tab_widths: impl IntoIterator<Item = f32>,
        scroll_offset_x: f32,
    ) -> bool {
        if geometry.contains_tabs_viewport_x(x) {
            let pointer_x = (x - geometry.row_start_x).clamp(0.0, geometry.tabs_viewport_width);
            if Self::unified_titlebar_tab_shell_hit_test(pointer_x, y, tab_widths, scroll_offset_x)
            {
                return true;
            }
        }

        if !geometry.contains_action_rail_x(x) {
            return false;
        }

        geometry.new_tab_button_contains(x, y)
    }

    pub(crate) fn vertical_tab_shell_hit_test(
        pointer_x: f32,
        pointer_y: f32,
        strip_width: f32,
        tab_heights: impl IntoIterator<Item = f32>,
        scroll_offset_y: f32,
    ) -> bool {
        if pointer_x < 0.0 || pointer_x >= strip_width || pointer_y < 0.0 {
            return false;
        }

        let mut top = scroll_offset_y;
        for height in tab_heights {
            let bottom = top + height;
            if pointer_y >= top && pointer_y < bottom {
                return true;
            }
            top = bottom + TAB_ITEM_GAP;
        }

        false
    }

    pub(crate) fn vertical_tab_strip_interactive_hit_test(
        &self,
        x: f32,
        y: f32,
    ) -> bool {
        let local_y = y - self.chrome_height();
        if local_y < 0.0 {
            return false;
        }

        let strip_width = self.effective_vertical_tab_strip_width();
        let compact = self.vertical_tabs_minimized;
        let top_shelf_layout =
            Self::vertical_new_tab_shelf_layout(strip_width - TAB_STROKE_THICKNESS, compact);
        let bottom_shelf_layout = Self::vertical_bottom_shelf_layout();
        let list_height = self.effective_vertical_tabs_list_height();
        let scroll_offset_y: f32 = self.tab_strip.vertical_scroll_handle.offset().y.into();

        Self::vertical_tab_strip_interactive_hit_test_for_layout(
            x,
            local_y,
            strip_width,
            self.vertical_tab_strip_header_height(),
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            std::iter::repeat_n(TAB_ITEM_HEIGHT, self.tabs.len()),
            scroll_offset_y,
            compact,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn vertical_tab_strip_interactive_hit_test_for_layout(
        x: f32,
        y: f32,
        strip_width: f32,
        header_height: f32,
        top_shelf_layout: VerticalNewTabShelfLayout,
        bottom_shelf_layout: VerticalBottomShelfLayout,
        list_height: f32,
        tab_heights: impl IntoIterator<Item = f32>,
        scroll_offset_y: f32,
        compact: bool,
    ) -> bool {
        if x < 0.0 || x >= strip_width || y < 0.0 {
            return false;
        }

        if !compact && x >= (strip_width - 4.0).max(0.0) {
            return true;
        }

        let top_shelf_top = header_height;
        if Self::point_hits_rect(
            x,
            y,
            top_shelf_layout.button_x,
            top_shelf_top + top_shelf_layout.button_y,
            top_shelf_layout.button_width,
            top_shelf_layout.button_height,
        ) {
            return true;
        }

        let list_top = top_shelf_top + top_shelf_layout.shelf_height;
        let list_bottom = list_top + list_height;
        if y >= list_top && y < list_bottom {
            return Self::vertical_tab_shell_hit_test(
                x,
                y - list_top,
                strip_width,
                tab_heights,
                scroll_offset_y,
            );
        }

        let bottom_shelf_top = list_bottom;
        let (button_x, button_y) = Self::vertical_bottom_shelf_button_origin(bottom_shelf_layout);
        Self::point_hits_rect(
            x,
            y,
            button_x,
            bottom_shelf_top + button_y,
            bottom_shelf_layout.button_size,
            bottom_shelf_layout.button_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab_hit_test_y() -> f32 {
        TOP_STRIP_CONTENT_OFFSET_Y + TABBAR_HEIGHT - 1.0
    }

    fn vertical_hit_test_layout(
        strip_width: f32,
        compact: bool,
    ) -> (
        VerticalNewTabShelfLayout,
        VerticalBottomShelfLayout,
        f32,
        f32,
    ) {
        (
            TerminalView::vertical_new_tab_shelf_layout(
                strip_width - TAB_STROKE_THICKNESS,
                compact,
            ),
            TerminalView::vertical_bottom_shelf_layout(),
            TABBAR_HEIGHT,
            180.0,
        )
    }

    #[test]
    fn shell_hit_test_detects_tabs_and_respects_y_bounds() {
        let widths = [100.0, 120.0];
        let scroll_offset_x = 0.0;
        let tab_top = TOP_STRIP_CONTENT_OFFSET_Y + (TABBAR_HEIGHT - TAB_ITEM_HEIGHT);
        let tab_bottom = TOP_STRIP_CONTENT_OFFSET_Y + TABBAR_HEIGHT;
        let tab_y = tab_bottom - 1.0;
        let first_tab_left = TAB_HORIZONTAL_PADDING;
        let first_tab_right = first_tab_left + widths[0];
        let second_tab_left = first_tab_right + TAB_ITEM_GAP;

        assert!(TerminalView::unified_titlebar_tab_shell_hit_test(
            TAB_HORIZONTAL_PADDING + 20.0,
            tab_y,
            widths,
            scroll_offset_x
        ));
        assert!(!TerminalView::unified_titlebar_tab_shell_hit_test(
            TAB_HORIZONTAL_PADDING + 240.0,
            tab_y,
            widths,
            scroll_offset_x
        ));
        assert!(!TerminalView::unified_titlebar_tab_shell_hit_test(
            TAB_HORIZONTAL_PADDING + 20.0,
            TOP_STRIP_CONTENT_OFFSET_Y,
            widths,
            scroll_offset_x
        ));
        assert!(TerminalView::unified_titlebar_tab_shell_hit_test(
            TAB_HORIZONTAL_PADDING + 20.0,
            tab_top,
            widths,
            scroll_offset_x
        ));
        assert!(!TerminalView::unified_titlebar_tab_shell_hit_test(
            TAB_HORIZONTAL_PADDING + 20.0,
            tab_bottom,
            widths,
            scroll_offset_x
        ));
        assert!(!TerminalView::unified_titlebar_tab_shell_hit_test(
            first_tab_right,
            tab_y,
            [widths[0]],
            scroll_offset_x
        ));
        assert!(TerminalView::unified_titlebar_tab_shell_hit_test(
            second_tab_left,
            tab_y,
            widths,
            scroll_offset_x
        ));
    }

    #[test]
    fn interactive_hit_test_detects_tab_shell() {
        let geometry = TerminalView::tab_strip_geometry_for_viewport_width(1280.0);
        let x = geometry.row_start_x + TAB_HORIZONTAL_PADDING + 12.0;
        assert!(
            TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                x,
                tab_hit_test_y(),
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn interactive_hit_test_detects_new_tab_button() {
        let geometry = TerminalView::tab_strip_geometry_for_viewport_width(1280.0);
        let center_x = (geometry.button_start_x + geometry.button_end_x) * 0.5;
        let center_y = (geometry.button_start_y + geometry.button_end_y) * 0.5;
        assert!(
            TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                center_x,
                center_y,
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn interactive_hit_test_excludes_action_rail_empty_space() {
        let geometry = TerminalView::tab_strip_geometry_for_viewport_width(1280.0);
        let x = geometry.action_rail_start_x + 1.0;
        let y = (geometry.button_start_y + geometry.button_end_y) * 0.5;
        assert!(!geometry.new_tab_button_contains(x, y));
        assert!(geometry.contains_action_rail_x(x));
        assert!(
            !TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                x,
                y,
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn interactive_hit_test_excludes_gutter() {
        let geometry = TerminalView::tab_strip_geometry_for_viewport_width(1280.0);
        let x = geometry.gutter_start_x + (geometry.gutter_width * 0.5);
        assert!(geometry.contains_gutter_x(x));
        assert!(
            !TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                x,
                tab_hit_test_y(),
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn interactive_hit_test_excludes_expanded_left_inset_branding_space() {
        let base_left_inset = TerminalView::titlebar_left_padding_for_platform();
        let geometry = TerminalView::tab_strip_geometry_for_viewport_with_left_inset(
            1280.0,
            base_left_inset + 64.0,
        );
        let x = geometry.left_inset_width - 1.0;
        assert!(
            !TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                x,
                tab_hit_test_y(),
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn interactive_hit_test_respects_half_open_region_boundaries() {
        let geometry = TerminalView::tab_strip_geometry_for_viewport_width(1280.0);
        let tabs_boundary = geometry.tabs_viewport_end_x();
        assert!(!geometry.contains_tabs_viewport_x(tabs_boundary));

        let action_start = geometry.gutter_end_x();
        assert!(geometry.contains_action_rail_x(action_start));
        assert!(
            !TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                action_start,
                tab_hit_test_y(),
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );

        let action_end = geometry.action_rail_end_x();
        assert!(!geometry.contains_action_rail_x(action_end));
        assert!(
            !TerminalView::unified_titlebar_tab_interactive_hit_test_for_geometry(
                action_end,
                tab_hit_test_y(),
                geometry,
                [120.0, 120.0],
                0.0,
            )
        );
    }

    #[test]
    fn vertical_interactive_hit_test_detects_tab_rows() {
        let strip_width = 220.0;
        let compact = false;
        let (top_shelf_layout, bottom_shelf_layout, header_height, list_height) =
            vertical_hit_test_layout(strip_width, compact);
        let list_top = header_height + top_shelf_layout.shelf_height;

        assert!(TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            24.0,
            list_top + 12.0,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT, TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
    }

    #[test]
    fn vertical_interactive_hit_test_detects_shelf_buttons_and_resize_handle() {
        let strip_width = 220.0;
        let compact = false;
        let (top_shelf_layout, bottom_shelf_layout, header_height, list_height) =
            vertical_hit_test_layout(strip_width, compact);
        let top_button_x = top_shelf_layout.button_x + (top_shelf_layout.button_width * 0.5);
        let top_button_y =
            header_height + top_shelf_layout.button_y + (top_shelf_layout.button_height * 0.5);
        let bottom_shelf_top = header_height + top_shelf_layout.shelf_height + list_height;
        let (bottom_button_x, bottom_button_y) =
            TerminalView::vertical_bottom_shelf_button_origin(bottom_shelf_layout);

        assert!(TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            top_button_x,
            top_button_y,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
        assert!(TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            bottom_button_x + (bottom_shelf_layout.button_size * 0.5),
            bottom_shelf_top + bottom_button_y + (bottom_shelf_layout.button_size * 0.5),
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
        assert!(TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            strip_width - 1.0,
            24.0,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
    }

    #[test]
    fn vertical_interactive_hit_test_excludes_noninteractive_chrome_backgrounds() {
        let strip_width = 220.0;
        let compact = false;
        let (top_shelf_layout, bottom_shelf_layout, header_height, list_height) =
            vertical_hit_test_layout(strip_width, compact);
        let list_top = header_height + top_shelf_layout.shelf_height;

        assert!(!TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            24.0,
            12.0,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
        assert!(!TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            12.0,
            header_height + 8.0,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
        assert!(!TerminalView::vertical_tab_strip_interactive_hit_test_for_layout(
            24.0,
            list_top + TAB_ITEM_HEIGHT + 40.0,
            strip_width,
            header_height,
            top_shelf_layout,
            bottom_shelf_layout,
            list_height,
            [TAB_ITEM_HEIGHT],
            0.0,
            compact,
        ));
    }
}
