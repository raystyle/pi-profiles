---
metadata:
  node_type: memory
name: "eval-arm-a-server-side-pp-detection-no-reflection"
description: "A 臂实录:无反射型服务端原型污染检测 lab 用 json spaces 非破坏检测一次命中"
last_updated: 2026-10-10T00:49:57+08:00
created: 2026-10-10T00:49:57+08:00
---

# arm A - lab-detecting-server-side-prototype-pollution-without-polluted-property-reflection

- 目标:canonical 路径直达,range_launch launch-url 起实例(reused:false,无取号冗余)
- 面:Node/Express 商城,/my-account 地址表单 action=/my-account/change-address,sessionId 隐藏域
- 登入:GET /login 取 csrf → POST /login **JSON 体** {csrf,username,password} → 302 /my-account?id=wiener 并换新 session
- 基线:POST /my-account/change-address JSON 体 → 200 application/json 紧凑体(197B),响应回显 username/firstname/lastname/address_*/isAdmin
- 检测载荷(单发):同体追加 "__proto__":{"json spaces":10} → 同一响应即变 10 空格缩进(306B),Object.prototype 污染成立,非破坏(HTML 页不受影响)
- 判据:banner_verdict 首页 → solved:true / solved_class:true / congrats 行
- 关键点:该 lab 无属性回显,选非破坏且可自证的 json spaces 面;status 面会把整站响应改码、挡住横幅复验,故不取
- 件链:page_read → range_launch → http_session(get/post) → banner_verdict;零 sh_run,零手写件
- 轮次:6 次 rs_execute 达成,无需 restart node,无需第二载荷

