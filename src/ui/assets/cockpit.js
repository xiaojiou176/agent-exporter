const state = {
  threads: [],
  focusedThreadId: null,
  selectedThreadIds: new Set(),
  expandedWorkspaceKeys: new Set(),
  discoveryMeta: null,
  activeExportJobId: null,
  exportPollTimer: null,
  prefs: {
    locale: "en",
    workspaceLabels: {},
  },
};

const STRINGS = {
  en: {
    "hero.eyebrow": "local-first export cockpit",
    "hero.title": "Export Codex threads or workspace-local Claude sessions into local archive workbenches.",
    "hero.lead":
      "The cockpit prefers live Codex thread discovery, augments it with workspace-local Claude sessions, groups them by workspace, and exports each selected session back into its own workspace shell.",
    "hero.note":
      "This is still a local helper after the CLI path. The public front door remains the CLI quickstart.",
    "meta.launchRoot": "launch root",
    "meta.codexHome": "codex home",
    "meta.discoveryMode": "discovery mode",
    "detection.eyebrow": "detection",
    "detection.title": "Detected workspaces and threads",
    "detection.searchLabel": "Search detected threads",
    "detection.searchPlaceholder": "Search by title, workspace, model, or thread id",
    "selection.eyebrow": "selected threads",
    "selection.empty": "Choose one or more threads to inspect export details.",
    "selection.none": "No threads selected.",
    "selection.single": "1 thread selected.",
    "selection.multi": "{count} threads selected across {workspaceCount} workspaces.",
    "selection.clear": "Clear selection",
    "selection.summaryTitle": "Batch export summary",
    "selection.summaryCount": "Selected count",
    "selection.summaryWorkspaces": "Workspaces",
    "selection.summaryThreads": "Selected threads",
    "action.eyebrow": "action",
    "action.aiSummary": "Add optional AI summary",
    "action.aiInstructions": "Optional AI summary instructions",
    "action.aiPlaceholder": "Extra instructions for the AI synthesis, if needed.",
    "action.aiProfile": "Optional AI summary profile",
    "action.aiProfilePlaceholder": "Codex profile for the AI synthesis",
    "action.aiPreset": "Optional AI summary preset",
    "action.aiPresetPlaceholder": "handoff, bug-rca, decision, release...",
    "action.aiModel": "Optional AI summary model",
    "action.aiModelPlaceholder": "Model override for the AI synthesis",
    "action.aiProvider": "Optional AI summary provider",
    "action.aiProviderPlaceholder": "Provider override for the AI synthesis",
    "action.button.single": "Export selected thread",
    "action.button.multi": "Export {count} threads",
    "action.note":
      "Uses the canonical Codex export path, then publishes archive / reports / evidence shells for each affected workspace.",
    "result.eyebrow": "result",
    "result.empty": "No export has run yet.",
    "result.done": "Exported {count} thread(s) across {workspaceCount} workspace(s).",
    "result.workspace": "Workspace",
    "result.thread": "Thread",
    "result.exportedFile": "Exported transcript",
    "result.openArchive": "Open archive shell",
    "result.openReports": "Open reports shell",
    "result.openEvidence": "Open evidence shell",
    "result.openSummary": "Open AI summary",
    "result.copyBundle": "Copy export block",
    "result.copyPath": "Copy path",
    "result.path": "Path",
    "result.warning": "Warning",
    "result.progress": "Export progress",
    "result.running": "Running {phase} for {elapsed}",
    "result.phaseQueued": "queued",
    "result.phaseExporting": "raw export",
    "result.phaseAiSummary": "AI summary",
    "result.phasePublishing": "publish",
    "result.phaseCompleted": "completed",
    "result.stepRunning": "Running",
    "result.stepCompleted": "Completed",
    "result.stepWarning": "Completed with warning",
    "result.stepFailed": "Failed",
    "cli.eyebrow": "cli equivalent",
    "cli.note":
      "This cockpit is a local helper, not a hidden execution layer. The commands below show the closest CLI path for the current selection.",
    "cli.empty": "Select one or more threads to preview the equivalent CLI command.",
    "refresh": "Refresh",
    "renameWorkspace": "Rename",
    "renameWorkspacePrompt": "Rename this workspace group",
    "selectAll": "Select all",
    "unselectAll": "Clear group",
    "thread.untitled": "Untitled thread {id}",
    "thread.modelUnknown": "unknown model",
    "thread.updated": "updated {time}",
    "thread.title": "Title",
    "thread.id": "Thread",
    "thread.workspace": "Workspace",
    "thread.workspacePath": "Workspace path",
    "thread.model": "Model",
    "thread.connector": "Connector",
    "thread.updatedAt": "Updated",
    "thread.createdAt": "Created",
    "thread.cwd": "CWD",
    "thread.artifactPath": "Artifact",
    "thread.discovery": "Discovery",
    "thread.select": "Select thread",
    "workspace.threads": "{count} thread(s)",
    "status.loadingThreads": "Loading active top-level Codex threads…",
    "status.loadFailed": "Failed to load threads: {error}",
    "status.none": "No active top-level Codex threads were found.",
    "status.detected": "Detected {count} active top-level Codex thread(s) across {groupCount} workspace group(s).",
    "status.showing": "Showing {count} of {total} thread(s) across {groupCount} workspace group(s).",
    "status.noFilter": "No threads match the current search.",
    "status.exportRunning": "Running export for the selected threads…",
    "status.exportFailed": "Export failed: {error}",
    "status.exportStarting": "Starting export job…",
    "time.justNow": "just now",
    "time.minutesAgo": "{count}m ago",
    "time.hoursAgo": "{hours}h ago",
    "time.hoursMinutesAgo": "{hours}h {minutes}m ago",
    "time.daysAgo": "{count}d ago",
    "time.unknown": "unknown",
    "locale.toggle": "中文",
  },
  zh: {
    "hero.eyebrow": "本地优先导出驾驶舱",
    "hero.title": "把 Codex 对话或工作区内的 Claude session 导出到本地归档工作台。",
    "hero.lead":
      "这个 cockpit 会优先使用 live Codex thread discovery，再补上 workspace-local Claude session，按 workspace 分组，并把每个选中的 session 导回它自己的 workspace shell。",
    "hero.note":
      "它仍然是 CLI 之后的本地辅助面，不是隐藏执行层。公开 front door 依然是 CLI quickstart。",
    "meta.launchRoot": "启动根目录",
    "meta.codexHome": "Codex Home",
    "meta.discoveryMode": "发现模式",
    "detection.eyebrow": "发现",
    "detection.title": "已发现的工作区与会话",
    "detection.searchLabel": "搜索已发现的会话",
    "detection.searchPlaceholder": "按标题、工作区、模型或线程 ID 搜索",
    "selection.eyebrow": "已选会话",
    "selection.empty": "请选择一个或多个会话以查看导出详情。",
    "selection.none": "当前没有选中任何会话。",
    "selection.single": "当前选中了 1 个会话。",
    "selection.multi": "当前选中了 {count} 个会话，涉及 {workspaceCount} 个工作区。",
    "selection.clear": "清空选择",
    "selection.summaryTitle": "批量导出摘要",
    "selection.summaryCount": "已选数量",
    "selection.summaryWorkspaces": "涉及工作区",
    "selection.summaryThreads": "已选会话",
    "action.eyebrow": "动作",
    "action.aiSummary": "添加可选 AI 摘要",
    "action.aiInstructions": "可选 AI 摘要说明",
    "action.aiPlaceholder": "如有需要，可追加摘要说明。",
    "action.aiProfile": "可选 AI 摘要 Profile",
    "action.aiProfilePlaceholder": "用于 AI 摘要的 Codex profile",
    "action.aiPreset": "可选 AI 摘要 Preset",
    "action.aiPresetPlaceholder": "handoff、bug-rca、decision、release 等",
    "action.aiModel": "可选 AI 摘要模型",
    "action.aiModelPlaceholder": "用于 AI 摘要的模型覆盖",
    "action.aiProvider": "可选 AI 摘要 Provider",
    "action.aiProviderPlaceholder": "用于 AI 摘要的 provider 覆盖",
    "action.button.single": "导出所选会话",
    "action.button.multi": "导出 {count} 个会话",
    "action.note":
      "会走 canonical Codex export path，并为每个受影响的 workspace 生成 archive / reports / evidence shell。",
    "result.eyebrow": "结果",
    "result.empty": "还没有执行任何导出。",
    "result.done": "已导出 {count} 个会话，涉及 {workspaceCount} 个工作区。",
    "result.workspace": "工作区",
    "result.thread": "会话",
    "result.exportedFile": "已导出文件",
    "result.openArchive": "打开 archive shell",
    "result.openReports": "打开 reports shell",
    "result.openEvidence": "打开 evidence shell",
    "result.openSummary": "打开 AI 摘要",
    "result.copyBundle": "复制本次结果",
    "result.copyPath": "复制路径",
    "result.path": "路径",
    "result.warning": "警告",
    "result.progress": "导出进度",
    "result.running": "正在进行 {phase}，已持续 {elapsed}",
    "result.phaseQueued": "排队中",
    "result.phaseExporting": "原始导出",
    "result.phaseAiSummary": "AI 摘要",
    "result.phasePublishing": "发布 shell",
    "result.phaseCompleted": "已完成",
    "result.stepRunning": "进行中",
    "result.stepCompleted": "已完成",
    "result.stepWarning": "完成但有警告",
    "result.stepFailed": "失败",
    "cli.eyebrow": "CLI 等效项",
    "cli.note":
      "这个 cockpit 是本地辅助面，不是隐藏执行层。下面这些命令展示了当前选择最接近的 CLI 路径。",
    "cli.empty": "请选择一个或多个会话以预览对应的 CLI 命令。",
    "refresh": "刷新",
    "renameWorkspace": "重命名",
    "renameWorkspacePrompt": "重命名这个工作区分组",
    "selectAll": "全选",
    "unselectAll": "清空分组",
    "thread.untitled": "未命名会话 {id}",
    "thread.modelUnknown": "未知模型",
    "thread.updated": "{time}更新",
    "thread.title": "标题",
    "thread.id": "线程",
    "thread.workspace": "工作区",
    "thread.workspacePath": "工作区路径",
    "thread.model": "模型",
    "thread.connector": "连接器",
    "thread.updatedAt": "最近更新",
    "thread.createdAt": "创建时间",
    "thread.cwd": "CWD",
    "thread.artifactPath": "工件路径",
    "thread.discovery": "发现来源",
    "thread.select": "选择会话",
    "workspace.threads": "{count} 个会话",
    "status.loadingThreads": "正在加载活跃主会话…",
    "status.loadFailed": "加载会话失败：{error}",
    "status.none": "没有发现活跃的主会话。",
    "status.detected": "已发现 {count} 个活跃主会话，分布在 {groupCount} 个工作区分组中。",
    "status.showing": "当前显示 {count}/{total} 个会话，分布在 {groupCount} 个工作区分组中。",
    "status.noFilter": "当前搜索条件没有匹配到任何会话。",
    "status.exportRunning": "正在导出已选会话…",
    "status.exportFailed": "导出失败：{error}",
    "status.exportStarting": "正在启动导出任务…",
    "time.justNow": "刚刚",
    "time.minutesAgo": "{count}分钟前",
    "time.hoursAgo": "{hours}小时前",
    "time.hoursMinutesAgo": "{hours}小时{minutes}分钟前",
    "time.daysAgo": "{count}天前",
    "time.unknown": "未知",
    "locale.toggle": "EN",
  },
};

const threadListEl = document.getElementById("thread-list");
const detailEl = document.getElementById("thread-detail");
const resultStatusEl = document.getElementById("result-status");
const resultLinksEl = document.getElementById("result-links");
const detectionStatusEl = document.getElementById("detection-status");
const refreshButtonEl = document.getElementById("refresh-button");
const exportButtonEl = document.getElementById("export-button");
const aiSummaryToggleEl = document.getElementById("ai-summary-toggle");
const aiSummaryPanelEl = document.getElementById("ai-summary-panel");
const aiSummaryInstructionsEl = document.getElementById("ai-summary-instructions");
const aiSummaryProfileEl = document.getElementById("ai-summary-profile");
const aiSummaryPresetEl = document.getElementById("ai-summary-preset");
const aiSummaryModelEl = document.getElementById("ai-summary-model");
const aiSummaryProviderEl = document.getElementById("ai-summary-provider");
const threadSearchEl = document.getElementById("thread-search");
const threadSearchStatusEl = document.getElementById("thread-search-status");
const workspaceRootValueEl = document.getElementById("workspace-root-value");
const codexHomeValueEl = document.getElementById("codex-home-value");
const discoveryModeValueEl = document.getElementById("discovery-mode-value");
const commandPreviewEl = document.getElementById("command-preview");
const localeToggleEl = document.getElementById("locale-toggle");
const selectionSummaryEl = document.getElementById("selection-summary");
const clearSelectionButtonEl = document.getElementById("clear-selection-button");

function locale() {
  return state.prefs.locale === "zh" ? "zh" : "en";
}

function t(key, vars = {}) {
  const table = STRINGS[locale()] ?? STRINGS.en;
  let template = table[key] ?? STRINGS.en[key] ?? key;
  for (const [name, value] of Object.entries(vars)) {
    template = template.replaceAll(`{${name}}`, String(value));
  }
  return template;
}

function trimmedValue(element) {
  return element?.value?.trim() || "";
}

function previewArg(value) {
  const text = String(value ?? "");
  return /[\s"'\\]/.test(text) ? JSON.stringify(text) : text;
}

function selectedThreads() {
  return state.threads.filter((thread) => state.selectedThreadIds.has(thread.threadId));
}

function focusedThread() {
  return state.threads.find((thread) => thread.threadId === state.focusedThreadId) ?? null;
}

function filteredThreads() {
  const query = threadSearchEl?.value?.trim().toLowerCase() ?? "";
  if (!query) return state.threads;
  return state.threads.filter((thread) =>
    [
      thread.displayName,
      thread.modelProvider,
      thread.workspaceLabel,
      thread.workspacePath,
      thread.cwd,
      thread.connectorKind,
      thread.threadId,
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(query)),
  );
}

function groupThreadsByWorkspace(threads) {
  const groups = new Map();
  for (const thread of threads) {
    const key = thread.workspaceKey ?? thread.workspacePath ?? thread.cwd ?? "unknown-workspace";
    if (!groups.has(key)) {
      groups.set(key, {
        workspaceKey: key,
        workspaceLabel: thread.workspaceLabel ?? "Unknown workspace",
        workspacePath: thread.workspacePath ?? thread.cwd ?? "(unknown)",
        updatedAt: thread.updatedAt ?? 0,
        threads: [],
      });
    }
    const group = groups.get(key);
    group.updatedAt = Math.max(group.updatedAt ?? 0, thread.updatedAt ?? 0);
    group.threads.push(thread);
  }

  return Array.from(groups.values())
    .sort((left, right) => (right.updatedAt ?? 0) - (left.updatedAt ?? 0))
    .map((group) => ({
      ...group,
      threads: group.threads.sort((left, right) => {
        const updatedDiff = (right.updatedAt ?? 0) - (left.updatedAt ?? 0);
        if (updatedDiff !== 0) return updatedDiff;
        return String(left.displayName).localeCompare(String(right.displayName));
      }),
    }));
}

function formatRelativeTime(value) {
  if (typeof value !== "number" || Number.isNaN(value)) return t("time.unknown");
  const asDate =
    value > 1_000_000_000_000
      ? value
      : value > 1_000_000_000
        ? value * 1000
        : null;
  if (!asDate) return t("time.unknown");
  const diffMinutes = Math.max(0, Math.floor((Date.now() - asDate) / 60_000));
  if (diffMinutes < 1) return t("time.justNow");
  if (diffMinutes < 60) return t("time.minutesAgo", { count: diffMinutes });
  if (diffMinutes < 24 * 60) {
    const hours = Math.floor(diffMinutes / 60);
    const minutes = diffMinutes % 60;
    if (minutes === 0) return t("time.hoursAgo", { hours });
    return t("time.hoursMinutesAgo", { hours, minutes });
  }
  const days = Math.floor(diffMinutes / (24 * 60));
  return t("time.daysAgo", { count: days });
}

function formatDurationSince(timestamp) {
  if (!timestamp) return t("time.unknown");
  const start = new Date(timestamp).getTime();
  if (Number.isNaN(start)) return t("time.unknown");
  const diffMinutes = Math.max(0, Math.floor((Date.now() - start) / 60_000));
  if (diffMinutes < 1) return t("time.justNow");
  if (diffMinutes < 60) return t("time.minutesAgo", { count: diffMinutes });
  const hours = Math.floor(diffMinutes / 60);
  const minutes = diffMinutes % 60;
  if (hours < 24) {
    if (minutes === 0) return t("time.hoursAgo", { hours });
    return t("time.hoursMinutesAgo", { hours, minutes });
  }
  const days = Math.floor(hours / 24);
  return t("time.daysAgo", { count: days });
}

function humanPhaseLabel(phase) {
  if (!phase) return t("result.phaseQueued");
  if (phase.startsWith("exporting_raw_")) return t("result.phaseExporting");
  if (phase.startsWith("ai_summary_")) return t("result.phaseAiSummary");
  if (phase.startsWith("publishing ")) return t("result.phasePublishing");
  if (phase === "completed") return t("result.phaseCompleted");
  if (phase === "queued") return t("result.phaseQueued");
  if (phase === "failed") return t("result.stepFailed");
  return phase;
}

function applyStaticText() {
  document.documentElement.lang = locale() === "zh" ? "zh-CN" : "en";
  document.getElementById("hero-eyebrow").textContent = t("hero.eyebrow");
  document.getElementById("hero-title").textContent = t("hero.title");
  document.getElementById("hero-lead").textContent = t("hero.lead");
  document.getElementById("hero-note").textContent = t("hero.note");
  document.getElementById("launch-root-label").textContent = t("meta.launchRoot");
  document.getElementById("codex-home-label").textContent = t("meta.codexHome");
  document.getElementById("discovery-mode-label").textContent = t("meta.discoveryMode");
  document.getElementById("detection-eyebrow").textContent = t("detection.eyebrow");
  document.getElementById("detection-title").textContent = t("detection.title");
  document.getElementById("thread-search-label").textContent = t("detection.searchLabel");
  threadSearchEl.placeholder = t("detection.searchPlaceholder");
  document.getElementById("selected-thread-eyebrow").textContent = t("selection.eyebrow");
  document.getElementById("action-eyebrow").textContent = t("action.eyebrow");
  document.getElementById("ai-summary-label").textContent = t("action.aiSummary");
  document.getElementById("ai-summary-instructions-label").textContent = t("action.aiInstructions");
  aiSummaryInstructionsEl.placeholder = t("action.aiPlaceholder");
  document.getElementById("ai-summary-profile-label").textContent = t("action.aiProfile");
  aiSummaryProfileEl.placeholder = t("action.aiProfilePlaceholder");
  if (aiSummaryPresetEl) {
    document.getElementById("ai-summary-preset-label").textContent = t("action.aiPreset");
    aiSummaryPresetEl.placeholder = t("action.aiPresetPlaceholder");
  }
  document.getElementById("ai-summary-model-label").textContent = t("action.aiModel");
  aiSummaryModelEl.placeholder = t("action.aiModelPlaceholder");
  document.getElementById("ai-summary-provider-label").textContent = t("action.aiProvider");
  aiSummaryProviderEl.placeholder = t("action.aiProviderPlaceholder");
  document.getElementById("result-eyebrow").textContent = t("result.eyebrow");
  document.getElementById("cli-eyebrow").textContent = t("cli.eyebrow");
  document.getElementById("cli-note").textContent = t("cli.note");
  clearSelectionButtonEl.textContent = t("selection.clear");
  localeToggleEl.textContent = t("locale.toggle");
  refreshButtonEl.textContent = t("refresh");
  document.getElementById("action-note").textContent = t("action.note");
}

async function loadPreferences() {
  try {
    const response = await fetch("/api/preferences");
    const data = await response.json();
    state.prefs.locale = data.locale === "zh" ? "zh" : "en";
    state.prefs.workspaceLabels = data.workspaceLabels ?? {};
  } catch (_error) {
    state.prefs.locale = "en";
    state.prefs.workspaceLabels = {};
  }
}

async function persistPreferences() {
  await fetch("/api/preferences", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(state.prefs),
  });
}

function toggleThreadSelection(threadId) {
  if (state.selectedThreadIds.has(threadId)) {
    state.selectedThreadIds.delete(threadId);
    state.focusedThreadId =
      state.threads.find((thread) => state.selectedThreadIds.has(thread.threadId))?.threadId ?? null;
  } else {
    state.selectedThreadIds.add(threadId);
    state.focusedThreadId = threadId;
  }
}

function setAllThreadsInGroup(group, shouldSelect) {
  for (const thread of group.threads) {
    if (shouldSelect) {
      state.selectedThreadIds.add(thread.threadId);
    } else {
      state.selectedThreadIds.delete(thread.threadId);
    }
  }
  if (shouldSelect && group.threads[0]) {
    state.focusedThreadId = group.threads[0].threadId;
  } else {
    state.focusedThreadId =
      state.threads.find((thread) => state.selectedThreadIds.has(thread.threadId))?.threadId ?? null;
  }
}

async function renameWorkspace(group) {
  const current = state.prefs.workspaceLabels[group.workspacePath] ?? group.workspaceLabel;
  const next = window.prompt(t("renameWorkspacePrompt"), current);
  if (next === null) return;
  const trimmed = next.trim();
  if (trimmed) {
    state.prefs.workspaceLabels[group.workspacePath] = trimmed;
  } else {
    delete state.prefs.workspaceLabels[group.workspacePath];
  }
  await persistPreferences();
  state.threads = state.threads.map((thread) =>
    thread.workspacePath === group.workspacePath
      ? {
          ...thread,
          workspaceLabel: state.prefs.workspaceLabels[group.workspacePath] ?? group.workspacePath.split("/").pop(),
        }
      : thread,
  );
  renderThreads();
  renderSelection();
}

function renderThreads() {
  threadListEl.innerHTML = "";
  const visibleThreads = filteredThreads();

  if (state.threads.length === 0) {
    detectionStatusEl.textContent = t("status.none");
    threadSearchStatusEl.textContent = t("selection.none");
    renderSelection();
    return;
  }

  const allGroups = groupThreadsByWorkspace(state.threads);
  const visibleGroups = groupThreadsByWorkspace(visibleThreads);
  detectionStatusEl.textContent = t("status.detected", {
    count: state.threads.length,
    groupCount: allGroups.length,
  });
  threadSearchStatusEl.textContent = t("status.showing", {
    count: visibleThreads.length,
    total: state.threads.length,
    groupCount: visibleGroups.length,
  });

  if (visibleThreads.length === 0) {
    threadListEl.innerHTML = `<p class="muted">${t("status.noFilter")}</p>`;
    renderSelection();
    return;
  }

  if (
    state.focusedThreadId &&
    !visibleThreads.some((thread) => thread.threadId === state.focusedThreadId)
  ) {
    state.focusedThreadId =
      visibleThreads.find((thread) => state.selectedThreadIds.has(thread.threadId))?.threadId ?? null;
  }

  const focusedGroupKey = state.focusedThreadId
    ? visibleThreads.find((thread) => thread.threadId === state.focusedThreadId)?.workspaceKey
    : visibleGroups.find((group) =>
        group.threads.some((thread) => state.selectedThreadIds.has(thread.threadId)),
      )?.workspaceKey;
  if (focusedGroupKey) {
    state.expandedWorkspaceKeys.add(focusedGroupKey);
  }

  for (const group of visibleGroups) {
    const section = document.createElement("details");
    section.className = "workspace-group";
    if (state.expandedWorkspaceKeys.has(group.workspaceKey)) {
      section.open = true;
    }
    section.addEventListener("toggle", () => {
      if (section.open) {
        state.expandedWorkspaceKeys.add(group.workspaceKey);
      } else {
        state.expandedWorkspaceKeys.delete(group.workspaceKey);
      }
    });

    const summary = document.createElement("summary");
    summary.className = "workspace-group-head";

    const summaryLeft = document.createElement("div");
    const title = document.createElement("div");
    title.className = "workspace-group-title";
    title.textContent = `📁 ${group.workspaceLabel}`;
    const path = document.createElement("div");
    path.className = "workspace-group-path";
    path.textContent = group.workspacePath;
    summaryLeft.append(title, path);

    const summaryRight = document.createElement("div");
    summaryRight.className = "workspace-group-actions";
    const count = document.createElement("span");
    count.className = "workspace-group-count";
    count.textContent = t("workspace.threads", { count: group.threads.length });

    const renameButton = document.createElement("button");
    renameButton.type = "button";
    renameButton.className = "mini-button";
    renameButton.textContent = t("renameWorkspace");
    renameButton.addEventListener("click", async (event) => {
      event.preventDefault();
      event.stopPropagation();
      await renameWorkspace(group);
    });

    const allSelected = group.threads.every((thread) => state.selectedThreadIds.has(thread.threadId));
    const selectAllButton = document.createElement("button");
    selectAllButton.type = "button";
    selectAllButton.className = "mini-button";
    selectAllButton.textContent = allSelected ? t("unselectAll") : t("selectAll");
    selectAllButton.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      setAllThreadsInGroup(group, !allSelected);
      renderThreads();
      renderSelection();
    });

    summaryRight.append(count, renameButton, selectAllButton);
    summary.append(summaryLeft, summaryRight);
    section.append(summary);

    const list = document.createElement("div");
    list.className = "workspace-group-list";

    for (const thread of group.threads) {
      const row = document.createElement("button");
      row.type = "button";
      row.className = `thread-row${state.selectedThreadIds.has(thread.threadId) ? " is-selected" : ""}${state.focusedThreadId === thread.threadId ? " is-focused" : ""}`;
      row.title = thread.displayName;

      const check = document.createElement("span");
      check.className = `thread-row-check${state.selectedThreadIds.has(thread.threadId) ? " is-checked" : ""}`;
      check.textContent = state.selectedThreadIds.has(thread.threadId) ? "✓" : "";

      const body = document.createElement("div");
      body.className = "thread-row-body";

      const threadTitle = document.createElement("div");
      threadTitle.className = "thread-row-title";
      threadTitle.textContent = thread.displayName || t("thread.untitled", { id: thread.threadId.slice(0, 8) });

      const meta = document.createElement("div");
      meta.className = "thread-row-meta";
      meta.textContent = `${thread.modelProvider ?? t("thread.modelUnknown")} · ${t("thread.updated", {
        time: formatRelativeTime(thread.updatedAt),
      })} · ${thread.connectorKind ?? "codex"} · ${thread.threadId}`;

      body.append(threadTitle, meta);
      row.append(check, body);
      row.addEventListener("click", () => {
        toggleThreadSelection(thread.threadId);
        renderThreads();
        renderSelection();
      });
      list.append(row);
    }

    section.append(list);
    threadListEl.append(section);
  }
}

function renderDiscoveryMeta(data) {
  state.discoveryMeta = data;
  workspaceRootValueEl.textContent = data.workspaceRoot ?? "(unknown)";
  codexHomeValueEl.textContent = data.codexHome ?? "(unknown)";
  discoveryModeValueEl.textContent = data.discoveryMode ?? "(unknown)";
}

function renderCommandPreview() {
  const selected = selectedThreads();
  if (selected.length === 0 || !state.discoveryMeta) {
    commandPreviewEl.textContent = t("cli.empty");
    return;
  }

  const aiSummaryEnabled = Boolean(aiSummaryToggleEl?.checked);
  const aiSummaryProfile = trimmedValue(aiSummaryProfileEl);
  const aiSummaryPreset = trimmedValue(aiSummaryPresetEl);
  const aiSummaryModel = trimmedValue(aiSummaryModelEl);
  const aiSummaryProvider = trimmedValue(aiSummaryProviderEl);
  const aiSummaryInstructions = trimmedValue(aiSummaryInstructionsEl);

  const lines = [
    "cargo run -- ui cockpit \\",
    `  --workspace-root ${previewArg(state.discoveryMeta.workspaceRoot)} \\`,
    `  --codex-home ${previewArg(state.discoveryMeta.codexHome)}`,
    "",
  ];

  const byWorkspace = groupThreadsByWorkspace(selected);
  for (const group of byWorkspace) {
    lines.push(`# ${group.workspaceLabel}`);
    for (const thread of group.threads.slice(0, 6)) {
      if (thread.connectorKind === "claude-code" && thread.sessionPath) {
        lines.push("cargo run -- export claude-code \\");
        lines.push(`  --session-path ${previewArg(thread.sessionPath)} \\`);
      } else {
        lines.push("cargo run -- export codex \\");
        lines.push(`  --thread-id ${previewArg(thread.threadId)} \\`);
      }
      lines.push("  --format markdown \\");
      lines.push("  --destination workspace-conversations \\");
      if (aiSummaryEnabled) {
        lines.push("  --ai-summary \\");
        if (aiSummaryProfile) {
          lines.push(`  --ai-summary-profile ${previewArg(aiSummaryProfile)} \\`);
        }
        if (aiSummaryPreset) {
          lines.push(`  --ai-summary-preset ${previewArg(aiSummaryPreset)} \\`);
        }
        if (aiSummaryModel) {
          lines.push(`  --ai-summary-model ${previewArg(aiSummaryModel)} \\`);
        }
        if (aiSummaryProvider) {
          lines.push(`  --ai-summary-provider ${previewArg(aiSummaryProvider)} \\`);
        }
        if (aiSummaryInstructions) {
          lines.push(`  --ai-summary-instructions ${previewArg(aiSummaryInstructions)} \\`);
        }
      }
      lines.push(`  --workspace-root ${previewArg(thread.workspacePath)}`);
      lines.push("");
    }
    if (group.threads.length > 6) {
      lines.push(`# +${group.threads.length - 6} more thread(s)`);
      lines.push("");
    }
  }

  commandPreviewEl.textContent = lines.join("\n");
}

function renderSelection() {
  const selected = selectedThreads();
  clearSelectionButtonEl.disabled = selected.length === 0;
  aiSummaryPanelEl.hidden = !aiSummaryToggleEl?.checked;
  aiSummaryInstructionsEl.disabled = !aiSummaryToggleEl?.checked;
  aiSummaryProfileEl.disabled = !aiSummaryToggleEl?.checked;
  if (aiSummaryPresetEl) aiSummaryPresetEl.disabled = !aiSummaryToggleEl?.checked;
  aiSummaryModelEl.disabled = !aiSummaryToggleEl?.checked;
  aiSummaryProviderEl.disabled = !aiSummaryToggleEl?.checked;

  if (selected.length === 0) {
    selectionSummaryEl.textContent = t("selection.none");
    detailEl.className = "detail-empty";
    detailEl.textContent = t("selection.empty");
    exportButtonEl.disabled = true;
    exportButtonEl.textContent = t("action.button.single");
    renderCommandPreview();
    return;
  }

  const workspaceCount = new Set(selected.map((thread) => thread.workspacePath)).size;
  selectionSummaryEl.textContent =
    selected.length === 1
      ? t("selection.single")
      : t("selection.multi", { count: selected.length, workspaceCount });

  exportButtonEl.disabled = false;
  exportButtonEl.textContent =
    selected.length === 1
      ? t("action.button.single")
      : t("action.button.multi", { count: selected.length });

  if (selected.length === 1) {
    const thread = selected[0];
    detailEl.className = "thread-detail";
    detailEl.innerHTML = `
      <dl class="detail-list">
        <div class="detail-row">
          <dt>${t("thread.title")}</dt>
          <dd>${thread.displayName}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.connector")}</dt>
          <dd>${thread.connectorKind ?? "codex"}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.id")}</dt>
          <dd>${thread.threadId}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.workspace")}</dt>
          <dd>${thread.workspaceLabel ?? "(unknown)"}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.workspacePath")}</dt>
          <dd>${thread.workspacePath ?? thread.cwd ?? "(none)"}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.model")}</dt>
          <dd>${thread.modelProvider ?? t("thread.modelUnknown")}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.updatedAt")}</dt>
          <dd>${formatRelativeTime(thread.updatedAt)}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.createdAt")}</dt>
          <dd>${formatRelativeTime(thread.createdAt)}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.cwd")}</dt>
          <dd>${thread.cwd ?? "(none)"}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.artifactPath")}</dt>
          <dd>${thread.sessionPath ?? thread.rolloutPath ?? "(none)"}</dd>
        </div>
        <div class="detail-row">
          <dt>${t("thread.discovery")}</dt>
          <dd>${thread.sourceKind}</dd>
        </div>
      </dl>
    `;
  } else {
    const list = selected
      .slice(0, 8)
      .map((thread) => `<li>${thread.displayName}</li>`)
      .join("");
    const more = selected.length > 8 ? `<p class="muted">+${selected.length - 8}</p>` : "";
    detailEl.className = "thread-detail";
    detailEl.innerHTML = `
      <div class="detail-batch">
        <p class="eyebrow">${t("selection.summaryTitle")}</p>
        <dl class="detail-list">
          <div class="detail-row">
            <dt>${t("selection.summaryCount")}</dt>
            <dd>${selected.length}</dd>
          </div>
          <div class="detail-row">
            <dt>${t("selection.summaryWorkspaces")}</dt>
            <dd>${workspaceCount}</dd>
          </div>
        </dl>
        <div class="detail-row">
          <dt>${t("selection.summaryThreads")}</dt>
          <dd><ul class="selection-list">${list}</ul>${more}</dd>
        </div>
      </div>
    `;
  }

  renderCommandPreview();
}

function renderResultLinks(response) {
  resultLinksEl.innerHTML = "";
  for (const workspace of response.workspaces ?? []) {
    const card = document.createElement("div");
    card.className = "result-card";

    const head = document.createElement("div");
    head.className = "result-card-head";

    const titleStack = document.createElement("div");

    const title = document.createElement("div");
    title.className = "result-card-title";
    title.textContent = `${t("result.workspace")}: ${workspace.workspaceLabel}`;
    const path = document.createElement("div");
    path.className = "result-card-path";
    path.textContent = workspace.workspacePath;
    titleStack.append(title, path);

    head.append(titleStack);

    if (workspace.copyBundleText) {
      const copyBundleButton = document.createElement("button");
      copyBundleButton.type = "button";
      copyBundleButton.className = "mini-button";
      copyBundleButton.textContent = t("result.copyBundle");
      copyBundleButton.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(workspace.copyBundleText);
        } catch (_error) {
          // ignore clipboard failures; the grouped result remains visible on screen
        }
      });
      head.append(copyBundleButton);
    }

    const links = document.createElement("div");
    links.className = "result-card-links";

    for (const thread of workspace.threads ?? []) {
      const threadCard = document.createElement("div");
      threadCard.className = "result-thread-card";

      const threadTitle = document.createElement("div");
      threadTitle.className = "result-path-label";
      threadTitle.textContent = `${t("result.thread")}: ${thread.displayName}`;
      threadCard.append(threadTitle);

      for (const transcriptPath of thread.transcriptPaths ?? []) {
        threadCard.append(buildResultPathRow(t("result.exportedFile"), transcriptPath));
      }

      for (const aiSummaryPath of thread.aiSummaryPaths ?? []) {
        threadCard.append(buildResultPathRow(t("result.openSummary"), aiSummaryPath));
      }

      links.append(threadCard);
    }

    for (const [labelKey, target] of [
      ["result.openArchive", workspace.archiveShellPath],
      ["result.openReports", workspace.reportsShellPath],
      ["result.openEvidence", workspace.integrationShellPath],
    ]) {
      if (!target) continue;
      links.append(buildResultPathRow(t(labelKey), target));
    }

    card.append(head, links);
    resultLinksEl.append(card);
  }
}

function buildResultPathRow(label, targetPath) {
  const row = document.createElement("div");
  row.className = "result-path-row";

  const meta = document.createElement("div");
  meta.className = "result-path-meta";

  const rowLabel = document.createElement("div");
  rowLabel.className = "result-path-label";
  rowLabel.textContent = label;

  const rowPath = document.createElement("div");
  rowPath.className = "result-path-value";
  rowPath.textContent = targetPath;

  meta.append(rowLabel, rowPath);

  const actions = document.createElement("div");
  actions.className = "result-path-actions";

  const copyButton = document.createElement("button");
  copyButton.type = "button";
  copyButton.className = "mini-button";
  copyButton.textContent = t("result.copyPath");
  copyButton.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(targetPath);
    } catch (_error) {
      // ignore clipboard failures; the path remains visible for manual copy
    }
  });

  const openLink = document.createElement("a");
  openLink.href = `file://${targetPath}`;
  openLink.target = "_blank";
  openLink.rel = "noreferrer";
  openLink.textContent = label;

  actions.append(copyButton, openLink);
  row.append(meta, actions);
  return row;
}

function renderExportJob(job) {
  const elapsed = formatDurationSince(job.startedAt);
  resultStatusEl.textContent =
    job.status === "failed"
      ? t("status.exportFailed", { error: job.errorMessage ?? "unknown" })
      : t("result.running", {
          phase: humanPhaseLabel(job.currentPhase),
          elapsed,
        });

  resultLinksEl.innerHTML = "";

  const progressCard = document.createElement("div");
  progressCard.className = "result-card";
  const progressTitle = document.createElement("div");
  progressTitle.className = "result-card-title";
  progressTitle.textContent = t("result.progress");
  const progressBody = document.createElement("div");
  progressBody.className = "result-card-path";
  progressBody.textContent = `${job.exportedCount ?? 0} / ${selectedThreads().length} exported`;
  progressCard.append(progressTitle, progressBody);

  const steps = document.createElement("div");
  steps.className = "result-card-links";
  for (const step of job.steps ?? []) {
    const chip = document.createElement("span");
    chip.className = "result-chip";
    const label =
      step.status === "completed"
        ? t("result.stepCompleted")
        : step.status === "warning"
          ? t("result.stepWarning")
          : step.status === "failed"
            ? t("result.stepFailed")
            : t("result.stepRunning");
    const suffix =
      step.status === "running" ? ` · ${formatDurationSince(step.startedAt)}` : "";
    chip.textContent = `${label}: ${step.label}${suffix}`;
    steps.append(chip);
  }
  progressCard.append(steps);
  resultLinksEl.append(progressCard);

  if (job.warnings?.length) {
    for (const warning of job.warnings) {
      const card = document.createElement("div");
      card.className = "result-card";
      const title = document.createElement("div");
      title.className = "result-card-title";
      title.textContent = t("result.warning");
      const body = document.createElement("div");
      body.className = "result-card-path";
      body.textContent = warning;
      card.append(title, body);
      resultLinksEl.append(card);
    }
  }
}

async function pollExportJob(jobId) {
  if (state.exportPollTimer) {
    clearTimeout(state.exportPollTimer);
    state.exportPollTimer = null;
  }
  state.activeExportJobId = jobId;

  const tick = async () => {
    const response = await fetch(`/api/export/jobs/${jobId}`);
    const data = await response.json();
    renderExportJob(data);
    if (data.status === "completed" || data.status === "completed_with_warnings") {
      resultStatusEl.textContent = t("result.done", {
        count: data.exportedCount ?? 0,
        workspaceCount: data.workspaceCount ?? 0,
      });
      renderResultLinks(data);
      state.exportPollTimer = null;
      exportButtonEl.disabled = selectedThreads().length === 0;
      return;
    }
    if (data.status === "failed") {
      state.exportPollTimer = null;
      exportButtonEl.disabled = selectedThreads().length === 0;
      return;
    }
    state.exportPollTimer = setTimeout(() => {
      void tick();
    }, 1000);
  };

  await tick();
}

async function loadThreads() {
  refreshButtonEl.disabled = true;
  detectionStatusEl.textContent = t("status.loadingThreads");
  try {
    const response = await fetch("/api/discovery");
    const data = await response.json();
    renderDiscoveryMeta(data);
    state.threads = Array.isArray(data.threads) ? data.threads : [];
    for (const thread of state.threads) {
      if (state.prefs.workspaceLabels[thread.workspacePath]) {
        thread.workspaceLabel = state.prefs.workspaceLabels[thread.workspacePath];
      }
    }
    renderThreads();
    renderSelection();
  } catch (error) {
    detectionStatusEl.textContent = t("status.loadFailed", { error: String(error) });
  } finally {
    refreshButtonEl.disabled = false;
  }
}

async function exportSelected() {
  const selected = selectedThreads();
  if (selected.length === 0) return;
  const aiSummaryEnabled = Boolean(aiSummaryToggleEl?.checked);
  const aiSummaryProfile = trimmedValue(aiSummaryProfileEl);
  const aiSummaryPreset = trimmedValue(aiSummaryPresetEl);
  const aiSummaryModel = trimmedValue(aiSummaryModelEl);
  const aiSummaryProvider = trimmedValue(aiSummaryProviderEl);
  const aiSummaryInstructions = trimmedValue(aiSummaryInstructionsEl);

  exportButtonEl.disabled = true;
  resultStatusEl.textContent = t("status.exportStarting");
  resultLinksEl.innerHTML = "";

  try {
    const response = await fetch("/api/export", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        selections: selected.map((thread) => ({
          threadId: thread.threadId,
          connectorKind: thread.connectorKind || "codex",
          workspacePath: thread.workspacePath,
          workspaceLabel: thread.workspaceLabel,
          sessionPath: thread.sessionPath || null,
        })),
        aiSummary: aiSummaryEnabled,
        aiSummaryInstructions: aiSummaryInstructions || null,
        aiSummaryProfile: aiSummaryProfile || null,
        aiSummaryPreset: aiSummaryPreset || null,
        aiSummaryModel: aiSummaryModel || null,
        aiSummaryProvider: aiSummaryProvider || null,
      }),
    });
    const data = await response.json();
    if (!response.ok) {
      throw new Error(data.message ?? "Export failed");
    }
    await pollExportJob(data.jobId);
  } catch (error) {
    resultStatusEl.textContent = t("status.exportFailed", { error: String(error) });
  } finally {
    if (!state.exportPollTimer) {
      exportButtonEl.disabled = selectedThreads().length === 0;
    }
  }
}

refreshButtonEl.addEventListener("click", () => {
  void loadThreads();
});

threadSearchEl?.addEventListener("input", () => {
  renderThreads();
  renderSelection();
});

aiSummaryToggleEl?.addEventListener("change", () => {
  renderSelection();
});

aiSummaryInstructionsEl?.addEventListener("input", () => {
  renderCommandPreview();
});

aiSummaryProfileEl?.addEventListener("input", () => {
  renderCommandPreview();
});

aiSummaryPresetEl?.addEventListener("input", () => {
  renderCommandPreview();
});

aiSummaryModelEl?.addEventListener("input", () => {
  renderCommandPreview();
});

aiSummaryProviderEl?.addEventListener("input", () => {
  renderCommandPreview();
});

clearSelectionButtonEl?.addEventListener("click", () => {
  state.selectedThreadIds.clear();
  state.focusedThreadId = null;
  renderThreads();
  renderSelection();
});

localeToggleEl?.addEventListener("click", async () => {
  state.prefs.locale = locale() === "zh" ? "en" : "zh";
  await persistPreferences();
  applyStaticText();
  renderThreads();
  renderSelection();
});

exportButtonEl.addEventListener("click", () => {
  void exportSelected();
});

void (async function bootstrap() {
  await loadPreferences();
  applyStaticText();
  await loadThreads();
})();
