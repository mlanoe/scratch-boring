//! End-to-end test proving `sync_speech_bubbles` actually writes onto a
//! real `Text2d` component through the real Bevy scheduler (not just that
//! `SpeechBubbles`' own plain-data state is correct headless --
//! `tests/looks.rs` already covers that). Reuses `assets/looks.json`
//! (Sprite1: say "Hello!" then think "Hmm...").
//!
//! No `TextPlugin` needed -- `sync_speech_bubbles` writes the `Text2d`
//! component field directly (via its own derived `DerefMut<Target=String>`),
//! not through `Text2dWriter` (which needs `TextPlugin`'s font-layout
//! resources and isn't otherwise necessary here -- every bubble is always
//! one plain string, never rich/multi-span text).
//!
//! The bubble text now lives on its own `SpeechBubbleAnchor` entity (a real
//! `ChildOf` child of the sprite's own entity, see that struct's own doc
//! comment in `scratch.br`), not on the sprite's own `Text2d` directly --
//! also confirms the anchor's own local `Transform.translation.y` is
//! offset above the sprite (Sprite1 has no real costume asset in this
//! raw-JSON fixture, so `TargetCostumeDims` falls back to its own
//! documented 40x40 default -- half-height 20, plus the fixed 10px
//! padding `sync_speech_bubbles` adds -- expected offset 30.0).

use bevy::input::ButtonInput;
use bevy::prelude::*;
use scratch_boring::{
    play_queued_sounds, run_pending_threads, setup_scratch_project, sync_keyboard_state,
    sync_mouse_state, sync_speech_bubbles, ProjectPath, SpeechBubbleAnchor,
};
use std::sync::Arc;

fn test_app() -> App {
    let path: Arc<str> = Arc::from("assets/looks.json");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((AssetPlugin::default(), ImagePlugin::default()))
        // sync_keyboard_state/sync_mouse_state are in this test's own
        // Update chain (they run unconditionally in the real app too) and
        // need these resources to exist -- same reason tests/input.rs
        // inserts them manually rather than via InputPlugin.
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(ProjectPath { path })
        .add_systems(Startup, setup_scratch_project)
        .add_systems(
            Update,
            (
                sync_keyboard_state,
                sync_mouse_state,
                run_pending_threads,
                play_queued_sounds,
                sync_speech_bubbles,
            )
                .chain(),
        );
    app
}

#[test]
fn sprite1s_bubble_anchor_gets_the_final_text_and_sits_above_the_sprite() {
    let mut app = test_app();
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&SpeechBubbleAnchor, &Text2d, &Transform)>();
    let (_, text, transform) = query
        .iter(world)
        .find(|(anchor, ..)| anchor.target_id.as_ref() == "Sprite1")
        .expect("Sprite1 should have its own SpeechBubbleAnchor+Text2d child entity");
    assert_eq!(text.0, "Hmm...", "the think should have overwritten the earlier say");
    assert_eq!(transform.translation.y, 30.0, "the bubble anchor's own local translation should sit above Sprite1's placeholder half-height (20) plus the fixed 10px padding");
}
