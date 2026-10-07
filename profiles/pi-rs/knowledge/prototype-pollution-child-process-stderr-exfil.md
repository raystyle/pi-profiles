---
title: "prototype-pollution-child-process-stderr-exfil"
---

# prototype-pollution-child-process-stderr-exfil

# prototype-pollution-child-process-stderr-exfil

Node 服务端原型链污染触到 `child_process` 时的命令执行 gadget 与**免 OOB 外带通道**(实测于 `lab-exfiltrating-sensitive-data-via-server-side-prototype-pollution`,不需要 Burp Collaborator)。

## 命令执行 gadget

```json
"__proto__": {"shell":"vim","input":":! <shell 命令>\n"}
```

- `shell:"vim"` 使 runner 以 vim 当 shell;`input` 喂 vim 的 stdin,`\n` 结尾的 `:!` 行作为 Ex 命令执行 ⇒ 任意 OS 命令。
- 与 `execArgv` 型(`fork` 专用的 `--eval=require('child_process')…`)不是同一 gadget。先判 runner:`execArgv` 污染无影响 ⇒ 不是 `fork` ⇒ 改用 `shell`/`input`。

## 外带信道:任务 API 的 `error.message`

- 触发端点(维护任务)失败时,响应把 **Node `Command failed: <cmd>` + 完整子进程 stderr** 放进 `error.message`。
- 因此"把输出写 stderr"就是回显:
  `:! cat <目标文件>; cat <目标文件> 1>&2; cat <目标文件> | base64 -w0 1>&2`
- vim 非终端运行必然以 code 1 失败 ⇒ 该信道稳定。明文可能被终端转义粘成一行,**用同一轮的 base64 段反推明文边界**(base64 段解码即明文,可定位明文起点)。
- 把文件写到 webroot 再取的做法通常失败(child 的 CWD 下没有 `resources/`、`image/` 等目录,报 `No such file or directory`)⇒ 优先用 stderr 信道。

## 纪律

- 污染后先读 source 端点的回显确认键已落 `Object.prototype`。
- 触发端点多为 JSON 体;表单编码会 500 —— 别把 500 当"污染生效"。

## Links

- evidences: [[prototype-pollution-family]]
