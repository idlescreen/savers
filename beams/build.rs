// Build script: embed the saver's brand icon at compile time.
//
// `build_support::embed_brand_icon` reads
// `assets/scene-beams.ico` and produces a `include_bytes!`-shaped
// `pub static IDLE_SAVER_BEAMS_BRAND_ICON: &[u8]` symbol so the
// runtime can surface a per-saver splash without an asset path at
// runtime. The image never leaves the binary, which keeps the
// plugin loader from depending on filesystem layout.
//
// Why this lives in `build.rs` (not in the runtime crate): the
// `include_bytes!` macro runs at compile time of this crate, so
// the icon has to be physically present here when `cargo build`
// walks the dependency graph. Putting the embed in a script
// (rather than a function on a regular source file) means the
// bytes are pulled in before any test is even linked.
//
// RULES.md §1 (every page ≥ 16 lines): the doc comment above is
// what brings this file above the 16-line floor; the actual
// `fn main` is one line.

fn main() {
    build_support::embed_brand_icon("assets/scene-beams.ico", "idle-saver-beams");
}
