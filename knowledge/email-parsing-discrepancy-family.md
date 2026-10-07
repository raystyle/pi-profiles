---
title: "email-parsing-discrepancy-family"
---

# email-parsing-discrepancy-family

族结论:邮箱校验器的**接受集** ≠ 投递可达集;分歧点常在**发信端(第二个 parser)的取域规则**,而不是校验器本身。校验器通过 ≠ 确认信投递成功;唯一 ground truth 是 exploit 邮箱客户端的收件表。

## 校验器:多道闸门 + 精确模型(实测 `POST /register`)

**闸门必须用 response 长度区分**,只 grep 文案会把"域不符"误判成"接受":

| 判定 | body 长 | 文案 |
| --- | --- | --- |
| 真接受 | 3126 | `Please check your emails for your account registration link` |
| 语法拒 | 3850 | `Invalid email` |
| 安全过滤 | 3878 | `Registration blocked for security reasons` |
| 域不符 | 3893 | `Only emails with the ginandjuice.shop domain are allowed` |

- 接受集(实测):域必须**恰好**等于允许域(小写、无尾点、无子域);local 为 RFC-5322 atext(`!#$%&'*+-/=?^_`{|}~.`+字母数字),
  拒绝 `_ : ; 空格 \ " ( ) [ ] < > ,`、双 `@`、非 ASCII。
- 校验器会**先解码 local 里的 RFC-2047 encoded-word 再校验**:解码后仍须是 atext(解码出 `@` → 语法拒);
  域侧**不解码**,原始串里必须有字面 `@`。
- 安全过滤只挡 `=[0-9a-fA-F]{2}`(Q-encoding);base64 形 `=?x?b?…?=`、UTF-7 形能过过滤(但仍受上面的 atext 约束)。

## 分歧(族的核心结论)

- ②③ 那类"看起来被接受"的形态(`X@D.E`、`X@E.D`)实测落在 **3893 = 域不符**,不是接受。
- 投递侧从未到达:UUCP bang path `E!x`、percent hack `x%E`(裸/编码)、base64 解出 `@`/地址表的形、`%2540` 双编码 ——
  投递的收件域始终是允许域 D。第二个 parser 的取域规则仍是未决面。

## 探法(效率)

一次 POST 矩阵(`cache_probe` / `form_sweep` / `lab_http`),每行换 username,**按 body 长度**分类四个闸门;
再单独 `GET /email` 看谁真的收到。两步分开,别把"过校验"当"成功"。

## Links

- evidences: [[records/lab-logic-flaws-bypassing-access-controls-using-email-address-parsing-discrepancies]]
