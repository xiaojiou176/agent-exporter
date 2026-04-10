# agent-exporter Design System

## Purpose

这份 `MASTER` 锁的不是 CSS，而是**工作台前门的叙事顺序**。

它负责统一：

- 产品句子
- proof ladder
- does / does not prove 文案
- secondary surface 何时出现、出现到什么程度

## Tone

- `local-first`
- `archival dossier`
- `Swiss minimal`
- `evidence before theatre`

说得更直白一点：

> 它应该像一个可信的档案工作台前厅，  
> 不是 newsletter，也不是 hosted SaaS 首页。

## Color Direction

- 正文：浅纸面背景
- 正文字色：深 slate / navy
- 主强调色：单一蓝色，只给 CTA、proof 跳转、关键状态
- 深色块：只给 code / proof / evidence 区块

### Preferred tokens

| Token | Value | Use |
| --- | --- | --- |
| `ink-strong` | `#0F172A` | 标题与正文 |
| `ink-muted` | `#334155` | 次级说明 |
| `accent-primary` | `#0369A1` | CTA / proof jump |
| `surface-soft` | `#F8FAFC` | 浅底卡片 |
| `surface-deep` | `#020617` | 深色 code / proof 面板 |

## Type Rules

- 标题短、硬、少形容词
- 首屏不要连续解释三层 surface theory
- 命令块前先有一句人话
- `does / does not prove` 用表格或 callout 固定组件

## Proof Ladder

| Level | It proves | Typical artifact |
| --- | --- | --- |
| `L1` | CLI quickstart 真能跑 | command block |
| `L2` | transcript export 真会留下 HTML receipt | transcript HTML |
| `L3` | archive shell 真把本地 workbench 串起来 | proof page / proof ladder diagram |

## Front-Door Structure

首屏固定按这个顺序排：

1. 一句话产品句子
2. 三步 first success
3. 你会得到什么
4. `does / does not prove`
5. 再介绍 secondary surfaces

## Secondary Surface Visibility

- secondary surfaces 必须可见
- 但只能作为“后续入口卡片”
- 不得在首屏和 primary path 争同一层注意力

## Companion Docs Roles

- `docs/index.md`：入口页，告诉人先做什么
- `docs/README.md`：目录页，告诉人去哪看
- `docs/repo-map.md`：地图页，告诉人东西放哪
- `docs/archive-shell-proof.md`：证明页，告诉人当前 proof 到底证明了什么

## Anti-Patterns

- 上来先上架构课
- 只用一张图解释所有 proof
- 把 secondary surfaces 摊成第一屏
- 用 hosted/platform 语言暗示当前已经不是 local-first
