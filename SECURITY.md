# Security Policy

## Reporting a Vulnerability

如果你发现了会影响 `agent-exporter` 的安全问题，请不要在公开 Issue 里直接披露细节。

请至少在报告中包含：

1. 影响范围
2. 最小复现步骤
3. 预期风险等级
4. 是否涉及 credential / host-safety / filesystem / remote execution

## Current Security Scope

当前仓库最敏感的边界主要有：

- 本地 app-server 启动与 host safety
- 本地 artifact 写盘路径
- CLI 参数组合与命令调用
- tracked text files / fixtures / public docs 中的 credential 泄漏
- `.gitignore` 对本地 agent tooling 与 env 目录的覆盖完整性

## Current Preventive Gates

当前仓库已经把下面这些门禁放进 repo-owned contract：

- `cargo test --test host_safety_contract`
  - 防止 banned host-control primitives 回流到运行时代码里
- `cargo test --test security_contract`
  - 防止 `.gitignore` 漏掉 `.cursor/`、`.venv/` 等敏感本地目录
  - 防止 tracked text files 出现 live secret material
- `cargo clippy --all-targets --all-features -- -D warnings`
  - 防止新增代码在严格 lint 下继续积累技术债

这几条门禁现在也已经进入 CI required path。

## Response Expectations

- 会先确认问题是否可复现
- 如果问题成立，会优先修 blocker，再同步文档 truth
