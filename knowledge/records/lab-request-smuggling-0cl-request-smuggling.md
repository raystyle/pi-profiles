---
title: lab-request-smuggling-0cl-request-smuggling
---

# lab-request-smuggling-0cl-request-smuggling

> evidences: [[h2-smuggling-family]]

- 题面:0.CL request smuggling;Carlos 每 5s 开首页,让他执行 `alert()`。基于 Kettle《HTTP/1.1 Must Die》。
- 实例(批50):https://0a450054042817e780fd179b00380068.web-security-academy.net
- 判定:**stuck / infra-blocked**(20 候选零 holding ⇒ H-V 前提在本 infra 否证;停止投轮次)

## 20 候选全扫(新实例 `desync_probe <inst>/ --jar <jar>`,判据 = 同连接「该请求 + 探针 GET」响应条数)

| 类 | 候选(响应数 → 状态) |
| --- | --- |
| **0 响应(前端按真 CL 等 body ⇒ 探针永不被应答)** | `cl-colon-space`、`cl-tab-name`、`cl-lower` |
| 1 响应 400 | `cl-dup-diff`、`cl-obs-fold`、`cl-space-val`、`cl-negative`、`cl-hex`、`cl-empty-val` |
| 1 响应 403(边缘拒) | `cl-plus-te`、`te-plus-cl`、`te-bare`、`te-x`、`te-colon-space`、`te-dup`、`te-obs-fold`、`te-identity`、`te-chunked-case` |
| 1 响应 200(普通页 8470B) | `cl-plus-sign`、`cl-leading-zero` |

`suspects = []`。

## 判读

- 0 响应那三格**不是**"无人 holding",而是**前端解析了畸变 CL 名并按真 CL 等 body**(探针被吃进 body、
  自己拿不到响应)—— 即"前端计数"。0.CL 要求「前端**不**计数、后端计数」,恰相反 ⇒ **H-V 头 holding
  前提在本 infra 不成立**(跨批 44/47/49/50 稳定)。
- `cl-lower`(小写 `content-length`)也是 0 响应,与批49 把 `cl-lower` 记为 1 响应(200)不同 —— 实例间
  该格行为有漂移,但都不产生 holding。
- 结论与蒸馏一致:**本 infra 不存在第一跳 0.CL holding 原语**,整套双 desync 编排无米之炊 ⇒ 记
  infra-blocked,不再投轮次。判据面如要复用,应把 `desync_probe` 的三态(answered_probe / first_byte_ms /
  class ∈ front_desync|backend_hold|front_hold|normal)补上,把 0 响应单独标成 `front_hold` 而非"非 suspect"。

## 复现命令

```
range_launch launch 4BAD74C3…EFC8CC66 --jar <jar>
desync_probe <inst>/ --jar <jar> --read-ms 2500
```

## 关系

- 族:[[h2-smuggling-family]]、[[request-smuggling-family]];方法见 [[h2-tunnelling-and-h2cl-practice]]、[[desync-delivery-deficit-and-window-law]]。
