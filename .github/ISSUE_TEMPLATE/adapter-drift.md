---
name: Adapter drift (unknown events)
about: Report new event kinds from an adapter that need schema support
title: '[ADAPTER DRIFT] <adapter_name>: unknown event kinds'
labels: 'adapter,drift'
assignees: ''
---

## Adapter Name

<!-- Which adapter is emitting unknown events? -->
`<adapter_name>`

## Unknown Event Kinds

<!-- List the unknown event kinds detected -->
<!-- Example: task:started, task:completed, etc. -->

- `event_kind_1`
- `event_kind_2`
- `event_kind_3`

## Raw Event Samples

<!-- Paste raw event JSON from the diagnostics panel -->
<!-- This helps understand the event structure -->

<details>
<summary>Sample 1</summary>

```json
{
  "paste": "raw event here"
}
```
</details>

<details>
<summary>Sample 2</summary>

```json
{
  "paste": "raw event here"
}
```
</details>

## CLI Version

<!-- From diagnostics panel: daemon/schema versions -->
- Daemon: `vX.Y.Z`
- Schema: `sX.Y.Z`

## Expected Behavior

<!-- What should happen with these events? -->
<!-- Example: These events should be tracked as Needle lifecycle events -->

## Context

<!-- Any additional context about the adapter or events -->
<!-- Example: These events appear after updating the CLI to version X.Y.Z -->
