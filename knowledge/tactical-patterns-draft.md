---
title: 战术模式草稿(跨题稳定的机制层模式)
---

# 战术模式草稿(跨题稳定的机制层模式)

> 层位:[[web-vuln-methods]](总纲/族路由)与六族判型矩阵之间的**战术模式层**。每条 = 判型触发 → 机制原理 → 件组合(能力面 + 件名)→ 实证面 + 反例面。
> 纪律:只收**跨题复现 >=2** 的模式(标实证题数);抽象到 HTTP/浏览器/DNS 语义,不绑实例域名/端点/题名;件以「能力面(件名)」双写;反例同等收录并标反例数。
> 状态:单向链下游草稿(候选),由编审并入 seed;本文件不改 seed。题面级细节留在 records/族档。

## A. 竞态族

### P1 写侧证窗 → 读侧播撒(两段式并发)
- **判型触发**:同端点多请求/会话内单令牌;校验与结算分处两端点;令牌随时间或按用户派生。
- **机制原理**:「读-判-写」非原子,窗口对**同刻到达**的并发可见。写侧(创建/校验)要**齐发**争同刻窗口;读侧(消费/查询)要**播撒**——目标只是撞上半态对象的任一时刻,齐发只在窗口起点撞一次。
- **件组合**:单包并发原语(`h2_burst`)或预热连接齐发(`race_send`)证写侧窗口可达(错投/超额/重复即窗口证据)→ 二值分桶(`race_spread --class` 体长类、`blind_oracle --classify`)→ 读侧按窗口时长播撒(`race_spread --window-ms/--interval-ms/--workers`)。
- **实证(3)**:race/single-endpoint、race/multi-endpoint、race/time-sensitive。
- **反例(1)**:race/partial-construction —— 写侧窗口可见(并发注册多 INSERT),读侧 confirm 跨三种并发模型(h1 齐发/播撒/h2 真单包)数百次全 miss ⇒ 不可见的是**半态本身**(NULL 或未提交事务),不是采样不足。此时换的是 oracle 维度,不是样本量。

### P2 真单包并发原语与其预算律
- **判型触发**:竞态动作应在 ms 级窗口内生效却恒 miss;多连接齐发仍落在窗口之后。
- **机制原理**:同一 TCP 报文内多 stream 由服务器 worker 并发启动,到站间隔从「TCP 握手级」压到「微秒级」;HTTP/1.1 多连接各带握手/调度抖动,难以真正同刻。
- **件组合**:`h2_burst`(一条 h2 连接、一次 write、独立 stream;`single_packet_likely` + `burst_bytes` 自证单包性)→ 预算 ≈1400B,超了拆包即失同刻。会话级串行(PHP 原生会话等)会吃掉同刻效果 ⇒ 每请求**独立会话**(各自 csrf),或改不带会话的腿。
- **实证(2)**:race/single-endpoint、race/time-sensitive(单包翻牌)。
- **反例(2)**:race/partial-construction(真单包 1319B、`single_packet_likely=true` 仍零命中 ⇒ 墙不在同刻);smuggle/0cl(该前置计数并等 body ⇒ 单包无收益)。

## B. 走私族

### P3 第一跳 desync 计数 oracle(先证畸变头吃不吃)
- **判型触发**:拟做双级失步/0.CL/H2.CL 编排,但不知该前置对哪种畸变头敏感。
- **机制原理**:畸变头 + 探针写**同一条连接**,以「该连接随后收到几条 HTTP 响应」为计数 oracle:多一条 = 前置把余字节当下一请求开头(失步成立);恰好一条 = 原语无效;零响应 = 前置在等 body(不是失步)。计数比状态码/长度差分更硬,直接测失步本身。
- **件组合**:`desync_probe`(20 条候选逐一计数;零计数原语直接划掉,不为它写第二跳)。
- **实证(3)**:smuggle/0cl(多实例)、smuggle/cl-te、smuggle/te-cl(基础组的单连接逐响应判读)。
- **反例(1)**:smuggle/0cl —— 畸变 CL **名**被前端解析并按真 CL 等 body(0 响应),与 0.CL 要求的「前端不计数」相反 ⇒ 该 infra 无 H-V holding 原语,记 infra-blocked。读法边界:0 响应 ≠ suspect。

### P4 未完成帧 → 下一请求补全 → 读交接响应
- **判型触发**:需要让「响应落到下一个到达者身上」(越权页、缓存投毒、捕获他人请求)。
- **机制原理**:走私请求声明长度 > 实发字节 ⇒ 后端等余字节 ⇒ 响应延迟到**下一请求到达时才生成**;前端把这迟到的响应当该请求的响应还给你(或受害者)。**完整帧**(自带结尾空行)响应立即生成、被前端丢弃 ⇒ 不交付。
- **件组合**:帧构造(`conn_reuse --cl-te/--te-cl`、`h2cl_seq` arm/follow、`pause_desync` 暂停式)→ 紧跟一条普通请求(新连接即可)→ 读交接响应;缓存交付走 `smuggle_win`/`smuggle_arm`/`smuggle_seq`(逐轮 arm + settle/probe + cooldown,`--ledger` 记字节账)。
- **实证(5)**:smuggle/cl-te-bypass、smuggle/te-cl-bypass、smuggle/capture-requests、smuggle/cache-poison、smuggle/h2-cl。
- **反例(1)**:三个 h2 系 lab 实测**前端不复用后端连接** ⇒ arm 后新连接 follow-up 恒 200,跨用户交付链断(同连接内可见、跨连接不可交付)。

## C. 盲注族

### P5 无回显换信道:三梯 oracle + 控制值标定
- **判型触发**:注入无回显(响应同形/异步);题面要求「取回数据」但无同步信道。
- **机制原理**:把「真假/数据」编码进另一条可见信道——错误文本(类型/占位符/字段缺失)、时延(需 2x 稳定差才算信号)、出网(回调即判定)。布尔 WHERE 必须**先匹配一行**(前缀用服务端已入库的真实值),否则条件恒假零命中。
- **件组合**:布尔臂 `blind_oracle`(`--template` 含 `{I}`/`{C}` + `--place cookie|header|body|query` + `--true` 标记);先发已知真/假的**控制值**标定分辨率,再逐位提取;时延臂先 `nap` 定基线;出网臂见 P8。
- **实证(4)**:blind/conditional-responses、blind/oob、blind/oob-exfil、blind/out-of-band-exfil(shellshock 同型)。
- **反例(2)**:blind/oob-exfil —— 自建 OOB 域 40+ 载荷零命中(平台只放行官方域);race/partial-construction 的时延不可用(抖动 > 信号,单状态无命中可标定)。

## D. 缓存族

### P6 键面三问 → 未键控面 → 续毒保活
- **判型触发**:响应带缓存头(`X-Cache`/`Age`/`Vary`/`Cache-Control`),或 `Pragma: x-get-cache-key` 读得到键;页面存在反射点。
- **机制原理**:缓存键取哪些请求分量 ≠ 源站渲染读哪些分量,差值即未键控面(污染一个键让受害者踩中)。TTL 短 ⇒ 投毒必须按 TTL 周期续,直到受害者命中;TTL 内先到者会把干净页写回。
- **件组合**:键侦察 `cache_probe`(多请求看 `X-Cache`/`X-Cache-Key`/命中转移)→ 未键控面探测(`header_fuzz` 逐头、`url_fuzz` 分隔符、`cache_probe` 头/体槽)→ 续毒保活(`poison_loop` 定时、`fatget_poison` GET 带体、`raw_poison` 字节级)→ 干净请求验 `X-Cache: hit`。
- **实证(6)**:cache/unkeyed-param、cache/unkeyed-query、cache/unkeyed-header、cache/key-injection、cache/multiple-headers、cache/normalization。
- **反例(2)**:cache/combining —— **裸根路径校验会把干净页写回**挤掉投毒(校验读要带随机参数);host/cache-poison-ambiguous —— 缓存只坐在明确路由内(另一域响应从不入本键)。

### P7 字节级请求行投毒(高层库会重编码)
- **判型触发**:载荷含 `<`/`>`/`'`/`%`/编码序列,或需保留请求目标原字节。
- **机制原理**:高层 HTTP 库会规范化/重编码请求目标(ureq 对 `<>`、`%XX` 会再编码)⇒ 依赖原字节的键/反射面投不进去,必须发字节级请求行。
- **件组合**:`raw_poison`(字节级请求行 + 重复)→ `raw_http`/`raw_matrix` 成组对比变体 → `poison_loop` 续毒。
- **实证(3)**:cache/unkeyed-query、cache/key-injection、cache/unkeyed-header。
- **反例(1)**:cache/key-injection —— 键**头名大小写敏感**(`origin:` 大小写不同即异键),大写投毒永不命中;同题内反例,警示键构造要逐字节复刻受害者请求。

## E. OOB 族

### P8 OOB 判定两半:到达即判定 / 自持标签读回
- **判型触发**:同步无信道、目标出站受限、题面明写「须用官方 Collaborator」。
- **机制原理**:**检测面**只看到达——把回调打向任意**随机**官方子域,一次 DNS 查询即翻检测型题(不需持有子域)。**读取面**要读数据——数据埋在标签里,必须由**自持 secret 派生标签**再轮询官方轮询主机,交互**读出即消费**。判定语义(到达)与读取语义(持有)是两件事。
- **件组合**:检测腿 `dns_oob`/`oob_serve`(自建权威)或直接任意 `*.oastify.com` 子域;读取腿 `burp_collab new`(派生标签)→ 注入 → `burp_collab poll`(取回 `subDomain` / `data.request`)。取回值必须闭环使用(登录取会话、`/submitSolution` 交答案)。
- **实证(4)**:oob/blind(检测一发即解)、oob/blind-exfil、oob/out-of-band-exfil、oob/shellshock。
- **反例(2)**:自建 `oob.dthack.io` 对 **lab 侧**不可达(b16/b18/b19 共 40+ 载荷、含强制外部递归的**两腿判别器**,仍零命中)⇒「注入未执行」与「出站被挡」必须分离;随机标签可翻检测题但**读不回**数据。

## F. 浏览器族

### P9 用户激活载荷必须真手势(合成 click 是假阴性)
- **判型触发**:载荷需要用户激活(`javascript:` URL、`ng-focus`/`onfocus`/`autofocus`、需 keydown 的交互门)。
- **机制原理**:Chrome 对合成 `el.click()` 不派发**用户激活**;`javascript:` URL 只在真激活下先百分号解码再执行 ⇒ 合成点击的负结果不是否证。判读必须走真输入管线。
- **件组合**:`page_alert --click 'selector'`(CDP 输入管线)或 `--keys`/`--driver`;判读预挂的 `window.__labAlerts`。
- **实证(2)**:xss/javascript-url-blocked(真点击 `%28` → 执行)、dom/web-messages-and-a-javascript-url。
- **反例(1)**:曾据合成 `a.click()` 判「现代 Chrome 不解码 javascript:」,改用真手势后同载荷执行 ⇒ 需激活的行为一律用真输入判读。

### P10 渲染面判读:先聚焦、再读副作用
- **判型触发**:focus/blur 类射点;框架事件(如 `ng-focus`)经异步队列。
- **机制原理**:未聚焦文档里 Chrome 只设 `activeElement`、**不派发 focus 事件** ⇒ focus 类载荷系统性假阴性;`ng-*` 事件经 `$applyAsync` 排队,同一次 `eval` 读副作用也假阴性(下一次 eval 才可见)。
- **件组合**:`browser_suite call Page.bringToFront`(先让文档 hasFocus)→ `page_alert`/`page_eval_batch` 触发 → **下一次** eval 读副作用;跨源子帧判读改 `fetch` beacon + 服务器访问日志(`page_alert` 只读顶层 `window.__labAlerts`)。
- **实证(3)**:dom/web-messages-and-json-parse、xss/javascript-url-blocked、csti/angular-sandbox-escape-and-csp。
- **反例(1)**:未聚焦时 ng-focus 载荷全 `fired=false`,补 `Page.bringToFront` 后同载荷立即 `fired=true`。

## 汇编备注

- oracle 的抽象分类(布尔/字节计数/时延/出网/二阶/嵌套)已在 seed [[hunter-oracle-engineering]];本层只补**族内可操作的两段式与预算律**(P1/P2/P5/P8)。
- 「解析分歧即攻击面」的抽象已在 [[hunter-differential-method]];本层补**第一跳计数 oracle 与交付律**(P3/P4)。
- 反例总纪律:窗口/半态/交付链的**负结果**要标「同一 infra 复现 + 读法边界」才记死(P3 的 0 响应、P4 的跨连接、P7 的大小写键)。
- 件名按 catalog 实名核对(2026-10:catalog 的 piece 名与版本);件会演进,能力面稳定,引用以能力面为准。
