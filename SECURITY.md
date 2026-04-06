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

## Response Expectations

- 会先确认问题是否可复现
- 如果问题成立，会优先修 blocker，再同步文档 truth
