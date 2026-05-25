//! Card row entity: one entry in the configuration device list.
//!
//! Simpler than `AudioDeviceRow` — cards don't have volumes, so this only
//! shows a display name and a profile dropdown. Owns its own `Dropdown`
//! entity for profile selection.

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, CardInfo};
use crate::ui::StyledExt;
use crate::ui::components::dropdown::DropdownEvent as DdEvt;
use crate::ui::h_flex;
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};

const ROW_HEIGHT: f32 = 64.0;

// ── Card row entity ────────────────────────────────────────────────────────

/// One row in the configuration device list. Owns its own `Dropdown` for
/// profile selection.
pub(crate) struct CardRow {
    pub(crate) card_index: u32,
    display_name: String,
    profile_dropdown: Entity<crate::ui::components::dropdown::Dropdown>,
    /// Stored for potential direct command dispatch from the row.
    #[allow(dead_code)]
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: PaWakeup,
    focus_handle: FocusHandle,
    #[allow(dead_code)]
    dropdown_sub: Subscription,
}

impl CardRow {
    /// Create a new card row from a `CardInfo` snapshot.
    pub(crate) fn new(
        card: &CardInfo,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: PaWakeup,
        cx: &mut Context<Self>,
    ) -> Self {
        let display_name = card
            .description
            .clone()
            .unwrap_or_else(|| card.name.clone());
        let profiles: Vec<String> = card.profiles.iter().map(|p| p.name.clone()).collect();
        let selected_idx = card
            .active_profile
            .as_ref()
            .and_then(|active| profiles.iter().position(|p| p == active))
            .unwrap_or(0);

        let profile_dropdown = cx.new(|cx| {
            crate::ui::components::dropdown::Dropdown::new(profiles, selected_idx, cx)
                .placeholder("unknown")
        });

        let dropdown_sub = cx.subscribe(&profile_dropdown, {
            let cmd_tx = cmd_tx.clone();
            move |this, _dd, event: &DdEvt, _cx| {
                if let DdEvt::Selected(_idx, profile) = event {
                    let _ = cmd_tx.send(AudioCommand::SetCardProfile(
                        this.card_index,
                        profile.clone(),
                    ));
                    this.wakeup.wake();
                }
            }
        });

        Self {
            card_index: card.index,
            display_name,
            profile_dropdown,
            cmd_tx,
            wakeup,
            focus_handle: cx.focus_handle(),
            dropdown_sub,
        }
    }

    /// Sync row state from a fresh card snapshot.
    pub(crate) fn update_from_card(&mut self, card: &CardInfo, cx: &mut Context<Self>) {
        self.display_name = card
            .description
            .clone()
            .unwrap_or_else(|| card.name.clone());
        let profiles: Vec<String> = card.profiles.iter().map(|p| p.name.clone()).collect();
        let selected_idx = card
            .active_profile
            .as_ref()
            .and_then(|active| profiles.iter().position(|p| p == active))
            .unwrap_or(0);
        self.profile_dropdown
            .update(cx, |d, cx| d.set_items(&profiles, selected_idx, cx));
        cx.notify();
    }
}

impl Focusable for CardRow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CardRow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;
        h_flex()
            .justify_between()
            .id(SharedString::from(format!("card-{}", self.card_index)))
            .px_4()
            .h(px(ROW_HEIGHT))
            .border_b_1()
            .border_color(colors.border_subtle)
            .hover(|el| el.bg(colors.hover_overlay))
            .child(
                h_flex()
                    .gap_2()
                    .w_full()
                    .child(
                        div()
                            .styled(text_styles.heading)
                            .child(SharedString::from(self.display_name.clone())),
                    )
                    .when(self.profile_dropdown.read(cx).has_items(), |el| {
                        el.child(self.profile_dropdown.clone())
                    }),
            )
    }
}
