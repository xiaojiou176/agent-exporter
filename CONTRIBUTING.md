# Contributing

## 当前贡献重点

当前仓库已经从“只有骨架”进入“Codex dual-source + Claude second connector 已 landed”的阶段。

所以最欢迎的贡献顺序是：

1. 文档与实现继续保持同步
2. 继续完善 Markdown / JSON / HTML 三输出与 Codex / Claude 路径的回归验证
3. archive browsing / publish
4. 后续 connector 扩展

不欢迎的顺序是：

1. 先加 GUI
2. 先加 Search / index
3. 先加多 connector
4. 先加 remote service

---

## 提交前最小检查

```bash
cargo fmt
cargo test
cargo run -- connectors
cargo run -- scaffold
cargo run -- export codex --thread-id <thread-id>
cargo run -- export codex --source local --thread-id <thread-id>
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl
cargo run -- export codex --thread-id <thread-id> --format json
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format json
cargo run -- export codex --thread-id <thread-id> --format html
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format html
```

---

## 文档规则

如果你改了以下任一内容，必须同步更新文档：

- connector 支持范围
- source contract
- output format
- CLI 命令面
- roadmap / implementation order

---

## 开发规则

1. 不要把某个 connector 的私有解析逻辑直接塞进 `cli.rs`
2. 不要把 local direct-read 假装成 canonical export
3. 不要在没有 ADR/文档更新前改导出语义
4. 新增 connector 前，先更新 docs/reference
5. 不要破坏 `complete / incomplete` 与默认 round-based Markdown contract
6. 不要把 `local` 结果包装成 canonical / complete truth
7. 不要为 Claude 再造第二套 transcript / output 模型
