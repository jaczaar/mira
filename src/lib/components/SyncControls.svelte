<script lang="ts">
  import { onMount } from "svelte";
  import { syncState, syncTasksToCalendar } from "../stores/sync";
  import { loadAssignedTasks } from "../stores/tasks";
  import { config, hasToken } from "../stores/config";
  import { googleAccount, loadGoogleAuthStatus } from "../stores/google";

  onMount(async () => {
    await loadGoogleAuthStatus();
  });

  async function handleRefresh() {
    const jqlFilter = $config.jql_filter;
    await loadAssignedTasks(jqlFilter || undefined);
  }

  async function handleSyncToCalendar() {
    try {
      await syncTasksToCalendar();
    } catch (error) {
      console.error("Sync failed:", error);
    }
  }

  const canSync = $derived($hasToken && $config.selected_calendar && $googleAccount);
</script>

<div class="sync-controls">
  <div class="main-actions">
    <button
      class="action-btn refresh"
      onclick={handleRefresh}
      disabled={$syncState.status === "syncing" || !$hasToken}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="23 4 23 10 17 10" />
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
      </svg>
      Refresh
    </button>

    <button
      class="action-btn sync"
      onclick={handleSyncToCalendar}
      disabled={$syncState.status === "syncing" || !canSync}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
        <line x1="16" y1="2" x2="16" y2="6" />
        <line x1="8" y1="2" x2="8" y2="6" />
        <line x1="3" y1="10" x2="21" y2="10" />
      </svg>
      {$syncState.status === "syncing" ? "Syncing..." : "Sync to Calendar"}
    </button>

  </div>

  {#if $syncState.status === "syncing"}
    <div class="progress">
      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {($syncState.progress / $syncState.total) * 100}%"
        ></div>
      </div>
      <span class="progress-text">
        {$syncState.progress}/{$syncState.total}
      </span>
    </div>
  {/if}

  {#if $syncState.lastSync}
    <p class="last-sync">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
      Last sync: {new Date($syncState.lastSync).toLocaleString()}
    </p>
  {/if}

  {#if !canSync}
    <p class="warning">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      {#if !$hasToken}
        Configure Jira credentials in Settings to enable sync.
      {:else if !$googleAccount}
        Connect Google Calendar in Settings to enable sync.
      {:else if !$config.selected_calendar}
        Select a calendar in Settings to enable sync.
      {/if}
    </p>
  {/if}
</div>

<style>
  .sync-controls {
    padding: 0 0 16px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .main-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s var(--ease-out);
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .action-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .action-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .action-btn.sync {
    background: var(--accent-blue-dim);
    border-color: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .action-btn.sync:hover:not(:disabled) {
    background: var(--accent-blue-dim);
    border-color: var(--accent-blue-glow);
    box-shadow: var(--shadow-glow-blue);
  }

  .action-btn.ghost {
    background: transparent;
    border-color: var(--border-default);
    color: var(--text-secondary);
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 14px;
  }

  .progress-bar {
    flex: 1;
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--gradient-brand);
    transition: width 0.4s var(--ease-out);
    border-radius: 2px;
  }

  .progress-text {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    min-width: 40px;
    text-align: right;
  }

  .last-sync {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 10px 0 0;
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .warning {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 10px 0 0;
    font-size: 12px;
    color: var(--accent-amber);
    padding: 8px 12px;
    background: var(--accent-amber-dim);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(251, 191, 36, 0.15);
  }
</style>
