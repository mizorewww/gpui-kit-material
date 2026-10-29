//! The Gallery shell: maps list rows to pages, frames each row as a slice of
//! its page's panel, and owns navigation and keyboard traversal across
//! virtual rows. The shared GPUI list measures only the viewport; Kit
//! Component List requires precomputed sizes, while MessageScroller imposes
//! log/tail semantics inappropriate for this page.
use super::*;

impl Gallery {
    /// The page holding list row `row`, and the row's index in that page
    /// (0 is the header).
    pub(super) fn locate(&self, row: usize) -> (usize, usize) {
        let page = self.headers.partition_point(|header| *header <= row) - 1;
        (page, row - self.headers[page])
    }

    pub(super) fn row_count(&self) -> usize {
        self.row_focus.len()
    }

    /// Splice the list where a page's body row count changed (e.g. a
    /// window-width-dependent grid), keeping every other row's measurement.
    pub(super) fn sync_row_counts(&mut self, window: &Window, cx: &mut Context<Self>) {
        for index in 0..self.pages.len() {
            let count = self.pages[index].row_count(window, cx);
            let previous = self.counts[index];
            if count == previous {
                continue;
            }
            let start = self.headers[index] + 1;
            let top = self.list.logical_scroll_top();
            let handles: Vec<_> = (0..count).map(|_| cx.focus_handle()).collect();
            self.row_focus
                .splice(start..start + previous, handles.iter().cloned());
            self.list
                .splice_focusable(start..start + previous, handles.into_iter().map(Some));
            self.counts[index] = count;
            for later in index + 1..self.pages.len() {
                self.headers[later] = self.headers[later] + count - previous;
                self.page_rows[later].set_header(self.headers[later]);
            }
            // Keep the reader on the same content when rows above change.
            if top.item_ix >= start + previous {
                self.list.scroll_to(ListOffset {
                    item_ix: top.item_ix + count - previous,
                    offset_in_item: top.offset_in_item,
                });
            } else if top.item_ix >= start {
                self.list.scroll_to(ListOffset {
                    item_ix: start + (top.item_ix - start).min(count.saturating_sub(1)),
                    offset_in_item: top.offset_in_item,
                });
            }
        }
    }

    /// Let pages forward controlled changes to playbacks of unmounted rows.
    pub(super) fn after_layout(&self, rendered: &[usize], cx: &mut App) {
        // Run after list prepaint, when measurements are current. Fast can
        // carry visible rows without calling the builder; current intersection
        // protects those too. Builder membership includes focused offscreen rows.
        let viewport = self.list.viewport_bounds();
        for (index, page) in self.pages.iter().enumerate() {
            let rows = &self.page_rows[index];
            let visible = |row: usize| {
                let global = rows.global(row);
                rendered.contains(&global)
                    || self.list.bounds_for_item(global).is_some_and(|bounds| {
                        let overlap = bounds.intersect(&viewport);
                        overlap.size.width > px(0.) && overlap.size.height > px(0.)
                    })
            };
            page.after_layout(&visible, cx);
        }
    }

    pub(super) fn interactive_row(&self, row: usize, cx: &App) -> bool {
        let (page, local) = self.locate(row);
        if local == 0 {
            self.pages[page].info().guideline.is_some()
        } else {
            self.pages[page].interactive(local - 1, cx)
        }
    }

    fn adjacent_interactive_row(&self, row: usize, reverse: bool, cx: &App) -> Option<usize> {
        if reverse {
            (0..row)
                .rev()
                .find(|index| self.interactive_row(*index, cx))
        } else {
            (row + 1..self.row_count()).find(|index| self.interactive_row(*index, cx))
        }
    }

    pub(super) fn scroll_to_section(&self, page: usize) {
        self.list.scroll_to(ListOffset {
            item_ix: self.headers[page],
            offset_in_item: px(0.),
        });
    }

    pub(super) fn render_navigation(&self, compact: bool, cx: &mut Context<Self>) -> Stateful<Div> {
        div()
            .id("gallery-navigation")
            .role(Role::Navigation)
            .aria_label("Gallery pages")
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.navigation_scroll)
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .when(compact, |nav| nav.w_full().max_h(rem(160.)).border_b_1())
            .when(!compact, |nav| nav.w(rem(176.)).h_full().border_r_1())
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                gpui_base::v_flex()
                    .p(rem(12.))
                    .pt(rem(32.))
                    .gap(rem(12.))
                    .child(
                        accessible_text("gallery-navigation-title", "Browse components")
                            .text_size(TEXT_AUX),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(4.))
                            .when(compact, |items| items.flex_row().flex_wrap())
                            .children(pages::PAGES.iter().enumerate().map(|(index, entry)| {
                                MaterialButton::new(("navigate", index))
                                    .variant(ButtonVariant::Tonal)
                                    .button_size(ButtonSize::ExtraSmall)
                                    .toggle(true)
                                    .selected(self.selected_section == index)
                                    .toggle_on_click(false)
                                    .child(entry.info.title)
                                    .when(!compact, |button| button.w_full().justify_start())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.selected_section = index;
                                        this.scroll_to_section(index);
                                        cx.notify();
                                    }))
                                    .reveal_handle(
                                        ("reveal-navigation", index),
                                        &self.navigation_scroll,
                                    )
                            })),
                    ),
            )
    }

    pub(super) fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (page, local) = self.locate(index);
        let first = local == 0;
        let last = local == self.counts[page];
        let (content, gap) = if first {
            (
                spec::page_header(self.pages[page].info()).into_any_element(),
                0.,
            )
        } else {
            (
                self.pages[page].render_row(local - 1, window, cx),
                self.pages[page].row_gap(local - 1, cx),
            )
        };
        let compact = is_compact(window);
        let padding = if compact {
            spec::PANEL_PADDING_COMPACT
        } else {
            spec::PANEL_PADDING
        };
        let page_gap = if compact {
            spec::PAGE_GAP_COMPACT
        } else {
            spec::PAGE_GAP
        };
        let radius = rem(spec::PANEL_RADIUS);
        let row = div()
            .id(("gallery-row", index))
            .when(index == 0, |row| {
                row.debug_selector(|| "gallery-first-row".into())
            })
            .track_focus(&self.row_focus[index])
            .tab_stop(false)
            .w_full()
            .max_w(rem(spec::PAGE_MAX_WIDTH))
            .px(rem(page_gap))
            .when(first, |row| row.pt(rem(page_gap)))
            .when(last, |row| row.pb(rem(page_gap)))
            .child(
                div()
                    .when(first, |panel| {
                        let id = self.pages[page].info().id;
                        panel.debug_selector(move || format!("page-{id}"))
                    })
                    .w_full()
                    .bg(spec::panel_background())
                    .px(rem(padding))
                    .pt(rem(gap))
                    .when(first, |panel| panel.rounded_t(radius).overflow_hidden())
                    .when(last, |panel| panel.rounded_b(radius).pb(rem(padding)))
                    .child(content),
            );
        // List item roots are positioned directly at the list's left edge.
        // A real flex parent centers the panel.
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(row)
            .into_any_element()
    }

    // The public traversal API exposes the rendered tab order, not candidate
    // enumeration. Scan at most one cycle, as Base Root does for focus traps.
    pub(super) fn focus_row(
        &mut self,
        row: usize,
        reverse: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = &self.row_focus[row];
        let original = window.focused(cx);
        let mut visited = Vec::new();
        loop {
            if reverse {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            if target.contains_focused(window, cx) {
                return;
            }
            let current = window.focused(cx);
            if visited.contains(&current) {
                break;
            }
            visited.push(current);
        }
        // A controlled row may have become fully disabled. Continue through the
        // logical catalog instead of trapping focus on an empty row container.
        if let Some(original) = original {
            original.focus(window, cx);
        }
        if let Some(next) = self.adjacent_interactive_row(row, reverse, cx) {
            self.list.scroll_to(ListOffset {
                item_ix: next,
                offset_in_item: px(0.),
            });
            self.pending_focus = Some((next, reverse));
            cx.notify();
        }
    }

    pub(super) fn advance_gallery_focus(
        &mut self,
        reverse: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if gpui_base::active_focus_trap(window, cx).is_some() {
            cx.propagate();
            return;
        }
        let previous = window.focused(cx);
        let current_row = self
            .row_focus
            .iter()
            .position(|focus| focus.contains_focused(window, cx));
        let from_navigation = self.navigation_focus.contains_focused(window, cx);
        if current_row.is_none() && !from_navigation {
            cx.propagate();
            return;
        }
        if reverse {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        if let Some(row) = current_row {
            if self.row_focus[row].contains_focused(window, cx) {
                return;
            }
        } else if self.navigation_focus.contains_focused(window, cx) {
            return;
        }
        let target = match current_row {
            Some(row) => self.adjacent_interactive_row(row, reverse, cx),
            None if reverse => (0..self.row_count())
                .rev()
                .find(|row| self.interactive_row(*row, cx)),
            None => (0..self.row_count()).find(|row| self.interactive_row(*row, cx)),
        };
        if let Some(target) = target {
            if let Some(previous) = previous {
                previous.focus(window, cx);
            }
            self.list.scroll_to(ListOffset {
                item_ix: target,
                offset_in_item: px(0.),
            });
            self.pending_focus = Some((target, reverse));
            cx.notify();
        }
    }
}
