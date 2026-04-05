# External Repo Reading List

这份文档记录的是：

> 做 `agent-exporter` 时，哪些外部仓真的值得参考，  
> 以及每个仓到底该借哪一层、不该抄哪一层。

这里的原则很简单：

- **不是谁 stars 高就先学谁**
- **不是谁功能多就先学谁**
- **谁最直接解决我们当前阶段的问题，谁优先**

---

## 当前最值得参考的 4 个仓

### 1. `claude-conversation-extractor`

定位：

- 本地 undocumented JSONL 会话的 direct-read 导出工具

最该借：

- local direct-read exporter 的 framing
- `detailed transcript` 思维
- 极轻量 CLI

最不该照抄：

- 单来源小工具的整体形态
- 输出目录策略
- 把 local direct-read 直接当 canonical truth

适合在我们这里扮演的角色：

> `local source` 参考一号位

### 2. `claude-code-log`

定位：

- 会话转 HTML/Markdown + TUI 浏览 + 项目层级组织

最该借：

- 项目层级导出组织
- HTML/Markdown 双输出思维
- 会话目录组织、缓存、浏览体验

最不该照抄：

- 浏览器日志站式的产品重心
- 把 exporter 直接做成 history website

适合在我们这里扮演的角色：

> 输出层 / 浏览层参考

### 3. `claude-code-transcripts`

定位：

- 发布型 transcript / HTML transcript / 分享型输出

最该借：

- CLI 子命令切分
- output directory 设计
- local/json/all 模式划分
- publish/shareable transcript 思维

最不该照抄：

- 把 v1 重心放到 publishable HTML
- 把 share/gist 能力当成首要目标

适合在我们这里扮演的角色：

> CLI 命令面 + 发布型 transcript 参考

### 4. `coding_agent_session_search`

定位：

- 跨 agent 统一搜索 / 统一 schema / 多 connector 平台

最该借：

- connector 世界观
- 多 agent schema
- robot/CLI/TUI 分层
- 长期 v2/v3 演化方向

最不该照抄：

- v1 直接做成大而全平台
- 一开始就把 search/index/semantic/MCP server 全拉进来

适合在我们这里扮演的角色：

> v2/v3 方向参考，而不是 v1 模板

---

## 当前参考优先级矩阵

| Repo | 现在值不值得看 | 该借哪一层 | 不该照抄哪一层 | 对 `agent-exporter` 的角色 |
| --- | --- | --- | --- | --- |
| `claude-conversation-extractor` | 必须看 | local direct-read、detailed transcript、offline framing | 单来源、小工具整体形态 | `local source` 参考 |
| `claude-code-log` | 必须看 | 项目层级导出、TUI 浏览、HTML/Markdown 双输出 | 浏览器日志站式产品重心 | 输出层 / 浏览层参考 |
| `claude-code-transcripts` | 必须看 | publishable transcript、分页、分享输出、子命令切分 | 过早把 HTML 发布当主目标 | CLI / output 参考 |
| `coding_agent_session_search` | 必须看 | connector 视角、统一 schema、长期 search/index 方向 | v1 直接做成这么重的平台 | v2/v3 方向参考 |

---

## 当前 phase 顺序该借谁

### v1

目标：

- 先把 Codex 导出做对
- 先建立 archive core
- 先稳住 contract

该借：

- `claude-conversation-extractor`
- `claude-code-transcripts`
- `claude-code-log`

### v2

目标：

- 加 local direct-read
- 补更完整的 output 面
- 开始出现多 source

该借：

- `claude-conversation-extractor`
- `claude-code-log`
- `coding_agent_session_search`

### Phase 3

目标：

- 证明 second connector 能接进共享 archive / Markdown contract
- 最小范围接入 Claude Code
- 不做 auto-discovery / HTML-first / browser-first 体验层

该借：

- `claude-code-transcripts`
- `claude-code-log`
- `claude-conversation-extractor`

### Later

目标：

- 走向更强的 output / browse / search / index

该借：

- `coding_agent_session_search`

---

## 一句话总建议

如果只能记一句话，就记这句：

> **当前 second connector proof 先学 `claude-code-transcripts + claude-code-log + claude-conversation-extractor`，**
> **以后再学 `coding_agent_session_search`。**

这样做的好处是：

- 今天先把导出做对
- 明天再把平台做大

而不是第一天就把自己做成一个搜索平台。

---

## 当前已采纳的借鉴结果

本仓当前已经明确采纳了下面几层，而没有把整个参考仓整包搬进来：

### 已采纳

1. **`claude-code-transcripts` 的命令收口思路**
   - 当前真实 public export 命令收口为 `export codex ...`
   - source 维度只扩到 `app-server|local`
2. **`claude-code-log` 的 typed transcript -> renderer 分层**
   - source 先把上游 payload 变成 typed archive core
   - output 再把 typed archive core 渲染成 Markdown
3. **bundle-aware 的输出心态**
   - 当前一次导出会生成 thread-scoped Markdown part 文件，而不是继续停在 plan-only stdout
4. **`claude-conversation-extractor` 的 local direct-read framing**
   - local source 已落地为 Codex connector 的第二入口
   - 但没有把整个单来源小工具边界搬进 repo
5. **Phase 3 Claude second connector proof**
   - `claude-code --session-path <PATH>` landed
   - shared archive core / shared Markdown contract reused
   - no auto-discovery / TUI / browser / HTML-first drift

### 明确未采纳

1. `claude-conversation-extractor` 的 local direct-read 主路径
2. `claude-code-log` 的 TUI / browser / HTML-first 重心
3. `claude-code-transcripts` 的 publish/share/web/gist 能力
4. `coding_agent_session_search` 的 search/index/pages/models/sources 平台壳
