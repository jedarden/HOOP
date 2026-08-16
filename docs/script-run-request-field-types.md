# `ScriptRunRequest` field types

Source: [`hoop-daemon/src/api_scripts.rs`](../hoop-daemon/src/api_scripts.rs),
the `ScriptRunRequest` definition around line 165.

`ScriptRunRequest` is a custom, project-defined Rust struct with two public
fields:

| Field | Declared type | Generic type parameters | Classification |
| --- | --- | --- | --- |
| `args` | `Vec<String>` | `Vec<T>` where `T = String` | `Vec` is a standard-library generic collection and `String` is a standard-library owned string type. Neither is custom or a Rust language primitive. |
| `project` | `Option<String>` | `Option<T>` where `T = String` | `Option` is a standard-library generic enum and `String` is a standard-library owned string type. Neither is custom or a Rust language primitive. |

The complete type nesting is:

- `args` → `Vec<T>` → `String`
- `project` → `Option<T>` → `String`

There are no custom field types and no language primitive fields such as
`bool`, `i32`, or `u64`. The custom type in this definition is the enclosing
`ScriptRunRequest` struct itself.

The `args` field also has `#[serde(default)]`, so an omitted `args` property
deserializes to an empty `Vec<String>`.
