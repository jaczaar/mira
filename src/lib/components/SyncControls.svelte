<script lang="ts">
  import { onMount } from "svelte";
  import { syncState, syncTasksToCalendar, syncCalendarToWorklogs } from "../stores/sync";
  import { loadAssignedTasks } from "../stores/tasks";
  import { config, hasToken } from "../stores/config";
  import { googleAccount, loadGoogleAuthStatus } from "../stores/google";

  let worklogStartDate = $state(
    new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split("T")[0]
  );
  let worklogEndDate = $state(new Date().toISOString().split("T")[0]);
  let showWorklogPanel = $state(false);

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

  async function handleSyncWorklogs() {
    try {
      await syncCalendarToWorklogs(worklogStartDate, worklogEndDate);
    } catch (error) {
      console.error("Worklog sync failed:", error);
    }
  }

  const canSync = $derived($hasToken && $config.selected_calendar && $googleAccount);
</script>

<div class="sync-controls">
  <div class="main-actions">
    <button
      class="refresh-btn"
      onclick={handleRefresh}
      disabled={$syncState.status === "syncing" || !$hasToken}
    >
      Refresh Tasks
    </button>

    <button
      class="sync-btn"
      onclick={handleSyncToCalendar}
      disabled={$syncState.status === "syncing" || !canSync}
    >
      {$syncState.status === "syncing" ? "Syncing..." : "Sync to Calendar"}
    </button>

    <button
      class="worklog-btn"
      onclick={() => (showWorklogPanel = !showWorklogPanel)}
      disabled={$syncState.status === "syncing" || !canSync}
    >
      Log Time to Jira
    </button>
  </div>

  {#if showWorklogPanel}
    <div class="worklog-panel">
      <h4>Sync Calendar Events as Jira Worklogs</h4>
      <p class="hint">
        This will find calendar events matching Jira task patterns and log them as work time.
      </p>

      <div class="date-range">
        <div class="date-input">
          <label for="start-date">From</label>
          <input id="start-date" type="date" bind:value={worklogStartDate} />
        </div>
        <div class="date-input">
          <label for="end-date">To</label>
          <input id="end-date" type="date" bind:value={worklogEndDate} />
        </div>
      </div>

      <div class="panel-actions">
        <button class="sync-worklogs" onclick={handleSyncWorklogs}>
          Sync Worklogs
        </button>
        <button class="cancel" onclick={() => (showWorklogPanel = false)}>
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if $syncState.status === "syncing"}
    <div class="progress">
      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {($syncState.progress / $syncState.total) * 100}%"
        ></div>
      </div>
      <span class="progress-text">
        {$syncState.progress} / {$syncState.total}
      </span>
    </div>
  {/if}

  {#if $syncState.lastSync}
    <p class="last-sync">Last sync: {new Date($syncState.lastSync).toLocaleString()}</p>
  {/if}

  {#if !canSync}
    <p class="warning">
      {#if !$hasToken}
        Configure your Jira credentials in Settings to enable sync.
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
    background: white;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 20px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .main-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  button {
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn {
    background: #f5f5f7;
    color: #1d1d1f;
  }

  .refresh-btn:hover:not(:disabled) {
    background: #e8e8ed;
  }

  .sync-btn {
    background: #0071e3;
    color: white;
  }

  .sync-btn:hover:not(:disabled) {
    background: #0077ed;
  }

  .worklog-btn {
    background: #34c759;
    color: white;
  }

  .worklog-btn:hover:not(:disabled) {
    background: #30d158;
  }

  .worklog-panel {
    margin-top: 16px;
    padding: 16px;
    background: #f5f5f7;
    border-radius: 8px;
  }

  .worklog-panel h4 {
    margin: 0 0 8px;
    font-size: 15px;
  }

  .worklog-panel .hint {
    font-size: 13px;
    color: #86868b;
    margin-bottom: 16px;
  }

  .date-range {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
  }

  .date-input {
    flex: 1;
  }

  .date-input label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .date-input input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d2d2d7;
    border-radius: 6px;
    font-size: 14px;
  }

  .panel-actions {
    display: flex;
    gap: 8px;
  }

  .sync-worklogs {
    background: #34c759;
    color: white;
  }

  .cancel {
    background: #e5e5e5;
    color: #1d1d1f;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 16px;
  }

  .progress-bar {
    flex: 1;
    height: 6px;
    background: #e5e5e5;
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #0071e3;
    transition: width 0.3s;
  }

  .progress-text {
    font-size: 12px;
    color: #86868b;
    min-width: 60px;
  }

  .last-sync {
    margin-top: 12px;
    font-size: 12px;
    color: #86868b;
  }

  .warning {
    margin-top: 12px;
    font-size: 13px;
    color: #ff9500;
    padding: 8px 12px;
    background: #fff9e6;
    border-radius: 6px;
  }
</style>
