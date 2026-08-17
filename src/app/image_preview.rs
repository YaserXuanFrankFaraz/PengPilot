//! Window-modal preview for image attachments.
//!
//! Attachment tiles already render through GPUI's asynchronous image cache.
//! Opening a preview therefore only changes in-memory UI state; the frame path
//! never probes the filesystem. The same path also powers the file-manager action.

use gpui::{KeyBinding, actions};

use super::*;

actions!(waku_image_preview, [DismissImagePreview]);

const IMAGE_PREVIEW_CONTEXT: &str = "ImagePreview";
const IMAGE_PREVIEW_ANIMATION_DURATION: Duration = Duration::from_millis(140);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "escape",
        DismissImagePreview,
        Some(IMAGE_PREVIEW_CONTEXT),
    )]);
}

pub(super) struct ImagePreviewState {
    image: Arc<gpui::Image>,
    name: SharedString,
    /// Absolute path when the preview came from a file (Imagine / library).
    /// Needed to copy into the asset library; `None` for in-memory blobs.
    pub(super) source_path: Option<PathBuf>,
    pub(super) prompt: Option<String>,
    focus: FocusHandle,
    close_focus: FocusHandle,
    previous_focus: Option<FocusHandle>,
    generation: u64,
}

pub(super) fn image_format_for_name(name: &str) -> Option<gpui::ImageFormat> {
    let extension = Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())?
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some(gpui::ImageFormat::Png),
        "jpg" | "jpeg" => Some(gpui::ImageFormat::Jpeg),
        "webp" => Some(gpui::ImageFormat::Webp),
        "gif" => Some(gpui::ImageFormat::Gif),
        "svg" => Some(gpui::ImageFormat::Svg),
        "bmp" => Some(gpui::ImageFormat::Bmp),
        "tif" | "tiff" => Some(gpui::ImageFormat::Tiff),
        "ico" => Some(gpui::ImageFormat::Ico),
        "pnm" | "pbm" | "pgm" | "ppm" => Some(gpui::ImageFormat::Pnm),
        _ => None,
    }
}

pub(super) fn attachment_menu_items(path: PathBuf, _can_reveal: bool) -> Vec<MenuItem> {
    vec![
        MenuItem::new(tr!("common.reveal_in_finder"), move |_, cx| {
            crate::platform::reveal_in_file_manager(&path, cx);
        })
        .icon("icons/folder.svg"),
    ]
}

fn preview_image_path(url: &str) -> Option<PathBuf> {
    if let Some(path) = crate::blob_store::shared_path_for(url) {
        return path.is_file().then_some(path);
    }
    let path = Path::new(url);
    path.is_absolute()
        .then(|| path.to_path_buf())
        .filter(|path| path.is_file())
}

fn load_preview_image(url: &str) -> Option<(Arc<gpui::Image>, SharedString)> {
    if let Some(decoded) = crate::md::render::decode_data_url(url) {
        return Some((decoded, SharedString::from("image")));
    }
    let path = preview_image_path(url)?;
    let format = image_format_for_name(path.to_str()?)?;
    let bytes = std::fs::read(&path).ok()?;
    (!bytes.is_empty()).then(|| {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image");
        (
            Arc::new(gpui::Image::from_bytes(format, bytes)),
            SharedString::from(name.to_owned()),
        )
    })
}

impl Waku {
    pub(super) fn open_image_preview(
        &mut self,
        image: Arc<gpui::Image>,
        name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_image_preview_with_source(image, name, None, None, window, cx);
    }

    pub(super) fn open_image_preview_with_source(
        &mut self,
        image: Arc<gpui::Image>,
        name: SharedString,
        source_path: Option<PathBuf>,
        prompt: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.image_preview_generation = self.image_preview_generation.wrapping_add(1);
        let generation = self.image_preview_generation;
        let focus = cx.focus_handle();
        self.image_preview = Some(ImagePreviewState {
            image,
            name,
            source_path,
            prompt,
            focus: focus.clone(),
            close_focus: cx.focus_handle(),
            previous_focus: window.focused(cx),
            generation,
        });

        // The preview is deferred onto GPUI's overlay plane. Wait until that
        // subtree has joined the dispatch tree before focusing it, so Escape
        // is reliable on the first key press.
        let weak = cx.entity().downgrade();
        window.on_next_frame(move |window, _| {
            window.on_next_frame(move |window, cx| {
                let mut should_focus = false;
                let _ = weak.update(cx, |this, _| {
                    should_focus = this
                        .image_preview
                        .as_ref()
                        .is_some_and(|preview| preview.generation == generation);
                });
                if should_focus {
                    window.focus(&focus, cx);
                }
            });
        });
        cx.notify();
    }

    /// Open a transcript image URL in the modal preview. Accepts `data:`,
    /// `waku-blob:`, and absolute filesystem paths (Grok Imagine output).
    pub(super) fn open_image_url(
        &mut self,
        url: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((image, name)) = load_preview_image(url) else {
            if let Some(path) = preview_image_path(url) {
                crate::platform::reveal_in_file_manager(&path, cx);
            }
            return;
        };
        self.open_image_preview_with_source(
            image,
            name,
            preview_image_path(url),
            None,
            window,
            cx,
        );
    }

    fn close_image_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = self.image_preview.take() else {
            return;
        };
        self.image_preview_generation = self.image_preview_generation.wrapping_add(1);
        if let Some(previous_focus) = preview.previous_focus {
            window.focus(&previous_focus, cx);
        } else {
            let focus = self.composer_focus(cx);
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    pub(super) fn render_image_preview(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let preview = self.image_preview.as_ref()?;
        let theme = Theme::current(cx);
        let image_source = preview.image.clone();
        let name = preview.name.clone();
        let can_save = preview.source_path.is_some();
        let already_saved = preview
            .source_path
            .as_ref()
            .and_then(|path| self.store.library_asset_saved_for_source(path).ok().flatten())
            .is_some();
        let focus = preview.focus.clone();
        let close_focus = preview.close_focus.clone();
        let generation = preview.generation;

        let close = div()
            .id("image-preview-close")
            .absolute()
            .top(px(14.0))
            .right(px(14.0))
            .track_focus(&close_focus)
            .tab_index(0)
            .size(px(32.0))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .cursor_default()
            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.48))
            .focus_visible(|style| style.border_1().border_color(theme.accent))
            .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 0.0, 0.66)))
            .active(|style| style.opacity(0.8))
            .tooltip(Tooltip::text(tr!("attachments.close_preview")))
            .child(icon("icons/x.svg", 13.0, gpui::white()))
            .on_click(cx.listener(|this, _, window, cx| {
                this.close_image_preview(window, cx);
                cx.stop_propagation();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    this.close_image_preview(window, cx);
                    cx.stop_propagation();
                }
            }));

        let unavailable_color = gpui::white().opacity(0.78);
        let image = div()
            .size_full()
            .min_h_0()
            .flex()
            .items_center()
            .justify_center()
            .cursor_default()
            .child(
                img(image_source)
                    .size_full()
                    .object_fit(ObjectFit::Contain)
                    .with_fallback(move || {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(8.0))
                            .text_size(px(12.0))
                            .text_color(unavailable_color)
                            .child(icon("icons/alert.svg", 18.0, unavailable_color))
                            .child(tr_cow!("attachments.preview_unavailable"))
                            .into_any_element()
                    }),
            );
        let content = div()
            .relative()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(div().w_full().flex_1().min_h_0().child(image))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .max_w(px(420.0))
                            .px(px(11.0))
                            .py(px(5.0))
                            .rounded_full()
                            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.48))
                            .text_size(px(11.5))
                            .text_color(gpui::white().opacity(0.9))
                            .truncate()
                            .child(name),
                    )
                    .when(can_save, |row| {
                        row.child(
                            div()
                                .id("image-preview-save-library")
                                .px(px(12.0))
                                .py(px(5.0))
                                .rounded_full()
                                .bg(if already_saved {
                                    gpui::hsla(0.0, 0.0, 0.0, 0.32)
                                } else {
                                    theme.accent
                                })
                                .text_size(px(11.5))
                                .text_color(gpui::white())
                                .cursor_pointer()
                                .hover(|style| style.opacity(0.88))
                                .child(if already_saved {
                                    tr!("library.already_saved")
                                } else {
                                    tr!("library.save")
                                })
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save_current_preview_to_library(window, cx);
                                    cx.stop_propagation();
                                })),
                        )
                    }),
            )
            .child(close);

        let layer = div()
            .id(SharedString::from(format!(
                "image-preview-layer-{generation}"
            )))
            .absolute()
            .inset_0()
            .occlude()
            .track_focus(&focus)
            .key_context(IMAGE_PREVIEW_CONTEXT)
            .on_action(cx.listener(|this, _: &DismissImagePreview, window, cx| {
                this.close_image_preview(window, cx);
            }))
            .tab_group()
            .tab_stop(false)
            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.82))
            .p(px(36.0))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.close_image_preview(window, cx);
                }),
            )
            .child(content)
            .with_animation(
                SharedString::from(format!("image-preview-enter-{generation}")),
                Animation::new(IMAGE_PREVIEW_ANIMATION_DURATION).with_easing(ease_out_quint()),
                |element, delta| element.opacity(delta),
            );

        Some(gpui::deferred(layer).with_priority(5).into_any_element())
    }
}
