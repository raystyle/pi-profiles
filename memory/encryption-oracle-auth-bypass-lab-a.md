---
metadata:
  node_type: memory
name: "Encryption Oracle Auth Bypass lab A臂实录"
description: "ECB 固定 IV 的 notification oracle：长度探针判模型、解密回显槽验块、拼 cookie 得 administrator"
last_updated: 2026-10-10T04:33:25+08:00
created: 2026-10-10T04:33:25+08:00
---

## lab-logic-flaws-authentication-bypass-via-encryption-oracle (arm A)

- 实例入口：`range_launch launch-url /web-security/logic-flaws/examples/lab-logic-flaws-authentication-bypass-via-encryption-oracle --jar /tmp/cj1.json` → `https://0a97001d04e87d32806e1c6e008e0078.web-security-academy.net/`
- 两个加密面：登录页勾 Stay logged in → `stay-logged-in` cookie（base64、32 B）；评论表单 email 非法 → 响应 `Set-Cookie: notification=<base64 密文>`，页面回显 `Invalid email address: <email>`。
- 模型判定只看密文长度：输入 1/4/16/40 字符 → cookie 32/32/48/64 B = pad32(23+len)，即 明文 = "Invalid email address: "（23 B）+ 输入，无 IV 前缀。15 字符输入给 48 B 是排除 IV||C 模型的判别点（IV||C 会预测 32 B）。
- 模式判定：重复块探针（email = 9 字填充 + 两个相同 16 B 块）→ 密文第 3、4 块相同 → ECB + 固定 IV（确定性：同明文必同密文，块可跨调用搬运）。
- 解密回显槽：把任意值塞进 `notification` cookie，页面直接回显其明文——先把 `stay-logged-in` 的值喂进去，回显 `wiener:1791577705588`，明文格式确认。
- 伪造：ECB 下按 16 B 对齐取块。调用一 X = 9 填充 + `administrator:17` → 取块 3；调用二 X = 9 填充 + `91577705588` + 5×0x05（合法 PKCS7）→ 取块 3；两块拼接 base64 = 32 B cookie。
- 验证链：notification 槽回显 `administrator:1791577705588` → `GET /admin`（只带伪造 cookie）200 出用户面板 → `GET /admin/delete?username=carlos` 302 → `banner_verdict` solved:true。
- 工具：项目层件 `blk_forge`（`blocks` / `dupes` / `take --n` / `cat` / `xor` / `selftest`），覆盖 base64 分组查看、重复块判定（ECB）、取块拼接、等长 XOR（CBC IV 重建形态）。
- 可复用要点：密文长度是模型判别器（有无 IV 前缀差 16 B）；解密回显槽等于免费明文确认面；ECB 把加密 oracle 变成任意块装配器，无需密钥。

