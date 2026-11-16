package ca.jakelandau.filmlogger.ui

/** Third-party libraries shown in Settings. Keep in step with the manifests. */
internal data class Credit(val name: String, val license: String)

internal data class CreditGroup(val title: String, val items: List<Credit>)

internal val CREDIT_GROUPS = listOf(
    CreditGroup(
        "Rust core",
        listOf(
            Credit("chrono", "MIT OR Apache-2.0"),
            Credit("reqwest", "MIT OR Apache-2.0"),
            Credit("rusqlite", "MIT"),
            Credit("serde", "MIT OR Apache-2.0"),
            Credit("serde_json", "MIT OR Apache-2.0"),
            Credit("thiserror", "MIT OR Apache-2.0"),
            Credit("SQLite (via rusqlite)", "Public domain"),
        ),
    ),
    CreditGroup(
        "Desktop",
        listOf(
            Credit("Tauri", "MIT OR Apache-2.0"),
            Credit("tauri-plugin-dialog", "MIT OR Apache-2.0"),
            Credit("tauri-build", "MIT OR Apache-2.0"),
            Credit("@tauri-apps/api", "MIT OR Apache-2.0"),
            Credit("@tauri-apps/plugin-dialog", "MIT OR Apache-2.0"),
            Credit("Vite", "MIT"),
            Credit("TypeScript", "Apache-2.0"),
            Credit("Vitest", "MIT"),
        ),
    ),
    CreditGroup(
        "Android",
        listOf(
            Credit("Kotlin", "Apache-2.0"),
            Credit("AndroidX Core KTX", "Apache-2.0"),
            Credit("AndroidX Activity Compose", "Apache-2.0"),
            Credit("AndroidX Lifecycle Runtime KTX", "Apache-2.0"),
            Credit("Jetpack Compose (UI, Material 3, icons)", "Apache-2.0"),
            Credit("Coil Compose / Coil SVG", "Apache-2.0"),
            Credit("JNA", "Apache-2.0 OR LGPL-2.1"),
            Credit("UniFFI", "MPL-2.0"),
        ),
    ),
    CreditGroup(
        "Test-only",
        listOf(
            Credit("tokio", "MIT"),
            Credit("wiremock", "MIT OR Apache-2.0"),
            Credit("tempfile", "MIT OR Apache-2.0"),
        ),
    ),
    CreditGroup(
        "Assets and data",
        listOf(
            Credit(
                "Official rating symbols and country flags",
                "Wikimedia Commons contributors (individual file licenses)",
            ),
        ),
    ),
)

internal const val TMDB_ATTRIBUTION =
    "This product uses the TMDB API but is not endorsed or certified by TMDB. " +
        "Film metadata and images come from The Movie Database (TMDB)."
