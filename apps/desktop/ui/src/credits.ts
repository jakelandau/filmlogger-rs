// Third-party libraries and attribution shown in Settings. Keep this list in
// step with the manifests; it intentionally lists direct dependencies, not
// the full transitive tree.
export interface Credit {
  name: string;
  license: string;
}

export interface CreditGroup {
  title: string;
  items: Credit[];
}

export const CREDIT_GROUPS: CreditGroup[] = [
  {
    title: "Rust core",
    items: [
      { name: "chrono", license: "MIT OR Apache-2.0" },
      { name: "reqwest", license: "MIT OR Apache-2.0" },
      { name: "rusqlite", license: "MIT" },
      { name: "serde", license: "MIT OR Apache-2.0" },
      { name: "serde_json", license: "MIT OR Apache-2.0" },
      { name: "thiserror", license: "MIT OR Apache-2.0" },
      { name: "SQLite (via rusqlite)", license: "Public domain" },
    ],
  },
  {
    title: "Desktop",
    items: [
      { name: "Tauri", license: "MIT OR Apache-2.0" },
      { name: "tauri-plugin-dialog", license: "MIT OR Apache-2.0" },
      { name: "tauri-plugin-updater", license: "MIT OR Apache-2.0" },
      { name: "tauri-plugin-process", license: "MIT OR Apache-2.0" },
      { name: "tauri-build", license: "MIT OR Apache-2.0" },
      { name: "@tauri-apps/api", license: "MIT OR Apache-2.0" },
      { name: "@tauri-apps/plugin-dialog", license: "MIT OR Apache-2.0" },
      { name: "@tauri-apps/plugin-updater", license: "MIT OR Apache-2.0" },
      { name: "@tauri-apps/plugin-process", license: "MIT OR Apache-2.0" },
      { name: "Vite", license: "MIT" },
      { name: "TypeScript", license: "Apache-2.0" },
      { name: "Vitest", license: "MIT" },
    ],
  },
  {
    title: "Android",
    items: [
      { name: "Kotlin", license: "Apache-2.0" },
      { name: "AndroidX Core KTX", license: "Apache-2.0" },
      { name: "AndroidX Activity Compose", license: "Apache-2.0" },
      { name: "AndroidX Lifecycle Runtime KTX", license: "Apache-2.0" },
      { name: "Jetpack Compose (UI, Material 3, icons)", license: "Apache-2.0" },
      { name: "Coil Compose / Coil SVG", license: "Apache-2.0" },
      { name: "JNA", license: "Apache-2.0 OR LGPL-2.1" },
      { name: "UniFFI", license: "MPL-2.0" },
    ],
  },
  {
    title: "Test-only",
    items: [
      { name: "tokio", license: "MIT" },
      { name: "wiremock", license: "MIT OR Apache-2.0" },
      { name: "tempfile", license: "MIT OR Apache-2.0" },
    ],
  },
  {
    title: "Assets and data",
    items: [
      {
        name: "Official rating symbols and country flags",
        license: "Wikimedia Commons contributors (individual file licenses)",
      },
    ],
  },
];

export const TMDB_ATTRIBUTION =
  "This product uses the TMDB API but is not endorsed or certified by TMDB. " +
  "Film metadata and images come from The Movie Database (TMDB).";
