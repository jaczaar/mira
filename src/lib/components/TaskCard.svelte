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
        return "#34c759";
      case "indeterminate":
        return "#0071e3";
      default:
        return "#86868b";
    }
  }

  function getPriorityColor(priority: string | null): string {
    switch (priority?.toLowerCase()) {
      case "highest":
      case "critical":
        return "#ff3b30";
      case "high":
        return "#ff9500";
      case "medium":
        return "#ffcc00";
      case "low":
        return "#34c759";
      case "lowest":
        return "#86868b";
      default:
        return "#86868b";
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
      <div class="task-due">Due: {task.due_date}</div>
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
        <span class="synced-badge">Synced</span>
      {/if}
    </div>
  {/if}

  <div class="task-actions">
    {#if onSync}
      <button class="action-btn sync" onclick={() => onSync(task)}>
        {task.calendar_event_uid ? "Re-sync" : "Sync"}
      </button>
    {/if}
    {#if onLogTime && !compact}
      <button class="action-btn log" onclick={() => onLogTime(task)}>
        Log Time
      </button>
    {/if}
    <a href={task.url} target="_blank" rel="noopener" class="action-btn view">
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
    background: white;
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: box-shadow 0.2s;
  }

  .task-card.compact {
    border-radius: 0;
    box-shadow: none;
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
    font-size: 14px;
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

  .task-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .task-card.compact:hover {
    background: #f9f9f9;
    box-shadow: none;
  }

  .task-card.synced {
    border-left: 3px solid #34c759;
  }

  .task-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .task-key {
    font-size: 13px;
    font-weight: 600;
    color: #0071e3;
  }

  .task-status {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--status-color) 15%, white);
    color: var(--status-color);
    font-weight: 500;
  }

  .task-summary {
    margin: 0 0 12px;
    font-size: 15px;
    font-weight: 500;
    color: #1d1d1f;
    line-height: 1.4;
  }

  .task-meta,
  .compact-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 8px;
  }

  .compact-meta {
    margin-bottom: 0;
  }

  .task-meta span,
  .compact-meta span {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 4px;
    background: #f5f5f7;
    color: #86868b;
  }

  .priority {
    background: color-mix(in srgb, var(--priority-color) 15%, white) !important;
    color: var(--priority-color) !important;
  }

  .synced-badge {
    background: #e8f8ec !important;
    color: #34c759 !important;
  }

  .task-time {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: #86868b;
    margin-bottom: 8px;
  }

  .task-due {
    font-size: 12px;
    color: #ff9500;
    margin-bottom: 8px;
  }

  .task-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid #f0f0f0;
  }

  .action-btn {
    padding: 6px 12px;
    border: none;
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
    text-decoration: none;
    transition: background-color 0.2s;
  }

  .action-btn.sync {
    background: #0071e3;
    color: white;
  }

  .action-btn.sync:hover {
    background: #0077ed;
  }

  .action-btn.log {
    background: #34c759;
    color: white;
  }

  .action-btn.log:hover {
    background: #30d158;
  }

  .action-btn.view {
    background: #f5f5f7;
    color: #1d1d1f;
  }

  .action-btn.view:hover {
    background: #e8e8ed;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    font-size: 11px;
    color: #34c759;
  }

  .sync-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #34c759;
  }

  .sync-time {
    color: #86868b;
  }
</style>
