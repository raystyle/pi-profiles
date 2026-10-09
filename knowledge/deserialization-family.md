---
title: 反序列化族:识别记号、三栈手法与改后生效判读
---

# 反序列化族:识别记号、三栈手法与改后生效判读

Keywords: deserialization, insecure deserialization, gadget chain, rO0, php object injection, marshal, cookies, serialization, 反序列化

层位:判型归 [[web-vuln-methods]] 注入族;本层写打法:先认栈(识别记号),
再选手法(改位/改类型/功能滥用/gadget 链),终判「改后对象须真实生效」。
[[injection-family]] 的反序列化单行升格本节;campaign 实录 143-149 为疾影线。

## 识别记号(先认栈再动手)

- Java:`rO0AB` 开头的 base64(ObjectOutputStream 魔数 ac ed 00 05),常见于
  cookie/session 参数;十六进制原始形 `aced0005`。
- PHP:`O:<len>:"<class>":<n>:{...}` 对象串或 `a:<n>:{...}` 数组串,常见于
  cookie;base64/hex 包裹时先解外层。
- Ruby:`\x04\x08` 开头的 Marshal 原始串(hex 形 `0408`),base64 包裹常见。
- 未知栈:魔数直查(上面的四种字节形)比试错快;改位前先原样重放确认
  「服务器接受我构造的序列化串」这一前提。

## 手法面(按代价升序)

1. **改位提权**:对象里带角色/管理员位的字段直接改值重序列化;前提是校验
   读该字段而构造无签名。
2. **改类型**:序列化格式里的类型声明与实际值错配(PHP `i`/`s` 标签互改、
   loose comparison 恒真形 `0 == "string"`)绕过校验。
3. **功能滥用**:改对象内属性(如文件路径/收件人)让既有方法做开发者没想
   到的事,无需新代码执行。
4. **gadget 链**:调起应用可达类链执行命令/请求;PHP 常见预建链、Ruby 有
   通用公开链、Java 依赖具体库版本(Apache Commons 系最著录)。外部生成
   优先(公开 gadget 生成器/ysoserial 族),本仓不自造链,即 件面只做投递。

## 件组合

- 侦察/改位/改类型:`http_session` 取序列化串 + 文本手术刀改字节 +
  `http_session` 回投;URL-safe base64 变体注意 `+/` 与 `-_` 的互转。
- Java 投递:`java_ser`(本地编译序列化对象出 base64,可直写会话 cookie)、
  `jclass`(类装配/stored jar)。
- 预置字典/链外生成后的投递走 `http_session`/`upload`;gadget 生成本身
  外部化。

## 判读边界(反例面)

- **改后对象须真实生效于受限动作**(改完 admin 位只回放登录页不算成,
  必须让一个受权限控制的动作(删号/改邮箱/受限页)以新身份真实完成;
  ):判定锚同横幅纪律(banner_verdict/终页)。
- 签名/加密壳在位时,改串即失效(回错或会话重置),即 先验壳(HMAC 尾、
  IV 结构)再谈改位。
- 魔数对不上时不要硬猜栈:改串失败的大头是栈认错,不是手法错。

## 相关

- [[injection-family]](判型面)、[[tactical-patterns]](P5 三梯 oracle:
  反序列化报错/时延/出网同落一律)、[[oob-callback-family]](gadget 链的
  带外验证信道)。
