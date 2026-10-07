---
title: browse cdp 注入库:rs 件操作浏览器
---

# browse cdp 注入库:rs 件操作浏览器

`cdp` 是 browse_rs 的 CDP 连接层 crate(仓 `crates/cdp`),只做 browser 级
WebSocket 会话、flatten attach 与 sessionId 路由,不含 `goto()`/`click()`
语义封装;单文件 rs 脚本经 rs_execute 运行时,调用方直接写
`session.call("Page.navigate", params)`。高层方言用法见 browse-dialect.md。

## 注入机制(脚本源不含 path 依赖)

- 脚本源只写 `cdp::` 前缀,不声明 `[dependencies]` 的 path 依赖。
- rs_execute 运行时物化一份副本(agent 目录 `rust-materialized/`):把
  `cdp = { path = "<crate 目录>" }` 插进 cargo 块(已有
  `//! [dependencies]` 头则其下插入,否则于 `//!` 头内新建 cargo 块)。
- crate 目录解析序(ADR-0011):包内 vendored 副本
  (`vendor/cdp`,随 npm/bun 分发)为缺省;`PI_BROWSE_RS_DIR` 指到的
  browse_rs 检出可覆写(开发迭代用);`~/repos/browse_rs` 传统检出兜底。
  无需本机检出 browse_rs,vendored 副本即够编译。
- 源保持机器无关、可提交;脚本自带 `//! cdp = ...` 行即跳过注入(自声明),
  不用 `cdp::` 则原样运行。`pi_rust_lib` 走同一机制。

## 核心 API 面

- `Session::new() -> Arc<Session>`;`session.connect_opts(ConnectOptions{..})`
  连接 browser 级端点(非某个 tab)。线索三选一:`ws_url` / `port` /
  `profile_dir`(读该目录 `DevToolsActivePort`);`timeout_ms` 缺省 5000。
- 引擎拉起:`cdp::spawn::spawn_engine(chrome, profile_dir, headless, extra_args)
  -> Result<Child>` 起隔离 clean-chrome(`--remote-debugging-port=0`、`--no-sandbox`);
  `std::mem::forget(child)` 让它活过脚本(常驻)。`headless` 形参决定无头,
  无显示会话(ssh)下即使传 false 也会自动补 `--headless`。
  `cdp::spawn::wait_devtools_ready(profile_dir).await` 拿 WS URL(冷启动上限 15s)。
- 标准起法(见 web_fetch.rs `attach()`):先 `connect_opts` 试附着,Err 再
  `spawn_engine` + 加大 `timeout_ms` 重连。
- `session.call(method, params: serde_json::Value).await -> Result<Value>`:
  任意 CDP 命令直发;`Browser.`/`Target.`/`Storage.`/`SystemInfo.`/`Tethering.`/
  `Tracing.`/`Extensions.` 前缀走 browser 端点(不附 sessionId)。内建守卫拦
  `Browser.close` 与非自建 tab 的 `Target.closeTarget`;单次调用 30s 超时。
- `list_page_targets().await -> Vec<PageTarget>`(target_id/title/url/type_/own);
  `use_target(id).await` 选定活动 tab(仅 own=true 可关)。
- 事件面:`wait_for(method, ms).await`、`peek_events(method, n).await`;
  事件环形缓冲上限 1000 条。

## 异步运行时要求

API 全是 async(tokio)。脚本自建运行时并 block_on:

```rust
let rt = pi_rust_lib::tokio::runtime::Builder::new_multi_thread()
    .worker_threads(2).enable_all().build().unwrap();
rt.block_on(async { /* ... */ });
```

外层用 `pi_rust_lib::tokio::time::timeout(Duration, ..)` 配 `PI_TIMEOUT_SECS`
兜底。常用域:`Page.enable/navigate/captureSnapshot`;`Network.enable` 后读
`Network.responseReceived` 取 status/终址/headers;`Runtime.evaluate` 取值。
