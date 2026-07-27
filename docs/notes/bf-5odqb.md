# bf-5odqb — Provision Termux + Termux:API on Pixel 6 (one-time HOOP ADB bridge setup)

> Verified operator runbook + automation-boundary analysis. Dispatched
> 2026-07-26 against the live Pixel 6 (`100.88.10.113`). **Bead left OPEN** —
> acceptance is unreachable by an unattended agent (see
> [Why this bead is not auto-closeable](#why-this-bead-is-not-auto-closeable)).

## TL;DR

This bead models the missing **phone-side half** of the HOOP↔Pixel 6 bridge as
tracked work. The host-side artifacts are all shipped and verified end-to-end
up to the missing receiver (see [bf-357bc](bf-357bc.md)). This run re-confirmed
the phone is still un-provisioned, **nailed down precisely why no adb-only path
can complete it**, and produced the device-specific operator runbook below. The
provisioning itself is a manual operator step and was **not** performed: it
installs apps onto the operator's personal phone and requires at least one
interactive session *inside* the Termux app that adb cannot drive.

When the operator runs [Quick start](#quick-start-operator-runbook), re-run
bf-357bc — the host side already passes every path, so only criterion #1's
phone-visible half should flip PASS with no code change.

## Verified environment (2026-07-26)

| Check | Result |
|-------|--------|
| ADB → Pixel 6 | ✅ `adb-check` → `connected: 100.88.10.113:5555 (authorized)`, state `device` |
| Device / OS | Pixel 6, **Android 16 / API 36** → `POST_NOTIFICATIONS` applies (Android 13+) |
| Free storage `/data` | ✅ 92 G free / 110 G — no concern |
| F-Droid installed | ✅ `org.fdroid.fdroid` present → in-phone install path available |
| Termux installed | ❌ `pm list packages \| grep termux` → empty; `/data/data/com.termux` does not exist |
| Termux:API installed | ❌ `com.termux.api` absent |
| `HOOP_NOTIFY` receiver registered | ❌ `pm query-receivers --components -a HOOP_NOTIFY` → **`No receivers found`** |
| `HOOP_DICTATE_START` receiver registered | ❌ `No receivers found` (inbound half also un-provisioned) |
| Host reachability to APK sources | ✅ `f-droid.org` → HTTP 200 (0.5 s); `github.com` → HTTP 200 (0.9 s) — host can download APKs |
| Host-side `notify-phone` installed | ✅ `~/.hoop/scripts/notify-phone` (+`.yml`), executable → acceptance test is runnable the moment the phone side is up |

State is **unchanged** from bf-357bc runs #1/#2 — stable, not transient.

## Why this bead is not auto-closeable

The bridge splits cleanly into an **adb-automatable** half and a
**human-in-Termux** half. The agent can do the first; it cannot do the second,
and the second is required for every one of the three acceptance criteria.

### What an agent *could* do via adb (host → phone, no human-in-Termux)

- `adb install` the Termux + Termux:API APKs (host reaches f-droid.org /
  github.com; the package manager installs as `system`, not as the `shell` user).
- `adb shell pm grant com.termux android.permission.POST_NOTIFICATIONS`.

### What an agent *cannot* do — the actual blockers

1. **`pkg install termux-api sox curl`** runs as Termux's app uid inside the
   Termux shell. `adb shell` is the `shell` user (uid 2000); `/data/data` is
   `system:system` mode `drwxrwx--x`, so `shell` has only traverse (`--x`)
   permission and **cannot write into `/data/data/com.termux/…` or run `pkg`**.
   Verified this run: `ls -ld /data/data` → `drwxrwx--x system system`.
2. **Placing and starting the listener** has the same uid wall: Termux home
   (`/data/data/com.termux/files/home`) is owned by Termux's app uid, not
   writable by `shell`. The script must be `cp`'d there *from inside Termux*,
   then `nohup ~/hoop-listener.sh …` started there.
3. **The RUN_COMMAND escape hatch is chicken-and-egg.** Driving Termux from adb
   via the `com.termux.RUN_COMMAND` service requires
   `allow-external-apps=true` in `~/.termux/termux.properties` — which must be
   created/edited *inside Termux first*. So the very first `pkg install` and
   the first listener bootstrap need a human typing in the Termux app. After
   that one manual enable, subsequent commands can be automated — but the first
   run cannot.

Because criteria 1–3 all depend on `pkg install termux-api` + a running
listener, **no adb-only path reaches acceptance.** Driving the Termux *UI* with
`adb shell input text` is theoretically possible but fragile (focus-dependent,
space handling, collides with operator use of a personal device) and is not an
appropriate unattended mutation of the operator's phone.

Additionally, this is an **outward-facing change to the operator's personal
phone** (installing apps, granting a permission) — not durably authorized for
unattended application, and consistent with bf-357bc's framing of provisioning
as a manual operator step.

## ⚠️ Caveat on acceptance criterion #1

The bead's criterion #1 — *"`pm query-receivers --components -a HOOP_NOTIFY`
returns the termux listener"* — is **likely structurally unachievable as
written**, regardless of correct provisioning. `scripts/termux-hoop-listener.sh`
catches `HOOP_NOTIFY` via `termux-broadcast-receiver`, which registers a
**runtime** (dynamic) `BroadcastReceiver` in the Termux:API process for the
duration of each blocking call — *not* a static, manifest-declared receiver.
`pm query-receivers` lists only manifest receivers, so it will probably keep
reporting `No receivers found` even when the listener is working perfectly.

**Authoritative substitutes** (these are what actually prove the bridge, and
what the operator should verify instead of chasing criterion #1):

- The listener log shows `HOOP_NOTIFY receiver started (action=HOOP_NOTIFY)`.
- Firing the broadcast logs `HOOP_NOTIFY received: <title>` in
  `~/.hoop-listener.log` **and** posts a shade notification titled
  e.g. `🔴 HOOP: Capacity Alert`.

If criterion #1 is ever meant to pass literally, that requires a real Android
manifest receiver (a compiled app), which is outside the Termux:API design. The
two substitutes above are the substantive acceptance checks.

## Quick start (operator runbook)

All steps marked **[manual]** must be done by the operator *inside the Termux
app on the phone*. Steps marked **[adb]** the operator can run from the coding
host (`100.64.0.1`). Assumes `adb-check` reports the Pixel 6 connected
(reconnect with `adb-connect <port>` if the Wireless Debugging port changed).

### 1. Install Termux + Termux:API **[manual, on phone]**

From **F-Droid** (already installed), **not Google Play** — the Play build of
Termux is abandoned and breaks API calls. Install both:

- `com.termux` (Termux)
- `com.termux.api` (Termux:API)

> *Alternative (host-assisted):* on the coding host, download the two APKs from
> f-droid.org (reachable: HTTP 200) and `adb install termux.apk && adb install
> termux-api.apk`. Same F-Droid artifacts, just delivered over adb. Either way
> the apps must come from F-Droid signing, not Play.

### 2. Inside Termux: install the packages the listener needs **[manual]**

Open the Termux app and run:

```bash
pkg install termux-api sox curl
```

`termux-api` provides `termux-notification` **and** `termux-broadcast-receiver`
(both used by the return path); `sox` and `curl` are needed by the dictation
half of the same listener.

### 3. Place and start the listener **[manual]**

Still inside Termux, get the script onto the phone and into Termux home. From
the coding host, push it to shared storage the shell user *can* write:

```bash
# [adb]  push to /sdcard (shell can write here)
adb push scripts/termux-hoop-listener.sh /sdcard/hoop-listener.sh
```

Then inside Termux, move it into home and launch it:

```bash
# [manual, inside Termux]
cp /sdcard/hoop-listener.sh ~/hoop-listener.sh
chmod +x ~/hoop-listener.sh
nohup ~/hoop-listener.sh > ~/.hoop-listener.log 2>&1 &
```

Confirm the receiver came up — the log should show:

```
[hoop-listener] … HOOP_NOTIFY receiver started (action=HOOP_NOTIFY)
```

(If it instead says `HOOP_NOTIFY receiver disabled: termux-notification not
installed`, step 2 didn't take — re-run `pkg install termux-api` and restart.)

### 4. Grant Termux notification permission **[adb or manual]**

Android 16 (API 36) → `POST_NOTIFICATIONS` is required. Quickest via adb:

```bash
# [adb]
adb shell pm grant com.termux android.permission.POST_NOTIFICATIONS
```

(Or on the phone: Settings → Apps → Termux → Notifications → allow.)

### 5. Restart the listener after step 2 **[manual]**

If the listener was started before `pkg install termux-api`, `notify_loop()`
skips the receiver. After step 2, inside Termux:

```bash
pkill -f hoop-listener.sh
nohup ~/hoop-listener.sh > ~/.hoop-listener.log 2>&1 &
```

## Acceptance verification (run after the runbook above)

```bash
# Criterion 1 (substitutes — see caveat above; literal pm query-receivers
# will likely still say "No receivers found" even when working):
adb shell pm query-receivers --components -a HOOP_NOTIFY   # informational only
adb shell 'grep "HOOP_NOTIFY receiver started" ~/.hoop-listener.log'  # ← real check

# Criterion 2 — fire a synthetic capacity_alert from the host:
echo '{"kind":"capacity_alert","summary":"85% util","project":"spaxel"}' \
  | ~/.hoop/scripts/notify-phone
# → a shade notification titled "🔴 HOOP: Capacity Alert" appears on the Pixel 6

# Criterion 3 — the broadcast reached the phone:
adb shell 'grep "HOOP_NOTIFY received" ~/.hoop-listener.log'
# → "HOOP_NOTIFY received: 🔴 HOOP: Capacity Alert"
```

When criteria 2 + 3 pass (criterion 1 via the log substitute), the phone side is
provisioned: **close `bf-5odqb`**, then re-run **bf-357bc** — no code change is
expected, only criterion #1's phone-visible half should flip PASS.

## Status

- **bf-5odqb:** left OPEN — not completable by an agent; pending operator
  provisioning per [Quick start](#quick-start-operator-runbook). `bf-5odqb`
  remains a **blocker of `bf-357bc`** until the operator runs the runbook.
- **bf-357bc:** still OPEN, blocked by this bead. Host side already PASSes every
  verifiable path; phone-side delivery is the sole remaining gap.
- No code changed this run — the artifacts under test are already at `HEAD`. The
  only output of this dispatch is this note.
