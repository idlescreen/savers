// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Shim: keeps the test files in their own directory so `src/ascii/` stays
//! under the eight-file ceiling from `AGENTS.md`.

#[path = "ascii_tests.rs"]
mod ascii_tests;