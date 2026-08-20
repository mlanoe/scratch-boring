//! End-to-end test for `sensing_askandwait`/`sensing_answer` -- the real
//! pausable scheduler's fourth genuinely-yielding block, and the first one
//! that resumes based on real keyboard *text* input rather than a
//! `ScratchTimer` threshold or a `Block` condition (see `FrameKind::
//! WaitingForAnswer`'s own doc comment in runtime.br).
//!
//! `assets/ask_and_wait.json`: Sprite1's green-flag script asks "what is
//! your name?" and waits, then sets a Stage variable to whatever `answer`
//! reports.
//!
//! ```text
//! when green flag clicked
//! ask "what is your name?" and wait
//! set myvar to (answer)
//! ```

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_input::ButtonState;
use scratch_boring::{
    check_key_press_hats, despawn_pending_clones, handle_ask_text_input, play_queued_sounds, run_pending_threads,
    setup_scratch_project, spawn_pending_clones, sync_ask_ui, sync_keyboard_state, sync_mouse_state, sync_scratch_timer,
    sync_variable_watchers, AskBoxText, ProjectPath, VariableWatcherTag,
};
use std::sync::Arc;

fn type_char(app: &mut App, ch: &str) {
    // `key_code` itself doesn't matter here -- `handle_ask_text_input`
    // only branches on `KeyCode::Enter`/`KeyCode::Backspace` explicitly,
    // appending `event.text` for anything else -- `KeyCode::KeyA` is just
    // a real, valid variant, not a claim about which physical key this
    // simulates.
    app.world_mut().resource_mut::<Messages<KeyboardInput>>().write(KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key: Key::Character(ch.into()),
        state: ButtonState::Pressed,
        text: Some(ch.into()),
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

fn press_enter(app: &mut App) {
    app.world_mut().resource_mut::<Messages<KeyboardInput>>().write(KeyboardInput {
        key_code: KeyCode::Enter,
        logical_key: Key::Enter,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

#[test]
fn ask_and_wait_shows_the_box_captures_typed_text_and_resumes_with_the_answer() {
    let path: Arc<str> = Arc::from("assets/ask_and_wait.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        // `MessageReader<KeyboardInput>` (`handle_ask_text_input`'s own
        // `SystemParam`) needs the `Messages<KeyboardInput>` resource to
        // already exist -- `MinimalPlugins` alone doesn't register it
        // (real games get this for free from `InputPlugin`, which isn't
        // part of `MinimalPlugins`); a real `cargo test` run confirmed
        // "Message not initialized" without this.
        .add_message::<KeyboardInput>()
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                sync_scratch_timer,
                check_key_press_hats,
                handle_ask_text_input,
                run_pending_threads,
                play_queued_sounds,
                spawn_pending_clones,
                despawn_pending_clones,
                sync_ask_ui,
                sync_variable_watchers,
            )
                .chain(),
        );

    // Tick 1: the green-flag script runs up to `ask and wait`, which
    // pushes a PendingAsk and pauses -- `set myvar to answer` hasn't run
    // yet.
    app.update();

    // `Vars.is_asking()`/`.answer` aren't externally checkable from this
    // test crate (both private -- no `pub` on either in the Boring
    // source), so "still paused, showing the real question" is verified
    // entirely through the observable ask-box entity instead, same
    // "check what's actually rendered" philosophy every other ECS test in
    // this project already follows.
    {
        let world = app.world_mut();
        let mut query = world.query::<(&AskBoxText, &Text2d, &Visibility)>();
        let (_, text2d, visibility) = query.iter(world).next().expect("the ask box entity should always exist");
        assert_eq!(*visibility, Visibility::Visible, "the ask box should be visible while a question is pending");
        assert!(text2d.0.contains("what is your name?"), "the ask box should show the real question, got: {}", text2d.0);
    }

    // Type "Ada" character by character, then press Enter -- all buffered
    // in the same tick's message queue, so `handle_ask_text_input`
    // processes the whole sequence (and `run_pending_threads`, chained
    // right after it, resumes the now-answered thread) within this one
    // `app.update()` call.
    type_char(&mut app, "A");
    type_char(&mut app, "d");
    type_char(&mut app, "a");
    press_enter(&mut app);
    app.update();

    // The ask box should have hidden itself again once answered.
    {
        let world = app.world_mut();
        let mut query = world.query::<(&AskBoxText, &Visibility)>();
        let (_, visibility) = query.iter(world).next().expect("the ask box entity should still exist");
        assert_eq!(*visibility, Visibility::Hidden, "the ask box should hide itself again once answered");
    }

    // Checked via the real on-screen watcher entity (same proven pattern
    // tests/watchers.rs already uses) -- confirms both that `sensing_
    // answer` reports the real typed text *and* that the script's own
    // "set myvar to answer" statement actually ran once the ask resumed
    // (not just that some internal state updated with nothing ever
    // reading it back).
    let world = app.world_mut();
    let mut query = world.query::<(&VariableWatcherTag, &Text2d)>();
    let (_, text2d) = query.iter(world).next().expect("myvar's own watcher entity should exist");
    assert_eq!(text2d.0, "myvar: Ada", "the script's own \"set myvar to answer\" should have run once the ask resumed, got: {}", text2d.0);
}
