# codex-thread-archive Blueprint

这份文档是当前 `agent-exporter` 的产品/架构蓝图。

它回答的问题是：

> 如果我们现在真要落地一个先做稳 Codex、再接第二个 connector、再接第二种输出格式的工具，它最合理的长相是什么？

---

## 一句话定义

`agent-exporter` 当前的最小目标不是“万能会话平台”，而是：

> **一个 Rust CLI-first 的 transcript/archive exporter，**
> **先做稳 Codex，再证明第二个 connector 也能接进来。**

它当前已经做完 Codex dual-source，并且落下了一个最小 `Claude Code` second connector proof。

---

## 设计目标

1. **先把 Codex 导出做对**
2. **先继承当前 CodexMonitor contract**
3. **先把 source / core / output 边界稳定下来**
4. **以后能接 Claude Code，而不是以后再大拆**

---

## 非目标

当前明确不做：

1. Search / index
2. 知识库 / semantic retrieval
3. GUI / Web UI
4. 远程服务
5. 多 connector 同步交付

---

## 三层架构

### 1. source

负责从具体来源加载 transcript 原料。

当前规划：

- `codex app-server source`（default canonical）
- `codex local direct-read source`（landed archival second entrance）
- `claude-code session-path source`（landed minimal second connector）

### 2. core

负责统一 archive contract。

这里要解决的不是“从哪读”，而是：

- round 如何组织
- complete / incomplete / degraded 如何表达
- transcript 的 typed model 长什么样

### 3. output

负责把 core model 渲染成：

- Markdown
- JSON
- HTML

---

## 推荐交付顺序

### Phase 1

先做，而且当前已经落地：

- `codex app-server source`
- typed archive core
- markdown export

原因：

- 最贴当前 CodexMonitor contract
- 最容易验证 `complete / incomplete`

### Phase 2

已落地：

- `codex local direct-read source`

原因：

- 它很有价值
- 但它代表 archival truth，不是 canonical truth

### Phase 3

已落地：

- `claude-code --session-path <PATH>`

原因：

- 用最小范围证明 second connector 可以复用现有 archive core
- 不必先做自动发现 / HTML / browser 体验层

### Phase 4

先做：

- Claude replay hardening / fidelity 提升

原因：

- second connector 已 landed，但 replay fidelity 仍有提升空间
- 先把 Claude 语义打稳，再进入展示层更安全

### Phase 5

已落地：

- JSON renderer

原因：

- 现在已经证明 shared transcript/core/output 不只服务 Markdown
- 但仍然没有把仓库拖进 browse / publish / platform 层

### Phase 6

已落地：

- HTML renderer

原因：

- 现在已经证明 shared transcript/core/output 不只服务 Markdown / JSON
- 但仍然把 HTML 收在单文件 transcript export，而不是 browse / publish shell

### Phase 7

已落地：

- archive browsing / publish

原因：

- 现在已经能为 workspace conversations 生成本地 archive index
- 但仍然没有进入 search / semantic retrieval / 平台壳

### Phase 8

再做：

- search / index / semantic retrieval

---

## 推荐 CLI 命令面

当前已经落地的最小集合是：

```bash
agent-exporter connectors
agent-exporter scaffold
agent-exporter export codex --thread-id <id>
agent-exporter export codex --source local --thread-id <id>
agent-exporter export codex --source local --rollout-path <path>
agent-exporter export claude-code --session-path <path>
agent-exporter export codex --thread-id <id> --format json
agent-exporter export claude-code --session-path <path> --format json
agent-exporter export codex --thread-id <id> --format html
agent-exporter export claude-code --session-path <path> --format html
agent-exporter publish archive-index --workspace-root <repo-root>
agent-exporter export codex --source app-server --thread-id <id> --destination workspace-conversations --workspace-root <repo-root>
```

未来扩展：

```bash
agent-exporter export claude-code --session-path <path>
```

---

## 状态语义

当前 v1 已落地两层状态语义：

| 状态 | 含义 |
| --- | --- |
| `complete` | canonical export，来自主真源 |
| `incomplete` | fallback 成功，但历史不保证完整 |

当前 Phase 2 已额外落地：

| 状态 | 含义 |
| --- | --- |
| `degraded` | archival/local best-effort，不等于 canonical parity |

---

## 目录组织建议

```text
src/
├── cli.rs
├── connectors/
│   ├── claude_code.rs
│   ├── codex/
│   │   ├── app_server.rs
│   │   └── mod.rs
│   └── mod.rs
├── core/
│   ├── archive.rs
│   └── mod.rs
├── model/
│   └── mod.rs
└── output/
    ├── html.rs
    ├── json.rs
    ├── markdown.rs
    └── mod.rs
```

---

## 当前蓝图背后的参考来源

这个蓝图不是拍脑袋定的，而是融合了 3 类来源：

1. **CodexMonitor**
   - 当前导出 contract
2. **官方 Codex**
   - `thread/read` / sqlite / rollout / turns 真相层
3. **外部参考仓**
   - local direct-read exporter
   - transcript output
   - CLI 设计
   - 多 agent connector 方向

---

## 最后一句话

`agent-exporter` 现在最该做的，不是“做很多”，而是：

> **先把 Codex transcript/export 这一件事做对，**
> **并把未来扩展的边界提前设计好。**

当前这句话已经从蓝图进入实现：

- canonical source 已落地
- archival local source 已落地
- typed archive core 已落地
- Markdown export 已落地
- JSON export 已落地
- HTML export 已落地
- 未来扩展边界仍然保持收口
