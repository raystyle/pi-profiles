---
title: lab-scanning-non-standard-data-structures
---

# lab-scanning-non-standard-data-structures

> evidences: [[web-vuln-methods]]

- 题面:漏洞藏在"非标准数据结构"里(需 Burp「Scan selected insertion point」),手动利用后删 carlos;wiener:peter。
- 实例(批51):https://0a4f000003d659d386b6de6b00110042.web-security-academy.net
- 判定:**stuck**(cookie 两半的解析/定位语义已量化死;真正可注入的非标准结构仍未找到)

## 应用面(blog 克隆)

`/`、`/post?postId=N`、`POST /post/comment`(csrf/postId/comment/name/email/website,全部 HTML 转义)、`/login`(hidden csrf)、`/my-account`(显示 `Your username is: X`)、`/my-account/change-email`(email + csrf)、`/admin`(401)、`/logout`,`/resources/static/*`。

## 会话 cookie = 唯一"非标准结构":`<username>%3a<32 位 base62 token>`

批51 把它的语义钉死(纠正批50 的判读):

1. **500 的成因是"两半不匹配",不是"用户名未知"**:身份由 **token** 决定,app 要求 cookie 里的 username 与该 token 对应用户**相等**,不等即 500。故**任何** username 半的改动(含 `wiener' OR '1'='1`、`%27%20OR%201=1--%20-`、`wiener' UNION SELECT 1,2,3,4,5,6,7,8-- -`、`wiener'-- -`、``wiener'' OR ''1''=''1``)都是同一个 500(digest aa136edf6af38e3e)——这些读数**不能**当作"用户名不可注入"的证据,它们全被 mismatch 掩盖。
2. **username 半不可注入**(与上条独立):注入形状与真值形状给出同一个 500 类,没有 SQL 错误类可分。
3. **token 半是精确匹配**:`<tok>' OR '1'='1` / `' OR 1=1-- -` -> /admin 401、/my-account 302(未登录);token 截断 8 位、大写化 -> 302 ⇒ 无前缀/大小写宽容。
4. **重复 session cookie:后一个生效**(`session=administrator:<tok>; session=wiener:<tok>` -> 200 正常 wiener 会话;反序 -> 500),无 split-brain。
5. **cookie 里塞第三个冒号**(`wiener%3a<tok>%3aadministrator`)-> 401(= 未登录),无越权。

## 其余插入点

- `/my-account/change-email` 首次被扫:10 值(sqli/ssti/cmdi/crlf/xss/path)全 302(Location `/my-account?id=wiener`)无差分 —— 该槽只存不判。
- `/admin/delete?username=carlos` 直接带 wiener 会话 -> 401,动作端点也过角色门。
- cookie 的 CRLF 面(批50 四腿:原样 200、username 内 `%0d%0a` 500、裸 CRLF 302+新匿名 cookie、token 尾 `%0d%0a` 302)仍判**关闭**。

## 复现命令

```
http_session get <inst>/login --jar j          # csrf + 匿名 cookie 形:`:<32位>`
http_session post <inst>/login --form csrf=… --form username=wiener --form password=peter --jar j
raw_matrix @cookie-battery.json   # username/token 两半的注入矩阵(见上四类判读)
form_sweep <inst>/my-account/change-email email 'x@x.com,…,x@x.com|id' --field csrf=… --jar j   # 只存不判,全 302
```

## 关系

- 族:[[web-vuln-methods]];姊妹题 [[lab-discovering-vulnerabilities-quickly-with-targeted-scanning]];应用形状见 [[portswigger-burp-scanner-essential-skills-labs-app-shapes-and-traps]]。
