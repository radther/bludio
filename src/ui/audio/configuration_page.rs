//! Audio card configuration page: shows all PulseAudio hardware cards.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<ConfigurationPage> → Vec<Entity<CardRow>>
//!     → Entity<Dropdown>

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, AudioState};
use crate::ui::audio::card_row::CardRow;
use crate::ui::{StyledExt, h_flex, v_flex};
use gpui::{Context, Entity, Render, SharedString, Window, div, prelude::*, px};

// ── Configuration page entity ──────────────────────────────────────────────

/// Owns a list of `CardRow` entities and coordinates their state.
pub(crate) struct ConfigurationPage {
    rows: Vec<Entity<CardRow>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: Option<PaWakeup>,
    connected: bool,
    error: Option<String>,
}

impl ConfigurationPage {
    /// Create a new configuration page entity.
    pub(crate) fn new(
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: Option<PaWakeup>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            rows: Vec::new(),
            cmd_tx,
            wakeup,
            connected: false,
            error: None,
        }
    }

    /// Sync rows with the latest audio state cards.
    pub(crate) fn sync_cards(
        &mut self,
        state: &AudioState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.connected = state.connected;
        self.error.clone_from(&state.error);

        let Some(wakeup) = self.wakeup else {
            return;
        };
        let cmd_tx = self.cmd_tx.clone();

        for card in &state.cards {
            if let Some(row) = self
                .rows
                .iter()
                .find(|r| r.read(cx).card_index == card.index)
            {
                row.update(cx, |row, row_cx| {
                    row.update_from_card(card, row_cx);
                });
            } else {
                let row =
                    cx.new(|row_cx| CardRow::new(card, cmd_tx.clone(), wakeup, window, row_cx));
                self.rows.push(row);
            }
        }
        self.rows
            .retain(|r| state.cards.iter().any(|c| c.index == r.read(cx).card_index));
    }
}

impl Render for ConfigurationPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(colors.surface)
                    .border_b_1()
                    .border_color(colors.border)
                    .child(div().styled(text_styles.heading).child("Configuration")),
            )
            .when_some(self.error.clone(), |el, err| {
                el.child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(colors.error_background)
                        .text_color(colors.danger)
                        .child(SharedString::from(format!("Error: {err}"))),
                )
            })
            .when(!self.connected, |el| {
                el.child(
                    h_flex()
                        .justify_center()
                        .flex_1()
                        .text_color(colors.text_secondary)
                        .child("Connecting to PulseAudio..."),
                )
            })
            .when(self.connected && self.rows.is_empty(), |el| {
                el.child(
                    h_flex()
                        .justify_center()
                        .h(px(200.0))
                        .text_color(colors.text_secondary)
                        .child("No audio cards found"),
                )
            })
            .when(self.connected && !self.rows.is_empty(), |el| {
                el.child(
                    div()
                        .id("audio-card-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .children(self.rows.clone()),
                )
            })
    }
}
