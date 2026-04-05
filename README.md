# agent-exporter

`agent-exporter` 是一个 **本地优先、CLI 优先、可扩展到多 Agent CLI** 的会话导出工具仓库。

当前先把五件事做对：

> **先把 Codex 的 canonical export 做对，**
> **再用最小 Claude Code connector 证明这套架构能接第二个来源，**
> **再用最小 JSON exporter 证明 shared transcript core 能接第二种输出，**
> **再用最小 HTML exporter 证明同一份 transcript 也能直接变成静态可读页面，**
> **再用最小 archive index 证明这些页面已经可以本地浏览和静态发布。**

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
- 一条真实可用的 `export codex --thread-id ...` 导出主链
- 一条真实可用的 `export claude-code --session-path ...` 导出主链
- 一条真实可用的 `--format markdown|json|html` 输出命令面
- 一条真实可用的 `publish archive-index --workspace-root <repo>` 本地归档索引命令
- 第一批 ADR / 参考文档与实现对齐

当前阶段**还没有**完成的是：

- search / index / semantic retrieval
- 多 agent archive 平台化

---

## 当前范围

### 当前做

- Codex 对话记录导出
- 先冻结导出 contract 和 source 分层
- 先写清楚“参考谁、继承什么、不要抄什么”

### 当前不做

- Search / index / knowledge base
- hosted / remote publish
- GUI / Web UI
- 远程服务
- 多 connector 同步交付

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
9. **当前最高优先级：search / index，但不能直接膨胀成平台壳**

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

### Archive index contract

当前还额外支持：

- `publish archive-index --workspace-root <repo>`
  - 扫描 `<repo>/.agents/Conversations` 里已经导出的 HTML transcript
  - 生成一个本地可打开的 `index.html`
  - 使用相对链接串起 transcript 页面
  - 不做搜索、分页、gist、web publish

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

后续文档和实现会继续沿着这条线推进：

1. search / index / multi-agent archive 平台化

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
```

`cargo test` 现在也承担 host safety gate: 如果运行时代码里重新出现危险宿主机原语，测试会直接失败。

---

## 贡献入口

见 [CONTRIBUTING.md](./CONTRIBUTING.md)。
