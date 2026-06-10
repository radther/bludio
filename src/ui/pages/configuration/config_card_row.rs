//! Card row entity: one entry in the configuration device list.
//!
//! Simpler than `AudioDeviceRow` — cards don't have volumes, so this only
//! shows a display name, a profile dropdown, and optionally a Bluetooth codec
//! dropdown. Owns its own `Dropdown` entities for profile and codec selection.

use crate::backend::audio::pulse::PaWakeup;
use crate::backend::audio::{AudioCommand, CardInfo};
use crate::ui::StyledExt;
use crate::ui::components::dropdown::DropdownEvent as DdEvt;
use crate::ui::theme::TextStyleSet;
use crate::ui::{h_flex, v_flex};
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, Render, SharedString, Subscription, Window, div,
    prelude::*,
};

/// Bundled parameters for the `build_codec_dropdown` helper.
struct CodecDropdownParams<'a> {
    card: &'a CardInfo,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    #[allow(dead_code)]
    wakeup: PaWakeup,
}

// ── Card row entity ────────────────────────────────────────────────────────

/// One row in the configuration device list. Owns its own `Dropdown` for
/// profile selection and an optional `Dropdown` for Bluetooth codec selection.
pub(crate) struct CardRow {
    pub(crate) card_index: u32,
    card_name: String,
    display_name: String,
    profile_dropdown: Entity<crate::ui::components::dropdown::Dropdown>,
    codec_dropdown: Option<Entity<crate::ui::components::dropdown::Dropdown>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: PaWakeup,
    focus_handle: FocusHandle,
    _dropdown_sub: Subscription,
    _codec_dropdown_sub: Option<Subscription>,
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

        let _dropdown_sub = cx.subscribe(&profile_dropdown, {
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

        let (codec_dropdown, _codec_dropdown_sub) = Self::build_codec_dropdown(
            CodecDropdownParams {
                card,
                cmd_tx: cmd_tx.clone(),
                wakeup,
            },
            cx,
        );

        Self {
            card_index: card.index,
            card_name: card.name.clone(),
            display_name,
            profile_dropdown,
            codec_dropdown,
            cmd_tx: cmd_tx.clone(),
            wakeup,
            focus_handle: cx.focus_handle(),
            _dropdown_sub,
            _codec_dropdown_sub,
        }
    }

    /// Build the optional codec dropdown and its subscription.
    fn build_codec_dropdown(
        params: CodecDropdownParams<'_>,
        cx: &mut Context<Self>,
    ) -> (
        Option<Entity<crate::ui::components::dropdown::Dropdown>>,
        Option<Subscription>,
    ) {
        let card = params.card;
        if card.codecs.is_empty() {
            return (None, None);
        }

        let codec_names: Vec<String> = card.codecs.iter().map(|c| c.description.clone()).collect();
        let selected_idx = card
            .active_codec
            .as_ref()
            .and_then(|active| card.codecs.iter().position(|c| &c.name == active))
            .unwrap_or(0);

        let codec_names_for_lookup: Vec<String> =
            card.codecs.iter().map(|c| c.name.clone()).collect();

        let dropdown = cx.new(|cx| {
            crate::ui::components::dropdown::Dropdown::new(codec_names, selected_idx, cx)
                .placeholder("unknown")
        });

        let sub = cx.subscribe(&dropdown, {
            let card_name = card.name.clone();
            let codec_names = codec_names_for_lookup.clone();
            let cmd_tx = params.cmd_tx;
            move |this, _dd, event: &DdEvt, _cx| {
                if let DdEvt::Selected(idx, _description) = event
                    && let Some(codec_name) = codec_names.get(*idx)
                {
                    let _ = cmd_tx.send(AudioCommand::SetCardCodec(
                        this.card_index,
                        card_name.clone(),
                        codec_name.clone(),
                    ));
                    this.wakeup.wake();
                }
            }
        });

        (Some(dropdown), Some(sub))
    }

    /// Sync row state from a fresh card snapshot.
    pub(crate) fn update_from_card(&mut self, card: &CardInfo, cx: &mut Context<Self>) {
        self.display_name = card
            .description
            .clone()
            .unwrap_or_else(|| card.name.clone());
        self.card_name.clone_from(&card.name);

        // Sync profile dropdown
        let profiles: Vec<String> = card.profiles.iter().map(|p| p.name.clone()).collect();
        let selected_idx = card
            .active_profile
            .as_ref()
            .and_then(|active| profiles.iter().position(|p| p == active))
            .unwrap_or(0);
        self.profile_dropdown
            .update(cx, |d, cx| d.set_items(&profiles, selected_idx, cx));

        // Sync or create codec dropdown
        if !card.codecs.is_empty() {
            let codec_names: Vec<String> =
                card.codecs.iter().map(|c| c.description.clone()).collect();
            let selected_idx = card
                .active_codec
                .as_ref()
                .and_then(|active| card.codecs.iter().position(|c| &c.name == active))
                .unwrap_or(0);
            if let Some(ref dd) = self.codec_dropdown {
                dd.update(cx, |d, cx| d.set_items(&codec_names, selected_idx, cx));
            } else {
                let (dd, sub) = Self::build_codec_dropdown(
                    CodecDropdownParams {
                        card,
                        cmd_tx: self.cmd_tx.clone(),
                        wakeup: self.wakeup,
                    },
                    cx,
                );
                self.codec_dropdown = dd;
                self._codec_dropdown_sub = sub;
            }
        } else {
            self.codec_dropdown = None;
            self._codec_dropdown_sub = None;
        }

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
        let text_styles = TextStyleSet::default();
        v_flex()
            .id(SharedString::from(format!("card-{}", self.card_index)))
            .px_8()
            .hover(|el| el.bg(colors.background_hover))
            .gap_2()
            .items_start()
            .w_full()
            .child(
                div()
                    .styled(text_styles.body)
                    .child(SharedString::from(self.display_name.clone())),
            )
            .child(
                h_flex()
                    .gap_2()
                    .when(self.profile_dropdown.read(cx).has_items(), |el| {
                        el.child(self.profile_dropdown.clone())
                    })
                    .when(
                        self.codec_dropdown
                            .as_ref()
                            .is_some_and(|dd| dd.read(cx).has_items()),
                        |el| {
                            if let Some(ref dd) = self.codec_dropdown {
                                el.child(dd.clone())
                            } else {
                                el
                            }
                        },
                    ),
            )
    }
}
