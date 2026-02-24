use super::super::tab_chrome;
use super::super::tabs::TabDropMarkerSide;
use super::super::*;

impl TerminalView {
    fn render_tabbar_new_tab_button(
        &self,
        bg: gpui::Rgba,
        hover_bg: gpui::Rgba,
        border: gpui::Rgba,
        hover_border: gpui::Rgba,
        text: gpui::Rgba,
        hover_text: gpui::Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id("tabbar-new-tab")
            .w(px(TABBAR_NEW_TAB_BUTTON_SIZE))
            .h(px(TABBAR_NEW_TAB_BUTTON_SIZE))
            .rounded(px(TABBAR_NEW_TAB_BUTTON_RADIUS))
            .bg(bg)
            .border_1()
            .border_color(border)
            .text_color(text)
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.disarm_titlebar_window_move();
                    this.add_tab(cx);
                    cx.stop_propagation();
                }),
            )
            .hover(move |style| {
                style
                    .bg(hover_bg)
                    .border_color(hover_border)
                    .text_color(hover_text)
            })
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(TABBAR_NEW_TAB_ICON_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .mt(px(TABBAR_NEW_TAB_ICON_BASELINE_NUDGE_Y))
                    .child("+"),
            )
            .into_any_element()
    }

    pub(crate) fn render_tab_strip(
        &mut self,
        window: &Window,
        colors: &TerminalColors,
        font_family: &SharedString,
        tabbar_bg: gpui::Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tab_strip_layout = self.tab_strip_layout(window);
        self.set_tab_strip_layout_snapshot(tab_strip_layout);
        let tab_strip_geometry = tab_strip_layout.geometry;
        let tabbar_left_inset_width = tab_strip_geometry.left_inset_width;
        let tabbar_right_inset_width = tab_strip_geometry.right_inset_width;
        let tabbar_action_rail_width = tab_strip_geometry.action_rail_width;
        let tabbar_new_tab_left = (tab_strip_geometry.button_start_x - tab_strip_geometry.action_rail_start_x)
            .max(0.0);
        let tabbar_new_tab_top = (tab_strip_geometry.button_start_y - TOP_STRIP_CONTENT_OFFSET_Y).max(0.0);
        let tabbar_gutter_width = tab_strip_geometry.gutter_width;
        let tab_strip_viewport_width = tab_strip_geometry.tabs_viewport_width;
        self.sync_tab_display_widths_for_viewport_if_needed(tab_strip_viewport_width);
        let tab_strip_content_width = self
            .tab_strip_fixed_content_width()
            .max(tab_strip_viewport_width);
        let tab_strip_overflow_state = self.tab_strip_overflow_state();

        let tab_stroke_color = tab_chrome::resolve_tab_stroke_color(
            tabbar_bg,
            colors.foreground,
            TAB_STROKE_FOREGROUND_MIX,
        );
        let mut inactive_tab_bg = colors.foreground;
        inactive_tab_bg.a = self.scaled_chrome_alpha(0.10);
        let mut active_tab_bg = tabbar_bg;
        active_tab_bg.a = 0.0;
        let mut hovered_tab_bg = colors.foreground;
        hovered_tab_bg.a = self.scaled_chrome_alpha(0.13);
        let mut active_tab_text = colors.foreground;
        active_tab_text.a = 0.95;
        let mut inactive_tab_text = colors.foreground;
        inactive_tab_text.a = 0.7;
        let mut close_button_hover_bg = colors.foreground;
        close_button_hover_bg.a = self.scaled_chrome_alpha(0.24);
        let mut close_button_hover_text = colors.foreground;
        close_button_hover_text.a = 0.98;
        let mut tab_drop_marker_color = colors.cursor;
        tab_drop_marker_color.a = self.scaled_chrome_alpha(0.95);
        let mut tabbar_new_tab_bg = colors.foreground;
        tabbar_new_tab_bg.a = self.scaled_chrome_alpha(0.11);
        let mut tabbar_new_tab_hover_bg = colors.foreground;
        tabbar_new_tab_hover_bg.a = self.scaled_chrome_alpha(0.2);
        let mut tabbar_new_tab_border = colors.foreground;
        tabbar_new_tab_border.a = self.scaled_chrome_alpha(0.24);
        let mut tabbar_new_tab_hover_border = colors.cursor;
        tabbar_new_tab_hover_border.a = self.scaled_chrome_alpha(0.76);
        let mut tabbar_new_tab_text = colors.foreground;
        tabbar_new_tab_text.a = 0.9;
        let mut tabbar_new_tab_hover_text = colors.cursor;
        tabbar_new_tab_hover_text.a = 0.98;
        let mut tab_overflow_fade_soft = tabbar_bg;
        tab_overflow_fade_soft.a = self.scaled_chrome_alpha(0.18);
        let mut tab_overflow_fade_mid = tabbar_bg;
        tab_overflow_fade_mid.a = self.scaled_chrome_alpha(0.28);
        let mut tab_overflow_fade_strong = tabbar_bg;
        tab_overflow_fade_strong.a = self.scaled_chrome_alpha(0.38);

        let active_tab_index = (self.active_tab < self.tabs.len()).then_some(self.active_tab);
        let tab_chrome_layout = tab_chrome::compute_tab_chrome_layout(
            self.tabs.iter().map(|tab| tab.display_width),
            tab_chrome::TabChromeInput {
                active_index: active_tab_index,
                tabbar_height: TABBAR_HEIGHT,
                tab_item_height: TAB_ITEM_HEIGHT,
                horizontal_padding: TAB_HORIZONTAL_PADDING,
                tab_item_gap: TAB_ITEM_GAP,
            },
        );
        debug_assert!(tab_chrome_layout.tab_strokes.len() == self.tabs.len());

        let render_tab_stroke = |stroke: tab_chrome::StrokeRect| {
            div()
                .absolute()
                .left(px(stroke.x))
                .top(px(stroke.y))
                .w(px(stroke.w))
                .h(px(stroke.h))
                .bg(tab_stroke_color)
        };

        let mut tabs_scroll_content = div()
            .id("tabs-scroll-content")
            .flex_none()
            .w(px(tab_strip_content_width))
            .min_w(px(tab_strip_content_width))
            .h(px(TABBAR_HEIGHT))
            .flex()
            .relative()
            .items_end()
            .gap(px(TAB_ITEM_GAP))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                let hovered_changed = this.tab_strip.hovered_tab.take().is_some()
                    || this.tab_strip.hovered_tab_close.take().is_some();
                let drag_changed = if event.dragging() {
                    let (pointer_x, viewport_width) =
                        this.tab_strip_pointer_x_from_window_x(window, event.position.x);
                    this.update_tab_drag_preview(pointer_x, viewport_width, cx)
                } else {
                    if this.tab_strip.drag.is_some() {
                        this.commit_tab_drag(cx);
                        return;
                    }
                    false
                };
                if hovered_changed && !drag_changed {
                    cx.notify();
                }
            }));

        tabs_scroll_content = tabs_scroll_content.child(
            div()
                .id("tabs-left-padding-spacer")
                .flex_none()
                .w(px(TAB_HORIZONTAL_PADDING))
                .h(px(TABBAR_HEIGHT)),
        );
        for (index, tab) in self.tabs.iter().enumerate() {
            let switch_tab_index = index;
            let hover_tab_index = index;
            let close_tab_index = index;
            let is_active = index == self.active_tab;
            let is_hovered = self.tab_strip.hovered_tab == Some(index);
            let show_tab_close = Self::tab_shows_close(
                is_active,
                self.tab_strip.hovered_tab,
                self.tab_strip.hovered_tab_close,
                index,
            );
            let is_renaming = self.renaming_tab == Some(index);
            let tab_drop_marker_side = self.tab_drop_marker_side(index);
            let close_slot_width = if show_tab_close {
                TAB_CLOSE_SLOT_WIDTH
            } else {
                0.0
            };
            let label = Self::format_tab_label_for_render(
                &tab.title,
                Self::tab_title_char_budget(tab.display_width, close_slot_width),
            );
            let rename_text_color = if is_active {
                active_tab_text
            } else {
                inactive_tab_text
            };
            let mut rename_selection_color = colors.cursor;
            rename_selection_color.a = if is_active { 0.34 } else { 0.24 };

            let tab_bg = if is_active {
                active_tab_bg
            } else if is_hovered {
                hovered_tab_bg
            } else {
                inactive_tab_bg
            };
            let tab_strokes = tab_chrome_layout.tab_strokes[index];

            let mut close_text_color = if is_active {
                active_tab_text
            } else {
                inactive_tab_text
            };
            if !show_tab_close {
                close_text_color.a = 0.0;
            }

            let close_button = div()
                .w(px(close_slot_width))
                .h(px(TAB_CLOSE_HITBOX))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(5.0))
                .text_color(close_text_color)
                .text_size(px(12.0))
                .child("×")
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                        let is_active = close_tab_index == this.active_tab;
                        if Self::tab_shows_close(
                            is_active,
                            this.tab_strip.hovered_tab,
                            this.tab_strip.hovered_tab_close,
                            close_tab_index,
                        ) {
                            this.close_tab(close_tab_index, cx);
                            cx.stop_propagation();
                        }
                    }),
                )
                .on_mouse_move(
                    cx.listener(move |this, _event: &MouseMoveEvent, _window, cx| {
                        let mut hover_changed = false;
                        if this.tab_strip.hovered_tab != Some(hover_tab_index) {
                            this.tab_strip.hovered_tab = Some(hover_tab_index);
                            hover_changed = true;
                        }
                        if this.tab_strip.hovered_tab_close != Some(hover_tab_index) {
                            this.tab_strip.hovered_tab_close = Some(hover_tab_index);
                            hover_changed = true;
                        }
                        if hover_changed {
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }),
                )
                .hover(move |style| {
                    style
                        .bg(close_button_hover_bg)
                        .text_color(close_button_hover_text)
                })
                .cursor_pointer();

            let tab_shell = div()
                .flex_none()
                .relative()
                .bg(tab_bg)
                .w(px(tab.display_width))
                .h(px(TAB_ITEM_HEIGHT))
                .px(px(TAB_TEXT_PADDING_X))
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                        this.switch_tab(switch_tab_index, cx);
                        this.begin_tab_drag(switch_tab_index);
                        if event.click_count == 2 {
                            this.begin_rename_tab(switch_tab_index, cx);
                        }
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_move(
                    cx.listener(move |this, event: &MouseMoveEvent, window, cx| {
                        let mut hovered_changed =
                            if this.tab_strip.hovered_tab != Some(hover_tab_index) {
                                this.tab_strip.hovered_tab = Some(hover_tab_index);
                                true
                            } else {
                                false
                            };
                        if this.tab_strip.hovered_tab_close.take().is_some() {
                            hovered_changed = true;
                        }
                        let drag_changed = if event.dragging() {
                            let (pointer_x, viewport_width) =
                                this.tab_strip_pointer_x_from_window_x(window, event.position.x);
                            this.update_tab_drag_preview(pointer_x, viewport_width, cx)
                        } else {
                            false
                        };
                        if hovered_changed && !drag_changed {
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }),
                )
                .child(render_tab_stroke(tab_strokes.top))
                .children(tab_strokes.left_boundary.map(render_tab_stroke))
                .children(tab_strokes.right_boundary.map(render_tab_stroke));

            let drop_marker = tab_drop_marker_side.map(|side| {
                let marker_x = match side {
                    TabDropMarkerSide::Left => 0.0,
                    TabDropMarkerSide::Right => tab.display_width - TAB_DROP_MARKER_WIDTH,
                }
                .max(0.0);
                let marker_height = (TAB_ITEM_HEIGHT - (TAB_DROP_MARKER_INSET_Y * 2.0)).max(0.0);

                div()
                    .absolute()
                    .left(px(marker_x))
                    .top(px(TAB_DROP_MARKER_INSET_Y))
                    .w(px(TAB_DROP_MARKER_WIDTH))
                    .h(px(marker_height))
                    .bg(tab_drop_marker_color)
            });

            tabs_scroll_content = tabs_scroll_content.child(
                tab_shell
                    .child(div().flex_1().min_w(px(0.0)).h_full().relative().child(
                        if is_renaming {
                            self.render_inline_input_layer(
                                Font {
                                    family: font_family.clone(),
                                    weight: FontWeight::NORMAL,
                                    ..Default::default()
                                },
                                px(12.0),
                                rename_text_color.into(),
                                rename_selection_color.into(),
                                InlineInputAlignment::Left,
                                cx,
                            )
                        } else {
                            let title_text = div()
                                .size_full()
                                .flex()
                                .items_center()
                                .overflow_x_hidden()
                                .whitespace_nowrap()
                                .font_family(font_family.clone())
                                .text_color(rename_text_color)
                                .text_size(px(12.0))
                                .text_ellipsis();
                            title_text.child(label).into_any_element()
                        },
                    ))
                    .child(close_button)
                    .children(drop_marker),
            );
        }
        tabs_scroll_content = tabs_scroll_content.child(
            div()
                .id("tabs-right-padding-spacer")
                .flex_none()
                .w(px(TAB_HORIZONTAL_PADDING))
                .h(px(TABBAR_HEIGHT)),
        );

        for segment in &tab_chrome_layout.baseline_strokes {
            tabs_scroll_content = tabs_scroll_content.child(render_tab_stroke(*segment));
        }
        tabs_scroll_content = tabs_scroll_content.child(
            div()
                .id("tabs-baseline-tail-filler")
                .flex_1()
                .min_w(px(0.0))
                .h(px(TABBAR_HEIGHT))
                .relative()
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(px(tab_chrome_layout.baseline_y))
                        .h(px(TAB_STROKE_THICKNESS))
                        .bg(tab_stroke_color),
                ),
        );
        let tab_baseline_y = tab_chrome_layout.baseline_y;
        let left_overflow_indicator = tab_strip_overflow_state.left.then(|| {
            Self::render_overflow_indicator(
                "tabs-overflow-left",
                true,
                tab_overflow_fade_soft,
                tab_overflow_fade_mid,
                tab_overflow_fade_strong,
            )
        });
        let right_overflow_indicator = tab_strip_overflow_state.right.then(|| {
            Self::render_overflow_indicator(
                "tabs-overflow-right",
                false,
                tab_overflow_fade_soft,
                tab_overflow_fade_mid,
                tab_overflow_fade_strong,
            )
        });

        let tabs_row = div()
            .w_full()
            .h(px(TABBAR_HEIGHT))
            .flex()
            .children((tabbar_left_inset_width > 0.0).then(|| {
                div()
                    .id("tabbar-left-inset")
                    .relative()
                    .flex_none()
                    .w(px(tabbar_left_inset_width))
                    .h_full()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(tab_baseline_y))
                            .h(px(TAB_STROKE_THICKNESS))
                            .bg(tab_stroke_color),
                    )
                    .into_any_element()
            }))
            .child(
                div()
                    .id("tabs-scroll-viewport-lane")
                    .flex_none()
                    .w(px(tab_strip_viewport_width))
                    .min_w(px(0.0))
                    .h_full()
                    .relative()
                    .child(
                        div()
                            .id("tabs-scroll-viewport")
                            .absolute()
                            .left_0()
                            .top_0()
                            .right_0()
                            .bottom_0()
                            .overflow_x_scroll()
                            .track_scroll(&self.tab_strip.scroll_handle)
                            .child(tabs_scroll_content),
                    )
                    .children(left_overflow_indicator)
                    .children(right_overflow_indicator),
            )
            .children((tabbar_gutter_width > 0.0).then(|| {
                div()
                    .id("tabbar-action-gutter")
                    .relative()
                    .flex_none()
                    .w(px(tabbar_gutter_width))
                    .h_full()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(tab_baseline_y))
                            .h(px(TAB_STROKE_THICKNESS))
                            .bg(tab_stroke_color),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(((tabbar_gutter_width - TAB_STROKE_THICKNESS) * 0.5).max(0.0)))
                            .top(px(TAB_STROKE_THICKNESS))
                            .bottom_0()
                            .w(px(TAB_STROKE_THICKNESS))
                            .bg(tab_stroke_color),
                    )
                    .into_any_element()
            }))
            .children((tabbar_action_rail_width > 0.0).then(|| {
                div()
                    .id("tabbar-action-rail")
                    .relative()
                    .flex_none()
                    .w(px(tabbar_action_rail_width))
                    .h_full()
                    .on_scroll_wheel(cx.listener(Self::handle_tab_strip_action_rail_scroll_wheel))
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                        let hovered_changed = this.tab_strip.hovered_tab.take().is_some()
                            || this.tab_strip.hovered_tab_close.take().is_some();
                        if !event.dragging() {
                            if hovered_changed {
                                cx.notify();
                            }
                            return;
                        }

                        let (pointer_x, viewport_width) =
                            this.tab_strip_pointer_x_from_window_x(window, event.position.x);
                        if !this.update_tab_drag_preview(pointer_x, viewport_width, cx)
                            && hovered_changed
                        {
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .absolute()
                            .left(px(tabbar_new_tab_left))
                            .top(px(tabbar_new_tab_top))
                            .child(self.render_tabbar_new_tab_button(
                                tabbar_new_tab_bg,
                                tabbar_new_tab_hover_bg,
                                tabbar_new_tab_border,
                                tabbar_new_tab_hover_border,
                                tabbar_new_tab_text,
                                tabbar_new_tab_hover_text,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(tab_baseline_y))
                            .h(px(TAB_STROKE_THICKNESS))
                            .bg(tab_stroke_color),
                    )
                    .into_any_element()
            }))
            .children((tabbar_right_inset_width > 0.0).then(|| {
                div()
                    .id("tabbar-right-inset")
                    .relative()
                    .flex_none()
                    .w(px(tabbar_right_inset_width))
                    .h_full()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(tab_baseline_y))
                            .h(px(TAB_STROKE_THICKNESS))
                            .bg(tab_stroke_color),
                    )
                    .into_any_element()
            }))
            .into_any_element();

        tabs_row
    }

    fn render_overflow_indicator(
        id: &'static str,
        left_side: bool,
        soft: gpui::Rgba,
        mid: gpui::Rgba,
        strong: gpui::Rgba,
    ) -> AnyElement {
        let lane = if left_side {
            div()
                .id(id)
                .absolute()
                .top_0()
                .left_0()
                .bottom_0()
                .w(px(9.0))
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(strong),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(3.0))
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(mid),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(6.0))
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(soft),
                )
        } else {
            div()
                .id(id)
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .w(px(9.0))
                .child(
                    div()
                        .absolute()
                        .right_0()
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(strong),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(3.0))
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(mid),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(6.0))
                        .top_0()
                        .bottom_0()
                        .w(px(3.0))
                        .bg(soft),
                )
        };

        lane.into_any_element()
    }
}
