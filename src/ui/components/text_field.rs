//! Self-contained, single-line text input component.
//!
//! Drop-in entity that handles all keyboard and IME input internally.
//! Emits `TextFieldEvent`s via `cx.emit()` (EventEmitter pattern).
//!
//! Based on gpui's `input.rs` example and Zed's `ui_input`/`editor` crates.

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Element, Entity, EntityInputHandler,
    EventEmitter, FocusHandle, Focusable, GlobalElementId, InspectorElementId, IntoElement,
    KeyDownEvent, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, PaintQuad, Pixels, Point,
    Render, ShapedLine, SharedString, Style, TextAlign, TextRun, Window, div, hsla, point,
    prelude::*, px, relative, size,
};
use std::ops::Range;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// ── Constants ──────────────────────────────────────────────────────────────

/// Interval between cursor blink toggles, matches Zed's `CURSOR_BLINK_INTERVAL`.
const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the TextField for parent views to handle.
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
            .map(|(i, _)| i + 1)
            .unwrap_or(0);
        // Find end of word
        let end = self.content[clamped..]
            .char_indices()
            .find(|(_, c)| !c.is_alphanumeric())
            .map(|(i, _)| clamped + i)
            .unwrap_or(len);
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

// ── Render ─────────────────────────────────────────────────────────────────

impl Render for TextField {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.weak_entity();
        let focus_handle = self.focus_handle(cx);

        div()
            .track_focus(&focus_handle)
            .cursor(CursorStyle::IBeam)
            .flex()
            .flex_grow()
            .px_1()
            .min_h(px(20.))
            .on_key_down(move |event: &KeyDownEvent, _window, app| {
                let Some(this) = this.upgrade() else {
                    return;
                };
                let key = event.keystroke.key.clone();
                let modifiers = event.keystroke.modifiers;

                this.update(app, |this, cx| {
                    // Determine platform-appropriate modifier for shortcuts:
                    // Ctrl on Linux/Windows, Cmd on macOS.
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
                        // All other character keys are handled by EntityInputHandler
                        // (IME / platform text input routed via window.handle_input())
                        _ => {}
                    }
                });
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    this.on_mouse_down(event, cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.on_mouse_up(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.on_mouse_up(cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                this.on_mouse_move(event, cx);
            }))
            .child(TextFieldElement {
                entity: cx.entity(),
            })
    }
}

// ── Custom Element (paints text, selection, cursor, and calls handle_input) ─

struct TextFieldElement {
    entity: Entity<TextField>,
}

struct TextFieldPrepaint {
    line: Option<ShapedLine>,
    selection: Option<PaintQuad>,
    cursor: Option<PaintQuad>,
}

impl IntoElement for TextFieldElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextFieldElement {
    type RequestLayoutState = ();
    type PrepaintState = TextFieldPrepaint;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.entity.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0.6, 1.))
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
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let focused = input.focus_handle.is_focused(window);

        // Selection highlight or cursor
        let (selection, cursor_quad) = if selected_range.is_empty() {
            // No selection — paint blinking cursor if focused
            let cursor_visible = focused && input.blink_visible;
            (
                None,
                if cursor_visible {
                    Some(gpui::fill(
                        Bounds::new(
                            point(bounds.left() + cursor_pos, bounds.top()),
                            size(px(2.), bounds.bottom() - bounds.top()),
                        ),
                        hsla(210. / 360., 0.7, 0.55, 1.),
                    ))
                } else {
                    None
                },
            )
        } else {
            // Has selection — always highlight it, plus optionally show cursor
            let sel_start = line.x_for_index(selected_range.start);
            let sel_end = line.x_for_index(selected_range.end);
            (
                Some(gpui::fill(
                    Bounds::from_corners(
                        point(bounds.left() + sel_start, bounds.top()),
                        point(bounds.left() + sel_end, bounds.bottom()),
                    ),
                    hsla(210. / 360., 0.6, 0.5, 0.3),
                )),
                // Also show a cursor at the active end if focused
                if focused && input.blink_visible {
                    Some(gpui::fill(
                        Bounds::new(
                            point(bounds.left() + cursor_pos, bounds.top()),
                            size(px(2.), bounds.bottom() - bounds.top()),
                        ),
                        hsla(210. / 360., 0.7, 0.55, 1.),
                    ))
                } else {
                    None
                },
            )
        };

        TextFieldPrepaint {
            line: Some(line),
            selection,
            cursor: cursor_quad,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.entity.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            gpui::ElementInputHandler::new(bounds, self.entity.clone()),
            cx,
        );

        // Paint selection background first (so it's behind text)
        if let Some(selection_quad) = prepaint.selection.take() {
            window.paint_quad(selection_quad);
        }

        // Paint text
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            TextAlign::Left,
            None,
            window,
            cx,
        )
        .ok();

        // Paint cursor on top
        if let Some(cursor_quad) = prepaint.cursor.take() {
            window.paint_quad(cursor_quad);
        }

        // Update layout/bounds for mouse interaction, and handle focus transitions
        let focused = self.entity.read(cx).focus_handle.is_focused(window);
        self.entity.update(cx, |input, cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            // Edge-detect focus changes for blink lifecycle
            if focused && !input.was_focused {
                input.start_blinking(cx);
            } else if !focused && input.was_focused {
                input.blink_visible = false;
                cx.notify();
            }
            input.was_focused = focused;
        });
    }
}
