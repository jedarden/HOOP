# bf-357bc — End-to-end verify notify-phone delivers a notification to the Pixel 6

> Verification note for bead `bf-357bc` (ADR-6 §24 hardware acceptance test for
> parent `bf-3l1bl`). Run 2026-07-26 against the live Pixel 6.

## TL;DR — bead left OPEN; the host-side artifact passes every verifiable path,
## but the phone-side delivery prerequisite is not provisioned, so a notification
## does **not** appear on the Pixel 6 and acceptance criterion #1 is unmet.

The task's stated precondition — *"With notify-phone installed in ~/.hoop/scripts/
and **the Termux listener running notify_loop()** …"* — is **false on this
hardware.** Termux is not installed on the Pixel 6, so the `HOOP_NOTIFY` broadcast
the script fires has no receiver to catch it and posts no shade notification. This
is a missing one-time phone-side setup (a documented operator step), not a defect
in any of the four artifacts under test. Those artifacts are already committed to
`HEAD` by the child beads and are correct.

The bead was therefore **not closed.** The host-side half is proven end-to-end;
once the operator runs the one-time phone provisioning (`scripts/hoop-adb setup`),
this bead can be retried and is expected to pass without code change.

## Environment

| Check | Result |
|-------|--------|
| `hoop` CLI on PATH | ✅ `/home/coding/.local/bin/hoop`; `hoop status --json` returns valid JSON |
| `notify-phone` installed | ✅ `~/.hoop/scripts/notify-phone` (+`.yml`), executable, **byte-identical to `hoop-daemon/examples/scripts/notify-phone`** |
| ADB → Pixel 6 | ✅ `adb-check` → `connected: 100.88.10.113:5555 (authorized)`, state `device` |
| Termux installed on phone | ❌ `pm list packages \| grep termux` → empty; `/data/data/com.termux` does not exist. Only `org.fdroid.fdroid` is installed. |
| `HOOP_NOTIFY` receiver registered | ❌ `pm query-receivers --components -a HOOP_NOTIFY` → **`No receivers found`** |
| Undelivered `HOOP_NOTIFY` broadcasts queued on device | **54** (piling up because no receiver catches them — direct evidence the broadcast reaches the device but goes undelivered) |

## Acceptance-criteria results

| # | Criterion | Result | Evidence |
|---|-----------|--------|----------|
| 1 | A notification appears on the Pixel 6 (real `capacity_alert` or the synthetic event) | ❌ **FAIL** | No receiver registered → broadcast queues undelivered → no shade notification. See "Root cause" below. |
| 2 | Repeated events update the same notification rather than stacking | ⚠️ **Cannot exercise on hardware** (no notification exists to update). Host side fires each event cleanly (exit 0); the coalescing mechanism is `termux-notification --id hoop` in `notify_loop()` (`scripts/termux-hoop-listener.sh:84`) — verified by code, not exercisable until Termux is installed. |
| 3 | `adb`-unreachable case logs and exits 0 (fail-soft re-verified on hardware) | ✅ **PASS** | `ADB_SERIAL=unreachable-device-0000 … notify-phone` → `[notify-phone] adb broadcast exited 1: adb: device 'unreachable-device-0000' not found (fail-soft).` exit **0**. |
| 4 | All four artifacts committed and pushed | ✅ **PASS** | `notify-phone`, `notify-phone.yml`, `scripts/termux-hoop-listener.sh`, `docs/notes/notify-phone-setup.md` are all in `HEAD` (committed by child beads `bf-22rd7`/`bf-4c5ja`/`bf-te1gs`); none modified in the working tree. |

## Host-side evidence (the notify-phone artifact — all paths PASS)

Direct invocation of the installed script (the daemon-free path; see "Daemon
invocation" note below for why `hoop script run` was avoided):

```
$ echo '{"kind":"capacity_alert","summary":"85% util","project":"spaxel"}' \
    | ~/.hoop/scripts/notify-phone
[notify-phone] sent (capacity_alert): 🔴 HOOP: Capacity Alert — [spaxel] 85% util
$ echo $?
0
```

The `sent (…)` log only prints when `deliver()` returns `True`, i.e. `adb` reached
the device and the `am broadcast` returned 0 — so the host→device hop is healthy.
Message formatting (glyph + kind title + `[project]` prefix + sanitized summary)
matches the `TITLES` table in the script and the setup doc.

Repeated events (criterion 2, host half) — each fires and exits 0:
```
[notify-phone] sent (capacity_alert): 🔴 HOOP: Capacity Alert — [spaxel] 87% util   exit=0
[notify-phone] sent (capacity_alert): 🔴 HOOP: Capacity Alert — [spaxel] 91% util   exit=0
```

adb-unreachable fail-soft (criterion 3) — bogus serial, non-destructive (does not
touch the live connection or the shared adb server):
```
$ echo '{"kind":"capacity_alert","summary":"90% util"}' \
    | ADB_SERIAL=unreachable-device-0000 ~/.hoop/scripts/notify-phone
[notify-phone] adb broadcast exited 1: adb: device 'unreachable-device-0000' not found (fail-soft).
$ echo $?
0
```

Bonus — `HOOP_NOTIFY_TIMEOUT` import-time fail-soft (the `bf-22rd7` hardening):
```
$ echo '{"kind":"capacity_alert","summary":"x"}' \
    | HOOP_NOTIFY_TIMEOUT=notanumber ~/.hoop/scripts/notify-phone
[notify-phone] HOOP_NOTIFY_TIMEOUT='notanumber' not an int; using default 10s (fail-soft).
[notify-phone] sent (capacity_alert): 🔴 HOOP: Capacity Alert — x
exit=0
```

## Root cause — phone-side Termux bridge never provisioned

`notify-phone` sends `am broadcast -a HOOP_NOTIFY`; `scripts/termux-hoop-listener.sh`'s
`notify_loop()` is the *only* thing registered to catch it (it then calls
`termux-notification`). That listener runs **inside Termux** on the phone, and
Termux is not installed here:

- `pm list packages | grep termux` → (empty)
- `/data/data/com.termux` → `No such file or directory`
- `pm query-receivers --components -a HOOP_NOTIFY` → **`No receivers found`**
- `dumpsys activity broadcasts` → **54** `act=HOOP_NOTIFY` intents queued
  undelivered (the broadcast reaches the device but has no sink).

Notably the **inbound** dictation half of the same bridge is also un-provisioned —
there is no `HOOP_DICTATE_*` receiver registered either. So this Pixel 6 has never
had the one-time `hoop-adb setup` run against it; the Termux bridge exists only as
shipped code + docs, not as running infrastructure.

This is out of scope for a verification bead: `docs/notes/notify-phone-setup.md`
("Quick start", "Prerequisites") and `scripts/hoop-adb setup` both describe the
one-time phone install as an **operator** manual step (install Termux + Termux:API
**from F-Droid** — the Play build is abandoned/broken; `pkg install termux-api`;
`adb push` the listener; grant notification permission; start the listener).
Provisioning an app onto the operator's personal phone is an outward-facing change
that was not authorized by this task and is not reliably automatable over adb alone
(`pkg install` and the background listener must run *inside* the Termux app).

## Daemon-invocation note (separate finding, not a criterion)

`echo … | hoop script run notify-phone` **hangs** (printed `Running script:
notify-phone`, then blocked past 120s). The running `hoop serve` daemon is a
long-lived instance (started Jul 09) and does not appear to enforce the manifest's
`timeout_secs: 15` — the spawned script likely stalls on `sys.stdin.read()` because
the daemon keeps stdin open without sending EOF. The script's own `NOTIFY_TIMEOUT`
caps only the `adb` call, not the stdin read. This is a daemon-side issue
orthogonal to the notify-phone artifact (which passes when invoked directly) and to
this bead's criteria; flagged here for the record. It does not affect the
acceptance verdict.

## Recommended remediation → retry

1. On the Pixel 6, perform the one-time provisioning documented in
   `docs/notes/notify-phone-setup.md` → "Quick start" / "Prerequisites"
   (equivalently `scripts/hoop-adb setup`):
   - Install **Termux** and **Termux:API** from F-Droid (not Play).
   - Inside Termux: `pkg install termux-api sox curl`.
   - `adb push scripts/termux-hoop-listener.sh …` and start it; confirm the log
     shows `HOOP_NOTIFY receiver started`.
   - Grant Termux notification permission (Android 13+: `pm grant com.termux
     android.permission.POST_NOTIFICATIONS`, or via Settings UI).
2. Re-run this verification. Because the host side already passes every path
   (criteria 3, 4, and the host halves of 1, 2), only criterion #1's phone-visible
   half should flip from FAIL to PASS; no code change is expected.
3. Once criterion 1 passes on provisioned hardware, close `bf-357bc`, which unblocks
   the parent umbrella `bf-3l1bl`.

If provisioning the phone is to be tracked rather than ad-hoc, file it as a new
bead blocking `bf-357bc` (the current open bridge `bf-62eb8` is the *automatic*
daemon→script hook and is a separate concern).
