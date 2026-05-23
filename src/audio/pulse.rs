//! PulseAudio backend: connection, listing, command execution, event subscription.
//!
//! Runs a dedicated std::thread with its own PA Mainloop. Communicates with
//! the GPUI thread via tokio::sync::mpsc channels for state updates and commands.
//! (tokio::sync::mpsc::UnboundedSender works from any thread — only the
//! receiver end needs a Tokio runtime.)

use crate::audio::{AudioCommand, AudioState, CardInfo, ProfileInfo, SinkInfo, SourceInfo};
use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlags, State as ContextState};
use pulse::mainloop::api::MainloopInnerType;
use pulse::mainloop::standard::IterateResult;
use pulse::mainloop::standard::Mainloop;
use pulse::proplist::Proplist;
use pulse::volume::Volume;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use tokio::sync::mpsc as tmpsc;

// ── Public entry point ─────────────────────────────────────────────────────

/// Opaque handle for waking the PulseAudio mainloop from another thread.
#[derive(Clone, Copy)]
pub(crate) struct PaWakeup {
    ptr: *mut std::ffi::c_void,
}

unsafe impl Send for PaWakeup {}

impl PaWakeup {
    /// Wake up the PA mainloop. Safe to call from any thread.
    pub(crate) fn wake(&self) {
        unsafe {
            libpulse_sys::mainloop::pa_mainloop_wakeup(self.ptr.cast());
        }
    }
}

/// Run the PulseAudio backend thread.
pub(crate) fn run_pa_thread_from_channels(
    cmd_rx: tmpsc::UnboundedReceiver<AudioCommand>,
    state_tx: tmpsc::UnboundedSender<AudioState>,
    wakeup_tx: mpsc::Sender<PaWakeup>,
) {
    if let Err(e) = run_pa_loop(cmd_rx, state_tx, wakeup_tx) {
        eprintln!("[audio] PulseAudio thread error: {e}");
    }
}

// ── Done-flag helper ───────────────────────────────────────────────────────

type DoneFlag = Rc<RefCell<bool>>;

fn new_done_flag() -> DoneFlag {
    Rc::new(RefCell::new(false))
}

/// Spin the mainloop until `done` becomes true (set by a callback).
fn spin_until(ml: &Rc<RefCell<Mainloop>>, done: &DoneFlag) {
    while !*done.borrow() {
        match ml.borrow_mut().iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                break;
            }
            IterateResult::Success(_) => {}
        }
    }
    // Reset the flag for next use
    *done.borrow_mut() = false;
}

// ── PA main loop ───────────────────────────────────────────────────────────

fn run_pa_loop(
    mut cmd_rx: tmpsc::UnboundedReceiver<AudioCommand>,
    state_tx: tmpsc::UnboundedSender<AudioState>,
    wakeup_tx: mpsc::Sender<PaWakeup>,
) -> Result<(), String> {
    let mut mainloop = Mainloop::new().ok_or("Failed to create PA mainloop")?;

    let mut proplist = Proplist::new().ok_or("Failed to create proplist")?;
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "Bludio")
        .map_err(|_| "Failed to set app name")?;

    let mut ctx = Context::new_with_proplist(&mainloop, "Bludio", &proplist)
        .ok_or("Failed to create PA context")?;

    ctx.connect(None, ContextFlags::NOFLAGS, None)
        .map_err(|_| "Failed to connect to PulseAudio")?;

    wait_for_ready(&mut mainloop, &ctx)?;

    // Extract raw mainloop pointer for cross-thread wakeup
    let ml_ptr: *mut std::ffi::c_void = {
        let inner = Rc::clone(&mainloop._inner);
        inner.get_ptr().cast()
    };
    let _ = wakeup_tx.send(PaWakeup { ptr: ml_ptr });

    let ml: Rc<RefCell<Mainloop>> = Rc::new(RefCell::new(mainloop));
    let pa_ctx: Rc<RefCell<Context>> = Rc::new(RefCell::new(ctx));
    let done = new_done_flag();

    // Send initial state
    let initial = build_audio_state(&pa_ctx, &ml, &done);
    let _ = state_tx.send(initial);

    // Command + event loop
    // The subscribe callback sets needs_rebuild on any PA event.
    // iterate(true) blocks until either a PA event arrives or wakeup() is
    // called from the GPUI thread (when a command is sent).
    let needs_rebuild: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    {
        let nr = needs_rebuild.clone();
        let mut ctx_mut = pa_ctx.borrow_mut();
        ctx_mut.set_subscribe_callback(Some(Box::new(
            move |_facility: Option<pulse::context::subscribe::Facility>,
                  _operation: Option<pulse::context::subscribe::Operation>,
                  _index: u32| {
                *nr.borrow_mut() = true;
            },
        )));
    }
    let _sub_op = pa_ctx.borrow_mut().subscribe(
        pulse::context::subscribe::Facility::Sink.to_interest_mask()
            | pulse::context::subscribe::Facility::Source.to_interest_mask()
            | pulse::context::subscribe::Facility::Card.to_interest_mask()
            | pulse::context::subscribe::Facility::Server.to_interest_mask(),
        |_success| {},
    );

    loop {
        // ── Process pending commands (non-blocking) ──
        // Commands arrive when the GPUI thread sends them; wakeup() was
        // already called, so iterate(true) will return immediately.
        if let Ok(cmd) = cmd_rx.try_recv() {
            execute_command(&pa_ctx, &ml, &cmd, &done);
            *needs_rebuild.borrow_mut() = true;
        }

        // ── Rebuild state if needed ──
        if *needs_rebuild.borrow() {
            *needs_rebuild.borrow_mut() = false;
            let state = build_audio_state(&pa_ctx, &ml, &done);
            let _ = state_tx.send(state);
        }

        // ── Block until PA event OR wakeup() from GPUI ──
        // Pure event-driven, no timeouts or delays.
        match ml.borrow_mut().iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => break,
            IterateResult::Success(_) => {}
        }
    }

    Ok(())
}

// ── PA connection helpers ──────────────────────────────────────────────────

fn wait_for_ready(mainloop: &mut Mainloop, ctx: &Context) -> Result<(), String> {
    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(_) => {}
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err("Mainloop error during connection".into());
            }
        }
        match ctx.get_state() {
            ContextState::Ready => break,
            ContextState::Failed | ContextState::Terminated => {
                return Err("PulseAudio connection failed".into());
            }
            _ => {}
        }
    }
    Ok(())
}

// ── Command execution ──────────────────────────────────────────────────────

fn execute_command(
    pa_ctx: &Rc<RefCell<Context>>,
    ml: &Rc<RefCell<Mainloop>>,
    cmd: &AudioCommand,
    done: &DoneFlag,
) {
    match cmd {
        AudioCommand::SetSinkVolume(index, vol) => {
            let index = *index;
            let vol = *vol;
            let v = volume_f64_to_pa(vol);
            let cv = make_channel_volumes(v);
            let mut intro = pa_ctx.borrow_mut().introspect();
            let d = done.clone();
            let _op = intro.set_sink_volume_by_index(
                index,
                &cv,
                Some(Box::new(move |_success| {
                    *d.borrow_mut() = true;
                })),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetSourceVolume(index, vol) => {
            let index = *index;
            let vol = *vol;
            let v = volume_f64_to_pa(vol);
            let cv = make_channel_volumes(v);
            let mut intro = pa_ctx.borrow_mut().introspect();
            let d = done.clone();
            let _op = intro.set_source_volume_by_index(
                index,
                &cv,
                Some(Box::new(move |_success| {
                    *d.borrow_mut() = true;
                })),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetSinkMute(index, mute) => {
            let index = *index;
            let mute = *mute;
            let mut intro = pa_ctx.borrow_mut().introspect();
            let d = done.clone();
            let _op = intro.set_sink_mute_by_index(
                index,
                mute,
                Some(Box::new(move |_success| {
                    *d.borrow_mut() = true;
                })),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetSourceMute(index, mute) => {
            let index = *index;
            let mute = *mute;
            let mut intro = pa_ctx.borrow_mut().introspect();
            let d = done.clone();
            let _op = intro.set_source_mute_by_index(
                index,
                mute,
                Some(Box::new(move |_success| {
                    *d.borrow_mut() = true;
                })),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetCardProfile(index, profile) => {
            let index = *index;
            let mut intro = pa_ctx.borrow_mut().introspect();
            let d = done.clone();
            let _op = intro.set_card_profile_by_index(
                index,
                profile,
                Some(Box::new(move |_success| {
                    *d.borrow_mut() = true;
                })),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetDefaultSink(name) => {
            let mut ctx_mut = pa_ctx.borrow_mut();
            let d = done.clone();
            let _op = ctx_mut.set_default_sink(
                name,
                Box::new(move |_success| {
                    *d.borrow_mut() = true;
                }),
            );
            spin_until(ml, done);
        }
        AudioCommand::SetDefaultSource(name) => {
            let mut ctx_mut = pa_ctx.borrow_mut();
            let d = done.clone();
            let _op = ctx_mut.set_default_source(
                name,
                Box::new(move |_success| {
                    *d.borrow_mut() = true;
                }),
            );
            spin_until(ml, done);
        }
    }
}

// ── Volume conversion ──────────────────────────────────────────────────────

const PA_VOLUME_NORM: f64 = 65536.0;

fn volume_f64_to_pa(vol: f64) -> Volume {
    let raw = (vol * PA_VOLUME_NORM) as u32;
    Volume(raw.clamp(0, Volume::MAX.0))
}

fn pa_volume_to_f64(vol: Volume) -> f64 {
    (vol.0 as f64) / PA_VOLUME_NORM
}

fn make_channel_volumes(v: Volume) -> pulse::volume::ChannelVolumes {
    let mut cv = pulse::volume::ChannelVolumes::default();
    // TODO: preserve channel count from the device instead of hardcoding 2.
    // PulseAudio handles channel count mismatch for volume setting, but
    // reading back from a multi-channel device may show a different value.
    cv.set(2, v);
    cv
}

// ── State building ─────────────────────────────────────────────────────────

fn build_audio_state(
    pa_ctx: &Rc<RefCell<Context>>,
    ml: &Rc<RefCell<Mainloop>>,
    done: &DoneFlag,
) -> AudioState {
    let sinks_data: Rc<RefCell<Vec<SinkInfo>>> = Rc::new(RefCell::new(Vec::new()));
    let sources_data: Rc<RefCell<Vec<SourceInfo>>> = Rc::new(RefCell::new(Vec::new()));
    let cards_data: Rc<RefCell<Vec<CardInfo>>> = Rc::new(RefCell::new(Vec::new()));
    let default_sink: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let default_source: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // ── List sinks ──
    {
        let sinks = sinks_data.clone();
        let intro = pa_ctx.borrow().introspect();
        let d = done.clone();
        let _op = intro.get_sink_info_list(move |result| match result {
            ListResult::Item(si) => {
                let vol = si.volume.get().first().copied().unwrap_or(Volume(0));
                sinks.borrow_mut().push(SinkInfo {
                    index: si.index,
                    name: si.name.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    description: si
                        .description
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    volume: pa_volume_to_f64(vol),
                    muted: si.mute,
                    is_default: false,
                    card_index: si.card,
                    active_profile: None,
                    available_profiles: Vec::new(),
                });
            }
            ListResult::End => *d.borrow_mut() = true,
            ListResult::Error => *d.borrow_mut() = true,
        });
        spin_until(ml, done);
    }

    // ── List sources ──
    {
        let sources = sources_data.clone();
        let intro = pa_ctx.borrow().introspect();
        let d = done.clone();
        let _op = intro.get_source_info_list(move |result| match result {
            ListResult::Item(si) => {
                let vol = si.volume.get().first().copied().unwrap_or(Volume(0));
                sources.borrow_mut().push(SourceInfo {
                    index: si.index,
                    name: si.name.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    description: si
                        .description
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    volume: pa_volume_to_f64(vol),
                    muted: si.mute,
                    is_default: false,
                    is_monitor: si.monitor_of_sink.is_some(),
                });
            }
            ListResult::End => *d.borrow_mut() = true,
            ListResult::Error => *d.borrow_mut() = true,
        });
        spin_until(ml, done);
    }

    // ── List cards ──
    {
        let cards = cards_data.clone();
        let intro = pa_ctx.borrow().introspect();
        let d = done.clone();
        let _op = intro.get_card_info_list(move |result| match result {
            ListResult::Item(ci) => {
                let active_profile = ci
                    .active_profile
                    .as_ref()
                    .and_then(|ap| ap.name.as_ref().map(|s| s.to_string()));
                let profiles: Vec<ProfileInfo> = ci
                    .profiles
                    .iter()
                    .map(|p| ProfileInfo {
                        name: p.name.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                        description: p
                            .description
                            .as_ref()
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                        available: p.available,
                    })
                    .collect();
                cards.borrow_mut().push(CardInfo {
                    index: ci.index,
                    name: ci.name.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    active_profile,
                    profiles,
                });
            }
            ListResult::End => *d.borrow_mut() = true,
            ListResult::Error => *d.borrow_mut() = true,
        });
        spin_until(ml, done);
    }

    // ── List server info (defaults) ──
    {
        let ds = default_sink.clone();
        let ds2 = default_source.clone();
        let intro = pa_ctx.borrow().introspect();
        let d = done.clone();
        let _op = intro.get_server_info(move |info| {
            *ds.borrow_mut() = info.default_sink_name.as_ref().map(|s| s.to_string());
            *ds2.borrow_mut() = info.default_source_name.as_ref().map(|s| s.to_string());
            *d.borrow_mut() = true;
        });
        spin_until(ml, done);
    }

    // ── Cross-reference: card profiles → sinks ──
    let cards = cards_data.borrow();
    let ds = default_sink.borrow();
    let dsrc = default_source.borrow();

    let card_map: std::collections::HashMap<u32, &CardInfo> =
        cards.iter().map(|c| (c.index, c)).collect();

    let mut sinks = sinks_data.borrow_mut();
    for sink in sinks.iter_mut() {
        if let Some(card_idx) = sink.card_index
            && let Some(card) = card_map.get(&card_idx)
        {
            sink.active_profile = card.active_profile.clone();
            sink.available_profiles = card.profiles.clone();
        }
        sink.is_default = ds.as_ref() == Some(&sink.name);
    }

    let mut sources = sources_data.borrow_mut();
    for source in sources.iter_mut() {
        source.is_default = dsrc.as_ref() == Some(&source.name);
    }

    AudioState {
        sinks: sinks.clone(),
        sources: sources.clone(),
        cards: cards.clone(),
        connected: true,
        error: None,
    }
}
