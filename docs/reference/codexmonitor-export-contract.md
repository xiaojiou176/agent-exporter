# CodexMonitor Export Contract

这份文档回答一个最基础的问题：

> 如果 `agent-exporter` 要从 CodexMonitor 当前能力里“继承导出语义”，那我们到底必须继承什么？

说得更直白一点：

**不是所有实现细节都要搬走。**
真正要继承的是 **contract**，也就是当前仓库已经对“完整导出”作出的承诺。

---

## 一句话结论

当前 CodexMonitor 对完整导出的 contract 可以压成 5 条：

1. **`thread/read` 是 primary source**
2. **`thread/resume` 只在 `includeTurns` 被 upstream 拒绝时 fallback**
3. **fallback 导出必须标记为 `incomplete`**
4. **输出路径只能是 `Downloads` 或 `<workspace>/.agents/Conversations`**
5. **Markdown 输出按 round 组织，并按 round 边界切 part**

---

## 必须继承的 contract

### 1. Primary source = `thread/read`

当前仓库已经把这件事写死在文档里了：

- [`docs/app-server-events.md`](../../../../[UIUX]CodexMonitor/CodexMonitor/docs/app-server-events.md) 的说明明确写到：
  - thread history hydration and complete Markdown export use `thread/read` as the primary persisted-history source

这个点非常重要，因为它决定了：

> 当前“完整导出”不是随便从本地缓存里拼一个 transcript，  
> 而是先尊重 app-server 暴露出来的 canonical thread 面。

### 2. `thread/resume` 只是 fallback，不是同级主真源

当前导出实现入口在：

- [`src-tauri/src/exports.rs`](../../../../[UIUX]CodexMonitor/CodexMonitor/src-tauri/src/exports.rs)

其中导出逻辑会：

1. 先调用 `read_thread`
2. 如果错误命中 `includeTurns is unavailable before first user message`
   或 `ephemeral threads do not support includeTurns`
3. 再退到 `resume_thread`

这说明：

> `resume` 是“救场路径”，不是“默认路径”。

### 3. fallback 必须标 `incomplete`

这个 contract 不只写在后端，也写在用户提示层。

后端返回结构里有：

- `is_complete`

前端成功提示里也明确写了：

- 当 `isComplete = false` 时，提示  
  `当前导出来自 live fallback，可能缺少尚未 materialize 的历史。`

这点很关键，因为它提醒未来 `agent-exporter`：

> 不能把“能导出来”误包装成“完整历史已经被证明导出来了”。

### 4. 输出路径 contract 很硬

当前实现只支持两个目标：

1. **Downloads**
2. **`<workspace>/.agents/Conversations`**

而且当 workspace 路径不可用时，当前实现不会静默写到别处，而是明确报错并建议：

> Try exporting to Downloads instead.

这说明：

> 路径不是一个小 UI 选项，而是当前产品 contract 的一部分。

### 5. Markdown 输出结构不是自由发挥

当前导出格式已经很清楚：

- 文件头：
  - 线程 ID
  - 导出时间
  - 条目数
  - 当前分片
  - 包含轮次
- 正文：
  - `# 第N轮`
  - `## 用户`
  - `## 助手`
  - `### 工具`

另外两个很重要的格式细节：

1. 如果 thread 没有显式 opening user message，但有 `preview`，会补一个 synthetic opening user message
2. 分片是按 **round 边界** 做的，不是粗暴按字节切

---

## 未来 `agent-exporter` 应该怎么继承

### v1 必须继承

1. `thread/read` primary source 思维
2. `resume` fallback only 思维
3. `complete / incomplete` 状态语义
4. Downloads / workspaceConversations 双路径语义
5. round-based markdown output

### v1 可以重新组织，但不能改义

1. CLI 命令面可以不同
2. 内部模块切分可以不同
3. 文件名实现细节可以重构
4. typed models 可以重写

但这些重构不能改变上面那 5 条 contract。

---

## 当前 `agent-exporter` v1 落地状态

当前仓已经把下面三件事真正落到了实现里：

1. **主路径**
   - 通过一次性本地 `codex app-server` 连接发送 `thread/read(includeTurns=true)`
2. **fallback**
   - 只在命中当前 CodexMonitor 已知的两类 `includeTurns` 拒绝错误时，才退到 `thread/resume`
3. **导出结果**
   - 输出真实 Markdown 文件
   - 明确写出 `complete / incomplete`
   - 按 round 组织，并按 round 边界 split part

说得更直白一点：

> `agent-exporter` v1 现在不是“计划这样做”，而是“已经按这条 contract 真正在代码里这样做”。  
> 以后如果改主路径、fallback 条件、输出目标语义或 round/part 规则，就不再是普通重构，而是 contract 变更。

---

## 最后一句话

如果以后我们在实现里改了下面任何一条，就不能只当“重构”处理，而要先改文档：

- primary source
- fallback 条件
- incomplete 语义
- 输出路径
- round/part 结构
