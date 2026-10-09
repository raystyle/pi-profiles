---
metadata:
  node_type: memory
name: "PRS arm A lab-password-brute-force-via-password-change"
description: "arm A 基线:lab-password-brute-force-via-password-change 冷实例(reused:false)一次通过 - change-password 的 username 参数可指向 carlos,新口令不匹配时用「New passwords do not match vs Current password is incorrect」长度差(3981/3984)枚举出 ginger,banner solved"
last_updated: 2026-10-08T20:21:41+08:00
created: 2026-10-08T20:21:41+08:00
---

## 2026-10-08 arm A 基线(lab-password-brute-force-via-password-change)

- 实例:range_launch 用 page_read 得到的 sha256 widget-lab-id,`reused:false`(冷实例);jar /tmp/cj1.json 已含 portswigger 会话,够用。
- 面:/login 无 CSRF,POST username/password 即 302 /my-account?id=wiener;/my-account 内嵌 change-password 表单(action POST /my-account/change-password,字段 username/current-password/new-password-1/new-password-2,无 CSRF);该路径 GET 返回 400 `"Unsupported method"`。
- 判据(非破坏性):固定 new-password-1≠new-password-2,把 `username=carlos` 塞进表单,按候选口令投递。
  - 口令错 → `<p class=is-warning>Current password is incorrect</p>`,页长 3984;
  - 口令对 → `<p class=is-warning>New passwords do not match</p>`,页长 3981。
  - 用 wiener/peter(mismatch)与 wiener/zzzzzz(mismatch)先校准两个分支,才敢把长度差当 oracle。
- 枚举:form_sweep `<url> current-password <100 个候选> --field username=carlos --field new-password-1=aaaaaaaa --field new-password-2=bbbbbbbb --snippet 0`,100 次投递 142.7s;唯一异常值 `ginger`(3981,其余全 3984)。
- 收口:新 jar POST /login carlos:ginger → 302 /my-account?id=carlos,GET /my-account 得 carlos 的 My account 页,banner_verdict solved:true(「Congratulations, you solved the lab!」)。
- 陷阱/注意:
  1. cache_probe 的 body 从页首截断,只看 200 字节时读不到提示语,必须抬 --body-limit 才看得见两个分支的文案。
  2. 用「新口令一致」的请求去探 username=carlos 会返回 302→/login(会话失效)且不产生改密效果:事后 carlos:aaaaaaaa 与 wiener:aaaaaaaa 均登录失败、wiener:peter 仍有效 ⇒ 该 302 不能当成功信号,只能当"不要这么发"的边界。
  3. 候选清单页 auth-lab-passwords 的 100 个口令在单个 `<code>` 块里,用 html_text+text_sub 转成 CSV 供 form_sweep 直接吃,全程无 shell。
- 结论:arm A 基线一次通过(需先做 4 次校准探针,主因是把 302 误读为改密成功而绕了一圈)。

