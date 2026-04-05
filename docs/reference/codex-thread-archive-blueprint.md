# codex-thread-archive Blueprint

这份文档是当前 `agent-exporter` 的产品/架构蓝图。

它回答的问题是：

> 如果我们现在真要落地一个只做 Codex 导出的工具，它最合理的长相是什么？

---

## 一句话定义

`agent-exporter` 的 v1 目标不是“万能会话平台”，而是：

> **一个 Rust CLI-first 的 Codex transcript/archive 工具。**

它当前先做 Codex，后续再扩展到 Claude Code 和其他 CLI。

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

- `codex app-server source`（v1）
- `codex local direct-read source`（v2）
- `claude-code source`（future）

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
- HTML（future）

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

再做：

- `codex local direct-read source`

原因：

- 它很有价值
- 但它代表 archival truth，不是 canonical truth

### Phase 3

再做：

- Claude Code connector

原因：

- 当前仓库先把 Codex 走通最重要
- connector 边界稳定后再接第二个来源，成本更低

---

## 推荐 CLI 命令面

当前已经落地的最小集合是：

```bash
agent-exporter connectors
agent-exporter scaffold
agent-exporter export codex --thread-id <id>
agent-exporter export codex --thread-id <id> --destination workspace-conversations --workspace-root <repo-root>
```

未来扩展：

```bash
agent-exporter export codex --source app-server
agent-exporter export codex --source local
agent-exporter export codex --rollout-path <path>    # only after local direct-read lands
agent-exporter export claude-code --session-path <path>
```

---

## 状态语义

当前 v1 已落地两层状态语义：

| 状态 | 含义 |
| --- | --- |
| `complete` | canonical export，来自主真源 |
| `incomplete` | fallback 成功，但历史不保证完整 |

未来如果 local direct-read 真的落地，再单独增加：

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
- typed archive core 已落地
- Markdown export 已落地
- 未来扩展边界仍然保持收口
