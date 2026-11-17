//! Contract test for the Tauri IPC surface: every `#[tauri::command]` must be
//! listed in `tauri::generate_handler!` and wrapped in `ui/src/api.ts`.
//!
//! The sources are embedded with `include_str!`, which resolves relative to
//! this file, so the test needs neither the Tauri runtime nor a built UI. A
//! separate file keeps `lib.rs` free of the marker literals scanned here.

const LIB: &str = include_str!("lib.rs");
const API: &str = include_str!("../../ui/src/api.ts");

/// The identifier after each `fn ` that follows a `#[tauri::command]`.
fn command_names() -> Vec<String> {
    let mut names = Vec::new();
    for (index, attribute) in LIB.match_indices("#[tauri::command]") {
        let after_attribute = &LIB[index + attribute.len()..];
        let Some(fn_start) = after_attribute.find("fn ") else {
            continue;
        };
        names.push(
            after_attribute[fn_start + "fn ".len()..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect(),
        );
    }
    names
}

/// The command names listed in `tauri::generate_handler![ ... ]`.
fn handler_names() -> Vec<String> {
    let marker = "tauri::generate_handler![";
    let start = LIB
        .find(marker)
        .unwrap_or_else(|| panic!("no {marker} in lib.rs"))
        + marker.len();
    let list = &LIB[start..];
    let end = list
        .find(']')
        .unwrap_or_else(|| panic!("no closing ] for {marker}"));
    list[..end]
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// The command names passed to `invoke<T>("name"` in `api.ts`. Wrappers must
/// keep the `invoke<T>("name", ...)` shape on one line.
fn api_names() -> Vec<String> {
    let mut names = Vec::new();
    for (index, marker) in API.match_indices("invoke<") {
        let after = &API[index + marker.len()..];
        let Some(generic_end) = after.find(">(") else {
            continue;
        };
        let Some(quoted) = after[generic_end + ">(".len()..].strip_prefix('"') else {
            continue;
        };
        if let Some(end) = quoted.find('"') {
            names.push(quoted[..end].to_owned());
        }
    }
    names
}

fn sorted(names: &[String]) -> Vec<String> {
    let mut names = names.to_vec();
    names.sort();
    names.dedup();
    names
}

/// The comma-separated names of `left` that are absent from `right`.
fn missing(left: &[String], right: &[String]) -> String {
    left.iter()
        .filter(|name| !right.contains(name))
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn commands_are_registered_and_wrapped() {
    let commands = sorted(&command_names());
    let handlers = sorted(&handler_names());
    let wrappers = sorted(&api_names());
    assert!(
        !commands.is_empty(),
        "no #[tauri::command] parsed from lib.rs"
    );

    assert_eq!(
        missing(&commands, &handlers),
        "",
        "commands missing from generate_handler!"
    );
    assert_eq!(
        missing(&handlers, &commands),
        "",
        "generate_handler! entries with no #[tauri::command]"
    );
    assert_eq!(
        missing(&commands, &wrappers),
        "",
        "commands missing from ui/src/api.ts"
    );
    assert_eq!(
        missing(&wrappers, &commands),
        "",
        "api.ts wrappers with no command"
    );
    assert_eq!(commands, handlers);
    assert_eq!(commands, wrappers);
}
