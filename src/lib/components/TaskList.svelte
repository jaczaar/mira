<script lang="ts">
  import TaskCard from "./TaskCard.svelte";
  import { tasks, tasksLoading, tasksError } from "../stores/tasks";
  import type { SyncedTask } from "../stores/tasks";

  interface Props {
    onSyncTask?: (task: SyncedTask) => void;
  }

  let { onSyncTask }: Props = $props();

  let filter = $state<"all" | "active" | "synced" | "done">("active");
  let viewMode = $state<"flat" | "grouped">("grouped");
  let searchQuery = $state("");

  interface EpicGroup {
    epicKey: string | null;
    epicSummary: string | null;
    tasks: SyncedTask[];
  }

  const filteredTasks = $derived.by(() => {
    switch (filter) {
      case "active":
        return $tasks.filter((t) => t.status_category !== "done");
      case "synced":
        return $tasks.filter((t) => t.calendar_event_uid !== undefined);
      case "done":
        return $tasks.filter((t) => t.status_category === "done");
      default:
        return $tasks;
    }
  });

  function fuzzyScore(query: string, text: string): number | null {
    const q = query.toLowerCase();
    const t = text.toLowerCase();
    if (!q) return 0;

    let score = 0;
    let tIndex = 0;
    let lastMatch = -1;
    let firstMatch = -1;

    for (const ch of q) {
      const found = t.indexOf(ch, tIndex);
      if (found === -1) return null;
      if (firstMatch === -1) firstMatch = found;
      score += found === lastMatch + 1 ? 3 : 1;
      lastMatch = found;
      tIndex = found + 1;
    }

    if (t.includes(q)) score += 10;
    score += Math.max(0, 10 - firstMatch);
    return score;
  }

  function taskMatchScore(task: SyncedTask, query: string): number | null {
    const tokens = query
      .trim()
      .toLowerCase()
      .split(/\s+/)
      .filter(Boolean);

    if (tokens.length === 0) return 0;

    const fields = [
      task.key,
      task.summary,
      task.project_key,
      task.project_name,
      task.status,
      task.priority || "",
      task.issue_type || "",
      task.parent_key || "",
      task.parent_summary || "",
      task.labels.join(" "),
    ];

    let total = 0;
    for (const token of tokens) {
      let best: number | null = null;
      for (const field of fields) {
        if (!field) continue;
        const score = fuzzyScore(token, field);
        if (score !== null && (best === null || score > best)) {
          best = score;
        }
      }
      if (best === null) return null;
      total += best;
    }

    return total;
  }

  const visibleTasks = $derived.by(() => {
    const query = searchQuery.trim();
    if (!query) return filteredTasks;

    const scored = filteredTasks
      .map((task) => ({ task, score: taskMatchScore(task, query) }))
      .filter((entry) => entry.score !== null)
      .sort((a, b) => (b.score ?? 0) - (a.score ?? 0));

    return scored.map((entry) => entry.task);
  });

  const groupedByEpic = $derived.by(() => {
    const groups = new Map<string, EpicGroup>();

    for (const task of visibleTasks) {
      if (task.is_epic) {
        groups.set(task.key, {
          epicKey: task.key,
          epicSummary: task.summary,
          tasks: [task],
        });
      }
    }

    for (const task of visibleTasks) {
      if (task.is_epic) continue;

      const parentKey = task.parent_key;
      if (parentKey && groups.has(parentKey)) {
        groups.get(parentKey)!.tasks.push(task);
      } else if (parentKey) {
        if (!groups.has(parentKey)) {
          groups.set(parentKey, {
            epicKey: parentKey,
            epicSummary: task.parent_summary || parentKey,
            tasks: [],
          });
        }
        groups.get(parentKey)!.tasks.push(task);
      } else {
        const noEpicKey = "__no_epic__";
        if (!groups.has(noEpicKey)) {
          groups.set(noEpicKey, {
            epicKey: null,
            epicSummary: null,
            tasks: [],
          });
        }
        groups.get(noEpicKey)!.tasks.push(task);
      }
    }

    const result = Array.from(groups.values()).sort((a, b) => {
      if (a.epicKey === null) return 1;
      if (b.epicKey === null) return -1;
      return (a.epicKey || "").localeCompare(b.epicKey || "");
    });

    return result;
  });
</script>

<div class="task-list">
  <div class="controls-bar">
    <div class="filter-bar">
      <button class:active={filter === "all"} onclick={() => (filter = "all")}>
        All ({$tasks.length})
      </button>
      <button class:active={filter === "active"} onclick={() => (filter = "active")}>
        Active ({$tasks.filter((t) => t.status_category !== "done").length})
      </button>
      <button class:active={filter === "synced"} onclick={() => (filter = "synced")}>
        Synced ({$tasks.filter((t) => t.calendar_event_uid).length})
      </button>
      <button class:active={filter === "done"} onclick={() => (filter = "done")}>
        Done ({$tasks.filter((t) => t.status_category === "done").length})
      </button>
    </div>

    <div class="search-bar">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        type="search"
        placeholder="Search tasks..."
        bind:value={searchQuery}
        aria-label="Search tasks"
      />
      {#if searchQuery}
        <button type="button" class="clear-search" onclick={() => (searchQuery = "")}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      {/if}
    </div>

    <div class="view-toggle">
      <button class:active={viewMode === "grouped"} onclick={() => (viewMode = "grouped")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="8" y1="6" x2="21" y2="6" />
          <line x1="8" y1="12" x2="21" y2="12" />
          <line x1="8" y1="18" x2="21" y2="18" />
          <line x1="3" y1="6" x2="3.01" y2="6" />
          <line x1="3" y1="12" x2="3.01" y2="12" />
          <line x1="3" y1="18" x2="3.01" y2="18" />
        </svg>
      </button>
      <button class:active={viewMode === "flat"} onclick={() => (viewMode = "flat")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <rect x="3" y="3" width="7" height="7" />
          <rect x="14" y="3" width="7" height="7" />
          <rect x="14" y="14" width="7" height="7" />
          <rect x="3" y="14" width="7" height="7" />
        </svg>
      </button>
    </div>
  </div>

  {#if $tasksLoading}
    <div class="skeleton-list">
      {#each Array(5) as _, i}
        <div class="skeleton-card" style="animation-delay: {i * 60}ms">
          <div class="skeleton-line short"></div>
          <div class="skeleton-line"></div>
          <div class="skeleton-line medium"></div>
        </div>
      {/each}
    </div>
  {:else if $tasksError}
    <div class="state-panel error">
      <p>Failed to load tasks</p>
      <p class="detail">{$tasksError}</p>
    </div>
  {:else if visibleTasks.length === 0}
    <div class="state-panel">
      {#if searchQuery}
        <p>No tasks match "{searchQuery}"</p>
        <button class="reset-btn" onclick={() => (searchQuery = "")}>Clear search</button>
      {:else}
        <p>No tasks found</p>
        {#if filter !== "all"}
          <button class="reset-btn" onclick={() => (filter = "all")}>Show all tasks</button>
        {/if}
      {/if}
    </div>
  {:else if viewMode === "flat"}
    <div class="tasks-grid">
      {#each visibleTasks as task, i (task.id)}
        <div style="animation: fadeInUp 0.3s var(--ease-out) {i * 40}ms both">
          <TaskCard {task} onSync={onSyncTask} />
        </div>
      {/each}
    </div>
  {:else}
    <div class="epic-groups">
      {#each groupedByEpic as group, gi (group.epicKey || "__no_epic__")}
        <div class="epic-group" style="animation: fadeInUp 0.3s var(--ease-out) {gi * 60}ms both">
          <div class="epic-header" class:no-epic={!group.epicKey}>
            {#if group.epicKey}
              <span class="epic-key">{group.epicKey}</span>
              <span class="epic-summary">{group.epicSummary}</span>
            {:else}
              <span class="epic-summary">Ungrouped</span>
            {/if}
            <span class="task-count">{group.tasks.filter(t => !t.is_epic).length}</span>
          </div>
          <div class="epic-tasks">
            {#each group.tasks.filter(t => !t.is_epic) as task (task.id)}
              <TaskCard {task} onSync={onSyncTask} compact={true} />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .task-list {
    width: 100%;
  }

  .controls-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border-subtle);
    flex-wrap: wrap;
    gap: 10px;
  }

  .filter-bar {
    display: flex;
    gap: 2px;
  }

  .filter-bar button {
    padding: 6px 12px;
    border: none;
    background: transparent;
    border-radius: 7px;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .filter-bar button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .filter-bar button.active {
    color: var(--text-primary);
  }

  .view-toggle {
    display: flex;
    gap: 2px;
  }

  .view-toggle button {
    padding: 6px 10px;
    border: none;
    background: transparent;
    border-radius: 7px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    display: flex;
    align-items: center;
  }

  .view-toggle button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .view-toggle button.active {
    color: var(--text-primary);
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 180px;
    max-width: 320px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0 12px;
    transition: border-color 0.2s;
  }

  .search-bar:focus-within {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .search-bar svg {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-bar input {
    flex: 1;
    padding: 7px 0;
    border: none;
    background: transparent;
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-primary);
    outline: none;
  }

  .search-bar input::placeholder {
    color: var(--text-tertiary);
  }

  .clear-search {
    padding: 2px;
    background: var(--bg-hover);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    color: var(--text-tertiary);
  }

  .clear-search:hover {
    color: var(--text-primary);
  }

  .tasks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 12px;
  }

  .epic-groups {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .epic-group {
    overflow: hidden;
  }

  .epic-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 4px;
  }

  .epic-header.no-epic {
    opacity: 0.7;
  }

  .epic-key {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 12px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    padding: 2px 8px;
    border-radius: 4px;
    letter-spacing: 0.01em;
  }

  .epic-summary {
    flex: 1;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .task-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    background: var(--bg-hover);
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }

  .epic-tasks {
    display: flex;
    flex-direction: column;
  }

  .state-panel {
    text-align: center;
    padding: 48px 24px;
    color: var(--text-secondary);
  }

  .state-panel.error {
    color: var(--accent-red);
  }

  .state-panel .detail {
    font-size: 13px;
    color: var(--text-tertiary);
    margin-top: 4px;
  }

  .state-panel .spinner {
    width: 28px;
    height: 28px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    margin: 0 auto 14px;
    animation: spin 0.8s linear infinite;
  }

  .reset-btn {
    margin-top: 12px;
    padding: 7px 16px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid var(--accent-blue-dim);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 13px;
  }

  .reset-btn:hover {
    background: var(--accent-blue-dim);
  }

  .skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px 0;
  }

  .skeleton-card {
    padding: 14px 16px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    animation: fadeIn 0.4s var(--ease-out) both;
  }

  .skeleton-line {
    height: 12px;
    background: var(--bg-hover);
    border-radius: 4px;
    margin-bottom: 10px;
    animation: pulse 1.4s ease-in-out infinite;
  }

  .skeleton-line:last-child {
    margin-bottom: 0;
  }

  .skeleton-line.short {
    width: 30%;
  }

  .skeleton-line.medium {
    width: 60%;
  }
</style>
