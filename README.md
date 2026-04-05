# agent-exporter

`agent-exporter` 是一个 **本地优先、CLI 优先、可扩展到多 Agent CLI** 的会话导出工具仓库。

当前只做一件事：

> **先把 Codex 的对话记录导出做好。**

但仓库从第一天开始就按“未来会接 Claude Code 和其他本地 CLI”来设计，所以不会把 Codex 的私有读取逻辑写死在整个项目里。

---

## 当前定位

你可以先把这个仓理解成一个“会话归档工具”的地基，而不是一个已经做完的产品。

当前阶段已经完成的是：

- Rust CLI 骨架
- `source / core / output` 分层
- `Codex app-server source`
- typed archive core
- round-based Markdown export
- 一条真实可用的 `export codex --thread-id ...` 导出主链
- 第一批 ADR / 参考文档与实现对齐

当前阶段**还没有**完成的是：

- local direct-read 实现
- Claude Code connector
- JSON / HTML exporter
- search / index / archive browsing

---

## 当前范围

### 当前做

- Codex 对话记录导出
- 先冻结导出 contract 和 source 分层
- 先写清楚“参考谁、继承什么、不要抄什么”

### 当前不做

- Claude Code connector
- Search / index / knowledge base
- GUI / Web UI
- 远程服务
- 多 connector 同步交付

---

## 当前建议路线

当前的主路线已经明确：

1. **先继承 CodexMonitor 的导出 contract**
2. **再参考官方 Codex 的 thread/source 真相层**
3. **v1 先做 Codex app-server source**
4. **现在已经落地：typed archive core + Markdown export**
5. **v2 再加 local direct-read source**
6. **Claude Code 放到后续 connector 扩展**

换句话说，v1 的重点不是“支持一切”，而是：

> **先把 Codex 的 canonical export 路径做对。**

---

## 仓库结构

```text
src/
├── cli.rs                # CLI 入口和命令面
├── connectors/           # 各 Agent CLI 的 source adapter
├── core/                 # transcript / archive contract
├── model/                # 共享模型
└── output/               # Markdown / JSON / HTML 输出层

docs/
├── adr/                  # 架构决策记录
└── reference/            # 上游 contract、外部参考仓、蓝图
```

---

## 快速开始

```bash
cargo run -- connectors
cargo run -- scaffold
cargo run -- export codex --thread-id <thread-id>
cargo run -- export codex \
  --thread-id <thread-id> \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo
```

说明：

- `connectors`：显示当前 connector 路线图
- `scaffold`：显示当前仓库状态和当前真实导出入口
- `export codex`：通过本地 `codex app-server` 走 canonical app-server path，生成真实 Markdown 归档文件

### 输出目标语义

当前 CLI 和 CodexMonitor 保持同一层语义，不会静默乱写文件：

- 默认目标：`Downloads`
- Repo 目标：`<workspace>/.agents/Conversations`

当你选择：

```bash
--destination workspace-conversations --workspace-root /path/to/repo
```

导出文件会落到：

```text
/path/to/repo/.agents/Conversations/
```

如果 `workspace-root` 不存在或不是目录，CLI 会直接报错，不会悄悄改写到别的地方。

### 完整性语义

当前 v1 明确保留两层状态：

- `complete`
  - 主路径 `thread/read(includeTurns=true)` 成功
- `incomplete`
  - 只有当上游明确拒绝 `includeTurns` 时，才降级到 `thread/resume`

说得更直白一点：

> 能导出来，不等于历史已经被证明完整。  
> 只要走了 live fallback，就必须老老实实标 `incomplete`。

### 高级调试 / 测试入口

默认情况下，CLI 会启动：

```bash
codex app-server
```

如果你需要接自定义 launcher 或测试替身，可以覆盖：

```bash
--app-server-command <command> --app-server-arg <arg> --app-server-arg <arg>
```

---

## 文档阅读顺序

如果你刚进入这个仓，推荐按下面顺序读：

1. [AGENTS.md](./AGENTS.md)
2. [CLAUDE.md](./CLAUDE.md)
3. [docs/README.md](./docs/README.md)
4. [docs/adr/ADR-0001-source-layering.md](./docs/adr/ADR-0001-source-layering.md)
5. [docs/adr/ADR-0002-codex-first-delivery.md](./docs/adr/ADR-0002-codex-first-delivery.md)
6. [docs/reference/codexmonitor-export-contract.md](./docs/reference/codexmonitor-export-contract.md)
7. [docs/reference/codex-upstream-reading-list.md](./docs/reference/codex-upstream-reading-list.md)
8. [docs/reference/external-repo-reading-list.md](./docs/reference/external-repo-reading-list.md)
9. [docs/reference/codex-thread-archive-blueprint.md](./docs/reference/codex-thread-archive-blueprint.md)

---

## 当前真理源

当前仓库有 3 层真理源，不能混着看：

1. **CodexMonitor contract**
   - 定义当前“完整导出”到底承诺什么
2. **官方 Codex 源码**
   - 定义 `thread/read`、sqlite、rollout、turns 的真实关系
3. **外部参考仓**
   - 提供可借鉴的实现思路、CLI 设计、输出策略

一个很重要的边界是：

> **外部参考仓只能帮助我们设计，不能推翻当前上游 contract。**

---

## 后续会补什么

后续文档和实现会继续沿着这条线推进：

1. `local direct-read` source
2. JSON / HTML renderer
3. Claude Code connector
4. 更丰富的 archive browsing / publishing 能力

---

## 开发命令

```bash
cargo fmt
cargo test
cargo run -- connectors
cargo run -- scaffold
cargo run -- export codex --thread-id <thread-id>
```

---

## 贡献入口

见 [CONTRIBUTING.md](./CONTRIBUTING.md)。
