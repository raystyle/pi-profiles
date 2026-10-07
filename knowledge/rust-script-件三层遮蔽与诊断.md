---
title: "rust-script 件三层遮蔽与诊断"
---

# rust-script 件三层遮蔽与诊断

Keywords: rs_execute, script catalog, bundled, project, global, shadow, doc_read, envelope

同名 rs 件按 bundled < global < project 三层覆盖:项目件层(`.pi-rs/rust-scripts/`,信任即生效)一旦存在同名件就直接赢,bundled 新版从此不执行,且无旁路开关。调试期的旧项目件副本是最常见的遮蔽源。

**判定手法**
- 按信封字段指纹判定跑的是哪一层:同一件名不同版本字段集不同(如 doc_read 的 `pages_needs_ocr`/`page_units`/`engine` 只在新版有);字段缺失优先怀疑遮蔽,其次才是功能故障。
- 按名比对三层:`diff .pi-rs/rust-scripts/<n>.rs <scripts 根>/<suite>/<n>.rs`,版本号与字节数一眼分辨。
- 验证被遮蔽的 bundled 件:另建临时副本手动注入 path 依赖(`materializeScript` 运行时也是这么做的),或直接删掉项目件副本解除遮蔽。
- `~/.pi-rs/agent/rust-materialized/` 是物化缓存;内容陈旧不代表 bundled 旧,以源码三层为准。

**件源构建契约(踩过的坑,写件自查)**
- `mdka` 3.2 只导出 `html_to_markdown` 系列(含 with 与 many 变体),没有 `from_html`。
- `let Some(x) = y else { report::failure(..) }` 的 else 分支必须发散:`report::failure` 返回 `()`,其后要补 `std::process::exit(1)`。
- `anydoc::to_markdown_bytes` 已返回 `Result<String, _>` 类型,不需要 `from_utf8_lossy` 再转。
- `doc_find` 命中 0 时仍打信封但进程退出 1(grep 纪律),不算件故障。

关联:
- [[tools/script-plane]]: 件目录分层与执行契约的出处。
- [[crates/mdka]]: mdka 的键 API 面。
