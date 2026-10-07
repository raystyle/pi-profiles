---
title: CSRF 族:SameSite 与令牌面
---

# CSRF 族:SameSite 与令牌面

同一类型漏洞:状态变更请求未真正绑定用户意图。缺陷在 SameSite 策略、
CSRF token 绑定、或允许同源表单代提交。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| SameSite=Strict 旁路 | 会话 cookie Strict,主站有兄弟域 | 兄弟域(同 registrable)XSS 发同站 websocket,携 Strict cookie | [[lab-samesite-strict-bypass-via-sibling-domain]] |
| 同源表单劫持 | 严格 CSP 但 `form-action` 未设 | 注入按钮让受害者**自身**同源提交 | [[lab-very-strict-csp-with-dangling-markup-attack]] |

## 共性

1. SameSite 按 **registrable domain** 判 same-site:兄弟子域即 same-site,
   可在兄弟域上找反射 XSS 作跳板,再发同站请求。
2. CSP 必须逐指令看:`form-action` 缺失即表单可外提/同源提交;token 绑定会话,
   跨会话复用会 400,故正解是让受害者自身提交。

## 判定与收尾要点

- 判定锚点:受害者账户状态被改/以受害者身份登录;请求被接受不算解。
- 交付后用 `solved_check` 收口。

## 相关族

- 点击劫持(同属受害者本尊请求)见 [[clickjacking-family]];XSS 语境见 [[xss-context-family]]、[[dom-xss-family]];
  方法论:web-vuln-methods(seed 层,按名引用)。
