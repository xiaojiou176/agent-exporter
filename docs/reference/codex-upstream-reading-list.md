# Official Codex Reading List

这份文档回答的问题是：

> 如果未来 `agent-exporter` 要真正理解 Codex 的 thread/source 真相层，官方源码里必须读哪几块？

说人话：

CodexMonitor 告诉我们“导出成品长什么样”，  
但 **官方 Codex** 才告诉我们“thread 这个东西到底是怎么被拼出来的”。

---

## 一句话结论

官方 Codex 里最该读的不是一大堆 UI 或整个 app-server，而是这 4 层：

1. **`thread/read` 聚合链**
2. **sqlite `threads` 元数据层**
3. **rollout 历史层**
4. **turn 重建层**

如果把这 4 层看懂了，未来的 `Codex source adapter` 基本就不会走歪。

---

## 必读锚点

### 1. `thread/read` 是 canonical 聚合入口

最关键文件：

- `codex-rs/app-server/src/codex_message_processor.rs`

最关键阅读点：

1. 解析 `thread_id`
2. 先尝试从 sqlite summary 读
3. 必要时定位 `rollout_path`
4. `includeTurns=true` 时再去读 rollout items
5. 最后 build turns

这一步给我们的核心启发是：

> `thread/read` 不是“直接把一个 json 文件读回来”。  
> 它是一个聚合器。

### 2. sqlite `threads` 表是元数据索引层

最关键文件：

- `codex-rs/state/src/runtime.rs`
- `codex-rs/state/src/runtime/threads.rs`

这一层回答的问题是：

- `state_5.sqlite` 在哪
- `threads` 表有哪些字段
- `rollout_path`、`cwd`、`title`、`model`、`git info` 等元数据从哪来

这层最适合被理解成：

> **目录卡**

它告诉你“卷宗在哪”，但它本身不是完整 transcript。

### 3. rollout 文件是正文层

最关键文件：

- `codex-rs/rollout/src/recorder.rs`

这一层回答的问题是：

- rollout history 怎么读
- 哪个路径才是真正的会话正文

这层最适合被理解成：

> **档案柜里的原始卷宗**

### 4. turns 是重建出来的，不是平铺白送的

最关键文件：

- `codex-rs/app-server-protocol/src/protocol/thread_history.rs`

这一层回答的问题是：

- rollout items 怎么变成 turns
- item index / turn id / 顺序语义怎么保留

这一层是未来 `archive core` 最值得参考的地方之一。

---

## 两个最容易被忽略的附加语义

### 1. thread name 不是原始 rollout 自带的

官方还有一层 `attach_thread_name(...)` 逻辑。

这说明：

> local direct-read 如果只读 rollout，不等于天然拿到了完整 thread name 语义。

### 2. status / interrupted 语义是 overlay 上去的

官方会把 stale `InProgress` turn 改成 `Interrupted`。

这说明：

> local direct-read 可以非常有价值，  
> 但它不是自动等于 canonical thread parity。

---

## 未来设计里应该怎么用这些锚点

### v1 必须参考

1. `thread/read` 聚合链
2. `threads` 元数据层
3. rollout history
4. `build_turns_from_rollout_items`

### v2 才需要更深参考

1. name attach
2. status overlay
3. interrupt 修正
4. 其他 loaded-thread / ephemeral 细节

---

## 当前 v1 已采用的建模边界

当前 `agent-exporter` 已经按官方 Codex 的三层真相把实现拆开：

1. **canonical summary**
   - 线程级别的稳定元数据
2. **archival turns**
   - 通过 rollout replay 重建出来的 `turns`
3. **app-server overlay**
   - `thread.name`
   - `thread.status`
   - stale in-progress turn 的 interrupted 修正可能性

这意味着：

> 我们没有把 rollout body 误说成 canonical thread parity，  
> 也没有把 sqlite `title` 误映射成 app-server `thread.name`。

---

## canonical truth vs archival truth

这两个词必须在仓库里讲清楚。

### canonical truth

来自：

- `thread/read`
- app-server 聚合结果

适合用在：

- “完整导出”
- complete / incomplete 语义
- v1 app-server source

### archival truth

来自：

- sqlite 元数据
- rollout jsonl
- 未来 local direct-read source

适合用在：

- 本地档案直读
- forensics / audit
- v2 local source

---

## 最后一句话

未来实现 `Codex source adapter` 时，最危险的误判不是“不会写代码”，而是：

> **把 archival truth 假装成 canonical truth。**

这份文档的目的，就是把这个边界提前写死。
