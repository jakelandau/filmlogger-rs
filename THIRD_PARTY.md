# Third-party libraries and assets

FilmLogger itself is licensed under **AGPL-3.0-or-later**. This file records the
third-party direct dependencies used by the Rust core, the Tauri desktop app,
and the Android app. Resolved transitive dependencies are pinned in
[`Cargo.lock`](Cargo.lock) and [`apps/desktop/ui/package-lock.json`](apps/desktop/ui/package-lock.json);
they retain their own licenses and are not relicensed by FilmLogger.

## Rust

| Library | Version | License | Source / license link |
|---|---:|---|---|
| chrono | 0.4.45 | MIT OR Apache-2.0 | <https://crates.io/crates/chrono/0.4.45> |
| reqwest | 0.12.28 | MIT OR Apache-2.0 | <https://crates.io/crates/reqwest/0.12.28> |
| rusqlite | 0.32.1 | MIT | <https://crates.io/crates/rusqlite/0.32.1> |
| serde | 1.0.229 | MIT OR Apache-2.0 | <https://crates.io/crates/serde/1.0.229> |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_json/1.0.154> |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | <https://crates.io/crates/thiserror/2.0.21> |
| tauri | 2.11.6 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri/2.11.6> |
| tauri-plugin-dialog | 2.7.3 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri-plugin-dialog/2.7.3> |
| tauri-plugin-updater | 2.12.0 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri-plugin-updater/2.12.0> |
| tauri-plugin-process | 2.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri-plugin-process/2.3.1> |
| tauri-build | 2.6.3 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri-build/2.6.3> |
| uniffi | 0.32.2 | MPL-2.0 | <https://github.com/mozilla/uniffi-rs/blob/main/LICENSE> |
| tokio (test-only) | 1.54.1 | MIT | <https://crates.io/crates/tokio/1.54.1> |
| wiremock (test-only) | 0.6.5 | MIT OR Apache-2.0 | <https://crates.io/crates/wiremock/0.6.5> |
| tempfile (test-only) | 3.27.0 | MIT OR Apache-2.0 | <https://crates.io/crates/tempfile/3.27.0> |

SQLite is compiled from the bundled amalgamation through `rusqlite` /
`libsqlite3-sys`; SQLite itself is in the **public domain**:
<https://www.sqlite.org/copyright.html>.

## Desktop UI

| Package | Version | License | Source / license link |
|---|---:|---|---|
| @tauri-apps/api | 2.12.4 | Apache-2.0 OR MIT | <https://www.npmjs.com/package/@tauri-apps/api/v/2.12.4> |
| @tauri-apps/plugin-dialog | 2.8.4 | MIT OR Apache-2.0 | <https://www.npmjs.com/package/@tauri-apps/plugin-dialog/v/2.8.4> |
| @tauri-apps/plugin-updater | 2.14.1 | MIT OR Apache-2.0 | <https://www.npmjs.com/package/@tauri-apps/plugin-updater/v/2.14.1> |
| @tauri-apps/plugin-process | 2.4.0 | MIT OR Apache-2.0 | <https://www.npmjs.com/package/@tauri-apps/plugin-process/v/2.4.0> |
| TypeScript | 5.9.3 | Apache-2.0 | <https://www.npmjs.com/package/typescript/v/5.9.3> |
| Vite | 6.4.5 | MIT | <https://www.npmjs.com/package/vite/v/6.4.5> |
| Vitest | 3.2.7 | MIT | <https://www.npmjs.com/package/vitest/v/3.2.7> |

## Android

| Library | Version | License | Source / license link |
|---|---:|---|---|
| Kotlin | 2.0.21 | Apache-2.0 | <https://github.com/JetBrains/kotlin/blob/master/license/LICENSE.txt> |
| Android Gradle Plugin (build-only) | 8.10.1 | Apache-2.0 | <https://developer.android.com/build/releases/gradle-plugin> |
| AndroidX Core KTX | 1.15.0 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/core> |
| AndroidX Activity Compose | 1.9.3 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/activity> |
| AndroidX Lifecycle Runtime KTX | 2.8.7 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/lifecycle> |
| Jetpack Compose UI | 1.7.6 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/compose-ui> |
| Jetpack Compose Material 3 | 1.3.1 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/compose-material3> |
| Jetpack Compose material-icons-core | 1.7.6 | Apache-2.0 | <https://developer.android.com/jetpack/androidx/releases/compose-material> |
| Coil Compose / Coil SVG | 2.7.0 | Apache-2.0 | <https://github.com/coil-kt/coil/blob/main/LICENSE> |
| JNA | 5.14.0 | Apache-2.0 OR LGPL-2.1 | <https://github.com/java-native-access/jna/blob/master/LICENSE> |
| UniFFI | 0.32.2 | MPL-2.0 | <https://github.com/mozilla/uniffi-rs/blob/main/LICENSE> |
| JUnit 4 (test-only) | 4.13.2 | EPL-1.0 | <https://github.com/junit-team/junit4/blob/main/LICENSE-junit.txt> |

The Android APK also packages JNA's `libjnidispatch.so` for `arm64-v8a`; its
license is the JNA license listed above. Compose, AndroidX, Kotlin
coroutines, and other Android transitive artifacts retain their upstream
Apache-2.0 licenses.

## Assets and data

| Asset | Source | License / attribution |
|---|---|---|
| TMDB logo | <https://www.themoviedb.org/about/logos-attribution> | The Movie Database (TMDB) logo attribution; TMDB API is not endorsed or certified by TMDB. |
| Official rating symbols and country flags | Wikimedia Commons; original files are in `apps/android/app/src/main/assets/cert-icons/official/` | Each file retains the license stated on its Wikimedia Commons file description page. |
| Film metadata, posters, and backdrops | The Movie Database (TMDB) API | Supplied under the TMDB API terms; not included in this repository. |

## AGPL-3.0-or-later compatibility

| License | Used by | Compatible with AGPL-3.0-or-later? |
|---|---|---|
| MIT / MIT OR Apache-2.0 | nearly every Rust crate and npm package | Yes — permissive, no additional restrictions. |
| Apache-2.0 | TypeScript, Kotlin, AndroidX, Compose, Coil, JNA (chosen arm) | Yes — Apache-2.0 is GPLv3-compatible, and AGPLv3 accepts GPLv3-compatible terms. |
| MPL-2.0 | uniffi (Rust and Kotlin bindings) | Yes — MPL-2.0 §3.3 explicitly allows a larger work under a secondary license, and AGPL-3.0 is one. The MPL-covered files keep their own license. |
| Public domain | SQLite (bundled via rusqlite) | Yes — no conditions. |
| EPL-1.0 | JUnit 4, test-only | Not compatible in a combined distribution (EPL-1.0 has no secondary-license clause), but it is a `testImplementation` dependency: it never enters the APK or any shipped artifact, so it does not affect the licensed work. Do not ship JUnit 4 with the binaries. |
| Wikimedia Commons per-file licenses | rating symbols and country flags | Assets are not relicensed by the AGPL; each file stays under its Commons license and requires the attribution recorded on its file description page. |

No dependency in the shipped product is GPL-only or AGPL-3.0-incompatible.
