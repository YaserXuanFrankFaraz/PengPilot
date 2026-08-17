//! Media library UI, shaped after CodePilot Gallery:
//! adaptive column waterfall, image-only cards, click → detail (preview + meta),
//! right-click → reveal / delete. Tags and favorites stay out of v1.

use super::*;
use crate::library::LibraryAsset;
use chrono::{Local, TimeZone};

const LIBRARY_COLUMN_GAP: f32 = 12.0;
const LIBRARY_MIN_COLUMN_WIDTH: f32 = 220.0;

impl Waku {
    pub(super) fn open_library_page(&mut self, cx: &mut Context<Self>) {
        self.library_page = true;
        self.board_visible = false;
        self.settings_page = None;
        self.library_detail_id = None;
        self.refresh_library_assets(cx);
        cx.notify();
    }

    pub(super) fn close_library_page(&mut self, cx: &mut Context<Self>) {
        if self.library_page {
            self.library_page = false;
            self.library_detail_id = None;
            cx.notify();
        }
    }

    pub(super) fn refresh_library_assets(&mut self, cx: &mut Context<Self>) {
        match self.store.list_library_assets() {
            Ok(assets) => {
                if let Some(id) = self.library_detail_id.as_ref() {
                    if !assets.iter().any(|asset| &asset.id == id) {
                        self.library_detail_id = None;
                    }
                }
                self.library_assets = assets;
            }
            Err(error) => self.show_toast(format!("library load failed: {error}")),
        }
        cx.notify();
    }

    pub(super) fn open_library_detail(&mut self, id: String, cx: &mut Context<Self>) {
        self.library_detail_id = Some(id);
        cx.notify();
    }

    pub(super) fn close_library_detail(&mut self, cx: &mut Context<Self>) {
        if self.library_detail_id.take().is_some() {
            cx.notify();
        }
    }

    pub(super) fn toggle_library_sort(&mut self, cx: &mut Context<Self>) {
        self.library_sort_newest = !self.library_sort_newest;
        cx.notify();
    }

    pub(super) fn save_current_preview_to_library(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(preview) = self.image_preview.as_ref() else {
            return;
        };
        let Some(source) = preview.source_path.clone() else {
            self.show_toast(tr!("library.save_needs_file"));
            return;
        };
        let session = self.selected_session();
        let request = crate::library::SaveLibraryAsset {
            source_path: source,
            prompt: preview.prompt.clone(),
            session_id: session.map(|session| session.id.to_string()),
            provider: session.map(|session| session.provider.id().to_owned()),
            model: session.and_then(|session| session.model.clone()),
        };
        match self.store.save_library_asset(request) {
            Ok(_) => {
                self.show_success_toast(tr!("library.saved"));
                if self.library_page {
                    self.refresh_library_assets(cx);
                }
            }
            Err(error) => self.show_toast(format!("save failed: {error}")),
        }
        let _ = window;
        cx.notify();
    }

    pub(super) fn delete_library_asset(&mut self, id: String, cx: &mut Context<Self>) {
        if let Err(error) = self.store.delete_library_asset(&id) {
            self.show_toast(format!("delete failed: {error}"));
            return;
        }
        if self.library_detail_id.as_deref() == Some(id.as_str()) {
            self.library_detail_id = None;
        }
        self.refresh_library_assets(cx);
        self.show_success_toast(tr!("library.deleted"));
    }

    fn library_assets_sorted(&self) -> Vec<LibraryAsset> {
        let mut assets = self.library_assets.clone();
        if self.library_sort_newest {
            assets.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
        } else {
            assets.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        }
        assets
    }

    pub(super) fn render_library_page(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = Theme::current(cx);
        let assets = self.library_assets_sorted();
        let pane_width = f32::from(window.viewport_size().width)
            - self.sidebar_rendered_width
            - self.right_panel_rendered_width;
        div()
            .relative()
            .flex_1()
            .h_full()
            .min_w_0()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .child(self.render_library_toolbar(assets.len(), &theme, cx))
            .child(if assets.is_empty() {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(13.0))
                    .text_color(theme.text_tertiary)
                    .child(tr!("library.empty"))
                    .into_any_element()
            } else {
                self.render_library_waterfall(&assets, pane_width.max(1.0), &theme, cx)
            })
            .children(self.render_library_detail_overlay(&theme, cx))
            .into_any_element()
    }

    fn render_library_toolbar(
        &self,
        count: usize,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Div {
        let sort_label = if self.library_sort_newest {
            tr!("library.sort_newest")
        } else {
            tr!("library.sort_oldest")
        };
        div()
            .flex_none()
            .h(px(40.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(13.0))
                    .text_color(theme.text)
                    .child(tr!("library.title")),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.text_tertiary)
                    .child(tr!("library.count", count = count)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("library-sort-toggle")
                    .px(px(10.0))
                    .py(px(5.0))
                    .rounded(px(7.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .cursor_pointer()
                    .text_size(px(12.0))
                    .text_color(theme.text_secondary)
                    .hover(|style| style.bg(theme.overlay))
                    .child(icon("icons/chevrons-up-down.svg", 13.0, theme.text_tertiary))
                    .child(sort_label)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_library_sort(cx);
                    })),
            )
    }

    fn render_library_waterfall(
        &self,
        assets: &[LibraryAsset],
        pane_width: f32,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content_width = (pane_width - 32.0).max(LIBRARY_MIN_COLUMN_WIDTH);
        let column_count = ((content_width + LIBRARY_COLUMN_GAP)
            / (LIBRARY_MIN_COLUMN_WIDTH + LIBRARY_COLUMN_GAP))
            .floor()
            .max(1.0) as usize;
        let column_width =
            (content_width - LIBRARY_COLUMN_GAP * (column_count.saturating_sub(1) as f32))
                / column_count as f32;
        let root = self.store.library_root();
        let mut columns: Vec<Vec<&LibraryAsset>> = (0..column_count).map(|_| Vec::new()).collect();
        for (index, asset) in assets.iter().enumerate() {
            columns[index % column_count].push(asset);
        }

        let mut row = div()
            .id("library-waterfall")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(16.0))
            .flex()
            .items_start()
            .gap(px(LIBRARY_COLUMN_GAP));
        for (column_index, column) in columns.into_iter().enumerate() {
            let mut stack = div()
                .id(SharedString::from(format!("library-col-{column_index}")))
                .w(px(column_width))
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(LIBRARY_COLUMN_GAP));
            for asset in column {
                stack = stack.child(self.render_library_card(asset, &root, column_width, theme, cx));
            }
            row = row.child(stack);
        }
        row.into_any_element()
    }

    fn render_library_card(
        &self,
        asset: &LibraryAsset,
        root: &std::path::Path,
        width: f32,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let path = asset.path_in(root);
        let id = asset.id.clone();
        let detail_id = id.clone();
        let delete_id = id.clone();
        let reveal_path = path.clone();
        let waku = cx.entity().downgrade();
        let menu = self.menu_handle(format!("library-card-{id}"), cx);
        let thumb_height = width;

        context_menu(
            div()
                .id(SharedString::from(format!("library-card-{id}")))
                .w(px(width))
                .rounded(px(10.0))
                .overflow_hidden()
                .bg(theme.inset)
                .border_1()
                .border_color(theme.border)
                .cursor_pointer()
                .hover(|style| style.border_color(theme.accent))
                .child(
                    div()
                        .w(px(width))
                        .h(px(thumb_height))
                        .overflow_hidden()
                        .child(img(path).size_full().object_fit(ObjectFit::Cover)),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.open_library_detail(detail_id.clone(), cx);
                })),
            SharedString::from(format!("library-card-menu-{id}")),
            &menu,
            move |_| {
                let reveal = reveal_path.clone();
                let delete = delete_id.clone();
                let delete_waku = waku.clone();
                vec![
                    MenuItem::new(tr!("common.reveal_in_finder"), move |_, cx| {
                        crate::platform::reveal_in_file_manager(&reveal, cx);
                    })
                    .icon("icons/folder.svg"),
                    MenuItem::new(tr!("library.delete"), move |_, cx| {
                        let _ = delete_waku.update(cx, |this, cx| {
                            this.delete_library_asset(delete.clone(), cx);
                        });
                    })
                    .icon("icons/trash.svg"),
                ]
            },
        )
        .into_any_element()
    }

    fn render_library_detail_overlay(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let id = self.library_detail_id.as_ref()?;
        let asset = self.library_assets.iter().find(|asset| &asset.id == id)?;
        let root = self.store.library_root();
        let path = asset.path_in(&root);
        let reveal_path = path.clone();
        let delete_id = asset.id.clone();
        let prompt = asset
            .prompt
            .clone()
            .unwrap_or_else(|| asset.filename.clone());
        let created = format_library_timestamp(asset.created_at);
        let provider = asset.provider.clone().unwrap_or_default();
        let model = asset.model.clone().unwrap_or_default();

        let preview = div()
            .id("library-detail-preview")
            .flex_1()
            .h_full()
            .min_w_0()
            .bg(gpui::black())
            .flex()
            .items_center()
            .justify_center()
            .p(px(24.0))
            .child(
                img(path)
                    .max_w_full()
                    .max_h_full()
                    .object_fit(ObjectFit::Contain),
            );

        let mut meta = div()
            .id("library-detail-meta")
            .w(px(320.0))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(24.0))
            .bg(theme.surface)
            .border_l_1()
            .border_color(theme.border)
            .overflow_y_scroll()
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_tertiary)
                    .child(tr!("library.prompt")),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.text)
                    .child(prompt),
            );

        if !provider.is_empty() || !model.is_empty() {
            meta = meta.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.0))
                    .children((!provider.is_empty()).then(|| {
                        div()
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(theme.border)
                            .text_size(px(11.0))
                            .text_color(theme.text_secondary)
                            .child(provider)
                    }))
                    .children((!model.is_empty()).then(|| {
                        div()
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(theme.border)
                            .text_size(px(11.0))
                            .text_color(theme.text_secondary)
                            .child(model)
                    })),
            );
        }

        meta = meta
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.text_tertiary)
                    .child(format!("{} · {}", tr!("library.created"), created)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .id("library-detail-reveal")
                            .w_full()
                            .h(px(34.0))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(theme.border)
                            .text_size(px(12.0))
                            .text_color(theme.text_secondary)
                            .cursor_pointer()
                            .hover(|style| style.bg(theme.overlay))
                            .child(tr!("common.reveal_in_finder"))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                crate::platform::reveal_in_file_manager(&reveal_path, cx);
                            })),
                    )
                    .child(
                        div()
                            .id("library-detail-delete")
                            .w_full()
                            .h(px(34.0))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme.danger_soft)
                            .text_size(px(12.0))
                            .text_color(theme.danger)
                            .cursor_pointer()
                            .hover(|style| style.opacity(0.9))
                            .child(tr!("library.delete"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.delete_library_asset(delete_id.clone(), cx);
                            })),
                    ),
            );

        Some(
            div()
                .id("library-detail-overlay")
                .absolute()
                .inset_0()
                .occlude()
                .bg(gpui::hsla(0.0, 0.0, 0.0, 0.55))
                .flex()
                .items_center()
                .justify_center()
                .p(px(32.0))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.close_library_detail(cx);
                    }),
                )
                .child(
                    div()
                        .id("library-detail-card")
                        .w_full()
                        .h_full()
                        .max_w(px(1100.0))
                        .max_h(px(720.0))
                        .rounded(px(12.0))
                        .overflow_hidden()
                        .bg(theme.surface)
                        .flex()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(preview)
                        .child(meta)
                        .child(
                            div()
                                .id("library-detail-close")
                                .absolute()
                                .top(px(12.0))
                                .right(px(12.0))
                                .size(px(28.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(gpui::hsla(0.0, 0.0, 0.0, 0.45))
                                .cursor_pointer()
                                .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 0.0, 0.65)))
                                .child(icon("icons/x.svg", 12.0, gpui::white()))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.close_library_detail(cx);
                                    cx.stop_propagation();
                                })),
                        ),
                )
                .into_any_element(),
        )
    }
}

fn format_library_timestamp(created_at: u64) -> String {
    match Local.timestamp_opt(created_at as i64, 0) {
        chrono::LocalResult::Single(datetime) => datetime.format("%Y-%m-%d %H:%M").to_string(),
        _ => created_at.to_string(),
    }
}
