# 项目级 lib(.pi-rs/lib)

rs 件间重复的逻辑沉淀为仓内共享 crate,与内置 `pi-rust-lib` 同级的一层。
件里只写 `pi_project_example::percent_encode(...)` 这类名字引用,rs_execute
物化时自动注入 `path` 依赖(与 `pi_rust_lib`/`cdp` 注入同机制,扫描方
`projectLibs`)。

| crate | 职责 | 来源 |
| --- | --- | --- |
| `pi_project_example` | `percent_encode`(RFC 3986 非保留集)与 `form_encode`(x-www-form-urlencoded 体) | phpser 的 url_encode 与 race_send 的 encode_forms 去重 |

分层口径:件内一次性的逻辑留在件里;两件以上重复即迁 lib 层;再经实战验证
通用后,由自省策展升入内置 `pi_rust_lib`(不自动灌入)。lib 层有三档:
原生注入(pi_rust_lib/cdp/jc 等产品 crate)、项目级(本目录,随仓走)、
全局级(`~/.pi-rs/agent/lib/`,跨项目个人库;同名项目级遮蔽全局级)。
新增 crate 即建子目录(`Cargo.toml` + `src/lib.rs`),件按 `[package].name`
引用,多 dep 并存。
