<script lang="ts">
  import type { SyncedTask } from "../stores/tasks";

  interface Props {
    task: SyncedTask;
    onSync?: (task: SyncedTask) => void;
    onLogTime?: (task: SyncedTask) => void;
    compact?: boolean;
  }

  let { task, onSync, onLogTime, compact = false }: Props = $props();

  function formatDuration(seconds: number | null): string {
    if (!seconds) return "-";
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  }

  function getStatusColor(category: string): string {
    switch (category) {
      case "done":
        return "var(--accent-green)";
      case "indeterminate":
        return "var(--accent-blue)";
      default:
        return "var(--text-tertiary)";
    }
  }

  function getPriorityColor(priority: string | null): string {
    switch (priority?.toLowerCase()) {
      case "highest":
      case "critical":
        return "var(--accent-red)";
      case "high":
        return "var(--accent-amber)";
      case "medium":
        return "#eab308";
      case "low":
        return "var(--accent-green)";
      case "lowest":
        return "var(--text-tertiary)";
      default:
        return "var(--text-tertiary)";
    }
  }
</script>

<div class="task-card" class:synced={task.calendar_event_uid} class:compact>
  <div class="task-header">
    <span class="task-key">{task.key}</span>
    <span
      class="task-status"
      style="--status-color: {getStatusColor(task.status_category)}"
    >
      {task.status}
    </span>
  </div>

  <h3 class="task-summary">{task.summary}</h3>

  {#if !compact}
    <div class="task-meta">
      <span class="project">{task.project_name}</span>
      {#if task.priority}
        <span
          class="priority"
          style="--priority-color: {getPriorityColor(task.priority)}"
        >
          {task.priority}
        </span>
      {/if}
      {#if task.issue_type}
        <span class="type">{task.issue_type}</span>
      {/if}
    </div>

    {#if task.time_estimate_seconds || task.time_spent_seconds}
      <div class="task-time">
        {#if task.time_estimate_seconds}
          <span class="estimate">
            Est: {formatDuration(task.time_estimate_seconds)}
          </span>
        {/if}
        {#if task.time_spent_seconds}
          <span class="spent">
            Logged: {formatDuration(task.time_spent_seconds)}
          </span>
        {/if}
      </div>
    {/if}

    {#if task.due_date}
      <div class="task-due">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
          <line x1="16" y1="2" x2="16" y2="6" />
          <line x1="8" y1="2" x2="8" y2="6" />
        </svg>
        Due: {task.due_date}
      </div>
    {/if}
  {:else}
    <div class="compact-meta">
      {#if task.priority}
        <span
          class="priority"
          style="--priority-color: {getPriorityColor(task.priority)}"
        >
          {task.priority}
        </span>
      {/if}
      {#if task.issue_type}
        <span class="type">{task.issue_type}</span>
      {/if}
      {#if task.time_estimate_seconds}
        <span class="estimate">Est: {formatDuration(task.time_estimate_seconds)}</span>
      {/if}
      {#if task.calendar_event_uid}
        <span class="synced-badge">
          <span class="sync-dot"></span>
          Synced
        </span>
      {/if}
    </div>
  {/if}

  <div class="task-actions">
    {#if onSync}
      <button class="act-btn primary" onclick={() => onSync(task)}>
        {task.calendar_event_uid ? "Re-sync" : "Schedule"}
      </button>
    {/if}
    {#if onLogTime && !compact}
      <button class="act-btn success" onclick={() => onLogTime(task)}>
        Log Time
      </button>
    {/if}
    <a href={task.url} target="_blank" rel="noopener" class="act-btn ghost">
      View
    </a>
  </div>

  {#if !compact && task.calendar_event_uid}
    <div class="sync-indicator">
      <span class="sync-dot"></span>
      Synced
      {#if task.last_synced}
        <span class="sync-time">
          {new Date(task.last_synced).toLocaleString()}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .task-card {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    padding: 16px;
    border: 1px solid var(--border-subtle);
    transition: all 0.25s var(--ease-out);
    position: relative;
  }

  .task-card.compact {
    border-radius: 0;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    padding: 12px 20px;
    display: grid;
    grid-template-columns: auto 1fr auto;
    grid-template-rows: auto auto;
    gap: 4px 16px;
    align-items: center;
  }

  .task-card.compact .task-header {
    grid-column: 1;
    grid-row: 1 / 3;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    margin-bottom: 0;
  }

  .task-card.compact .task-summary {
    grid-column: 2;
    grid-row: 1;
    margin: 0;
    font-size: 13px;
  }

  .task-card.compact .compact-meta {
    grid-column: 2;
    grid-row: 2;
  }

  .task-card.compact .task-actions {
    grid-column: 3;
    grid-row: 1 / 3;
    margin-top: 0;
    padding-top: 0;
    border-top: none;
  }

  .task-card:not(.compact):hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-glow-blue);
    transform: translateY(-1px);
  }

  .task-card.compact:hover {
    background: var(--bg-elevated);
  }

  .task-card.synced:not(.compact) {
    border-left: 2px solid var(--accent-green);
  }

  .task-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .task-key {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-blue);
    letter-spacing: 0.01em;
  }

  .task-status {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: color-mix(in srgb, var(--status-color) 12%, transparent);
    color: var(--status-color);
    font-weight: 500;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .task-summary {
    margin: 0 0 10px;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }

  .task-meta,
  .compact-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 8px;
  }

  .compact-meta {
    margin-bottom: 0;
  }

  .task-meta span,
  .compact-meta span {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .priority {
    background: color-mix(in srgb, var(--priority-color) 10%, transparent) !important;
    color: var(--priority-color) !important;
    border-color: color-mix(in srgb, var(--priority-color) 20%, transparent) !important;
  }

  .synced-badge {
    display: inline-flex !important;
    align-items: center;
    gap: 4px;
    background: var(--accent-green-dim) !important;
    color: var(--accent-green) !important;
    border-color: rgba(74, 222, 128, 0.2) !important;
  }

  .task-time {
    display: flex;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    margin-bottom: 8px;
  }

  .task-due {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--accent-amber);
    margin-bottom: 8px;
  }

  .task-actions {
    display: flex;
    gap: 6px;
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border-subtle);
  }

  .act-btn {
    padding: 5px 11px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.15s var(--ease-out);
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .act-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .act-btn.primary {
    background: var(--accent-blue-dim);
    border-color: rgba(91, 141, 239, 0.2);
    color: var(--accent-blue);
  }

  .act-btn.primary:hover {
    background: rgba(91, 141, 239, 0.2);
    box-shadow: 0 0 12px var(--accent-blue-dim);
  }

  .act-btn.success {
    background: var(--accent-green-dim);
    border-color: rgba(74, 222, 128, 0.15);
    color: var(--accent-green);
  }

  .act-btn.success:hover {
    background: rgba(74, 222, 128, 0.18);
  }

  .act-btn.ghost {
    background: transparent;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--accent-green);
  }

  .sync-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-green);
    flex-shrink: 0;
  }

  .sync-time {
    color: var(--text-tertiary);
  }
</style>
