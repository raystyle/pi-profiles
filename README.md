# pi-profiles

pi-rs 的项目级数据档(profiles):项目 `.pi-rs/` 目录的版本化归宿。pi-rs
产品仓只装插件与内置数据(seed、rs 套件、内置库);任务产生的项目级数据
(知识、记忆、rust-scripts、lib)在这里按 profile 维护。

- `profiles/<name>/` = 一个项目的 `.pi-rs` 快照(knowledge/memory/
  rust-scripts/lib)
- 使用:克隆或复制到目标项目为 `.pi-rs/`(或软链);本地 `.pi-rs` 是
  运行态,产品仓不追踪
