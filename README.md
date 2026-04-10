# agent-exporter

`agent-exporter` 是一个 **local-first archive and governance workbench for AI agent transcripts**。

[Try It In 3 Steps](#first-success-path) · [Docs Landing](https://xiaojiou176-open.github.io/agent-exporter/) · [Archive Shell Proof](https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html) · [Latest Release](https://github.com/xiaojiou176-open/agent-exporter/releases/latest)

![agent-exporter proof map showing CLI quickstart, transcript export, archive shell proof, reports shell, and integration evidence as one local-first workbench](./docs/assets/archive-shell-proof.svg)

## Product Kernel

一句话理解：

> 它先把 transcript 的导出、归档、检索、证据与治理做成一个本地优先工作台，
> 而不是先膨胀成 hosted service 或远程平台。

## Surface Model

- **Primary Surface:** `CLI-first`
- **Secondary Surface 1:** local archive shell / reports shell
- **Secondary Surface 2:** repo-owned integration pack
- **Secondary Surface 3:** read-only governance MCP bridge

这些 surface 可以一起存在，但不能互相借光，更不能把 operator-facing secondary surface 写成当前主门。

## Flagship Public Packet

当前阶段先打的旗舰公开包是：

> **GitHub repo + CLI quickstart + archive shell proof**

这不等于别的 surface 不做。
它们也要全部打完，但必须**分阶段过线**，不能把 secondary surface 的局部 readiness 误写成整仓已经 public-ready。

## Front Door Today

如果你第一次接触这个仓，推荐把入口顺序理解成：

1. **先走 CLI quickstart**
2. **再看 archive shell proof**
3. **最后按需要转去 reports shell / integration pack / governance lane**

说得更直白一点：

> 产品 kernel 里当然包含 evidence 与 governance，
> 但 Wave 1 的 front door 叙事必须仍然让 **CLI-first** 压住第一屏。

## First Success At A Glance

如果你只想先判断“值不值得复制第一条命令”，先看这一张卡片：

| Step | You run | You get |
| --- | --- | --- |
| `1` | `cargo run -- connectors` | 确认这仓现在接哪些 transcript source |
| `2` | `cargo run -- export codex ... --format html --destination workspace-conversations ...` | 一份可浏览的 HTML transcript receipt |
| `3` | `cargo run -- publish archive-index --workspace-root ...` | `.agents/Conversations/index.html` archive shell |

> 这三步像先装一台工作台的桌腿。  
> 先站起来，再讲抽屉、灯和分类系统。

## Public Entry Points

- **GitHub repo front door:** current primary onboarding path with the CLI quickstart
- **GitHub Pages landing:** a thin public docs layer that routes readers to the same product sentence and first-success path
- **Archive shell proof page:** a tracked public explanation of what the archive shell proves, what it does not prove, and how to reproduce it locally
- **Latest release shelf:** release/tag truth for the current public packet

## First Success Path

把 first success 理解成“先确认 connector，再导出一份 transcript，最后把它挂进 archive shell”。

1. **确认当前 connector 路线图**

   ```bash
   cargo run -- connectors
   ```

2. **导出一份 HTML transcript 到当前 workspace**

   ```bash
   cargo run -- export codex \
     --thread-id <thread-id> \
     --format html \
     --destination workspace-conversations \
     --workspace-root /absolute/path/to/repo
   ```

3. **生成 archive shell proof**

   ```bash
   cargo run -- publish archive-index --workspace-root /absolute/path/to/repo
   ```

成功信号：

- 你会得到一份 `.agents/Conversations/*.html` transcript export
- 你会得到 `.agents/Conversations/index.html` archive shell
- 这份 proof 是 **local-first HTML receipt**，不是 hosted demo，也不是 GitHub Pages live read-back

## Proof Ladder

把这仓的公开证明理解成一条 3 级梯子，而不是一张大而全的总图：

| Level | What it proves | Current asset |
| --- | --- | --- |
| `L1` | CLI 命令真能带你进门 | `First Success Path` |
| `L2` | transcript export 会留下可浏览 receipt | `.agents/Conversations/*.html` |
| `L3` | archive shell 会把 transcript / reports / integration evidence 串成 local-first workbench | [archive shell proof](https://xiaojiou176-open.github.io/agent-exporter/archive-shell-proof.html) |

## Public Boundary Right Now

先把当前公开边界记清楚，避免把“能生成”误写成“已经对外 live”：

- **public front door**：当前还是 GitHub repo + CLI quickstart，GitHub Pages 只是 companion docs surface，不是另一扇主门
- **first visible proof we can truthfully promise today**：tracked archive shell proof page + 本地可复现的 archive shell HTML receipt
- **secondary surfaces**：reports shell、integration pack、read-only governance MCP bridge 继续保留，但不抢主门
- **cannot claim yet**：`submit-ready`、`already approved`、`MCP-first`

## Does / Does Not Prove

| This repo can honestly prove today | This repo must not overclaim today |
| --- | --- |
| CLI quickstart works | hosted archive platform |
| transcript export can become HTML receipt | multi-user remote service |
| archive shell proof page explains the workbench truthfully | `submit-ready` / `listed-live` for non-GitHub lanes |
| secondary surfaces already exist as repo-owned lanes | `MCP-first` public positioning |

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

你可以先把这个仓理解成一个**本地优先的 archive / governance workbench**，而不是一个 hosted 平台。
不过当前 front door 的讲法要继续守住一个顺序：**先 CLI，再 archive shell proof，再 secondary surfaces**。

当前阶段已经完成的，不该再用“超长能力清单”去讲，而应该先用一张表看懂：

| Layer | 当前真相 | first proof / entry |
| --- | --- | --- |
| CLI core | `export codex`、`export claude-code`、`--format markdown|json|html` 都已 landed | CLI quickstart |
| Archive shell proof | `publish archive-index` 会生成 transcript browser、workspace backlinks 和 archive shell | `.agents/Conversations/index.html` |
| Reports shell | `search semantic|hybrid --save-report` 会生成 retrieval receipts 和 reports shell | `.agents/Search/Reports/index.html` |
| Integration pack | `integrate` / `doctor integrations` / `onboard` 已是 repo-owned companion lane | `.agents/Integration/Reports/index.html` |
| Governance lane | evidence / baseline / policy / remediation 已进入本地 workbench | archive shell Decision Desk + integration evidence reports |

更直白一点：

> 这仓已经不是“只有导出”的小工具，
> 但今天 public front door 仍然只应该先卖 **CLI quickstart + archive shell proof**，而不是把每条 side lane 都摊成第一屏。

当前阶段**还没有**完成的是：

- 按已经批准的 `Product Kernel` / `Surface Model`，把旗舰公开包继续收平
- 分阶段把 secondary surfaces 各自补到独立可判卷状态

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

## Current Buildout Order

当前最重要的 buildout 顺序，可以压缩成 5 步理解：

1. **先把 CLI export 主链做对**
2. **再把 transcript 变成 archive shell proof**
3. **再把 retrieval 结果保存成 reports receipts**
4. **再把 integration pack 做成 repo-owned companion lane**
5. **最后把 governance/evidence 保持为本地只读 lane，而不是第二个主门**

完整 capability ledger 与阶段落地记录继续放在：

- [docs/README.md](./docs/README.md)
- [CHANGELOG.md](./CHANGELOG.md)

换句话说，v1 的重点不是“支持一切”，而是：

> **先把 Codex 的 canonical export 路径做对，再把 archive shell proof 讲清楚。**

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

当前 blueprint 已经把 repo-local product kernel 和 side-lane hierarchy 讲清楚了。

后续若继续推进，优先顺序会是：

1. public surface / metadata / proof packet 收口
2. surface-by-surface closeout
3. Wave 3 的 packet / lane / final closeout 验证

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

如果你要把 `agent-exporter` 接进别的 agent/workflow，把它理解成**repo-owned companion lane**，不是当前 public 主门。

最稳的说明入口是：

- Codex: `docs/integrations/codex.md`
- Claude Code: `docs/integrations/claude-code.md`
- OpenClaw: `docs/integrations/openclaw.md`

模板入口见：

- `docs/integrations/templates/`

当前最重要的真实边界只有 4 条：

- **entry commands**：`integrate`、`doctor integrations`、`onboard`
- **saved evidence**：`--save-report` 会把结果写到 `.agents/Integration/Reports`
- **launcher policy**：默认优先 repo-local binary，不会静默改 Home，也不会在只读检查里偷偷 build
- **truth boundary**：integration evidence shell 是 secondary surface，不会回流 transcript corpus，也不代表整个产品变成 `MCP-first`

如果你需要更细的 host-by-host 接法、pack shape、launcher 细节或 OpenClaw 边界，直接看：

- `docs/integrations/*.md`
- `docs/integrations/templates/`

## License

当前仓库使用 **MIT License**。详见 [LICENSE](./LICENSE)。

`cargo test` 现在也承担 host safety gate: 如果运行时代码里重新出现危险宿主机原语，测试会直接失败。

---

## 贡献入口

见 [CONTRIBUTING.md](./CONTRIBUTING.md)。
