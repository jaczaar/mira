<script lang="ts">
  import { onMount } from "svelte";
  import TaskList from "../lib/components/TaskList.svelte";
  import SyncControls from "../lib/components/SyncControls.svelte";
  import Notifications from "../lib/components/Notifications.svelte";
  import TaskScheduler from "../lib/components/TaskScheduler.svelte";
  import PRCard from "../lib/components/PRCard.svelte";
  import PRList from "../lib/components/PRList.svelte";
  import PRScheduler from "../lib/components/PRScheduler.svelte";
  import JiraLinkModal from "../lib/components/JiraLinkModal.svelte";
  import { loadAssignedTasks, tasks } from "../lib/stores/tasks";
  import { loadConfig, config, hasToken } from "../lib/stores/config";
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

  // Combined and filtered work items
  type WorkItem =
    | { type: "task"; item: SyncedTask }
    | { type: "pr"; item: ScheduledPR };

  const filteredItems = $derived.by((): WorkItem[] => {
    let items: WorkItem[] = [];

    // Add tasks
    if (viewFilter !== "prs") {
      const taskItems: WorkItem[] = $tasks
        .filter((t) => t.status_category !== "done")
        .map((t) => ({ type: "task" as const, item: t }));
      items = [...items, ...taskItems];
    }

    // Add PRs
    if (viewFilter !== "tasks") {
      const prItems: WorkItem[] = $pullRequests.map((pr) => ({
        type: "pr" as const,
        item: pr,
      }));
      items = [...items, ...prItems];
    }

    // Apply additional filters
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

    // Sort by updated date (most recent first)
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

  <h1>Dashboard</h1>

  {#if !$hasToken && !$hasGitHubToken}
    <div class="setup-prompt">
      <h2>Welcome to Mira!</h2>
      <p>Get started by configuring your connections in Settings.</p>
      <p>
        You can connect to Jira (for tasks) and/or GitHub (for PR reviews).
      </p>
    </div>
  {:else}
    <SyncControls />

    <!-- Filter tabs -->
    <div class="filter-tabs">
      <button
        class="filter-tab"
        class:active={viewFilter === "all"}
        onclick={() => (viewFilter = "all")}
      >
        All ({taskCount + prCount})
      </button>
      {#if $hasToken}
        <button
          class="filter-tab task-tab"
          class:active={viewFilter === "tasks"}
          onclick={() => (viewFilter = "tasks")}
        >
          <span class="tab-badge task">Tasks</span>
          {taskCount}
        </button>
      {/if}
      {#if $hasGitHubToken}
        <button
          class="filter-tab pr-tab"
          class:active={viewFilter === "prs"}
          onclick={() => (viewFilter = "prs")}
        >
          <span class="tab-badge pr">PRs</span>
          {prCount}
        </button>
      {/if}
      <button
        class="filter-tab"
        class:active={viewFilter === "scheduled"}
        onclick={() => (viewFilter = "scheduled")}
      >
        Scheduled ({scheduledCount})
      </button>
      <button
        class="filter-tab"
        class:active={viewFilter === "needs-action"}
        onclick={() => (viewFilter = "needs-action")}
      >
        Needs Action
      </button>
      {#if $hasGitHubToken}
        <button class="refresh-btn" onclick={handleRefreshPRs} disabled={$prsLoading}>
          {$prsLoading ? "Loading..." : "Refresh PRs"}
        </button>
      {/if}
    </div>

    {#if $prsError}
      <div class="error-banner">
        GitHub Error: {$prsError}
      </div>
    {/if}

    <!-- Show task list for task-only view, combined view otherwise -->
    {#if viewFilter === "tasks"}
      <TaskList onSyncTask={handleSyncTask} onLogTime={handleLogTime} />
    {:else if viewFilter === "prs"}
      <PRList onSchedule={handleSchedulePR} onLinkJira={handleLinkJira} />
    {:else}
      <!-- Combined view -->
      {#if filteredItems.length === 0}
        <div class="empty">
          <p>No items to display</p>
        </div>
      {:else}
        <div class="combined-list">
          {#each filteredItems as workItem (workItem.type + "-" + (workItem.type === "task" ? workItem.item.id : workItem.item.id))}
            {#if workItem.type === "task"}
              <div class="work-item task-item">
                <div class="item-type-badge task">Task</div>
                <div class="item-content">
                  <div class="item-header">
                    <span class="item-key">{workItem.item.key}</span>
                    <span class="item-status">{workItem.item.status}</span>
                  </div>
                  <div class="item-summary">{workItem.item.summary}</div>
                  <div class="item-meta">
                    <span>{workItem.item.project_name}</span>
                    {#if workItem.item.priority}
                      <span class="priority">{workItem.item.priority}</span>
                    {/if}
                    {#if workItem.item.calendar_event_uid}
                      <span class="scheduled-badge">Scheduled</span>
                    {/if}
                  </div>
                </div>
                <div class="item-actions">
                  <button class="action-btn schedule" onclick={() => handleSyncTask(workItem.item)}>
                    {workItem.item.calendar_event_uid ? "Reschedule" : "Schedule"}
                  </button>
                  <a href={workItem.item.url} target="_blank" rel="noopener" class="action-btn view">
                    View
                  </a>
                </div>
              </div>
            {:else}
              <div class="work-item pr-item">
                <div class="item-type-badge pr">PR</div>
                <div class="item-content">
                  <div class="item-header">
                    <span class="item-key">{workItem.item.repo_name}</span>
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
                    <span class="author">{workItem.item.author}</span>
                    {#if workItem.item.linked_jira_key || workItem.item.jira_key}
                      <span class="jira-badge">{workItem.item.linked_jira_key || workItem.item.jira_key}</span>
                    {/if}
                    {#if workItem.item.calendar_event_uid}
                      <span class="scheduled-badge">Scheduled</span>
                    {/if}
                  </div>
                </div>
                <div class="item-actions">
                  <button class="action-btn schedule pr-btn" onclick={() => handleSchedulePR(workItem.item)}>
                    {workItem.item.calendar_event_uid ? "Reschedule" : "Schedule"}
                  </button>
                  <button class="action-btn link" onclick={() => handleLinkJira(workItem.item)}>
                    {workItem.item.linked_jira_key || workItem.item.jira_key ? "Jira" : "Link"}
                  </button>
                  <a href={workItem.item.url} target="_blank" rel="noopener" class="action-btn view">
                    View
                  </a>
                </div>
              </div>
            {/if}
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

  h1 {
    font-size: 28px;
    font-weight: 600;
    margin: 0 0 24px;
    color: #1d1d1f;
  }

  .setup-prompt {
    background: white;
    border-radius: 12px;
    padding: 40px;
    text-align: center;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .setup-prompt h2 {
    margin: 0 0 16px;
    font-size: 24px;
    color: #1d1d1f;
  }

  .setup-prompt p {
    margin: 8px 0;
    color: #86868b;
    line-height: 1.5;
  }

  .filter-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e5e5e5;
    flex-wrap: wrap;
    align-items: center;
  }

  .filter-tab {
    padding: 8px 16px;
    border: 1px solid #d2d2d7;
    background: white;
    border-radius: 20px;
    font-size: 13px;
    color: #1d1d1f;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .filter-tab:hover {
    background: #f5f5f7;
  }

  .filter-tab.active {
    background: #1d1d1f;
    color: white;
    border-color: #1d1d1f;
  }

  .tab-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .tab-badge.task {
    background: #dbeafe;
    color: #1d4ed8;
  }

  .tab-badge.pr {
    background: #ede9fe;
    color: #7c3aed;
  }

  .filter-tab.active .tab-badge.task {
    background: rgba(255, 255, 255, 0.2);
    color: white;
  }

  .filter-tab.active .tab-badge.pr {
    background: rgba(255, 255, 255, 0.2);
    color: white;
  }

  .refresh-btn {
    padding: 8px 16px;
    border: 1px solid #8b5cf6;
    background: white;
    border-radius: 20px;
    font-size: 13px;
    color: #8b5cf6;
    cursor: pointer;
    margin-left: auto;
  }

  .refresh-btn:hover:not(:disabled) {
    background: #ede9fe;
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner {
    padding: 12px 16px;
    background: #ffebea;
    border-radius: 8px;
    color: #ff3b30;
    font-size: 13px;
    margin-bottom: 16px;
  }

  .empty {
    text-align: center;
    padding: 40px;
    background: white;
    border-radius: 12px;
    color: #86868b;
  }

  .combined-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .work-item {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    padding: 16px;
    background: white;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: box-shadow 0.2s;
  }

  .work-item:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .work-item.task-item {
    border-left: 3px solid #0071e3;
  }

  .work-item.pr-item {
    border-left: 3px solid #8b5cf6;
  }

  .item-type-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 4px 8px;
    border-radius: 4px;
    white-space: nowrap;
  }

  .item-type-badge.task {
    background: #dbeafe;
    color: #1d4ed8;
  }

  .item-type-badge.pr {
    background: #ede9fe;
    color: #7c3aed;
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .item-key {
    font-size: 13px;
    font-weight: 600;
    color: #0071e3;
  }

  .pr-item .item-key {
    color: #6b7280;
  }

  .item-number {
    font-size: 13px;
    font-weight: 600;
    color: #8b5cf6;
  }

  .item-status {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: #f5f5f7;
    color: #86868b;
  }

  .role-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .role-badge.reviewer {
    background: #ede9fe;
    color: #7c3aed;
  }

  .role-badge.author {
    background: #dbeafe;
    color: #1d4ed8;
  }

  .draft-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: #fef3c7;
    color: #d97706;
  }

  .item-summary {
    font-size: 14px;
    font-weight: 500;
    color: #1d1d1f;
    margin-bottom: 8px;
    line-height: 1.4;
  }

  .item-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 12px;
    color: #86868b;
  }

  .item-meta span {
    padding: 2px 8px;
    background: #f5f5f7;
    border-radius: 4px;
  }

  .item-meta .priority {
    background: #fef3c7;
    color: #d97706;
  }

  .item-meta .jira-badge {
    background: #dbeafe;
    color: #1d4ed8;
    font-weight: 500;
  }

  .item-meta .scheduled-badge {
    background: #e8f8ec;
    color: #34c759;
  }

  .item-meta .author {
    background: transparent;
    color: #6b7280;
  }

  .item-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
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

  .action-btn.schedule {
    background: #0071e3;
    color: white;
  }

  .action-btn.schedule:hover {
    background: #0077ed;
  }

  .action-btn.schedule.pr-btn {
    background: #8b5cf6;
  }

  .action-btn.schedule.pr-btn:hover {
    background: #7c3aed;
  }

  .action-btn.link {
    background: #dbeafe;
    color: #1d4ed8;
  }

  .action-btn.link:hover {
    background: #bfdbfe;
  }

  .action-btn.view {
    background: #f5f5f7;
    color: #1d1d1f;
  }

  .action-btn.view:hover {
    background: #e8e8ed;
  }
</style>
