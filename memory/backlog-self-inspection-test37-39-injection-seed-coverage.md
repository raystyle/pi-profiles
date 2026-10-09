---
metadata:
  node_type: memory
name: "Backlog Self-Inspection Test37-39 Injection Seed Coverage"
description: "注入三连@65a5f6887:分隔符命令注入 5/23(临界达标但 P 层无具名条目)、输出重定向回读 2/26(直达但 P5 无落盘回读信道)、时间延迟 oracle 2/27(直达 P5 无缺口);建议 P22/P21 补分隔符词表、P5 补第四信道。上一轮 P12/P15 建议已原文落地"
last_updated: 2026-10-08T22:06:31+08:00
created: 2026-10-08T22:06:31+08:00
---

## 2026-10-08 回溯自省台(注入三连 37-39)@HEAD=65a5f6887

版本面貌:HEAD 推进到 65a5f6887「gitlink bump (lab 39)」。seed tactical-patterns 相对 598ec962f +7/-4 已提交,内容正是上一轮(题27-29)的两条建议原文落地:P12 触发词 += 「**跳过确认步/直发末步**」;P15 触发词 += 「**隐藏字段的价格篡改**(客户端提交价藏在加购表单)同落本条」,并把反例改为「负合计被显式拒时先试回调幅度落正带,卷带是幅度不可调时的退路」(pi 线 2)。P 条总数仍 24。

注入三连实测(seed 根 universe=64,iwe find --lexical):
- 「分隔符 命令 注入」→ tactical-patterns **rank 5/23**(临界前 5)= 达标但只靠通用词命中;top5 = injection-family>ssti-family>h2-tunnelling-and-h2cl-practice>host-header-family>tactical-patterns。P 层无具名覆盖命令分隔符词表(全文「分隔符」仅 P6 的 url_fuzz 缓存界符)。
- 「输出 重定向 回读」→ tactical-patterns **rank 2/26** = 直达;top5 = cache-poisoning-family>tactical-patterns>injection-family>request-forgery-family>js-reverse-debugging。但 P5 三梯只列 错误文本/时延/出网,无「输出重定向落盘 -> GET 回读」信道具名(全文无「重定向/回读/落盘」)。
- 「时间 延迟 oracle」→ tactical-patterns **rank 2/27** = 直达 P5(时延梯 + 变体注「命令注入时延」+ 题20/21 长度梯抖动读法边界),无缺口。

缺口建议(两条,均为措辞缺口):
1. P22(同面多形逐投)/P21(载体选型律)补:「命令分隔符面 —— `;` `&&` `||` `|` 反引号 `$()` 换行 `%0a`,按过滤缺口逐形投;被拒不等于面不通」。
2. P5 变体注补第四信道:「输出重定向落盘到 web 可写路径 -> 直接 GET 回读(件 `http_session`;判据=文件内容即数据),与错误文本/时延/出网并列」;或将该信道并入 P18 独立判定面。

