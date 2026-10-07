---
title: lab-race-conditions-partial-construction
---

# lab-race-conditions-partial-construction

> evidences: [[race-conditions-family]]

- 题面:Partial construction race;绕邮箱验证建号 → 登录 → 删 carlos。
- 实例(批50):https://0af3008703f359bc8471ef7100ff00f6.web-security-academy.net
- 判定:**stuck**(可见性探针实验已跑,但**探针自身即写者 ⇒ 实验被污染**;token 值面仍是墙)

## 本实例端点事实(批50 实测)

1. **注册页两值类**:新 INSERT `2636B`(`Please check your emails for your account registration link`);
   重复用户名 `3142B`(`An account already exists with that username`)。
2. **校验顺序 = email → 存在性 → INSERT**:`username=probeA`(已存在)+ 非法 email → `Invalid email address`(3119B);
   `username=probeA` + 合法 email + 1 字符密码 → **DUP(3142B)** ⇒ 密码无校验门。
   ⇒ 找不到「过存在性检查却因其它校验被挡」的**非变异**探针。
3. **登录被确认门挡死**:未确认账号 `POST /login`(probeA/pw) → 200 登录页(无 302,body 3801)
   ⇒ `/login` 不可作可见性探针。
4. **email 客户端只显示 `@exploit-<id>.exploit-server.net`**(Inbox is empty);而注册白名单要求
   `@ginandjuice.shop` ⇒ **无 token 可读化原语**(prescript 的 ② pivot 无落点)。

## 可见性实验(按 prescript 跑)

```
race_spread <inst>/register --method POST --body 'csrf=…&username=racew1&email=racew1@ginandjuice.shop&password=pw12345678' \
  --window-ms 12000 --workers 16 --class NEW:2636,DUP:3142 --hit-substr 'already exists' --extra-header 'Cookie: phpsessionid=…'
race_send <inst>/register --form csrf=… --form username=racew1 … --n 20 --jar <jar>
```

读数:read 侧 **sent 153 / DUP 149 / NEW 4**;write 侧 20/20 **DUP**。

**判读(本批关键)**:该实验**被污染** —— `/register` 探针本身就是写者:它的 INSERT 提交后,
后续探针全看到 DUP。write 侧 20/20 DUP 正是因为 read 侧探针已把 `racew1` 建好。
⇒ 「DUP 独有串命中」**不能**判 ②(行可见);同理「read 侧全 NEW」也几乎不可能出现。
**纪律:可见性探针必须是非变异的**(本实例无此端面,/register 做不到)。

## 仍在档的正面事实

- 注册行**提交很快**:同一波并发里,后到的存在性检查已能看到前者的 INSERT(153 探针 149 DUP)。
  ⇒ 假设①「INSERT 长事务未提交(包住慢发信)」**不像**成立;更像行早早提交,而 `token` 值不是 `''`
  (NULL 或插入即设)⇒ 绑空串的 `token[]=` 永不可中(累计 ≈530 次零命中,跨 h1 齐发/读侧播撒/h2 单包)。
- 未决面已收窄为「token 值/可读化」,**不是时序也不是样本量**;本 lab 无该原语 ⇒ 依纪律关闭本题轮次。

## 复现命令

```
http_session get <inst>/register --jar <jar> --out /tmp/reg.html            # csrf
http_session post <inst>/register --form username=probeA --form email=probeA@ginandjuice.shop … # 2636
http_session post <inst>/register --form username=probeA --form email=x@ginandjuice.shop …     # 3142
```

## 关系

- 族:[[race-conditions-family]];方法见 [[http-2-single-packet-race-burst-method]]。
