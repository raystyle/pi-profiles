---
metadata:
  node_type: memory
name: "Piece increment acceptance 42964415f"
description: "自省验收件增量 @42964415f(新会话新树):item1/3 过,item2 功能过但版本 1.2.2≠1.2.1,item4 raw_matrix --values 过 / range_launch 版本缺;根因=HEAD 8b588fd00 已越过增量提交"
last_updated: 2026-10-09T00:39:21+08:00
created: 2026-10-09T00:39:21+08:00
---

自省验收「件增量 @42964415f」,新会话新树,权威 AGENTS.md。逐项实测:

- item1 raw_matrix --selftest(1.1.0,--values 面盘展开):**过**。信封 {"data":{"selftest":"ok"},"next_suggestion":"sweep expansion covers line/headers/body/name","success":true}。
- item2 range_launch --selftest(1.2.1,体锚回捞):**过(功能面)/版本漂移**。实测版本 1.2.2,selftest {"data":{"selftest":"ok","body_candidates":["https://0aabc123.web-security-academy.net/","https://0bbcd456.web-security-academy.net/x"]},"next_suggestion":"instance-anchor candidates enumerate (hop short-circuit is chain-level)"} —— 体锚回捞面在。版本非 1.2.1 的原因:当前树 HEAD=8b588fd0007d 是 42964415f 的子提交。
- item3 raw_matrix --values 实测(example.com + @file 2 值词表):**过**。envelope data.variants=2,data.rows 长度 2,i0 name="alpha" i1 name="beta",两条 status=200/bytes=589/digest 同值;name 逐条等于词表值。
- item4 件目录面:raw_matrix args 行含 `--values <a,b,c|@wordlist>` = **过**;range_launch 版本断言 1.2.1 = **缺**(实测 1.2.2)。

根因与证据(全走件,未改文件,未提交):
- `git log --oneline 42964415f..HEAD -- .../hunter-suite/range_launch.rs` → 仅 8b588fd00「kimi G1+G2 - range_launch 1.2.2 hop short-circuit + candidates, find_files 1.1.2 arg-order self-heal」。
- `git show 42964415f:.../range_launch.rs | head` → `//! version: 1.2.1`(增量提交自身确为 1.2.1);`git show 42964415f` diff 证其新增 --selftest(body_fallback)与 find_academy_url 体锚回捞 + instance_from_body 标。
- `git merge-base --is-ancestor 42964415f HEAD` → yes,故当前树是增量的超集,版本面已被后续提交推进。

可复用配方:
- raw_matrix 模板 `{{V}}` 槽(行/头/体三面),--values 走内联 `a,b,c` 或 `@file`(每行一值,`#` 注释跳过)叉乘展开;spec `{host,port=443,tls=true,marker?,snippet?,variants:[{name?,line,headers?,body?}]}`;模板 name 空时,展开后的 row.name == 值,非空则 `tpl:值`;信封 data.variants=行数,data.rows 逐行 status/bytes/digest。
- 判定件增量前先 `git merge-base --is-ancestor <增量> HEAD` + `git log <增量>..HEAD -- <件路径>`:若树已越过增量,版本面断言应按「增量提交原文」与「运行树实跑」两栏分别回执。

纪律:一切走件;未改仓内文件(仅 /tmp 临时词表,已删);未 git 提交;未读题解/prs_c2coe/practice/eval 档案。

