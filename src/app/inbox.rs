use super::*;
use crate::work::{FocusZone, InboxCollection, Quadrant, WorkflowStatus};

impl Waku {
    pub(super) fn list_pane_visible(&self) -> bool {
        !self.board_visible
    }

    pub(super) fn session_matches_collection(&self, session: &AgentSession) -> bool {
        self.inbox_collection
            .contains(session.workflow_status, session.flagged)
    }

    pub(super) fn show_unfinished_action(
        &mut self,
        _: &ShowUnfinished,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.inbox_collection = InboxCollection::Unfinished;
        self.board_visible = false;
        cx.notify();
    }

    pub(super) fn show_flagged_action(
        &mut self,
        _: &ShowFlagged,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.inbox_collection = InboxCollection::Flagged;
        self.board_visible = false;
        cx.notify();
    }

    pub(super) fn show_archive_action(
        &mut self,
        _: &ShowArchive,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.inbox_collection = InboxCollection::Archive;
        self.board_visible = false;
        cx.notify();
    }

    pub(super) fn show_board_action(
        &mut self,
        _: &ShowBoard,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.board_visible = true;
        self.inbox_collection = InboxCollection::Unfinished;
        cx.notify();
    }

    pub(super) fn focus_nav_zone_action(
        &mut self,
        _: &FocusNavZone,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_zone = FocusZone::Nav;
        cx.notify();
    }

    pub(super) fn focus_list_zone_action(
        &mut self,
        _: &FocusListZone,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_zone = FocusZone::List;
        self.board_visible = false;
        cx.notify();
    }

    pub(super) fn focus_detail_zone_action(
        &mut self,
        _: &FocusDetailZone,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_zone = FocusZone::Detail;
        let handle = self.composer.read(cx).focus();
        window.focus(&handle, cx);
        cx.notify();
    }

    pub(super) fn set_session_workflow(&mut self, session_id: Uuid, next: WorkflowStatus, cx: &mut Context<Self>) {
        if let Some(session) = self.state.session_mut(session_id) {
            session.workflow_status = next;
        }
        cx.notify();
    }

    pub(super) fn toggle_session_flag(&mut self, session_id: Uuid, cx: &mut Context<Self>) {
        if let Some(session) = self.state.session_mut(session_id) {
            session.flagged = !session.flagged;
        }
        cx.notify();
    }

    pub(super) fn set_session_quadrant(
        &mut self,
        session_id: Uuid,
        quadrant: Quadrant,
        cx: &mut Context<Self>,
    ) {
        if let Some(session) = self.state.session_mut(session_id) {
            session.important = quadrant.important;
            session.urgent = quadrant.urgent;
        }
        cx.notify();
    }

    pub(super) fn render_nav_rail(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let theme = Theme::current(cx);
        let focused = self.focus_zone == FocusZone::Nav;
        div()
            .w(px(INBOX_NAV_RAIL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(if cfg!(target_os = "macos") {
                TITLEBAR_HEIGHT
            } else {
                8.0
            }))
            .pb(px(10.0))
            .gap(px(4.0))
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.sidebar_border)
            .when(focused, |rail| rail.border_color(theme.accent))
            .when(self.board_visible, |rail| {
                rail.children(self.render_client_window_controls(
                    super::window_chrome::WindowControlSide::Left,
                    window,
                    cx,
                ))
            })
            .child(self.nav_button(
                "nav-unfinished",
                "icons/list.svg",
                tr!("inbox.unfinished"),
                !self.board_visible && self.inbox_collection == InboxCollection::Unfinished,
                cx.listener(|this, _, window, cx| {
                    this.show_unfinished_action(&ShowUnfinished, window, cx);
                }),
                cx,
            ))
            .child(self.nav_button(
                "nav-flagged",
                "icons/star.svg",
                tr!("inbox.flagged"),
                !self.board_visible && self.inbox_collection == InboxCollection::Flagged,
                cx.listener(|this, _, window, cx| {
                    this.show_flagged_action(&ShowFlagged, window, cx);
                }),
                cx,
            ))
            .child(self.nav_button(
                "nav-archive",
                "icons/package.svg",
                tr!("inbox.archive"),
                !self.board_visible && self.inbox_collection == InboxCollection::Archive,
                cx.listener(|this, _, window, cx| {
                    this.show_archive_action(&ShowArchive, window, cx);
                }),
                cx,
            ))
            .child(self.nav_button(
                "nav-board",
                "icons/hexagon.svg",
                tr!("inbox.board"),
                self.board_visible,
                cx.listener(|this, _, window, cx| {
                    this.show_board_action(&ShowBoard, window, cx);
                }),
                cx,
            ))
            .child(div().flex_1())
            .child(self.nav_button(
                "nav-settings",
                "icons/settings.svg",
                tr!("common.settings"),
                false,
                cx.listener(|this, _, window, cx| {
                    this.open_settings_action(&OpenSettings, window, cx);
                }),
                cx,
            ))
    }

    fn nav_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        label: String,
        current: bool,
        on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = Theme::current(cx);
        div()
            .id(id)
            .tab_index(0)
            .w(px(34.0))
            .h(px(34.0))
            .rounded(px(8.0))
            .flex()
            .items_center()
            .justify_center()
            .cursor_default()
            .tooltip(Tooltip::text(label))
            .when(current, |button| button.bg(theme.sidebar_item_background))
            .hover(|button| button.bg(theme.overlay))
            .focus_visible(|style| style.border_1().border_color(theme.accent))
            .child(icon(
                icon_path,
                16.0,
                if current {
                    theme.text
                } else {
                    theme.text_tertiary
                },
            ))
            .on_click(on_click)
    }

    pub(super) fn render_quadrant_board(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let theme = Theme::current(cx);
        div()
            .flex_1()
            .h_full()
            .min_w_0()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .child(
                div()
                    .id("board-titlebar")
                    .h(px(TITLEBAR_HEIGHT))
                    .flex_none()
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(theme.border)
                    .when(!self.sidebar_visible, |header| {
                        header.children(self.render_client_window_controls(
                            super::window_chrome::WindowControlSide::Left,
                            window,
                            cx,
                        ))
                    })
                    .child(self.window_drag_region(
                        div()
                            .id("board-traffic-light-drag-region")
                            .w(px(leftover_traffic_light_clearance(self.sidebar_visible)))
                            .h_full()
                            .flex_none(),
                        cx,
                    ))
                    .child(
                        div()
                            .pl(px(16.0))
                            .pr(px(16.0))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w_0()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(13.0))
                                    .flex_none()
                                    .child(tr!("inbox.board")),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(theme.text_tertiary)
                                    .min_w_0()
                                    .truncate()
                                    .child(format!(
                                        "{} → {} → {} · {}",
                                        tr!("workflow.todo"),
                                        tr!("workflow.in_progress"),
                                        tr!("workflow.in_review"),
                                        tr!("inbox.complete")
                                    )),
                            ),
                    )
                    .child(self.window_drag_region(
                        div().id("board-titlebar-drag-region").h_full().flex_1(),
                        cx,
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .p(px(10.0))
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .flex()
                                    .gap(px(8.0))
                                    .child(self.render_quadrant(Quadrant::DO_NOW, cx))
                                    .child(self.render_quadrant(Quadrant::SCHEDULE, cx)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .flex()
                                    .gap(px(8.0))
                                    .child(self.render_quadrant(Quadrant::DELEGATE, cx))
                                    .child(self.render_quadrant(Quadrant::LATER, cx)),
                            ),
                    ),
            )
    }

    fn render_quadrant(&self, quadrant: Quadrant, cx: &mut Context<Self>) -> Div {
        let theme = Theme::current(cx);
        let accent = match (quadrant.important, quadrant.urgent) {
            (true, true) => theme.accent,
            (true, false) => theme.success,
            (false, true) => theme.warning,
            (false, false) => theme.text_ghost,
        };
        div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(px(10.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.raised)
            .child(
                div()
                    .px(px(10.0))
                    .py(px(8.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_l_4()
                    .border_color(accent)
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(12.5))
                            .child(tr!(quadrant.label_key())),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_tertiary)
                            .child(tr!(quadrant.hint_key())),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .border_t_1()
                    .border_color(theme.border)
                    .children(WorkflowStatus::LIVE.iter().copied().enumerate().map(
                        |(index, status)| {
                            self.render_quadrant_lane(quadrant, status, index < 2, cx)
                        },
                    )),
            )
    }

    fn render_quadrant_lane(
        &self,
        quadrant: Quadrant,
        status: WorkflowStatus,
        with_border: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = Theme::current(cx);
        let cards: Vec<Uuid> = self
            .state
            .sessions
            .iter()
            .filter(|session| {
                session.has_started()
                    && session.workflow_status == status
                    && session.important == quadrant.important
                    && session.urgent == quadrant.urgent
            })
            .map(|session| session.id)
            .collect();
        let count = cards.len();
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .when(with_border, |lane| {
                lane.border_r_1().border_color(theme.border)
            })
            .child(
                div()
                    .px(px(8.0))
                    .pt(px(6.0))
                    .pb(px(4.0))
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .text_size(px(11.0))
                    .text_color(theme.text_tertiary)
                    .child(workflow_dot(status, &theme))
                    .child(tr!(status.label_key()))
                    .child(
                        div()
                            .text_color(theme.text_ghost)
                            .child(SharedString::from(count.to_string())),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .px(px(6.0))
                    .pb(px(6.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .children(cards.into_iter().map(|id| self.render_board_card(id, cx))),
            )
    }

    fn render_board_card(&self, session_id: Uuid, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::current(cx);
        let Some(session) = self
            .state
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return div().into_any_element();
        };
        let selected = self.state.selected_session == Some(session_id);
        let title = session.display_title().to_string();
        div()
            .id(SharedString::from(format!("board-card-{session_id}")))
            .w_full()
            .px(px(8.0))
            .py(px(7.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(if selected { theme.accent } else { theme.border })
            .bg(theme.surface)
            .cursor_default()
            .hover(|card| card.bg(theme.overlay))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.board_visible = false;
                this.select_session(session_id, cx);
            }))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_clamp(2)
                    .child(SharedString::from(title)),
            )
            .child(
                div()
                    .mt(px(6.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(11.0))
                    .text_color(theme.text_tertiary)
                    .when(session.flagged, |meta| {
                        meta.child(div().text_color(theme.favorite).child("★"))
                    })
                    .child(SharedString::from(session.provider.short_name().to_string())),
            )
            .into_any_element()
    }
}

fn workflow_dot(status: WorkflowStatus, theme: &Theme) -> Div {
    let color = match status {
        WorkflowStatus::Todo => theme.text_tertiary,
        WorkflowStatus::InProgress => theme.accent,
        WorkflowStatus::InReview => theme.gauge,
        WorkflowStatus::Done => theme.success,
    };
    div().w(px(7.0)).h(px(7.0)).rounded_full().bg(color)
}

pub(super) fn workflow_label_color(status: WorkflowStatus, theme: &Theme) -> Hsla {
    match status {
        WorkflowStatus::Todo => theme.text_tertiary,
        WorkflowStatus::InProgress => theme.accent,
        WorkflowStatus::InReview => theme.gauge,
        WorkflowStatus::Done => theme.success,
    }
}
