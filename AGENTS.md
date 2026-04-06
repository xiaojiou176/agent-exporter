# agent-exporter Agent Guide

## Repo Summary

`agent-exporter` 是一个本地优先的 AI Agent CLI 会话导出仓库。

- 当前目标：**Codex canonical transcript/archive 导出**
- 未来目标：Claude Code、以及其他本地 CLI
- 当前阶段：**Codex dual-source + Claude Code minimal connector + shared JSON/HTML export + archive index + local metadata search + semantic retrieval + persistent local semantic index + hybrid retrieval + local multi-agent archive shell + local retrieval report artifacts + workspace-local transcript backlinks + local reports shell + reports-shell metadata search + minimal stdio MCP bridge + repo-owned integration materializer/doctor + drift-aware integration doctor hardening + platform-aware integration doctor diagnostics + integration pack-shape hardening + integration onboarding experience + integration evidence pack 已落地；当前已进入 post-Phase-26 产品裁决区；默认 Codex 主路径仍是 app-server**

---

## Core Rules

1. **文档先于代码**
   - 行为语义、边界、状态语义，先写进 `docs/`，再写实现。

2. **先守住上游真理源**
   - 当前完整导出的主真源，先按 CodexMonitor 的 `thread/read` contract 理解。
   - 官方 Codex 源码决定 thread/source 真相层，不要凭印象猜。

3. **source / core / output 分层**
   - `connectors/`：负责取数
   - `core/`：负责 transcript / archive contract
   - `output/`：负责 Markdown / JSON / HTML

4. **先小后大**
   - 继续坚持 local-first CLI，不往 hosted / platform shell 膨胀
   - 当前已 landed 的 local archive index / metadata search / semantic retrieval / hybrid retrieval 继续保留，不回退成“future plan”
   - 当前仍不做 hosted search/index/MCP server
   - v1 不做 GUI
   - v1 不做多 connector 并行上线

5. **可扩展，但不提前过度设计**
   - connector 边界要预留
   - 但不要为了未来十个 connector，把今天的实现搞得过重

6. **Host Safety 是硬边界**
   - 这个仓只允许管理自己直接 spawn 的 app-server child。
   - 禁止引入 `killall`、`pkill`、`kill -9`、`process.kill(...)`、`os.kill(...)`、`killpg(...)`、`osascript`、`System Events`、`AppleEvent`、`loginwindow`、`showForceQuitPanel`、`detached: true`、`.unref()`
   - `--app-server-command` 只允许 direct executable / repo-owned test double；host-control utility、shell launcher、inline-eval 入口一律拒绝。

---

## Read Order

1. `README.md`
2. `CLAUDE.md`
3. `docs/README.md`
4. `docs/adr/ADR-0001-source-layering.md`
5. `docs/adr/ADR-0002-codex-first-delivery.md`
6. `docs/reference/host-safety-contract.md`
7. `docs/reference/codexmonitor-export-contract.md`
8. `docs/reference/codex-upstream-reading-list.md`
9. `docs/reference/external-repo-reading-list.md`
10. `docs/reference/codex-thread-archive-blueprint.md`

---

## Near-Term Scope

### 当前实现目标

- `codex app-server source`
- `codex local direct-read source`
- `claude-code session-path connector`
- typed archive core
- markdown export
- json export
- html export
- archive index
- local metadata search
- semantic retrieval
- hybrid retrieval
- multi-agent local archive shell
- local retrieval report artifacts
- workspace-local transcript backlinks
- local reports shell
- reports-shell metadata search
- minimal stdio MCP bridge
- integration pack docs/templates
- repo-owned integration materializer
- repo-owned integration doctor
- drift-aware integration doctor hardening
- platform-aware integration doctor diagnostics
- integration pack-shape hardening
- integration onboarding experience
- integration evidence pack
- `export codex --thread-id ...` 真实 CLI 主链
- `export claude-code --session-path ...` 真实 CLI 主链
- `--format markdown|json|html`
- `publish archive-index --workspace-root <repo>`
- `integrate <platform> --target <dir>`
- `doctor integrations --platform <platform> --target <dir>`
- `onboard <platform> --target <dir>`
- `doctor integrations --platform <platform> --target <dir> --save-report`
- `onboard <platform> --target <dir> --save-report`
- `--source app-server|local`
- `degraded` archival semantics for local source
- integration pack 默认按 repo-local MCP bridge 理解
  - bridge 会先尝试 repo-local build 产物
  - 再尝试 `cargo run --manifest-path <repo>/Cargo.toml --bin agent-exporter --`
  - OpenClaw 当前是 bundle content / plugin skeleton，不是 repo-native runtime
  - doctor 现在还会检查 target 内容是否与当前 repo 重新材料化后的版本一致，并做 launcher probe
  - 如果 launcher 只剩 `cargo run` 回退，doctor 会保守报告 `partial`，不会在只读模式下触发 build
  - doctor 现在还会按平台检查 `.codex/config.toml`、`.mcp.json` 和 OpenClaw bundle manifests 的基本形状
  - doctor 现在还会进一步收紧 Codex `command/args` 形状，以及 Claude pack 本身的 `CLAUDE.md` / commands 形状
  - `onboard` 现在会把 materialize、doctor 和下一步建议串成一条更顺手的 first-run path
  - `integrate` / `onboard` 现在会拒绝明显的 live host/global roots，例如 `~/.codex`、`~/.claude*` 和 direct OpenClaw bundle/plugin roots
  - `doctor` / `onboard` 现在还能把当前结果保存成 integration evidence reports
  - integration evidence reports 默认写到当前工作目录下的 `.agents/Integration/Reports`
  - 这批 reports 是 integration-owned local artifacts，不回流 transcript corpus，也不混进 search-owned reports shell

### 当前明确非目标

- hosted / remote semantic retrieval
- hosted / remote publish
- hosted / remote MCP server
- Web UI / GUI
- Hosted service

---

## Validation

当前最小验证命令：

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
cargo run -- integrate codex --target /absolute/path/to/codex-pack
cargo run -- integrate claude-code --target /absolute/path/to/claude-pack
cargo run -- integrate openclaw --target /absolute/path/to/openclaw-pack
cargo run -- doctor integrations --platform codex --target /absolute/path/to/codex-pack
cargo run -- doctor integrations --platform codex --target /absolute/path/to/codex-pack --save-report
cargo run -- onboard codex --target /absolute/path/to/codex-pack --save-report
```

说明：

- `cargo test` 现在也承担 host safety gate，会拦截危险宿主机原语回流到运行时代码里。
