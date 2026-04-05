# Contributing

## 当前贡献重点

当前仓库已经从“只有骨架”进入“Codex-only v1 已落地主导出链”的阶段。

所以最欢迎的贡献顺序是：

1. 文档与实现继续保持同步
2. 完善 Codex canonical export 验证与测试
3. `local direct-read` 作为第二阶段能力
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
5. 不要破坏 `complete / incomplete` 与 round-based Markdown contract
