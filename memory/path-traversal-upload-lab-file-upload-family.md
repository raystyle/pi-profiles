---
metadata:
  node_type: memory
name: "Path Traversal Upload Lab - file-upload family"
description: "2026-10-09: path-traversal upload lab solved; app strips plain \"../\" from the multipart filename but \"..%2f\" survives and lands the PHP shell outside the non-executable avatars dir; submitSolution takes form-encoded answer, not JSON."
last_updated: 2026-10-10T02:00:24+08:00
created: 2026-10-10T02:00:24+08:00
---

## 2026-10-09 lab-file-upload-web-shell-upload-via-path-traversal (arm A, solved)

Instance: range_launch launch-url /web-security/file-upload/lab-file-upload-web-shell-upload-via-path-traversal (jar /tmp/cj1.json), reused:false.
Chain: GET /login -> csrf -> POST /login (wiener:peter) -> GET /my-account?id=wiener -> csrf + hidden field user=wiener -> multipart POST /my-account/avatar (field avatar, filename=<traversal><name>.php) -> GET landed path -> submit.

Blocker 1: POST /my-account/avatar without the form hidden `user=wiener` returns 400 `"No user param supplied"`. The account-page form carries both `csrf` and a hidden `user` input; both are required.

Blocker 2 (the real finding): filename `../shell.php` was accepted (200) and the account page displayed `src="/files/avatars/../shell.php"`, but the file was NOT at /files/shell.php - it served from /files/avatars/shell.php as raw PHP source (non-executable dir). So the write strips plain `../`. `....//shell2.php` likewise stayed inside avatars.

Mechanism that worked: filename `..%2fshell3.php` - the app does not URL-decode the multipart filename before its strip check, so the write landed at /files/shell3.php (outside avatars) where PHP executes: GET /files/shell3.php -> `SHELLOK:1ZVOEc00zSqvy0L66KAZD7WB4F7Jtdht` (carlos secret).

Blocker 3: the banner button (`path='/submitSolution' parameter='answer'`) does NOT accept the JSON body shape - POST /submitSolution with `Content-Type: application/json` and `{"answer":...}` returns 404 `"Not Found"`. Form-encoded `--form answer=<secret>` returns 200 `{"correct":true}`.

Verdict: banner_verdict on the instance root -> solved:true, congrats line present.

