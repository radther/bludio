//! Audio card configuration page: shows all PulseAudio hardware cards.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<ConfigurationPage> → Vec<Entity<CardRow>>
//!     → Entity<Dropdown>

use crate::backend::audio::pulse::PaWakeup;
use crate::backend::audio::{AudioCommand, AudioState};
use crate::backend::subsystem::SubsystemStatus;
use crate::ui::common::animation::FadeInAnimationExt;
use crate::ui::{h_flex, v_flex};
use crate::ui::components::page_header::page_header;
use crate::ui::pages::configuration::config_card_row::CardRow;
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};

// ── Configuration page entity ──────────────────────────────────────────────

/// Owns a list of `CardRow` entities and coordinates their state.
pub(crate) struct ConfigurationPage {
    rows: Vec<Entity<CardRow>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: Option<PaWakeup>,
    subsystem_status: SubsystemStatus,
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
            subsystem_status: SubsystemStatus::Connecting,
        }
    }

    /// Swap the command sender and wakeup (used after audio reconnect).
    pub(crate) fn update_cmd_tx(
        &mut self,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: Option<PaWakeup>,
    ) {
        self.cmd_tx = cmd_tx;
        self.wakeup = wakeup;
    }

    /// Set the wakeup handle after construction (async arrival).
    pub(crate) fn set_wakeup(&mut self, wakeup: PaWakeup) {
        self.wakeup = Some(wakeup);
    }

    /// Sync rows with the latest audio state cards.
    pub(crate) fn sync_cards(&mut self, state: &AudioState, cx: &mut Context<Self>) {
        self.subsystem_status = state.subsystem_status.clone();

        // Only sync card rows when connected.
        if self.subsystem_status != SubsystemStatus::Connected {
            self.rows.clear();
            return;
        }

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
                let row = cx.new(|row_cx| CardRow::new(card, cmd_tx.clone(), wakeup, row_cx));
                self.rows.push(row);
            }
        }
        self.rows
            .retain(|r| state.cards.iter().any(|c| c.index == r.read(cx).card_index));
    }
}

impl Render for ConfigurationPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(
                        "Configuration",
                        match &self.subsystem_status {
                            SubsystemStatus::Connected => format!(
                                "{} card{}",
                                self.rows.len(),
                                if self.rows.len() == 1 { "" } else { "s" }
                            ),
                            _ => String::new(),
                        },
                        colors,
                        text_styles,
                    ))
                    .with_fade_in_up("config-header", 1),
            )
            .child(match &self.subsystem_status {
                SubsystemStatus::Connecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Connecting to PulseAudio...")
                    .into_any_element(),
                SubsystemStatus::Disconnected(msg) => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.danger)
                    .child(format!("Error: {msg}"))
                    .into_any_element(),
                SubsystemStatus::Reconnecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Reconnecting to PulseAudio...")
                    .into_any_element(),
                SubsystemStatus::Connected => {
                    if self.rows.is_empty() {
                        h_flex()
                            .justify_center()
                            .h(px(200.0))
                            .text_color(colors.text_secondary)
                            .child("No audio cards found")
                            .into_any_element()
                    } else {
                        v_flex()
                            .id("audio-card-list")
                            .gap_4()
                            .flex_1()
                            .overflow_y_scroll()
                            .children(self.rows.iter().enumerate().map(|(i, row)| {
                                div()
                                    .child(row.clone())
                                    .with_fade_in_up(format!("config-row-{i}"), i + 2)
                                    .into_any_element()
                            }))
                            .into_any_element()
                    }
                }
            })
    }
}
