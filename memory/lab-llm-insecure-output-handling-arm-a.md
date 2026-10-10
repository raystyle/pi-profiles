---
metadata:
  node_type: memory
name: "Lab LLM insecure output handling arm A"
description: "LLM 不安全输出处理 XSS lab 一次过:零引号注入串 + import(this.src) 取件载荷"
last_updated: 2026-10-10T17:10:24+08:00
created: 2026-10-10T17:10:24+08:00
---

## 2026-10-10 lab-exploiting-insecure-output-handling-in-llms (arm A, solved)

- 路径 /web-security/llm-attacks/lab-exploiting-insecure-output-handling-in-llms,range_launch launch-url 直起实例(reused:false),约 98s 内解到 banner is-solved。
- 题面机制:聊天助手只有 password_reset / product_info 两个函数;product_info 返回该商品的 reviews 数组,助手回复在聊天页由 chat.js 的 contentCell.innerHTML 渲染 → 不可信输出直落 innerHTML。
- 注入面:商品评价(productId=1 皮夹克)+ 必过图形验证码。OCR tiny 把 6 位码读成 34DZun 被打回 invalid-captcha,肉眼读图得 4Roirc 才过;验证码一次失败即轮换,失败后必须重新取页 + 重读。
- 载荷设计(关键):评价里的注入串要让模型逐字回吐,而 reviews 经 JSON 转义会把双引号变成反斜杠引号。因此注入的 HTML 全串零引号:img 标签,src 指向 exploit server 的 e.js,onerror 调 import(this.src)。
  - import() 在页面 realm 执行模块,跨域取回的模块仍以 lab 源运行,同源 fetch 自带 cookie;
  - 恶意 JS 放 exploit server:Head 设 Content-Type application/javascript 与 Access-Control-Allow-Origin *,File 路径 /e.js;存储表单字段 responseFile/responseHead/responseBody + formAction=STORE,缺 responseFile 会 400;
  - 模块体:GET /my-account 用正则抓 csrf 值,再 POST /my-account/delete。
- 结果:carlos 会话里 img onerror 触发,模块以 carlos 会话删掉 carlos,破题。AI 日志 /openai/logs 可见受害者提问 Tell me about the reviews for the product with id 1 与助手原样回吐的标签,是链路直接证据。
- 复用点:凡「模型逐字回吐载荷」的注入题,优先选零引号 HTML,把 JS 全搬到 OOB 服务器用 import(this.src) 取件;不要在回吐串里塞 base64,长随机串模型会抄错。

