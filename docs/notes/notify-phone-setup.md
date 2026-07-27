# notify-phone — Pixel 6 fleet-notification delivery setup

> Implements the second half of **ADR-6** (`docs/plan/plan.md` §24,
> "Out-of-band fleet notification delivery"). Ships the **return path** from
> HOOP to the phone: `scripts/hoop-adb` + `scripts/termux-hoop-listener.sh`
> already carry the *inbound* half (push-to-talk dictation *into* HOOP);
> `notify-phone` carries fleet notifications *out* to the phone's notification
> shade.

This is the operator-script escape hatch (`docs/plan/plan.md` §22.3) — the same
pattern `hoop-daemon/examples/scripts/notify-pushover` demonstrates, but it
targets infrastructure this operator already runs for HOOP (the Tailscale ADB
link to the Pixel 6) rather than a third-party Pushover account nobody has
configured.

**Off by default**, exactly like `notify-pushover`. The daemon never runs it
unless the operator opts in by copying it into `~/.hoop/scripts/`.

> **Current status (2026-07-26).** The script, manifest, and phone-side
> receiver (the three artifacts this doc covers) are shipped. The *automatic*
> path — a fleet event firing this script with no human in the loop — still
> depends on the `FleetNotificationRing → script_trigger` bridge, which ADR-6 §24
> filed as follow-up bead work (`bf-62eb8`, still open). Until that bridge
> lands, an automatic `capacity_alert`/`convoy_complete` will **not** reach this
> script on its own; verify the full path end-to-end with the **manual /
> synthetic** commands in [Manual test](#manual-test), which exercise every
> artifact except the not-yet-wired daemon hook. When `bf-62eb8` merges, this
> status note is the only part of the doc that needs to change.

## Quick start (end-to-end)

If you just want a notification to appear on the Pixel 6:

1. **On the Pixel 6 (inside Termux)** — install the API package the receiver
   needs and restart the listener so its `notify_loop()` runs:
   ```bash
   pkg install termux-api          # provides termux-notification + termux-broadcast-receiver
   pkill -f hoop-listener.sh       # stop the old instance (started before notify_loop existed)
   nohup ~/hoop-listener.sh > ~/.hoop-listener.log 2>&1 &
   ```
   Confirm the receiver came up — the listener log should show
   `HOOP_NOTIFY receiver started`. (If it instead says
   `HOOP_NOTIFY receiver disabled: termux-notification not installed`,
   step 1 didn't take.)

2. **On the coding host** — opt in by copying the example into the live scripts
   dir the daemon supervises:
   ```bash
   mkdir -p ~/.hoop/scripts
   cp hoop-daemon/examples/scripts/notify-phone     ~/.hoop/scripts/notify-phone
   cp hoop-daemon/examples/scripts/notify-phone.yml ~/.hoop/scripts/notify-phone.yml
   chmod +x ~/.hoop/scripts/notify-phone
   ```

3. **Fire a test notification** (works today, no daemon bridge required):
   ```bash
   hoop script run notify-phone
   ```
   A notification titled `📣 HOOP: Test` should appear in the phone's
   notification shade, and the listener log should show
   `HOOP_NOTIFY received: …`.

See [Troubleshooting](#troubleshooting) if nothing appears.

## What it does

When invoked, `notify-phone`:

1. Reads the event JSON from stdin (the daemon-triggered path) **or** a
   positional arg / bare invocation (the manual modes — see
   [Manual test](#manual-test)).
2. Builds a `(title, content)` pair — a glyph + kind title, and the
   notification's `summary` field as the body (falls back to `details`, then to
   a placeholder).
3. Shells out over the **shared** ADB server (`localhost:5037`, the same one
   `hoop-adb` holds open) with
   `am broadcast -a HOOP_NOTIFY --es title … --es content …`.
4. The Termux listener on the phone catches that broadcast and posts a real
   Android notification via `termux-notification`.

The title/glyph mapping (from `TITLES` in the script):

| `kind` | Notification title |
|--------|--------------------|
| `capacity_alert` | `🔴 HOOP: Capacity Alert` |
| `convoy_complete` | `✅ HOOP: Convoy Complete` |
| anything else (incl. manual/test) | `📣 HOOP: <Kind>` |

**Automatic delivery path (ADR-6 §24 design).** The intent is that when a
`FleetNotification` whose kind the manifest subscribes to fires, the daemon
constructs an `EventContext` from the notification's `kind` (rendered as its
snake_case serde tag) and pipes the notification JSON to every matching script
on stdin — reusing the same `on:`/glob-matching machinery
(`matches_subscription` in `hoop-daemon/src/script_trigger.rs`) that already
drives `notify-pushover` against raw `NeedleEvent`s. That `FleetNotification →
EventContext` hook is the open bridge bead above; until it lands,
`trigger_matching_scripts` is only called against `NeedleEvent`s, never against
fleet events, so the manifest subscriptions have no live source to match yet.

## Prerequisites

This example **reuses the dictation bridge**, so the one-time phone setup is
exactly `hoop-adb setup` — there is no separate install for notifications. In
summary (full version: run `scripts/hoop-adb setup`):

### On the coding host

- `adb` in PATH (here: `~/.local/bin/adb`, backed by `~/.local/platform-tools/`).
- The Pixel 6 connected over Tailscale (device IP `100.88.10.113`). Run
  `adb-check`; if it reports the port changed, `adb-connect <new-port>` (the
  operator reads the new port off the phone's Wireless Debugging screen).

### On the Pixel 6 (inside Termux)

- Termux + Termux:API installed (F-Droid, **not** Google Play).
- `pkg install termux-api sox curl` — `termux-api` provides `termux-notification`
  *and* `termux-broadcast-receiver`, both used by the return path; `sox` and
  `curl` are needed by the dictation half the listener also runs.
- The listener script running: `scripts/termux-hoop-listener.sh`. Its
  `notify_loop()` background receiver registers for the `HOOP_NOTIFY` action and
  calls `termux-notification` for each broadcast. It logs
  `HOOP_NOTIFY receiver disabled: termux-notification not installed …` and
  degrades to a no-op if the Termux:API package is missing — dictation still
  works without it.
- **Restart the listener after installing `termux-api`** so `notify_loop()` is
  actually running (a listener started before the package existed skips the
  receiver). See step 1 of [Quick start](#quick-start-end-to-end).

> **Shared ADB server.** All agents on this host share `localhost:5037`. Do not
> send concurrent ADB input from multiple agents — interleaved taps/types
> produce gibberish. The notification path only sends an `am broadcast` (no UI
> input), so it does not collide with dictation or interactive control.

## Enabling it

```bash
# Opt in by copying the example into the live scripts dir the daemon supervises:
mkdir -p ~/.hoop/scripts
cp hoop-daemon/examples/scripts/notify-phone     ~/.hoop/scripts/notify-phone
cp hoop-daemon/examples/scripts/notify-phone.yml ~/.hoop/scripts/notify-phone.yml
chmod +x ~/.hoop/scripts/notify-phone

# Reload (the daemon hot-discovers scripts in ~/.hoop/scripts/), or restart
# the hoop service.
```

The manifest (`notify-phone.yml`) subscribes only to the two kinds meant to
interrupt the operator — not the bead-level firehose:

```yaml
on:
  - event: "capacity_alert"
  - event: "convoy_complete"
```

`event` is glob-matched against the notification kind's snake_case serde tag
(`capacity_alert` / `convoy_complete` / `stitch_beads_closed` /
`bead_created_by_hoop`). To receive more kinds, add them to the `on:` block. The
matching is implemented in `matches_subscription` / `glob_match`
(`hoop-daemon/src/script_trigger.rs`); it just isn't *fed* fleet events until the
bridge bead above lands.

## Configuration (environment variables)

All optional; sane defaults shown.

| Variable | Default | Purpose |
|----------|---------|---------|
| `HOOP_NOTIFY_ACTION` | `HOOP_NOTIFY` | Broadcast action. Override only if you renamed the listener's filter (`HOOP_NOTIFY_ACTION` on the phone side must match). |
| `ADB_SERIAL` | *(auto)* | Device serial, same env `hoop-adb` honors. Needed only if multiple devices are attached. |
| `HOOP_NOTIFY_TIMEOUT` | `10` | Hard wall-clock cap (seconds) on the `adb` call so an unreachable phone can't stall the script. The manifest's `timeout_secs` (15) is the outer bound. Parsed fail-soft at import time: a non-integer value logs a warning and falls back to `10` rather than crashing. |

## Privacy / §18 redaction

The notification JSON crosses the operator-script trust boundary — the same
boundary `notify-pushover` already crosses for raw `NeedleEvent`s. By design
(ADR-6 §24 Consequences), the bridge that feeds fleet events into scripts runs
the payload through the §18 redaction filter (`redaction::redact_json_value` in
`hoop-daemon/src/redaction.rs`; see `docs/concepts/privacy.md`) before it reaches
a script's stdin, so a secret accidentally pasted into a stitch title or
`summary` (e.g. an API key) is replaced with `[REDACTED]`. The redaction
function itself is already in place and tested; the *call site* on the
fleet-notification path is part of the open bridge bead, so until it lands, only
the **manual/synthetic** path exercises this script — and that path feeds it
exactly what you type, so don't pipe real secrets in by hand.

## Fail-soft behavior

Per `docs/plan/plan.md` §3 ("if HOOP dies, nothing else notices"), delivery
failure **never blocks** the code path that fired the event. Two layers enforce
this:

- The script **always exits 0** — a missing `adb`, no device, an unreachable
  phone, a timeout, or even a bad `HOOP_NOTIFY_TIMEOUT` value all degrade to a
  single `[notify-phone] …` line on stderr and exit 0. The only `__main__`
  effect is that log line.
- The daemon runs subscribed scripts fire-and-forget
  (`trigger_matching_scripts` in `hoop-daemon/src/script_trigger.rs`), so even a
  slow or wedged script can't stall the event that triggered it; the manifest's
  `timeout_secs: 15` caps it.

> **Important consequence for diagnosis:** because the script always exits 0, the
> daemon records the run as *completed successfully* **even when delivery
> failed**. The daemon's normal info/warn lines will *not* tell you a
> notification didn't go out. The **single `[notify-phone] … (fail-soft)` line
> on the script's stderr is the only signal** — see [Troubleshooting](#troubleshooting).

## Manual test

The script has three manual modes (no event JSON needed), all fail-soft:

```bash
# Bare invocation → posts a generic test notification
hoop script run notify-phone

# Positional arg → custom content (manual mode)
hoop script run notify-phone "hello from HOOP"

# Synthetic event JSON on stdin → exercises the real event path
echo '{"kind":"capacity_alert","summary":"85% util","project":"spaxel"}' \
  | hoop script run notify-phone
```

### End-to-end check on the phone

With the listener running and the script installed:

1. Run one of the manual commands above (or, once the bridge lands, trigger a
   real `capacity_alert`).
2. Watch the listener log on the phone (`~/.hoop-listener.log`) for
   `HOOP_NOTIFY received: …`.
3. A notification titled e.g. `🔴 HOOP: Capacity Alert` should appear in the
   phone's notification shade. Repeats update the same notification (`--id hoop`)
   rather than stacking.

## Troubleshooting

**Symptom: no notification appears on the phone.** For a manual `hoop script
run`, the script's stderr prints straight to your terminal; for a daemon-run
copy, it's captured in the script's piped stderr (not in the daemon's
info/warn lines — see [Fail-soft behavior](#fail-soft-behavior)). In either case
look for the single `[notify-phone] …` line, which pinpoints the cause:

| The `[notify-phone]` line | What it means | Fix |
|---------------------------|---------------|-----|
| `adb not found in PATH — skipping (fail-soft)` | `adb` isn't on PATH | Install platform-tools (`~/.local/bin/adb`); run `adb-check`. |
| `adb broadcast timed out after Ns — phone unreachable? … (fail-soft)` | ADB connected but the device didn't answer in `HOOP_NOTIFY_TIMEOUT`s | The Wireless Debugging port changed — run `adb-check`, then `adb-connect <new-port>` (read the port off the phone's Wireless Debugging screen). |
| `adb broadcast exited <N>: <stderr> (fail-soft)` | `adb` ran but the device-side `am broadcast` failed | Read the trailing device stderr; usually a Termux/listener problem (next row). |
| `adb vanished mid-call — skipping (fail-soft)` | The `adb` binary disappeared mid-call (rare) | Re-run; check platform-tools install. |
| `unexpected error during adb call: … (fail-soft)` | Anything else | The `repr` names the exception; treat as a bug if it recurs. |

If there is **no `[notify-phone]` line at all** (and no `sent (…)` success line),
the script didn't run — check that it's installed and executable at
`~/.hoop/scripts/notify-phone`.

**On the phone side**, confirm the receiver is healthy via the listener log
(`~/.hoop-listener.log`):

- `HOOP_NOTIFY receiver started (action=HOOP_NOTIFY)` — `notify_loop()` is
  running. If absent, the listener predates `notify_loop()` or wasn't restarted
  after `pkg install termux-api` — restart it (Quick start step 1).
- `HOOP_NOTIFY receiver disabled: termux-notification not installed` —
  `termux-api` isn't installed; `pkg install termux-api` and restart.
- `HOOP_NOTIFY received: …` — the broadcast reached the phone. If you see this
  but still no shade notification, `termux-notification` itself failed (next line
  in the log: `termux-notification failed`); confirm Termux:API is installed from
  F-Droid (the Google Play build is abandoned and breaks API calls) and that
  Termux has notification permission.

**`HOOP_NOTIFY_TIMEOUT=…` not an int; using default 10s (fail-soft)** is benign —
it just means the env var was set to garbage and the script fell back to the
default rather than crashing. Fix the value or unset it.

## Files

| File | Role |
|------|------|
| `hoop-daemon/examples/scripts/notify-phone` | The example script (opt-in; copy to `~/.hoop/scripts/`). Always exits 0; all failure modes degrade to one stderr line. |
| `hoop-daemon/examples/scripts/notify-phone.yml` | Manifest: `capacity_alert` + `convoy_complete` subscriptions, `timeout_secs: 15`, `overlap_policy: skip`. |
| `scripts/termux-hoop-listener.sh` | Phone-side listener — `notify_loop()` is the `HOOP_NOTIFY` receiver that calls `termux-notification`. |
| `scripts/hoop-adb` | The inbound half of the bridge; `hoop-adb setup` documents the one-time phone install this example reuses. |
| `hoop-daemon/src/script_trigger.rs` | `EventContext`, `matches_subscription`, `glob_match`, `trigger_matching_scripts` — the `on:`/glob-matching + fire-and-forget runner. The `FleetNotification → EventContext` hook that would auto-fire this script is the open bridge bead (`bf-62eb8`). |
| `hoop-daemon/src/redaction.rs` | `redact_json_value` — the §18 redaction the bridge will apply on the fleet→script path. |
| `docs/plan/plan.md` §24 (ADR-6) | The decision this implements. |
