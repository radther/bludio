//! Self-contained, single-line text input component.
//!
//! Drop-in entity that handles all keyboard and IME input internally.
//! Emits `TextFieldEvent`s via `cx.emit()` (`EventEmitter` pattern).
//!
//! Based on gpui's `input.rs` example and Zed's `ui_input`/`editor` crates.

use crate::ui::h_flex;
use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Entity, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    Pixels, Point, Render, RenderOnce, ShapedLine, SharedString, TextAlign, TextRun, Window,
    canvas, point, prelude::*, px, size,
};
use std::ops::Range;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// ── Constants ──────────────────────────────────────────────────────────────

/// Interval between cursor blink toggles, matches Zed's `CURSOR_BLINK_INTERVAL`.
const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the `TextField` for parent views to handle.
#[derive(Clone, Debug)]
pub(crate) enum TextFieldEvent {
    /// User pressed Enter. Contains the current text content.
    Confirmed(String),
    /// User pressed Escape.
    Cancelled,
}

// ── Entity ─────────────────────────────────────────────────────────────────

/// A self-contained, single-line text input field.
///
/// Handles all keyboard input, cursor movement, backspace/delete, IME
/// text input, selection, copy/paste/cut, and blinking cursor internally.
/// Emits `TextFieldEvent`s for Enter and Escape.
pub(crate) struct TextField {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    /// Optional character filter applied to all text input.
    filter_char: Option<Box<dyn Fn(char) -> bool>>,
    // ── Blink state ──────────────────────────────────────────────────────
    /// Whether the cursor is currently visible (toggled by blink timer).
    blink_visible: bool,
    /// Epoch counter to cancel stale blink timers when blinking is paused.
    blink_epoch: usize,
    /// Whether the text field was focused in the previous frame (for edge detection).
    was_focused: bool,
    /// Text alignment within the field.
    align: TextAlign,
}

impl TextField {
    /// Create a new empty text field.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: SharedString::default(),
            placeholder: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            filter_char: None,
            blink_visible: true,
            blink_epoch: 0,
            was_focused: false,
            align: TextAlign::Left,
        }
    }

    /// Set placeholder text displayed when the field is empty.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set a character filter: only characters passing the predicate are accepted.
    /// Use this to restrict input to digits, alphanumeric, etc.
    pub fn filter_char(mut self, filter: impl Fn(char) -> bool + 'static) -> Self {
        self.filter_char = Some(Box::new(filter));
        self
    }

    /// Set the text alignment within the field. Default: Left.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Get the current text content.
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Replace the entire content and reset selection.
    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.show_cursor_and_reset_blink(cx);
    }

    // ── Cursor / Selection helpers ──────────────────────────────────────

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    /// Returns the word range surrounding the given byte offset.
    /// Words are continuous runs of alphanumeric characters.
    fn word_range_at(&self, offset: usize) -> Range<usize> {
        if self.content.is_empty() {
            return 0..0;
        }
        let len = self.content.len();
        let clamped = offset.min(len);
        // Find start of word
        let start = self.content[..clamped]
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_alphanumeric())
            .map_or(0, |(i, _)| i + 1);
        // Find end of word
        let end = self.content[clamped..]
            .char_indices()
            .find(|(_, c)| !c.is_alphanumeric())
            .map_or(len, |(i, _)| clamped + i);
        start..end
    }

    // ── UTF-16 conversion helpers (for EntityInputHandler) ──────────────

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    // ── Blink management ────────────────────────────────────────────────

    /// Start the cursor blink cycle. Called when the text field gains focus.
    fn start_blinking(&mut self, cx: &mut Context<Self>) {
        self.blink_visible = false;
        self.blink_cursors(0, cx);
    }

    /// Show the cursor immediately and reset the blink timer.
    /// Called on every user interaction (keystroke, mouse click, etc.).
    fn show_cursor_and_reset_blink(&mut self, cx: &mut Context<Self>) {
        self.blink_visible = true;
        // Bump the epoch so any in-flight blink timer becomes a no-op,
        // then restart blinking after a pause.
        self.blink_epoch += 1;
        let epoch = self.blink_epoch;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CURSOR_BLINK_INTERVAL).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.resume_blinking(epoch, cx));
            }
        })
        .detach();
    }

    /// Resume blinking after the pause (only if epoch still matches).
    fn resume_blinking(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if epoch == self.blink_epoch {
            self.blink_visible = false;
            self.blink_cursors(epoch, cx);
        }
    }

    /// Toggle visibility and schedule the next toggle.
    fn blink_cursors(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if epoch == self.blink_epoch {
            self.blink_visible = !self.blink_visible;
            cx.notify();

            let next_epoch = epoch + 1;
            self.blink_epoch = next_epoch;
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(CURSOR_BLINK_INTERVAL).await;
                if let Some(this) = this.upgrade() {
                    this.update(cx, |this, cx| {
                        this.blink_cursors(next_epoch, cx);
                    });
                }
            })
            .detach();
        }
    }

    // ── Key handlers ────────────────────────────────────────────────────

    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, cx);
        }
        self.replace_text_in_range_impl(None, "");
        self.show_cursor_and_reset_blink(cx);
    }

    fn handle_delete(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range_impl(None, "");
        self.show_cursor_and_reset_blink(cx);
    }

    fn handle_left(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn handle_right(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn handle_select_left(&mut self, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn handle_select_right(&mut self, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn handle_home(&mut self, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn handle_end(&mut self, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn handle_select_all(&mut self, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn handle_copy(&self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            return;
        }
        let start = self.selected_range.start.min(self.selected_range.end);
        let end = self.selected_range.start.max(self.selected_range.end);
        let text = self.content[start..end].to_string();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    fn handle_paste(&mut self, cx: &mut Context<Self>) {
        let Some(item) = cx.read_from_clipboard() else {
            return;
        };
        let Some(text) = item.text() else {
            return;
        };
        // Apply character filter if set
        let filtered: String = if let Some(ref filter) = self.filter_char {
            text.chars().filter(|c| filter(*c)).collect()
        } else {
            text
        };
        if filtered.is_empty() {
            return;
        }
        self.replace_text_in_range_impl(None, &filtered);
        self.show_cursor_and_reset_blink(cx);
    }

    fn handle_cut(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            return;
        }
        self.handle_copy(cx);
        self.replace_text_in_range_impl(None, "");
        self.show_cursor_and_reset_blink(cx);
    }

    // ── Mouse handlers ──────────────────────────────────────────────────

    fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        self.is_selecting = true;
        // Keep cursor visible and reset blink timer on mouse interaction.
        self.show_cursor_and_reset_blink(cx);
        let click_pos = self.index_for_mouse_position(event.position);

        match event.click_count {
            1 => {
                // Single click: move cursor (with shift: extend selection)
                if event.modifiers.shift {
                    self.select_to(click_pos, cx);
                } else {
                    self.move_to(click_pos, cx);
                }
            }
            2 => {
                // Double click: select word
                let word_range = self.word_range_at(click_pos);
                self.selected_range = word_range;
                self.selection_reversed = false;
                cx.notify();
            }
            _ => {
                // Triple+ click: select all
                self.handle_select_all(cx);
            }
        }
    }

    fn on_mouse_up(&mut self, _cx: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    // ── Internal: text replacement without notifying ────────────────────

    fn replace_text_in_range_impl(&mut self, range_utf16: Option<Range<usize>>, new_text: &str) {
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .unwrap_or(self.selected_range.clone());

        // Apply character filter if set.
        let filtered: String = if let Some(ref filter) = self.filter_char {
            new_text.chars().filter(|c| filter(*c)).collect()
        } else {
            new_text.to_string()
        };

        self.content =
            (self.content[0..range.start].to_owned() + &filtered + &self.content[range.end..])
                .into();
        self.selected_range = range.start + filtered.len()..range.start + filtered.len();
        self.selection_reversed = false;
    }
}

// ── EntityInputHandler (IME / text input) ──────────────────────────────────

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::UTF16Selection> {
        Some(gpui::UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range_impl(range_utf16, new_text);
        self.show_cursor_and_reset_blink(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // For single-line input, we don't need to visually mark IME ranges.
        // Just replace the text.
        self.replace_text_in_range_impl(range_utf16, new_text);
        self.show_cursor_and_reset_blink(cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        pt: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&pt)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(pt.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

// ── Focusable ──────────────────────────────────────────────────────────────

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _app: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ── TextFieldComponent (RenderOnce element) ───────────────────────────────

/// The visual representation of a `TextField`.
///
/// Handles text layout, selection/cursor painting, IME input, and keyboard/mouse
/// event dispatch via `window.listener_for`. Text is rendered via `ShapedLine`
/// through a canvas, with selection and cursor as paint quads.
#[derive(IntoElement)]
struct TextFieldComponent {
    entity: Entity<TextField>,
}

impl TextFieldComponent {
    fn new(entity: Entity<TextField>) -> Self {
        Self { entity }
    }
}

impl RenderOnce for TextFieldComponent {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity = self.entity.clone();
        let focus_handle = entity.read(cx).focus_handle.clone();

        let entity_for_key = entity.clone();
        let entity_for_mouse = entity.clone();
        let entity_for_paint = entity.clone();

        h_flex()
            .track_focus(&focus_handle)
            .cursor(CursorStyle::IBeam)
            .justify_center()
            .flex_grow()
            .px_1()
            .min_h(px(20.))
            .relative()
            .on_key_down(window.listener_for(
                &entity_for_key,
                move |this, event: &KeyDownEvent, _window, cx| {
                    let key = event.keystroke.key.clone();
                    let modifiers = event.keystroke.modifiers;
                    let shortcut_mod = modifiers.control || modifiers.platform;

                    match key.as_str() {
                        "enter" => {
                            let text = this.content.to_string();
                            this.content = SharedString::default();
                            this.selected_range = 0..0;
                            this.selection_reversed = false;
                            this.blink_visible = true;
                            cx.emit(TextFieldEvent::Confirmed(text));
                            cx.notify();
                        }
                        "escape" => {
                            this.content = SharedString::default();
                            this.selected_range = 0..0;
                            this.selection_reversed = false;
                            this.blink_visible = true;
                            cx.emit(TextFieldEvent::Cancelled);
                            cx.notify();
                        }
                        "backspace" => this.handle_backspace(cx),
                        "delete" => this.handle_delete(cx),
                        "left" => {
                            if modifiers.shift {
                                this.handle_select_left(cx);
                            } else {
                                this.handle_left(cx);
                            }
                        }
                        "right" => {
                            if modifiers.shift {
                                this.handle_select_right(cx);
                            } else {
                                this.handle_right(cx);
                            }
                        }
                        "home" => {
                            if modifiers.shift {
                                this.select_to(0, cx);
                            } else {
                                this.handle_home(cx);
                            }
                        }
                        "end" => {
                            if modifiers.shift {
                                this.select_to(this.content.len(), cx);
                            } else {
                                this.handle_end(cx);
                            }
                        }
                        "a" if shortcut_mod => {
                            this.handle_select_all(cx);
                        }
                        "c" if shortcut_mod => {
                            this.handle_copy(cx);
                        }
                        "v" if shortcut_mod => {
                            this.handle_paste(cx);
                        }
                        "x" if shortcut_mod => {
                            this.handle_cut(cx);
                        }
                        _ => {}
                    }
                },
            ))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(
                    &entity_for_mouse,
                    |this, event: &MouseDownEvent, _window, cx| {
                        this.on_mouse_down(event, cx);
                    },
                ),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity_for_mouse, |this, _event, _window, cx| {
                    this.on_mouse_up(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                window.listener_for(&entity_for_mouse, |this, _event, _window, cx| {
                    this.on_mouse_up(cx);
                }),
            )
            .on_mouse_move(window.listener_for(
                &entity_for_mouse,
                |this, event: &MouseMoveEvent, _window, cx| {
                    this.on_mouse_move(event, cx);
                },
            ))
            .child(
                // Canvas handles: text painting, selection/cursor overlays, IME input
                canvas(
                    {
                        let entity = entity_for_paint.clone();
                        move |bounds, window, cx| {
                            let input = entity.read(cx);
                            let content = input.content.clone();
                            let is_empty = content.is_empty();
                            let selected_range = input.selected_range.clone();
                            let cursor = input.cursor_offset();
                            let align = input.align;
                            let style = window.text_style();
                            let theme = crate::ui::theme::theme(cx);
                            let colors = &theme.colors;

                            let (display_text, text_color) = if content.is_empty() {
                                (input.placeholder.clone(), colors.text_placeholder)
                            } else {
                                (content, style.color)
                            };

                            let run = TextRun {
                                len: display_text.len(),
                                font: style.font(),
                                color: text_color,
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            };
                            let runs = vec![run];

                            let font_size = style.font_size.to_pixels(window.rem_size());
                            let line = window.text_system().shape_line(
                                display_text,
                                font_size,
                                &runs,
                                None,
                            );

                            let cursor_pos = line.x_for_index(cursor);
                            let focused = input.focus_handle.is_focused(window);

                            // Horizontal offset for text alignment.
                            // When the field is empty, center the cursor in the field
                            // (placeholder text is centered separately by line.paint).
                            let line_width = if !is_empty && line.len() > 0 {
                                line.x_for_index(line.len())
                            } else {
                                px(0.0)
                            };
                            let align_offset_x = match align {
                                TextAlign::Left => px(0.0),
                                TextAlign::Center => (bounds.size.width - line_width) / 2.0,
                                TextAlign::Right => bounds.size.width - line_width,
                            };

                            let (selection, cursor_quad) = if selected_range.is_empty() {
                                let cursor_visible = focused && input.blink_visible;
                                (
                                    None,
                                    if cursor_visible {
                                        Some(gpui::fill(
                                            Bounds::new(
                                                point(
                                                    bounds.left() + align_offset_x + cursor_pos,
                                                    bounds.top(),
                                                ),
                                                size(px(2.), bounds.bottom() - bounds.top()),
                                            ),
                                            colors.accent,
                                        ))
                                    } else {
                                        None
                                    },
                                )
                            } else {
                                let sel_start = line.x_for_index(selected_range.start);
                                let sel_end = line.x_for_index(selected_range.end);
                                (
                                    Some(gpui::fill(
                                        Bounds::from_corners(
                                            point(
                                                bounds.left() + align_offset_x + sel_start,
                                                bounds.top(),
                                            ),
                                            point(
                                                bounds.left() + align_offset_x + sel_end,
                                                bounds.bottom(),
                                            ),
                                        ),
                                        colors.selection_background,
                                    )),
                                    if focused && input.blink_visible {
                                        Some(gpui::fill(
                                            Bounds::new(
                                                point(
                                                    bounds.left() + align_offset_x + cursor_pos,
                                                    bounds.top(),
                                                ),
                                                size(px(2.), bounds.bottom() - bounds.top()),
                                            ),
                                            colors.accent,
                                        ))
                                    } else {
                                        None
                                    },
                                )
                            };

                            // Return state for paint phase
                            (line, selection, cursor_quad, bounds, align, align_offset_x)
                        }
                    },
                    {
                        let entity = entity_for_paint;
                        move |_paint_bounds, state, window, cx| {
                            let (line, selection, cursor_quad, bounds, align, _align_offset_x) =
                                state;

                            // Handle IME and text input
                            let focus_handle = entity.read(cx).focus_handle.clone();
                            window.handle_input(
                                &focus_handle,
                                gpui::ElementInputHandler::new(bounds, entity.clone()),
                                cx,
                            );

                            // Paint selection background (behind text)
                            if let Some(selection_quad) = selection {
                                window.paint_quad(selection_quad);
                            }

                            // Paint text, vertically centered within the bounds.
                            // ShapedLine::paint uses align_width to position text for
                            // center/right alignment.
                            let line_height = window.line_height();
                            let text_origin = point(
                                bounds.origin.x,
                                bounds.origin.y + (bounds.size.height - line_height) / 2.0,
                            );
                            line.paint(
                                text_origin,
                                line_height,
                                align,
                                Some(bounds.size.width),
                                window,
                                cx,
                            )
                            .ok();

                            // Paint cursor on top
                            if let Some(cursor_quad) = cursor_quad {
                                window.paint_quad(cursor_quad);
                            }

                            // Update entity state for mouse hit-testing and focus transitions
                            let focused = entity.read(cx).focus_handle.is_focused(window);
                            entity.update(cx, |input, cx| {
                                input.last_layout = Some(line);
                                input.last_bounds = Some(bounds);
                                if focused && !input.was_focused {
                                    input.start_blinking(cx);
                                } else if !focused && input.was_focused {
                                    input.blink_visible = false;
                                    cx.notify();
                                }
                                input.was_focused = focused;
                            });
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
    }
}

// ── Render (Entity → RenderOnce) ──────────────────────────────────────────

impl Render for TextField {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TextFieldComponent::new(cx.entity())
    }
}
