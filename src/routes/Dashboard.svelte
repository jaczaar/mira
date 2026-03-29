<script lang="ts">
  import { onMount } from "svelte";
  import TaskList from "../lib/components/TaskList.svelte";
  import SyncControls from "../lib/components/SyncControls.svelte";
  import Notifications from "../lib/components/Notifications.svelte";
  import StatusBadge from "../lib/components/StatusBadge.svelte";
  import TaskScheduler from "../lib/components/TaskScheduler.svelte";
  import PRList from "../lib/components/PRList.svelte";
  import PRScheduler from "../lib/components/PRScheduler.svelte";
  import JiraLinkModal from "../lib/components/JiraLinkModal.svelte";
  import Connectors from "../lib/components/Connectors.svelte";
  import { loadAssignedTasks, tasks } from "../lib/stores/tasks";
  import { loadConfig, config, hasToken } from "../lib/stores/config";
  import { syncState } from "../lib/stores/sync";
  import {
    pullRequests,
    prsLoading,
    prsError,
    hasGitHubToken,
    loadPullRequests,
    checkGitHubToken,
  } from "../lib/stores/github";
  import type { SyncedTask } from "../lib/stores/tasks";
  import type { ScheduledPR } from "../lib/stores/github";

  let taskToSchedule = $state<SyncedTask | null>(null);
  let prToSchedule = $state<ScheduledPR | null>(null);
  let prToLink = $state<ScheduledPR | null>(null);

  type ViewFilter = "all" | "tasks" | "prs" | "scheduled" | "needs-action";
  let viewFilter = $state<ViewFilter>("all");
  let compactMode = $state(false);

  onMount(async () => {
    await loadConfig();
    await checkGitHubToken();

    if ($hasToken) {
      const jqlFilter = $config.jql_filter;
      await loadAssignedTasks(jqlFilter || undefined);
    }

    if ($hasGitHubToken) {
      await loadPullRequests();
    }
  });

  type WorkItem =
    | { type: "task"; item: SyncedTask }
    | { type: "pr"; item: ScheduledPR };

  const filteredItems = $derived.by((): WorkItem[] => {
    let items: WorkItem[] = [];

    if (viewFilter !== "prs") {
      const taskItems: WorkItem[] = $tasks
        .filter((t) => t.status_category !== "done")
        .map((t) => ({ type: "task" as const, item: t }));
      items = [...items, ...taskItems];
    }

    if (viewFilter !== "tasks") {
      const prItems: WorkItem[] = $pullRequests.map((pr) => ({
        type: "pr" as const,
        item: pr,
      }));
      items = [...items, ...prItems];
    }

    if (viewFilter === "scheduled") {
      items = items.filter((item) =>
        item.type === "task"
          ? item.item.calendar_event_uid !== undefined
          : item.item.calendar_event_uid !== undefined
      );
    } else if (viewFilter === "needs-action") {
      items = items.filter((item) =>
        item.type === "task"
          ? item.item.calendar_event_uid === undefined
          : item.item.calendar_event_uid === undefined
      );
    }

    items.sort((a, b) => {
      const getUpdated = (item: WorkItem): string => {
        if (item.type === "task") {
          return item.item.due_date || "";
        }
        return item.item.updated_at;
      };
      return getUpdated(b).localeCompare(getUpdated(a));
    });

    return items;
  });

  const taskCount = $derived(
    $tasks.filter((t) => t.status_category !== "done").length
  );
  const prCount = $derived($pullRequests.length);
  const scheduledCount = $derived(
    $tasks.filter((t) => t.calendar_event_uid).length +
      $pullRequests.filter((p) => p.calendar_event_uid).length
  );

  function handleSyncTask(task: SyncedTask) {
    taskToSchedule = task;
  }

  function handleLogTime(task: SyncedTask) {
    console.log("Log time for:", task.key);
  }

  function handleSchedulePR(pr: ScheduledPR) {
    prToSchedule = pr;
  }

  function handleLinkJira(pr: ScheduledPR) {
    prToLink = pr;
  }

  function handleCloseScheduler() {
    taskToSchedule = null;
    prToSchedule = null;
  }

  function handleCloseJiraLink() {
    prToLink = null;
  }

  function handleTaskScheduled() {
    console.log("Task scheduled successfully!");
  }

  function handlePRScheduled() {
    console.log("PR review scheduled successfully!");
  }

  function handleJiraLinked() {
    console.log("Jira issue linked successfully!");
  }

  async function handleRefreshPRs() {
    if ($hasGitHubToken) {
      await loadPullRequests();
    }
  }
</script>

<div class="dashboard">
  <Notifications />

  <div class="page-header">
    <div class="title-row">
      <h1>Mira</h1>
      <StatusBadge status={$syncState.status} message={$syncState.message} />
      <div style="margin-left: auto;"><Connectors /></div>
    </div>
    <div class="stats-row">
      {#if $hasToken}
        <span class="stat"><span class="stat-value">{taskCount}</span> tasks</span>
      {/if}
      {#if $hasGitHubToken}
        <span class="stat"><span class="stat-value">{prCount}</span> PRs</span>
      {/if}
      <span class="stat"><span class="stat-value accent">{scheduledCount}</span> scheduled</span>
    </div>
  </div>

  {#if !$hasToken && !$hasGitHubToken}
    <div class="setup-prompt">
      <div class="setup-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
        </svg>
      </div>
      <h2>Welcome to Mira</h2>
      <p>Connect your tools below to get started.</p>
      <p class="sub">Jira for tasks, GitHub for PRs, Google Calendar for scheduling.</p>
    </div>
  {:else}
    <SyncControls />

    <div class="filter-tabs">
      <button
        class="filter-tab"
        class:active={viewFilter === "all"}
        onclick={() => (viewFilter = "all")}
      >
        All
        <span class="count">{taskCount + prCount}</span>
      </button>
      {#if $hasToken}
        <button
          class="filter-tab"
          class:active={viewFilter === "tasks"}
          onclick={() => (viewFilter = "tasks")}
        >
          <span class="dot task"></span>
          Tasks
          <span class="count">{taskCount}</span>
        </button>
      {/if}
      {#if $hasGitHubToken}
        <button
          class="filter-tab"
          class:active={viewFilter === "prs"}
          onclick={() => (viewFilter = "prs")}
        >
          <span class="dot pr"></span>
          PRs
          <span class="count">{prCount}</span>
        </button>
      {/if}
      <button
        class="filter-tab"
        class:active={viewFilter === "scheduled"}
        onclick={() => (viewFilter = "scheduled")}
      >
        Scheduled
        <span class="count">{scheduledCount}</span>
      </button>
      <button
        class="filter-tab"
        class:active={viewFilter === "needs-action"}
        onclick={() => (viewFilter = "needs-action")}
      >
        Needs Action
      </button>
      <button class="density-btn" class:active={compactMode} onclick={() => compactMode = !compactMode} title={compactMode ? "Comfortable view" : "Compact view"}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          {#if compactMode}
            <line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" />
          {:else}
            <rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" />
          {/if}
        </svg>
      </button>
      {#if $hasGitHubToken}
        <button class="refresh-btn" onclick={handleRefreshPRs} disabled={$prsLoading}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
          {$prsLoading ? "Loading..." : "Refresh"}
        </button>
      {/if}
    </div>

    {#if $prsError}
      <div class="error-banner">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        GitHub Error: {$prsError}
      </div>
    {/if}

    {#if viewFilter === "tasks"}
      <TaskList onSyncTask={handleSyncTask} onLogTime={handleLogTime} />
    {:else if viewFilter === "prs"}
      <PRList onSchedule={handleSchedulePR} onLinkJira={handleLinkJira} />
    {:else}
      {#if filteredItems.length === 0}
        <div class="empty">
          <p>No items to display</p>
        </div>
      {:else}
        <div class="combined-list" class:compact={compactMode}>
          {#each filteredItems as workItem, i (workItem.type + "-" + (workItem.type === "task" ? workItem.item.id : workItem.item.id))}
            <div class="work-item" class:compact={compactMode} class:task-item={workItem.type === "task"} class:pr-item={workItem.type === "pr"} style="animation: fadeInUp 0.3s var(--ease-out) {Math.min(i, 15) * 30}ms both">
              {#if workItem.type === "task"}
                <div class="item-type-indicator task"></div>
                <div class="item-content">
                  <div class="item-header">
                    <span class="item-key task">{workItem.item.key}</span>
                    <span class="item-status">{workItem.item.status}</span>
                  </div>
                  <div class="item-summary">{workItem.item.summary}</div>
                  <div class="item-meta">
                    <span>{workItem.item.project_name}</span>
                    {#if workItem.item.priority}
                      <span class="priority">{workItem.item.priority}</span>
                    {/if}
                    {#if workItem.item.calendar_event_uid}
                      <span class="scheduled-badge">
                        <span class="sync-dot"></span>
                        Scheduled
                      </span>
                    {/if}
                  </div>
                </div>
                <div class="item-actions">
                  <button class="act-btn primary" onclick={() => handleSyncTask(workItem.item)}>
                    {workItem.item.calendar_event_uid ? "Reschedule" : "Schedule"}
                  </button>
                  <a href={workItem.item.url} target="_blank" rel="noopener" class="act-btn ghost">
                    View
                  </a>
                </div>
              {:else}
                <div class="item-type-indicator pr"></div>
                <div class="item-content">
                  <div class="item-header">
                    <span class="item-key pr">{workItem.item.repo_name}</span>
                    <span class="item-number">#{workItem.item.number}</span>
                    <span class="role-badge" class:author={workItem.item.pr_role === "author"} class:reviewer={workItem.item.pr_role === "reviewer"}>
                      {workItem.item.pr_role === "author" ? "Author" : "Review"}
                    </span>
                    {#if workItem.item.is_draft}
                      <span class="draft-badge">Draft</span>
                    {/if}
                  </div>
                  <div class="item-summary">{workItem.item.title}</div>
                  <div class="item-meta">
                    <span class="author-text">{workItem.item.author}</span>
                    {#if workItem.item.linked_jira_key || workItem.item.jira_key}
                      <span class="jira-badge">{workItem.item.linked_jira_key || workItem.item.jira_key}</span>
                    {/if}
                    {#if workItem.item.calendar_event_uid}
                      <span class="scheduled-badge">
                        <span class="sync-dot"></span>
                        Scheduled
                      </span>
                    {/if}
                  </div>
                </div>
                <div class="item-actions">
                  <button class="act-btn accent" onclick={() => handleSchedulePR(workItem.item)}>
                    {workItem.item.calendar_event_uid ? "Reschedule" : "Schedule"}
                  </button>
                  <button class="act-btn link" onclick={() => handleLinkJira(workItem.item)}>
                    {workItem.item.linked_jira_key || workItem.item.jira_key ? "Jira" : "Link"}
                  </button>
                  <a href={workItem.item.url} target="_blank" rel="noopener" class="act-btn ghost">
                    View
                  </a>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  {/if}


</div>

{#if taskToSchedule}
  <TaskScheduler
    task={taskToSchedule}
    onClose={handleCloseScheduler}
    onScheduled={handleTaskScheduled}
  />
{/if}

{#if prToSchedule}
  <PRScheduler
    pr={prToSchedule}
    onClose={handleCloseScheduler}
    onScheduled={handlePRScheduled}
  />
{/if}

{#if prToLink}
  <JiraLinkModal
    pr={prToLink}
    onClose={handleCloseJiraLink}
    onLinked={handleJiraLinked}
  />
{/if}

<style>
  .dashboard {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 24px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 6px;
  }

  h1 {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
    letter-spacing: -0.04em;
  }

  .stats-row {
    display: flex;
    gap: 16px;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .stat-value {
    font-family: var(--font-display);
    font-weight: 600;
    color: var(--text-secondary);
  }

  .stat-value.accent {
    color: var(--accent-blue);
  }

  .setup-prompt {
    padding: 56px 40px;
    text-align: center;
    animation: fadeInUp 0.4s var(--ease-out);
  }

  .setup-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: var(--gradient-brand);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 20px;
    color: white;
  }

  .setup-prompt h2 {
    margin: 0 0 8px;
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .setup-prompt p {
    margin: 4px 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .setup-prompt .sub {
    color: var(--text-tertiary);
    font-size: 14px;
  }


  .filter-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-subtle);
    flex-wrap: wrap;
    align-items: center;
  }

  .filter-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .filter-tab:hover {
    color: var(--text-secondary);
  }

  .filter-tab.active {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .filter-tab .count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .filter-tab.active .count {
    color: var(--text-secondary);
  }

  .filter-tab .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .filter-tab .dot.task {
    background: var(--accent-blue);
  }

  .filter-tab .dot.pr {
    background: var(--accent-purple);
  }

  .refresh-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 6px 12px;
    border: 1px solid var(--border-default);
    background: transparent;
    border-radius: 7px;
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .refresh-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .refresh-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--accent-red-dim);
    border-radius: var(--radius-md);
    color: var(--accent-red);
    font-size: 13px;
    margin-bottom: 16px;
    border: 1px solid rgba(248, 113, 113, 0.15);
  }

  .empty {
    text-align: center;
    padding: 48px 24px;
    color: var(--text-tertiary);
  }

  .combined-list {
    display: flex;
    flex-direction: column;
  }

  .work-item {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px 4px;
    border-bottom: 1px solid var(--border-subtle);
    transition: all 0.15s var(--ease-out);
  }

  .work-item:first-child {
    border-top: 1px solid var(--border-subtle);
  }

  .work-item:hover {
    background: var(--bg-hover);
  }

  .item-type-indicator {
    width: 3px;
    min-height: 36px;
    border-radius: 2px;
    flex-shrink: 0;
    align-self: stretch;
  }

  .item-type-indicator.task {
    background: var(--accent-blue);
  }

  .item-type-indicator.pr {
    background: var(--accent-purple);
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .item-key {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .item-key.task {
    color: var(--accent-blue);
  }

  .item-key.pr {
    color: var(--text-tertiary);
  }

  .item-number {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-purple);
  }

  .item-status {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: var(--bg-hover);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .role-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    padding: 1px 6px;
    border-radius: 4px;
    letter-spacing: 0.02em;
  }

  .role-badge.reviewer {
    background: var(--accent-purple-dim);
    color: var(--accent-purple);
  }

  .role-badge.author {
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .draft-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-amber-dim);
    color: var(--accent-amber);
  }

  .item-summary {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 6px;
    line-height: 1.4;
  }

  .item-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 11px;
  }

  .item-meta span {
    color: var(--text-tertiary);
  }

  .item-meta .priority {
    color: var(--accent-amber);
  }

  .item-meta .jira-badge {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--accent-blue);
  }

  .item-meta .scheduled-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-green);
  }

  .sync-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-green);
  }

  .item-meta .author-text {
    background: transparent;
    color: var(--text-tertiary);
    border: none;
    padding: 0;
  }

  .item-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
    align-self: center;
  }

  .act-btn {
    padding: 5px 10px;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.15s var(--ease-out);
    background: transparent;
    color: var(--text-tertiary);
  }

  .act-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .act-btn.primary {
    color: var(--accent-blue);
  }

  .act-btn.primary:hover {
    background: var(--accent-blue-dim);
  }

  .act-btn.accent {
    color: var(--accent-purple);
  }

  .act-btn.accent:hover {
    background: var(--accent-purple-dim);
  }

  .act-btn.link {
    color: var(--accent-blue);
  }

  .act-btn.link:hover {
    background: var(--accent-blue-dim);
  }

  .act-btn.ghost {
    color: var(--text-tertiary);
  }

  .density-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 1px solid var(--border-default);
    background: transparent;
    border-radius: 7px;
    color: var(--text-tertiary);
    cursor: pointer;
    margin-left: auto;
    transition: all 0.15s var(--ease-out);
  }

  .density-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .density-btn.active {
    color: var(--accent-blue);
    border-color: var(--accent-blue-dim);
    background: var(--accent-blue-dim);
  }

  .work-item.compact {
    padding: 8px 4px;
    gap: 10px;
  }

  .work-item.compact .item-summary {
    font-size: 13px;
    margin-bottom: 2px;
  }

  .work-item.compact .item-meta {
    display: none;
  }
</style>
