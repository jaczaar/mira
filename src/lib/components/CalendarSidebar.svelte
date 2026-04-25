<script lang="ts">
  import {
    sidebarTasks,
    addManualTask,
    importJiraTasks,
    moveTask,
    removeTask,
    type SidebarTask,
    type SidebarLane,
  } from "../stores/sidebarTasks";
  import { loadAssignedTasks, tasks } from "../stores/tasks";
  import { config, hasToken } from "../stores/config";

  interface Props {
    onScheduleActive: (task: SidebarTask) => void;
  }

  let { onScheduleActive }: Props = $props();

  let newTitle = $state("");
  let importing = $state(false);
  let importMessage = $state<string | null>(null);
  let importError = $state<string | null>(null);
  let dragId = $state<string | null>(null);
  let dragOverLane = $state<SidebarLane | null>(null);

  const backlog = $derived($sidebarTasks.filter((t) => t.lane === "backlog"));
  const active = $derived($sidebarTasks.filter((t) => t.lane === "active"));

  function handleAdd() {
    const title = newTitle.trim();
    if (!title) return;
    addManualTask(title);
    newTitle = "";
  }

  function handleAddKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  }

  async function handleImport() {
    if (!$hasToken || importing) return;
    importing = true;
    importMessage = null;
    importError = null;
    try {
      await loadAssignedTasks($config.jql_filter ?? undefined);
      const added = importJiraTasks(
        $tasks
          .filter((t) => t.status_category !== "done")
          .map((t) => ({ key: t.key, summary: t.summary, url: t.url }))
      );
      importMessage =
        added === 0 ? "Already up to date" : `Added ${added} task${added === 1 ? "" : "s"}`;
    } catch (err) {
      importError = err instanceof Error ? err.message : String(err);
    } finally {
      importing = false;
    }
  }

  function onDragStart(e: DragEvent, id: string) {
    dragId = id;
    if (e.dataTransfer) {
      e.dataTransfer.setData("text/plain", id);
      e.dataTransfer.effectAllowed = "move";
    }
  }

  function onDragEnd() {
    dragId = null;
    dragOverLane = null;
  }

  function onDragOver(e: DragEvent, lane: SidebarLane) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverLane = lane;
  }

  function onDragLeave(lane: SidebarLane) {
    if (dragOverLane === lane) dragOverLane = null;
  }

  function onDrop(e: DragEvent, lane: SidebarLane) {
    e.preventDefault();
    const id = dragId ?? e.dataTransfer?.getData("text/plain");
    if (id) moveTask(id, lane);
    dragId = null;
    dragOverLane = null;
  }
</script>

<aside class="cal-sidebar">
  <div class="sidebar-header">
    <h3>Tasks</h3>
  </div>

  <section
    class="lane"
    class:drop-target={dragOverLane === "backlog"}
    ondragover={(e) => onDragOver(e, "backlog")}
    ondragleave={() => onDragLeave("backlog")}
    ondrop={(e) => onDrop(e, "backlog")}
    role="list"
  >
    <header class="lane-header">
      <span class="lane-title">Backlog</span>
      <span class="lane-count">{backlog.length}</span>
    </header>

    <div class="add-row">
      <input
        type="text"
        class="add-input"
        placeholder="Add a task…"
        bind:value={newTitle}
        onkeydown={handleAddKey}
      />
      <button
        class="add-btn"
        onclick={handleAdd}
        disabled={!newTitle.trim()}
        title="Add task"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>

    <div class="import-row">
      <button
        class="import-btn"
        onclick={handleImport}
        disabled={!$hasToken || importing}
        title={$hasToken ? "Import assigned Jira issues" : "Connect Jira to import"}
      >
        {#if importing}
          <span class="spinner"></span>
          Importing…
        {:else}
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          Import from Jira
        {/if}
      </button>
      {#if importMessage}
        <span class="import-status ok">{importMessage}</span>
      {:else if importError}
        <span class="import-status err">{importError}</span>
      {/if}
    </div>

    <ul class="task-list">
      {#each backlog as task (task.id)}
        <li
          class="task-item"
          class:dragging={dragId === task.id}
          draggable="true"
          ondragstart={(e) => onDragStart(e, task.id)}
          ondragend={onDragEnd}
        >
          <div class="task-main">
            {#if task.jiraKey}
              <a
                class="jira-key"
                href={task.jiraUrl ?? "#"}
                target="_blank"
                rel="noopener"
                onmousedown={(e) => e.stopPropagation()}
              >
                {task.jiraKey}
              </a>
            {/if}
            <span class="task-title" title={task.title}>{task.title}</span>
          </div>
          <button
            class="remove-btn"
            onclick={() => removeTask(task.id)}
            title="Remove"
            aria-label="Remove task"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </li>
      {/each}
      {#if backlog.length === 0}
        <li class="empty-hint">Add tasks above or import from Jira.</li>
      {/if}
    </ul>
  </section>

  <section
    class="lane"
    class:drop-target={dragOverLane === "active"}
    ondragover={(e) => onDragOver(e, "active")}
    ondragleave={() => onDragLeave("active")}
    ondrop={(e) => onDrop(e, "active")}
    role="list"
  >
    <header class="lane-header">
      <span class="lane-title">Active</span>
      <span class="lane-count">{active.length}</span>
    </header>

    <ul class="task-list">
      {#each active as task (task.id)}
        <li
          class="task-item active"
          class:dragging={dragId === task.id}
          draggable="true"
          ondragstart={(e) => onDragStart(e, task.id)}
          ondragend={onDragEnd}
        >
          <div class="task-main">
            {#if task.jiraKey}
              <a
                class="jira-key"
                href={task.jiraUrl ?? "#"}
                target="_blank"
                rel="noopener"
                onmousedown={(e) => e.stopPropagation()}
              >
                {task.jiraKey}
              </a>
            {/if}
            <span class="task-title" title={task.title}>{task.title}</span>
          </div>
          <div class="task-actions">
            <button
              class="schedule-btn"
              onclick={() => onScheduleActive(task)}
              title="Schedule on calendar"
            >
              Schedule
            </button>
            <button
              class="remove-btn"
              onclick={() => removeTask(task.id)}
              title="Remove"
              aria-label="Remove task"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </li>
      {/each}
      {#if active.length === 0}
        <li class="empty-hint">Drag tasks here when you're ready to work on them.</li>
      {/if}
    </ul>
  </section>
</aside>

<style>
  .cal-sidebar {
    flex: 0 0 260px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
    overflow-y: auto;
    padding-right: 4px;
  }

  .sidebar-header h3 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .lane {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition: border-color 0.15s var(--ease-out), background 0.15s var(--ease-out);
  }

  .lane.drop-target {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
  }

  .lane-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 2px;
  }

  .lane-title {
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .lane-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    background: var(--bg-hover);
    padding: 1px 7px;
    border-radius: var(--radius-full);
  }

  .add-row {
    display: flex;
    gap: 6px;
  }

  .add-input {
    flex: 1;
    min-width: 0;
    padding: 6px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s var(--ease-out);
  }

  .add-input::placeholder {
    color: var(--text-tertiary);
  }

  .add-input:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid rgba(124, 172, 248, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    flex-shrink: 0;
  }

  .add-btn:hover:not(:disabled) {
    background: rgba(124, 172, 248, 0.2);
  }

  .add-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .import-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .import-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    background: transparent;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .import-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-style: solid;
  }

  .import-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .import-status {
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .import-status.ok {
    color: var(--accent-green);
  }

  .import-status.err {
    color: var(--accent-red);
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  .task-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 28px;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: grab;
    transition: border-color 0.12s var(--ease-out), background 0.12s var(--ease-out), opacity 0.12s var(--ease-out);
  }

  .task-item:hover {
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .task-item.dragging {
    opacity: 0.4;
    cursor: grabbing;
  }

  .task-item.active {
    border-left: 2px solid var(--accent-blue);
  }

  .task-main {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .jira-key {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--accent-blue);
    background: var(--accent-blue-dim);
    padding: 1px 6px;
    border-radius: 3px;
    text-decoration: none;
    flex-shrink: 0;
    letter-spacing: 0.01em;
  }

  .jira-key:hover {
    background: rgba(124, 172, 248, 0.2);
  }

  .task-title {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .task-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .schedule-btn {
    padding: 3px 8px;
    background: var(--accent-blue-dim);
    border: 1px solid rgba(124, 172, 248, 0.2);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 10px;
    font-weight: 500;
    color: var(--accent-blue);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .schedule-btn:hover {
    background: rgba(124, 172, 248, 0.2);
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
    flex-shrink: 0;
  }

  .remove-btn:hover {
    background: var(--bg-hover);
    color: var(--accent-red);
  }

  .empty-hint {
    padding: 10px 4px;
    font-size: 11px;
    color: var(--text-tertiary);
    text-align: center;
    font-style: italic;
  }
</style>
