// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

use super::super::Ascii;
use super::super::effect::EffectKind;
use crate::runner::{Screensaver, TerminalCell};
use std::time::Duration;

const COLS: usize = 80;
const ROWS: usize = 24;

#[test]
fn animation_completes_before_dwell_and_transition_even_with_short_cycle() {
    let mut saver = Ascii::new();
    // Configure a very short cycle dwell (0.1s)
    saver.params.cycle_secs = Some(0.1);
    saver.init(COLS, ROWS);
    saver.pin_effect(EffectKind::Wipe);

    let mut grid = vec![TerminalCell::default(); COLS * ROWS];

    // For the first 10 frames (0.16s), the animation is actively progressing.
    // In the prior buggy implementation, dwell_left would have expired at 0.1s
    // and cut off the animation prematurely.
    for _ in 0..10 {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
    }
    // Animation must still be active and not prematurely transitioned
    assert!(
        saver.engine.is_some(),
        "animation must complete before transitioning, not cut off by short dwell"
    );
    assert!(!saver.transition_active);

    // Now step until Wipe finishes its animation (~70 frames)
    let mut frames = 0;
    while saver.engine.is_some() && frames < 200 {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
        frames += 1;
    }

    // Now the engine is finished and dwell has started
    assert!(saver.engine.is_none());
    assert!(!saver.transition_active);
    assert!(grid.iter().any(|c| c.ch != ' '));

    // After 0.1s dwell (approx 7 frames of 16ms), transition triggers
    let mut transitioned = false;
    for _ in 0..15 {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
        if saver.transition_active {
            transitioned = true;
            break;
        }
    }
    assert!(
        transitioned,
        "smooth transition should begin after completing dwell"
    );
}

#[test]
fn transition_survives_degenerate_and_extreme_resizes() {
    let mut saver = Ascii::new();
    saver.params.cycle_secs = Some(1.0);
    saver.init(COLS, ROWS);
    saver.pin_effect(EffectKind::Wipe);

    // Step until animation settles
    let mut grid = vec![TerminalCell::default(); COLS * ROWS];
    for _ in 0..100 {
        saver.update(Duration::from_millis(16), COLS, ROWS);
        saver.draw(&mut grid, COLS, ROWS);
    }

    // Trigger transition
    saver.begin_transition();
    assert!(saver.transition_active);

    // Resize to 1x1 mid-transition
    let mut tiny_grid = vec![TerminalCell::default(); 1];
    saver.update(Duration::from_millis(16), 1, 1);
    saver.draw(&mut tiny_grid, 1, 1);
    assert_eq!(tiny_grid.len(), 1);

    // Resize to 0x0 mid-transition
    let mut zero_grid = Vec::new();
    saver.update(Duration::from_millis(16), 0, 0);
    saver.draw(&mut zero_grid, 0, 0);
    assert!(zero_grid.is_empty());

    // Resize back to huge grid (200x60)
    let mut huge_grid = vec![TerminalCell::default(); 200 * 60];
    saver.update(Duration::from_millis(16), 200, 60);
    saver.draw(&mut huge_grid, 200, 60);
    assert_eq!(huge_grid.len(), 12000);
}

#[test]
fn transition_pacing_scales_with_battery_power() {
    let mut saver = Ascii::new();
    saver.on_battery = true;
    saver.params.cycle_secs = Some(2.0);
    saver.init(COLS, ROWS);
    saver.pin_effect(EffectKind::Wipe);

    // Force settled state and transition
    saver.frame_grid = vec![
        TerminalCell {
            ch: 'A',
            fg: (255, 255, 255),
            bg: (0, 0, 0),
            bold: false,
        };
        COLS * ROWS
    ];
    saver.begin_transition();
    assert!(saver.transition_active);

    let initial_left = saver.transition_left;
    saver.update(Duration::from_millis(100), COLS, ROWS);

    // Step should be scaled by 0.55 on battery
    let elapsed = initial_left - saver.transition_left;
    let expected = 0.1 * super::super::BATTERY_SLOWDOWN;
    assert!(
        (elapsed - expected).abs() < 1e-4,
        "transition step must scale with battery rate: got {elapsed}, expected {expected}"
    );
}
