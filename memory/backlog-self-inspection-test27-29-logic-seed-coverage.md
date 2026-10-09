---
metadata:
  node_type: memory
name: "Backlog Self-Inspection Test27-29 Logic Seed Coverage"
description: "题27-29 logic 自省@598ec962f:数量负数下界 2/25 直达 P15;隐藏价格篡改 9/23、跳过确认步骤 6/23 均缺 → 两条都是检索措辞缺口(P15 补「隐藏字段/价格篡改」、P12 补「跳过确认步/直发末步」),无需新 P 条"
last_updated: 2026-10-08T21:38:08+08:00
created: 2026-10-08T21:38:08+08:00
---

## 2026-10-08 回溯自省台(题 27-29 logic 三连)@HEAD=598ec962f

版本面貌:HEAD 又推进(60494b5da → 598ec962f「gitlink bump (lab 29)」)。seed tactical-patterns 相对 60494b5da +10/-4 已提交:P5 加读法边界(题 20/21 长度梯先测抖动,随机尾注大于信号差时换信道)、P16 机制改写成「主形 = 词表按固定间隔插入自有有效凭据(题 24 每 2 失败插一次诱饵登录重置按 IP 计数)」、P20 加覆盖注(题 26 —— 驻留凭据可逆构造 base64 拼摘要的 stay-logged-in cookie 属本条退化形,取一枚样本离线清构形、预造全量重放)。P 条总数仍 24(无 P25+)。

三查询实测(seed 根 universe=64,iwe find --lexical):
- 「数量 负数 下界」→ tactical-patterns **rank 2/25** → 直达 P15(触发词「结账/兑换面只校验合计与余额,不校验条目合理性与符号」、机制「负数量/客户端价/有符号卷绕、总价无下界」)。
- 「隐藏 价格 篡改」→ **rank 9/23** → 不到(top5 是 business-logic-family>graphql-family>web-vuln-methods>ssti-family>js-reverse-debugging)。
- 「跳过 确认 步骤」→ **rank 6/23** → 差一位(top5 全是非 P 层文档:active-scan-insertion-and-judgment>debug-desk-four-lines>request-smuggling-family>cdp-interactive-debugging>web-vuln-methods)。

缺口判定 = 2 条,均为**检索措辞缺口**而非覆盖缺口(机制已在条目内):
- P15 正文有「客户端价/负数量/条目合理性/总价无下界」,但无「隐藏」「价格」「篡改」字面 → 建议 P15 触发词/机制补「隐藏字段、客户端价、价格篡改」。
- P12 正文有「删某步/跳验证/缺参落无校验分支」,但无「跳过」「确认步」字面(「确认」一词还被 P6「次级确认」等别处稀释 BM25)→ 建议 P12 触发词补「跳过确认步 / 直发末步 / 跳过校验步」。

备注:上一轮建议的 P25(离线凭证爆破)被并行会话落成 P20 的覆盖注(题 26)而非新条;P26(令牌-主体未绑定)尚未落地。

