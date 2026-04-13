# agent-exporter Design System

## Purpose

这份 `MASTER` 现在锁两层东西：

1. **公开面前门的信息架构**
2. **本地 workbench shell 的视觉语言**

说得更直白一点：

> 它不是“给页面加点样式”的笔记，
> 而是规定 `agent-exporter` 应该长得像什么、先讲什么、绝不能装成什么。

---

## North Star

### 核心原则

- `认知负担最小化`
- `美观`
- `产品化`
- `渐进式披露`
- `evidence before theatre`

### 产品气质

- `local-first`
- `developer-grade`
- `Swiss minimal`
- `calm but precise`
- `not a hosted SaaS façade`

如果用一句话概括：

> 它要像一个真正成熟的本地工程工作台前厅，
> 让人一眼知道从哪开始、下一步去哪、哪些只是侧门，
> 而不是像“功能很多所以全摊出来”的工具杂货铺。

---

## Donor Blend

### Primary donor

- **Vercel**
  - 学它的 `shadow-as-border`
  - 学它的留白控制
  - 学它的“信息像编译器一样被压缩到只剩必要结构”

### Supporting donors

- **Linear**
  - 学它的工具感层级和高密但不乱的信息组织
- **Raycast**
  - 学它的“developer tool 但仍然有产品完成度”的控制感
- **Apple**
  - 学它的 section 节奏与 scene 切换

### 明确不要抄的方向

- 不走 warm parchment / archive cosplay
- 不走 SaaS 渐变大横幅 + logo cloud + stats soup
- 不走 dashboard-card mosaic
- 不走 “功能越多越先堆出来” 的信息噪声

---

## Visual Thesis

### 总体画风

- 以 **冷白 / 雾蓝 / 深墨** 为主
- 背景是干净的 light surface，但带很轻的蓝色气场
- 卡片不是主角，**结构和节奏**才是主角
- 大标题要短、硬、像产品语句，不像营销 slogan
- code / proof / command block 才进入深色面板

### 为什么这样

`agent-exporter` 的可信度来自：

1. 它像工程工具，而不是营销页
2. 它有明确的 proof boundary
3. 它能把复杂 lane 讲得很清楚

所以视觉上必须支持：

- “主航道非常清楚”
- “侧门可见但不会抢戏”
- “每一块都像可操作的产品区，而不是纯装饰”

---

## Color System

### Preferred tokens

| Token | Value | Use |
| --- | --- | --- |
| `bg-canvas` | `#F8FBFF` | 主背景 |
| `bg-depth` | `#EEF3FB` | 底层渐变尾部 |
| `surface` | `rgba(255,255,255,0.88)` | 主要内容面 |
| `surface-strong` | `rgba(255,255,255,0.96)` | Hero / 重点面 |
| `ink-strong` | `#0F172A` | 标题与关键信息 |
| `ink-soft` | `#334155` | 正文 |
| `muted` | `#64748B` | 次级说明 |
| `line` | `rgba(15,23,42,0.08)` | 细边线 |
| `accent-primary` | `#2563EB` | 主 CTA / active state |
| `accent-strong` | `#1D4ED8` | hover / 强调 |
| `accent-soft` | `rgba(37,99,235,0.10)` | 标签 / 弱高亮 |
| `surface-deep` | `#0F172A` | code / proof / terminal 面板 |
| `surface-deep-2` | `#111827` | 深色块过渡 |

### Color laws

- 全局只保留 **一个主强调色：蓝色**
- 成功 / warning / fail 只给状态，不给大面积装饰
- 公开面不要突然跳出第二个品牌色
- 深色面板只服务于：
  - command blocks
  - evidence panels
  - code / proof blocks

---

## Typography

### Public docs

- **Primary:** `IBM Plex Sans`
- **Mono:** `JetBrains Mono`

原因：

- 比 Inter 更有工程感
- 比系统字体更有产品完成度
- 和 developer-tool / workbench 气质更贴

### Local generated shells

- 优先使用：
  - `IBM Plex Sans`
  - `JetBrains Mono`
- 本地无 web font 时回退：
  - `SF Pro Text / Display`
  - `Segoe UI`
  - `Menlo / SFMono`

### Type rules

- H1 像产品句子，不像文章标题
- H2 是分流路标，不是抽象理论标签
- mono label 只用于：
  - 小型 status label
  - command / path / state
  - tiny metadata
- 不要大段大写
- 不要用粗体堆层级，要用 size / spacing / grouping 建层级

---

## Layout Laws

### 首页骨架

固定顺序：

1. 一句话产品句子
2. 三步 first success
3. 你会得到什么
4. `does / does not prove`
5. proof ladder
6. 再打开 secondary surfaces

### Progressive disclosure

- 一屏内只回答“我该不该继续看”
- 第二屏才回答“我具体怎么开始”
- 第三屏开始回答“还有哪些 side lanes”
- 不要把 integration / governance / reports 一开始就全摊出来

### Section rule

每个 section 只做一件事：

- explain
- prove
- route
- compare

不能一段同时承担四件事。

---

## Component Grammar

### Hero

- 像 poster，不像文章导语
- 左边给一句产品句子 + 一段 lead
- 右边给 `at a glance`
- CTA 只保留 1 个 primary，其他都是 secondary

### Cards

- 不是默认 everywhere
- 只有在“这个块本身就是一个独立 decision unit / route unit”时才使用
- 常见合法 card：
  - first success step
  - proof ladder level
  - side lane entry
  - meta summary block

### Command blocks

- 深色
- 圆角大
- 像终端面板，不像 Markdown 默认代码块
- command 前先有人话解释这一步干嘛

### Tables

- 用于：
  - proof comparisons
  - lane maps
  - release shelf truth
- 不要把所有内容都塞进表格

### Details / disclosure

- 用来承接 second-ring info
- 非主路径内容优先折叠
- summary 必须像问题，不要像文件名

---

## Workbench Shell Rules

### Archive shell / reports shell / integration evidence

三类页面要像同一个产品家族：

- 同一套背景和 surface tokens
- 同一套 mono label 规则
- 同一套 search / filter button 语言
- 同一套 CTA 按钮和 focus ring

### 真实目标

让用户感觉：

- 这是一个统一的 local workbench
- transcript 是主航道
- reports 和 integration evidence 是平行侧门
- governance 是清晰可读的制度面，不是 UI 噪声

---

## Copy / IA Rules

- 先讲用户得到什么，再讲系统结构
- “这是哪条 lane” 要讲清楚
- “这不代表什么” 要明确写出来
- 不要上来解释三层 architecture theory
- 不要用 hosted / platform / live runtime 语言偷渡能力暗示

---

## Anti-Patterns

- warm archive cosplay
- generic SaaS landing tropes
- hero 上来就讲全部 surface theory
- secondary surfaces 抢主门
- 过度装饰的 gradients / blobs / neon
- 一页里同时出现多套颜色逻辑
- 看起来像“功能总表”，而不是“前门导览”
