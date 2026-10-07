---
title: 点击劫持族:叠层、对齐与 frame 防护
---

# 点击劫持族:叠层、对齐与 frame 防护

同一类型漏洞:受害者本尊点击被诱导落在敏感动作控件上。缺陷在帧防护缺失/
可绕过;利用面是 iframe 叠层 + 诱饵对齐。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| 表单预填 | 邮箱由 URL 参数预填 change-email | iframe 指预填页,诱饵钮对齐 Update email | [[lab-prefilled-form-input]] |
| frame buster | `if(top!=self)` 清空文档 | iframe `sandbox="allow-forms"`(禁脚本) | [[lab-frame-buster-script]] |
| 多步/确认页 | 动作分两步 | 用 `iframe.onload` 计数,门控第二击 | [[lab-multistep]] |
| 点击触发 DOM XSS | 敏感动作页带 DOM XSS | 诱饵点击提交表单引爆 | [[lab-exploiting-to-trigger-dom-based-xss]] |

## 共性

1. 量控件矩形(`browse` 取坐标),诱饵 `position:fixed` 对齐;iframe 半透明在上(z-index 2)。
2. 叠层语义:**bot 是坐标(命中测试)点击**,落在最上层元素;顶层点击信标无命中即证此为坐标点击。
3. frame buster 用 `sandbox="allow-forms"` 禁脚本绕过,表单仍可提交。

## 判定与收尾要点

- 判定锚点:受害者邮箱/状态实际被改(`solved_check` true);交付后 verified。
- 交付:`formAction=DELIVER_TO_VICTIM --follow`,读 `/log` 看 bot 访问。

## 相关族

- DOM XSS 引爆见 [[dom-xss-family]];CSRF 与本族同属「受害者本尊请求」见 [[csrf-family]];
  方法论:web-vuln-methods(seed 层,按名引用)。
