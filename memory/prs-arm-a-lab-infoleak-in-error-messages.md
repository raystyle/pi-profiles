---
metadata:
  node_type: memory
name: "PRS arm A lab-infoleak-in-error-messages"
description: "arm A 基线:lab-infoleak-in-error-messages 冷实例一次通过 - /product?productId=abc 触发 500 Java 栈迹,body 末尾印 Apache Struts 2 2.3.31,POST /submitSolution answer=2.3.31 → correct:true,banner solved"
last_updated: 2026-10-09T06:50:20+08:00
created: 2026-10-09T06:50:20+08:00
---

## 2026-10-09 arm A baseline: lab-infoleak-in-error-messages

- 冷实例(reused:false),一次通过。
- 路径:`range_launch launch-url /web-security/information-disclosure/exploiting/lab-infoleak-in-error-messages --jar /tmp/cj1.json` 直接出实例,无需 page_read 取号(题面需要的只是「提交第三方框架版本号」,由 description 一句话给出)。
- 触发面:购物站商品页 `/product?productId=<非数字>`。
- 泄漏:`http_dump` GET `/product?productId=abc` → 500,body 为 Java 栈迹(`NumberFormatException: For input string: "abc"`),**body 末行直接打印 `Apache Struts 2 2.3.31`**(body 尾部独立一行,长度 1767 字节;http_dump 的 body_preview 只截到栈头,版本行须落盘 `--out` 后 read 全文才见)。
- 提交:POST `/submitSolution` form `answer=2.3.31` → `{"correct":true}`。
- 判定:`banner_verdict` 首页 → solved_class=true + `Congratulations, you solved the lab!`。
- 件组合:banner_verdict + http_dump + http_session(POST/submitSolution);无 exploit server,无登录腿。
- 坑:http_dump 默认预览不含 body 尾部版本行,不落盘会误判「只有栈迹无版本」;submitSolution 的答案只要纯版本号 `2.3.31`,不带 `Apache Struts 2` 前缀。

