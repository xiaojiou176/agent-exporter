# agent-exporter

`agent-exporter` 是一个 **本地优先、CLI 优先、可扩展到多 Agent CLI** 的会话导出工具仓库。

当前先把五件事做对：

> **先把 Codex 的 canonical export 做对，**
> **再用最小 Claude Code connector 证明这套架构能接第二个来源，**
> **再用最小 JSON exporter 证明 shared transcript core 能接第二种输出，**
> **再用最小 HTML exporter 证明同一份 transcript 也能直接变成静态可读页面，**
> **再用最小 archive index 证明这些页面已经可以本地浏览和静态发布，**
> **再把它升级成一个 local-first multi-agent archive shell，**
> **再让 semantic / hybrid 检索结果也能保存成可链接的本地 retrieval reports，**
> **再把 workspace-local transcript 导航闭环补完整，**
> **再给 saved retrieval reports 补一张自己的本地 front door，**
> **再让这张 front door 也支持本地 metadata search。**

但仓库从第一天开始就按“未来会接 Claude Code 和其他本地 CLI”来设计，所以不会把 Codex 的私有读取逻辑写死在整个项目里。

## Host Safety

这个仓会启动一个本地 app-server 子进程，但它不是桌面自动化器，也不是宿主机清理脚本。

当前硬边界是：

- 只允许管理 repo 直接 spawn 的 app-server child
- 不允许 `killall`、`pkill`、`kill -9`、`process.kill(...)`、`os.kill(...)`、`killpg(...)`
- 不允许 `osascript`、`System Events`、`AppleEvent`、`loginwindow`、`showForceQuitPanel`
- 不允许 `detached` background runner 或 `.unref()` 这类“放出去就不认账”的模式
- `--app-server-command` 只接受 direct executable / repo-owned test double，不接受 shell launcher、host-control utility、inline-eval 入口

更完整的约束见 [docs/reference/host-safety-contract.md](./docs/reference/host-safety-contract.md)。

---

## 当前定位

你可以先把这个仓理解成一个“会话归档工具”的地基，而不是一个已经做完的产品。

当前阶段已经完成的是：

- Rust CLI 骨架
- `source / core / output` 分层
- `Codex app-server source`
- `Codex local direct-read source`
- `Claude Code session-path connector`
- typed archive core
- round-based Markdown export
- shared JSON export
- shared HTML export
- workspace conversations archive index
- local archive metadata search
- semantic retrieval
- persistent local semantic index
- hybrid retrieval
- multi-agent local archive shell
- local retrieval report artifacts
- workspace-local transcript backlinks
- local reports shell
- local reports-shell metadata search
- minimal stdio MCP bridge
- Codex / Claude Code / OpenClaw integration pack docs/templates
- repo-owned integration materializer
- repo-owned integration doctor
- drift-aware integration doctor hardening
- platform-aware integration doctor diagnostics
- integration pack-shape hardening
- integration onboarding experience
- forbidden-target onboarding guardrails
- integration evidence pack / exportable onboarding reports
- integration evidence shell search
- machine-readable integration evidence
- integration evidence timeline/diff
- evidence gate / explain
- baseline registry / policy packs / promotion engine/history
- remediation bundle studio
- read-only governance MCP surface + current-decision automation
- local governance workbench
- 一条真实可用的 `export codex --thread-id ...` 导出主链
- 一条真实可用的 `export claude-code --session-path ...` 导出主链
- 一条真实可用的 `--format markdown|json|html` 输出命令面
- 一条真实可用的 `publish archive-index --workspace-root <repo>` 本地归档索引命令
- 一条真实可用的 `integrate <platform> --target <dir>` 材料化主链
- 一条真实可用的 `doctor integrations --platform <platform> --target <dir>` 验收主链
- 第一批 ADR / 参考文档与实现对齐

当前阶段**还没有**完成的是：

- 当前 Phase 34 之后的新一轮产品裁决

---

## 当前范围

### 当前做

- Codex 对话记录导出
- 先冻结导出 contract 和 source 分层
- 先写清楚“参考谁、继承什么、不要抄什么”

### 当前不做

- hosted / remote search
- hosted / remote semantic platform
- hosted / remote publish
- hosted / browser-side GUI 平台壳
- browser-side execution
- 远程服务
- 多 connector 同步交付

### 当前已显式授权

- local-first GUI / Web UI Decision Desk
  - 仅限本地静态或本地服务决策台
  - 只负责看、比、判、引导下一步
  - 不 hosted
  - 不 cloud backend
  - 不 browser-side execute
  - 不合并 transcript/search/evidence corpus

---

## 当前建议路线

当前的主路线已经明确：

1. **先继承 CodexMonitor 的导出 contract**
2. **再参考官方 Codex 的 thread/source 真相层**
3. **v1 先做 Codex app-server source**
4. **现在已经落地：typed archive core + Codex dual source (`app-server` + `local`) + Markdown export**
5. **现在已经落地：Claude Code minimal `--session-path` second connector proof**
6. **现在已经落地：shared JSON export via `--format json`**
7. **现在已经落地：shared HTML export via `--format html`**
8. **现在已经落地：workspace conversations archive index via `publish archive-index`**
9. **现在已经落地：local archive metadata search**
10. **现在已经落地：semantic retrieval via `search semantic`**
11. **现在已经落地：persistent local semantic index reuse**
12. **现在已经落地：hybrid retrieval via `search hybrid`**
13. **现在已经落地：multi-agent archive 平台化 via upgraded `publish archive-index` shell**
14. **现在已经落地：local retrieval report artifacts via `search ... --save-report`**
15. **现在已经落地：workspace-local transcript backlinks for HTML exports**
16. **现在已经落地：local reports shell via `publish archive-index`**
17. **现在已经落地：local reports-shell metadata search**
18. **现在已经落地：minimal stdio MCP bridge for publish/search**
19. **现在已经落地：repo-owned integration materializer via `integrate <platform> --target <dir>`**
20. **现在已经落地：repo-owned integration doctor via `doctor integrations --platform <platform> --target <dir>`**
21. **现在已经落地：integration doctor drift checks + launcher probe**
22. **现在已经落地：platform-aware integration doctor diagnostics**
23. **现在已经落地：integration pack-shape hardening**
24. **现在已经落地：integration onboarding experience via `onboard <platform> --target <dir>`**
25. **现在已经落地：forbidden-target onboarding guardrails for live host/global roots**
26. **现在已经落地：integration evidence pack via `doctor/onboard --save-report` + `.agents/Integration/Reports/`**
27. **现在已经落地：integration evidence shell search via `.agents/Integration/Reports/index.html`**
28. **现在已经落地：machine-readable integration evidence via `report.json + index.json`**
29. **现在已经落地：integration evidence timeline/diff via `agent-exporter evidence diff --left <report> --right <report>`**
30. **现在已经落地：Local Evidence Decision Plane / Remediation Studio**
31. **现在已经落地：Baseline Registry / Policy Packs / Decision Promotion / Decision History**
32. **现在已经落地：Remediation Bundle Studio + Governance Workbench Polish**
33. **现在已经落地：Read-only Governance MCP + Current-Decision Automation**
34. **当前已进入 post-Phase-34 product decision 区，默认仍不膨胀成 hosted search / service**

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
cargo run -- export codex --source local --thread-id <thread-id>
cargo run -- export codex --source local --rollout-path /absolute/path/to/rollout.jsonl
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl
cargo run -- export codex --thread-id <thread-id> --format json
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format json
cargo run -- export codex --thread-id <thread-id> --format html
cargo run -- export claude-code --session-path /absolute/path/to/session.jsonl --format html
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
cargo run -- export codex \
  --source app-server \
  --thread-id <thread-id> \
  --destination workspace-conversations \
  --workspace-root /absolute/path/to/repo
```

说明：

- `connectors`：显示当前 connector 路线图
- `scaffold`：显示当前仓库状态和当前真实导出入口
- `export codex`：现在已经是一个双 source 命令面
  - 默认 `app-server`
  - 可显式切到 `local`
- `export claude-code`：现在已经是一个最小 second connector 入口
  - 只收 `--session-path`
  - 结果默认按 `degraded` 理解

### Output format contract

当前导出命令面已经支持：

- `--format markdown`
  - **默认值**
  - 保留当前 round-based Markdown contract
- `--format json`
  - 输出单文件 transcript JSON
  - 继续复用 shared archive core
  - 不发明比 transcript 更强的新语义
- `--format html`
  - 输出单文件 transcript HTML
  - 继续复用 shared archive core
  - 是静态可读文档，不是 browse / publish shell
  - 当输出目标是 `workspace-conversations` 时，会额外带上回 archive shell 的本地导航 backlink
  - 当输出目标是 `Downloads` 时，不会硬塞 workspace-only links

### Archive index contract

当前还额外支持：

- `publish archive-index --workspace-root <repo>`
  - 扫描 `<repo>/.agents/Conversations` 里已经导出的 HTML transcript
  - 生成一个本地可打开的 multi-agent archive shell `index.html`
  - 使用相对链接串起 transcript 页面
  - 现在已内置本地 metadata filter、connector/completeness facets、semantic / hybrid retrieval lane 说明，以及 saved retrieval report links
  - 现在也会在 `.agents/Search/Reports/index.html` 生成一张 local reports shell
  - local reports shell 现在也支持本地 report search 和 report-kind filter
  - semantic / hybrid retrieval 仍然留在 CLI，不会偷偷搬进浏览器侧
  - 不做 hosted publish、remote search service、gist、web publish

### Semantic retrieval contract

当前还额外支持：

- `search semantic --workspace-root <repo> --query "<text>"`
  - 对本地 archive corpus 做真实 embedding-based retrieval
  - 当前默认走本地模型目录
  - 如果本地模型文件不存在，命令会明确报错，不会退回关键词假装成语义检索
  - 会把 corpus embeddings 持久化到本地 sidecar index，并在相同模型资产的后续查询中复用

### Hybrid retrieval contract

当前还额外支持：

- `search hybrid --workspace-root <repo> --query "<text>"`
  - 会把 semantic score 和 lexical metadata score 组合成一个 explainable hybrid score
  - 继续复用现有 persistent semantic index，不再重造第二条 semantic persistence 主链
  - 不会静默改写 `search semantic` 的纯语义语义
  - 仍然是本地 CLI retrieval，不是 hosted / remote semantic platform

### Retrieval report contract

当前还额外支持：

- `search semantic --workspace-root <repo> --query "<text>" --save-report`
- `search hybrid --workspace-root <repo> --query "<text>" --save-report`
  - 会把本次检索结果保存成静态 HTML retrieval report
  - report 默认写到 `<repo>/.agents/Search/Reports`
  - report 是 search-owned local artifact，不会回流到 `.agents/Conversations` transcript corpus
  - archive shell 和 reports shell 都可以链接这些 report，但不会在浏览器里替你重新执行 retrieval

### Source contract

你可以先把它理解成“一栋房子现在有两扇合法入口，但正门还是原来的正门”：

- `--source app-server`
  - **默认值**
  - 代表 **canonical truth**
  - 继续使用 `thread/read` primary、`thread/resume` fallback
- `--source local`
  - **非默认**
  - 代表 **archival truth**
  - 支持：
    - `--thread-id <THREAD_ID>`：通过 `state_5.sqlite -> threads.rollout_path` 找 rollout
    - `--rollout-path <PATH>`：直接读本地 rollout jsonl

### 参数组合规则

- `app-server`
  - 允许：`--thread-id`
  - 禁止：`--rollout-path`
- `local`
  - 允许：`--thread-id`
  - 允许：`--rollout-path`
  - 但两者**不能同时给**

### CODEX_HOME 规则

在 `--source local` 下，`agent-exporter` 会按下面顺序找 Codex 本地账本：

1. `--codex-home <PATH>`
2. 环境变量 `CODEX_HOME`
3. 默认 `~/.codex`

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
- `degraded`
  - Codex `--source local` 的 archival replay 结果
  - Claude `--session-path` 的 local session-file import 结果
  - 结构 contract 与 canonical 保持一致
  - 但**不冒充 canonical parity**

说得更直白一点：

> 能导出来，不等于历史已经被证明完整。  
> 只要走了 live fallback，就必须老老实实标 `incomplete`。
> 只要走了 local archival source，就必须老老实实标 `degraded`。

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
6. [docs/reference/host-safety-contract.md](./docs/reference/host-safety-contract.md)
7. [docs/reference/codexmonitor-export-contract.md](./docs/reference/codexmonitor-export-contract.md)
8. [docs/reference/codex-upstream-reading-list.md](./docs/reference/codex-upstream-reading-list.md)
9. [docs/reference/external-repo-reading-list.md](./docs/reference/external-repo-reading-list.md)
10. [docs/reference/codex-thread-archive-blueprint.md](./docs/reference/codex-thread-archive-blueprint.md)

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

当前 blueprint 已经落地到本轮定义的最深阶段。

后续文档和实现若继续推进，会先进入：

1. 新的 post-Phase-34 产品裁决

---

## 开发命令

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
cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
cargo run -- search semantic --workspace-root /absolute/path/to/repo --query "how do I fix login issues"
cargo run -- search hybrid --workspace-root /absolute/path/to/repo --query "thread-1"
cargo run -- search semantic --workspace-root /absolute/path/to/repo --query "how do I fix login issues" --save-report
cargo run -- search hybrid --workspace-root /absolute/path/to/repo --query "thread-1" --save-report
cargo run -- evidence diff --left /absolute/path/to/report-a.json --right /absolute/path/to/report-b.json
cargo run -- evidence gate --baseline /absolute/path/to/report-a.json --candidate /absolute/path/to/report-b.json
cargo run -- evidence explain --report /absolute/path/to/report-b.json
cargo run -- evidence baseline list
cargo run -- evidence baseline show --name codex-main
cargo run -- evidence baseline promote --report /absolute/path/to/report-a.json --name codex-main
cargo run -- evidence policy list
cargo run -- evidence policy show --name codex
cargo run -- evidence promote --candidate /absolute/path/to/report-b.json --baseline-name codex-main
cargo run -- evidence history --baseline-name codex-main
cargo run -- integrate codex --target /absolute/path/to/codex-pack
cargo run -- integrate claude-code --target /absolute/path/to/claude-pack
cargo run -- integrate openclaw --target /absolute/path/to/openclaw-pack
cargo run -- doctor integrations --platform codex --target /absolute/path/to/codex-pack
cargo run -- doctor integrations --platform codex --target /absolute/path/to/codex-pack --explain
```

## Integration Pack

如果你要把 `agent-exporter` 接进别的 agent/workflow，目前最稳的入口是：

- Codex: `docs/integrations/codex.md`
- Claude Code: `docs/integrations/claude-code.md`
- OpenClaw: `docs/integrations/openclaw.md`

模板入口见：

- `docs/integrations/templates/`

当前真实边界：

- 已经准备好的：CLI-first templates / skills / command snippets / bundle skeletons / minimal stdio MCP bridge
- 当前 repo 还已经多了一层 repo-owned 接入主链：
  - `agent-exporter integrate codex --target <dir>`
  - `agent-exporter integrate claude-code --target <dir>`
  - `agent-exporter integrate openclaw --target <dir>`
  - `agent-exporter doctor integrations --platform <platform> --target <dir>`
- 当前 repo 现在还多了一条更顺手的 onboarding 主链：
  - `agent-exporter onboard codex --target <dir>`
  - `agent-exporter onboard claude-code --target <dir>`
  - `agent-exporter onboard openclaw --target <dir>`
- 当前 repo 现在还多了一条可保存、可复查的 integration evidence 主链：
  - `agent-exporter doctor integrations --platform <platform> --target <dir> --save-report`
  - `agent-exporter onboard <platform> --target <dir> --save-report`
  - report 默认写到当前工作目录下的 `.agents/Integration/Reports`
  - 现在还会同写 `report.html + report.json`，并维护 `index.html + index.json`
  - `agent-exporter evidence diff --left <report> --right <report>` 现在能解释两次 evidence 的变化
- 当前 repo 现在还多了一条判定/解释主链：
  - `agent-exporter evidence gate --baseline <report> --candidate <report>`
  - `agent-exporter evidence explain --report <report>`
  - `agent-exporter doctor integrations --platform <platform> --target <dir> --explain`
- 当前 repo 现在还多了一层只读 evidence/governance 工作台：
  - `publish archive-index` 会把 transcript/search/evidence 三壳导航和 Decision Desk 组织进同一个本地 front door
  - MCP bridge 现在也已经扩到 read-only evidence/governance tools，不再只覆盖 publish/search
- 当前 bridge 现在已经覆盖 publish/search/evidence/governance 只读工具，不代表整个 CLI 全量变成 MCP
- 当前 MCP bridge 默认依赖 repo 内的 `scripts/agent_exporter_mcp.py`
  - first-run 会优先尝试 repo-local `target/release/agent-exporter`
  - 没有 release binary 时，会继续尝试 repo-local `target/debug/agent-exporter`
  - 如果本地还没提前 build，它会再退到 `cargo run --manifest-path <repo>/Cargo.toml --bin agent-exporter --`
- 如果你要改成已安装 binary 或自定义 launcher，再显式设置 `AGENT_EXPORTER_BIN` / `AGENT_EXPORTER_ARGS`
- OpenClaw 当前准备好的是 **bundle content / plugin skeleton**，不是 repo-native OpenClaw runtime；接法见 `docs/integrations/openclaw.md`
- Installer 只会往显式 `--target` 下材料化，不会静默改你的 Home 目录
- `integrate` / `onboard` 现在还会直接拒绝明显的 live host/global roots：
  - `~/.codex`
  - `~/.claude*`
  - direct OpenClaw bundle/plugin roots，例如 `bundles/<name>`、`plugins/<name>`
- Doctor 只做只读 readiness 检查，不会偷偷替你装东西
- Doctor 现在还会额外检查：
  - 当前 target 内容是否和当前 repo 重新材料化后的版本一致
  - 当前 repo-local launcher 是否真的还能执行 `connectors`
  - 如果当前 launcher 只能回退到 `cargo run`，doctor 会保守停在 `partial`，不会在只读模式下为了一句 `ready` 触发 build
- Doctor 现在还会按平台补更具体的 shape checks：
  - Codex：`.codex/config.toml` 是否真像一个 project-scoped config
  - Claude Code：`.mcp.json` 是否真是一个可解析的 project-scoped MCP config
  - OpenClaw：bundle/plugin manifests 和 `.mcp.json` 是否真像一个合法 bundle
- Doctor 现在还会继续收紧 pack-shape 细节：
  - Codex：`command` + 非空 `args` 数组
  - Claude Code：`CLAUDE.md` 与 `.claude/commands/*.md` 的基本 pack 形状
- `onboard` 会把 `integrate + doctor + next steps` 串成一条更低摩擦的 first-run path
- integration evidence reports 是单独的本地 artifact：
  - root: `.agents/Integration/Reports`
  - front door: `.agents/Integration/Reports/index.html`
  - 不会回流 `.agents/Conversations` transcript corpus
  - 也不会混进 `.agents/Search/Reports` retrieval report 壳
- integration evidence shell 现在也已经支持本地静态搜索和 facet：
  - `platform`
  - `readiness`
  - 仍然只是静态 evidence shell，不会在浏览器里执行 doctor/onboard

## License

当前仓库使用 **MIT License**。详见 [LICENSE](./LICENSE)。

`cargo test` 现在也承担 host safety gate: 如果运行时代码里重新出现危险宿主机原语，测试会直接失败。

---

## 贡献入口

见 [CONTRIBUTING.md](./CONTRIBUTING.md)。
