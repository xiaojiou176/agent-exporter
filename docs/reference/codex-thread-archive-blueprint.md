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

1. hosted / remote search platform
2. 知识库 / semantic platform shell
3. hosted / browser-side GUI 平台壳
4. 远程服务
5. 多 connector 同步交付

> 注：
> 仓库早期的“当前不做 GUI / Web UI”在 Phase 30 已被用户显式覆盖为：
> **允许做 local-first GUI / Web UI decision desk**。
> 但这不改变上面的非目标边界：
> 仍然不做 hosted / browser executor / cloud backend。

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

已落地：

- local metadata search / index

原因：

- 现在已经能在 archive index 上做本地 metadata search
- 但仍然没有进入 semantic retrieval / embeddings / vector 层

### Phase 9

已落地：

- semantic retrieval

原因：

- 现在已经有独立的 embedding-based retrieval 命令
- 但仍然没有进入 persistent vector index / hosted semantic platform

### Phase 10

已落地：

- persistent local semantic index

### Phase 11

已落地：

- hybrid retrieval

### Phase 12

已落地：

- multi-agent archive 平台化

原因：

- 现在已经能把 archive index、metadata filter、semantic retrieval、hybrid retrieval 组织进同一个 local-first archive shell
- 但仍然没有把仓库拖进 hosted / remote search platform

---

当前 blueprint 已经落地到当前定义的最深阶段。下一步若继续推进，需要新的产品裁决，而不是默认跳进 hosted / remote platform。

### Phase 13

已落地：

- local retrieval report artifacts

原因：

- 现在已经能把 semantic / hybrid 检索结果保存成 local static reports
- archive shell 也已经能链接这些 reports
- 但仍然没有把 retrieval execution 搬进浏览器或远端平台

### Phase 14

已落地：

- workspace-local navigation backlinks

原因：

- 现在 workspace conversations 里的 transcript HTML 已经能回到 archive shell
- 但 Downloads HTML 仍保持为不带 workspace-only links 的 leaf artifact
- 仍然没有把 transcript 页面膨胀成 browse / retrieval shell

### Phase 15

已落地：

- local reports shell

原因：

- 现在 `.agents/Search/Reports/index.html` 已经能把 saved retrieval reports 组织成一张本地 front door
- archive shell、transcript、report 三类工件的导航闭环更完整
- 但仍然没有把检索执行搬进浏览器或远端平台

### Phase 18

已落地：

- reports-shell metadata search

原因：

- 现在 local reports shell 也已经能做本地 report search 和 report-kind filter
- 但它仍然只是静态 front door，不执行 retrieval

### Phase 20

已落地：

- minimal stdio MCP bridge

原因：

- 现在 publish/search 的高价值本地工具面已经能通过 MCP 被外部 agent 客户端接入
- 但 bridge 仍然是 local stdio server，不是 hosted MCP service

### Phase 21

已落地：

- repo-owned integration materializer / doctor

原因：

- 现在 integration pack 不再只是 templates/docs
- `integrate <platform> --target <dir>` 已经能把 Codex / Claude Code / OpenClaw 的 repo-owned 材料化资产写到显式 target
- `doctor integrations --platform <platform> --target <dir>` 已经能诚实报告 `ready / partial / missing`
- 但它仍然不会静默改用户 home/global config，也不会把 OpenClaw 写成 repo-native runtime

### Phase 26

已落地：

- integration evidence pack / exportable onboarding report

原因：

- 现在 `doctor` / `onboard` 已经不只会把结果打印到终端
- `--save-report` 已经能把 integration evidence 保存到 `.agents/Integration/Reports/`
- 同目录下也会生成一张静态 front door `index.html`
- 但这些 artifacts 仍然和 transcript corpus / retrieval reports 分仓，不会被 search/archive 主链误吃回去

### Phase 27

已落地：

- integration evidence shell search

原因：

- `.agents/Integration/Reports/index.html` 已经不只是 evidence 列表
- 它现在还支持本地静态搜索和 facet，至少覆盖 `platform` / `readiness`
- 但浏览器仍然只是静态 evidence shell，不执行 doctor/onboard/install

### Phase 28

已落地：

- machine-readable integration evidence

原因：

- `doctor/onboard --save-report` 已经不只写 `report.html`
- 现在还会同写 `report.json`
- `.agents/Integration/Reports/` 也会维护 `index.json`
- 这些 JSON 仍然只是 integration-owned local artifacts，不回流 transcript/search corpus

### Phase 29

已落地：

- integration evidence timeline / diff

原因：

- 现在已经有 `agent-exporter evidence diff --left <report> --right <report>`
- 它会解释 readiness、changed checks、以及 added/removed next steps
- 它比较的是已保存 evidence snapshots，不会重新执行 doctor/onboard/install

### Phase 30

已落地：

- local evidence decision plane / remediation studio

原因：

- `agent-exporter evidence gate --baseline <report> --candidate <report>` 现在能给出 `pass / warn / fail`
- `agent-exporter evidence explain --report <report>` 与 `doctor integrations --explain` 现在能给出 remediation order
- `publish archive-index` 现在已经会把 transcript/search/evidence 三壳导航、baseline/candidate、verdict、changed checks 和 remediation order 组织成一个本地 decision desk
- 现有 MCP bridge 也已经扩到 read-only evidence consumption surface

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
agent-exporter search semantic --workspace-root <repo-root> --query <text>
agent-exporter search hybrid --workspace-root <repo-root> --query <text>
agent-exporter export claude-code --session-path <path> --format html
agent-exporter publish archive-index --workspace-root <repo-root>
agent-exporter integrate codex --target <dir>
agent-exporter integrate claude-code --target <dir>
agent-exporter integrate openclaw --target <dir>
agent-exporter doctor integrations --platform <platform> --target <dir>
agent-exporter doctor integrations --platform <platform> --target <dir> --save-report
agent-exporter onboard codex --target <dir> --save-report
agent-exporter export codex --source app-server --thread-id <id> --destination workspace-conversations --workspace-root <repo-root>
```

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
├── integrations/
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
- archive index 已落地
- local metadata search 已落地
- repo-owned integration materializer / doctor 已落地
- integration evidence pack / exportable onboarding report 已落地
- integration evidence shell search 已落地
- machine-readable integration evidence 已落地
- integration evidence timeline/diff 已落地
- local evidence decision plane / remediation studio 已落地
- 未来扩展边界仍然保持收口
