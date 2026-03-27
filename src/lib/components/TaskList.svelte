<script lang="ts">
  import TaskCard from "./TaskCard.svelte";
  import { tasks, tasksLoading, tasksError } from "../stores/tasks";
  import type { SyncedTask } from "../stores/tasks";

  interface Props {
    onSyncTask?: (task: SyncedTask) => void;
    onLogTime?: (task: SyncedTask) => void;
  }

  let { onSyncTask, onLogTime }: Props = $props();

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

    // First pass: add all epics
    for (const task of visibleTasks) {
      if (task.is_epic) {
        groups.set(task.key, {
          epicKey: task.key,
          epicSummary: task.summary,
          tasks: [task],
        });
      }
    }

    // Second pass: add non-epic tasks to their parent groups
    for (const task of visibleTasks) {
      if (task.is_epic) continue;

      const parentKey = task.parent_key;
      if (parentKey && groups.has(parentKey)) {
        groups.get(parentKey)!.tasks.push(task);
      } else if (parentKey) {
        // Parent epic exists but not in filtered results - create a group for it
        if (!groups.has(parentKey)) {
          groups.set(parentKey, {
            epicKey: parentKey,
            epicSummary: task.parent_summary || parentKey,
            tasks: [],
          });
        }
        groups.get(parentKey)!.tasks.push(task);
      } else {
        // No parent - add to "No Epic" group
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

    // Convert to array and sort: epics with children first, then no epic
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
      <input
        type="search"
        placeholder="Search tasks..."
        bind:value={searchQuery}
        aria-label="Search tasks"
      />
      {#if searchQuery}
        <button type="button" class="clear-search" onclick={() => (searchQuery = "")}>
          Clear
        </button>
      {/if}
    </div>

    <div class="view-toggle">
      <button class:active={viewMode === "grouped"} onclick={() => (viewMode = "grouped")}>
        Grouped
      </button>
      <button class:active={viewMode === "flat"} onclick={() => (viewMode = "flat")}>
        Flat
      </button>
    </div>
  </div>

  {#if $tasksLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading tasks...</p>
    </div>
  {:else if $tasksError}
    <div class="error">
      <p>Failed to load tasks</p>
      <p class="error-detail">{$tasksError}</p>
    </div>
  {:else if visibleTasks.length === 0}
    <div class="empty">
      {#if searchQuery}
        <p>No tasks match "{searchQuery}"</p>
        <button onclick={() => (searchQuery = "")}>Clear search</button>
      {:else}
        <p>No tasks found</p>
        {#if filter !== "all"}
          <button onclick={() => (filter = "all")}>Show all tasks</button>
        {/if}
      {/if}
    </div>
  {:else if viewMode === "flat"}
    <div class="tasks-grid">
      {#each visibleTasks as task (task.id)}
        <TaskCard {task} onSync={onSyncTask} {onLogTime} />
      {/each}
    </div>
  {:else}
    <div class="epic-groups">
      {#each groupedByEpic as group (group.epicKey || "__no_epic__")}
        <div class="epic-group">
          <div class="epic-header" class:no-epic={!group.epicKey}>
            {#if group.epicKey}
              <span class="epic-key">{group.epicKey}</span>
              <span class="epic-summary">{group.epicSummary}</span>
            {:else}
              <span class="epic-summary">No Epic</span>
            {/if}
            <span class="task-count">{group.tasks.filter(t => !t.is_epic).length} tasks</span>
          </div>
          <div class="epic-tasks">
            {#each group.tasks.filter(t => !t.is_epic) as task (task.id)}
              <TaskCard {task} onSync={onSyncTask} {onLogTime} compact={true} />
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
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e5e5e5;
    flex-wrap: wrap;
    gap: 12px;
  }

  .filter-bar {
    display: flex;
    gap: 8px;
  }

  .filter-bar button,
  .view-toggle button {
    padding: 8px 16px;
    border: 1px solid #d2d2d7;
    background: white;
    border-radius: 20px;
    font-size: 13px;
    color: #1d1d1f;
    cursor: pointer;
    transition: all 0.2s;
  }

  .filter-bar button:hover,
  .view-toggle button:hover {
    background: #f5f5f7;
  }

  .filter-bar button.active,
  .view-toggle button.active {
    background: #1d1d1f;
    color: white;
    border-color: #1d1d1f;
  }

  .view-toggle {
    display: flex;
    gap: 4px;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 200px;
    max-width: 360px;
  }

  .search-bar input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #d2d2d7;
    border-radius: 20px;
    font-size: 13px;
  }

  .search-bar input:focus {
    outline: none;
    border-color: #0071e3;
  }

  .clear-search {
    padding: 6px 10px;
    border: 1px solid #d2d2d7;
    background: white;
    border-radius: 12px;
    font-size: 12px;
    color: #1d1d1f;
    cursor: pointer;
  }

  .clear-search:hover {
    background: #f5f5f7;
  }

  .tasks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }

  .epic-groups {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .epic-group {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .epic-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
  }

  .epic-header.no-epic {
    background: #86868b;
  }

  .epic-key {
    font-weight: 600;
    font-size: 14px;
    background: rgba(255, 255, 255, 0.2);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .epic-summary {
    flex: 1;
    font-size: 15px;
    font-weight: 500;
  }

  .task-count {
    font-size: 12px;
    opacity: 0.8;
  }

  .epic-tasks {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: #f0f0f0;
  }

  .loading,
  .error,
  .empty {
    text-align: center;
    padding: 40px;
    background: white;
    border-radius: 12px;
  }

  .loading .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #e5e5e5;
    border-top-color: #0071e3;
    border-radius: 50%;
    margin: 0 auto 16px;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    background: #ffebea;
    color: #ff3b30;
  }

  .error-detail {
    font-size: 13px;
    opacity: 0.8;
  }

  .empty {
    color: #86868b;
  }

  .empty button {
    margin-top: 12px;
    padding: 8px 16px;
    background: #0071e3;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }
</style>
