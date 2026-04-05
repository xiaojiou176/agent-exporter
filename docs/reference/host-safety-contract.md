# Host Safety Contract

`agent-exporter` 是一个导出 CLI，不是桌面自动化器，也不是宿主机清理器。

说得更直白一点，这个仓应该像“只会拿自己外卖号”的前台，而不是“能拿整栋楼总闸”的管理员。

## 为什么这个仓也要写这份合同

虽然 `agent-exporter` 当前没有 `killall`、`pkill`、`osascript` 这类高风险原语，但它确实会做一件和宿主机有关的事:

- 启动一个本地 Codex app-server 子进程
- 在结束时回收这个**自己亲手启动**的子进程

这类能力如果没有边界，很容易从“我只关自己开的那个子进程”滑成“我能顺手起任意命令、顺手清任意环境”。

这份文档的目的，就是把边界写死。

## 这个仓明确允许什么

当前只允许这一条运行时 closeout 路径：

1. 用 `std::process::Command` 启动一个**直接拥有的** app-server child
2. 在 `Drop` 里只对这个 `Child` handle 做回收

换句话说，允许的是“我自己开的那扇门，我自己关”，不允许的是“看到楼道里像我开的门，就全关了”。

## 这个仓明确禁止什么

下面这些原语或行为，不应该进入 `agent-exporter` 的运行时代码：

1. `killall`
2. `pkill`
3. `kill -9`
4. `process.kill(...)`
5. `os.kill(...)`
6. `killpg(...)`
7. `osascript`
8. `System Events`
9. `AppleEvent`
10. `loginwindow`
11. `showForceQuitPanel`
12. `detached: true` + `.unref()`

这些东西的共同问题是：它们更像“按名字、按模式、按桌面环境”动手，而不是只作用于本仓确权的直接 child。

## `--app-server-command` 的安全边界

这个仓保留了一个调试入口：

```bash
--app-server-command <command> --app-server-arg <arg>
```

它存在的意义是：

- 允许默认 `codex app-server`
- 允许 repo-owned test double
- 允许显式可审计的直接可执行文件

它**不是**给 shell 包装器、桌面脚本、宿主机控制命令开的后门。

当前规则是：

1. 拒绝 `osascript`、`killall`、`pkill`、`open`、`sh`、`bash`、`zsh`、`powershell` 等 host-control / shell-style launcher
2. 拒绝 `python -c`、`node -e` 这类 inline-eval 入口
3. 拒绝参数里出现 `System Events`、`loginwindow`、`showForceQuitPanel`、`killall`、`pkill` 等危险模式

## 当前验证门禁

当前仓把宿主机安全收成两层门禁：

1. `cargo test`
   - 会跑 `tests/host_safety_contract.rs`
   - 静态扫描 `src/**`，确认运行时代码没有把危险原语带回来
2. runtime validation
   - `AppServerLaunchConfig::validate_host_safety()`
   - 在 CLI request 构造和 app-server spawn 前各做一次检查

## 对 Agent 的执行要求

以后任何 Agent 在这个仓里做与进程相关的改动时，都要遵守下面 4 条：

1. 只能管理 repo 直接拥有的 app-server child
2. 不能引入 PID 级、进程组级、桌面级 cleanup
3. 不能把 GUI/AppleScript/host-control 能力塞进测试后门
4. 文档、代码、测试三层必须一起更新，不能只改其中一层
