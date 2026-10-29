//! A thin wrapper around the `uniffi-bindgen` CLI, so the Android build can
//! generate the Kotlin bindings without a separately installed tool.
//!
//! Usage (generate Kotlin bindings from the compiled cdylib):
//! ```sh
//! cargo run -p film-logger-ffi --example bindgen -- generate \
//!     -l kotlin --crate film_logger_ffi --out-dir <dir> <cdylib>
//! ```

fn main() {
    uniffi::uniffi_bindgen_main();
}
