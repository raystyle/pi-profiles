---
title: 反序列化族:gadget 链与 polyglot 入口
---

# 反序列化族:gadget 链与 polyglot 入口

同一类型漏洞:不可信字节被反序列化成对象,魔术方法/属性注入触发 gadget
链。族内入口两分:会话/参数直接携带序列化串,与文件解析(`phar://`)隐式触发。

## 子型判型矩阵

| 子型 | 判型特征 | 手法方向 | 实录 |
|---|---|---|---|
| PHP 自研 gadget 链 | 题面给带魔术方法的类源码 | 按 `__destruct`/`__wakeup` 构造对象链(批 32 solved) | [[lab-deserialization-developing-a-custom-gadget-chain-for-php-deserialization]] |
| PHAR 反序列化 | 有文件上传 + 可触发 `phar://` | JPEG+PHAR polyglot,metadata 内置链 | [[lab-deserialization-using-phar-deserialization-to-deploy-a-custom-gadget-chain]] |
| Java 自研 gadget 链 | 序列化 session + 站内 `/backup/` 泄漏类源码 | `readObject` 内 SQLi → 应用回显异常 → **报错型 SQLi 取数**(批 32 solved) | [[lab-deserialization-developing-a-custom-gadget-chain-for-java-deserialization]] |

## 共性

1. 判型:会话 cookie 是 base64 的序列化对象(Java)或题面给带魔术方法的类(PHP)。
2. PHP 侧:`phpser` 件按模板展开 NUL 字节(`%00` 私有/保护属性),出 base64/URL 编码;
   `b64` 解 data-URI;`bin_get` 抽 polyglot 内嵌 PHAR metadata(ascii run);`upload` 传二进制。
   **坑(批 10→32 教训)**:`phpser` 不重算长度,`s:<len>` 必须逐字节数对(NUL 也计 1);
   私有/保护属性名 = `1+类名长+1+属性名长`,差一位 PHP `unserialize()` 直接 false(表象是 500 `unserialize() failed`)。
3. PHAR 入口:上传 polyglot 后 `phar://<name>` 触发 metadata 反序列化(常在文件操作函数)。
4. Java 侧:序列化串用 `bin_get`/`b64` 读;自研链**照抄站内泄漏的类源码** + 仅编译用 stub,
   用真 JDK(`javac`→`ObjectOutputStream`→base64;件 `java_ser` 封装"编译源树→跑 Main→写 cookie jar")。
   类名须与目标一致(否则 `ClassNotFoundException`);应用强转失败(`ClassCastException`)不影响
   `readObject` 已执行——**异常回显就是副作用/取数通道**。

## 判定与收尾要点

- 判定锚点:目标动作实际发生(删文件、RCE、提权进 admin);序列化被解析不算解。
- 纪律:**题解目录不读**(如 serialization-examples 的 `solution/`),只用 `generic/` 骨架。

## 相关族

- 注入语境与模板 SSTI 见 seed injection-family(跨库不连,按名引用);
  方法论:web-vuln-methods(seed 层,按名引用)。
