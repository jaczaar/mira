<script lang="ts">
  import type { ScheduledPR } from "../stores/github";

  interface Props {
    pr: ScheduledPR;
    onSchedule?: (pr: ScheduledPR) => void;
    onLinkJira?: (pr: ScheduledPR) => void;
    compact?: boolean;
  }

  let { pr, onSchedule, onLinkJira, compact = false }: Props = $props();

  function formatTimeAgo(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMins = Math.floor(diffMs / (1000 * 60));

    if (diffDays > 0) return `${diffDays}d ago`;
    if (diffHours > 0) return `${diffHours}h ago`;
    if (diffMins > 0) return `${diffMins}m ago`;
    return "just now";
  }

  function getJiraKey(): string | null {
    return pr.linked_jira_key || pr.jira_key;
  }
</script>

<div class="pr-card" class:synced={pr.calendar_event_uid} class:compact>
  <div class="pr-header">
    <span class="pr-badge">PR</span>
    <span class="role-badge" class:author={pr.pr_role === "author"} class:reviewer={pr.pr_role === "reviewer"}>
      {pr.pr_role === "author" ? "Author" : "Review"}
    </span>
    <span class="pr-repo">{pr.repo_name}</span>
    {#if pr.is_draft}
      <span class="draft-badge">Draft</span>
    {/if}
  </div>

  <h3 class="pr-title">
    <span class="pr-number">#{pr.number}</span>
    {pr.title}
  </h3>

  {#if !compact}
    <div class="pr-meta">
      <div class="author">
        {#if pr.author_avatar}
          <img src={pr.author_avatar} alt={pr.author} class="avatar" />
        {/if}
        <span>{pr.author}</span>
      </div>
      <span class="branch-info">{pr.branch} → {pr.target_branch}</span>
      <span class="updated">Updated {formatTimeAgo(pr.updated_at)}</span>
    </div>

    {#if getJiraKey()}
      <div class="jira-link">
        <span class="jira-badge">{getJiraKey()}</span>
        {#if pr.linked_jira_key && pr.linked_jira_key !== pr.jira_key}
          <span class="linked-label">Linked</span>
        {:else}
          <span class="auto-label">Auto-detected</span>
        {/if}
      </div>
    {:else}
      <div class="no-jira">
        <span class="no-jira-label">No Jira issue linked</span>
      </div>
    {/if}
  {:else}
    <div class="compact-meta">
      <span class="author-compact">{pr.author}</span>
      <span class="updated-compact">{formatTimeAgo(pr.updated_at)}</span>
      {#if getJiraKey()}
        <span class="jira-badge">{getJiraKey()}</span>
      {/if}
      {#if pr.calendar_event_uid}
        <span class="synced-badge">Scheduled</span>
      {/if}
    </div>
  {/if}

  <div class="pr-actions">
    {#if onSchedule}
      <button class="action-btn schedule" onclick={() => onSchedule(pr)}>
        {pr.calendar_event_uid ? "Reschedule" : "Schedule"}
      </button>
    {/if}
    {#if onLinkJira && !compact}
      <button class="action-btn link" onclick={() => onLinkJira(pr)}>
        {getJiraKey() ? "Change Jira" : "Link Jira"}
      </button>
    {/if}
    <a href={pr.url} target="_blank" rel="noopener" class="action-btn view">
      View
    </a>
  </div>

  {#if !compact && pr.calendar_event_uid}
    <div class="sync-indicator">
      <span class="sync-dot"></span>
      Scheduled
      {#if pr.last_synced}
        <span class="sync-time">
          {new Date(pr.last_synced).toLocaleString()}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .pr-card {
    background: white;
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: box-shadow 0.2s;
    border-left: 3px solid #8b5cf6;
  }

  .pr-card.compact {
    border-radius: 0;
    box-shadow: none;
    padding: 12px 20px;
    display: grid;
    grid-template-columns: auto 1fr auto;
    grid-template-rows: auto auto;
    gap: 4px 16px;
    align-items: center;
  }

  .pr-card.compact .pr-header {
    grid-column: 1;
    grid-row: 1 / 3;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    margin-bottom: 0;
  }

  .pr-card.compact .pr-title {
    grid-column: 2;
    grid-row: 1;
    margin: 0;
    font-size: 14px;
  }

  .pr-card.compact .compact-meta {
    grid-column: 2;
    grid-row: 2;
  }

  .pr-card.compact .pr-actions {
    grid-column: 3;
    grid-row: 1 / 3;
    margin-top: 0;
    padding-top: 0;
    border-top: none;
  }

  .pr-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .pr-card.compact:hover {
    background: #f9f9f9;
    box-shadow: none;
  }

  .pr-card.synced {
    border-left-color: #34c759;
  }

  .pr-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .pr-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: #8b5cf6;
    color: white;
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

  .pr-repo {
    font-size: 13px;
    font-weight: 500;
    color: #6b7280;
  }

  .draft-badge {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
    background: #fef3c7;
    color: #d97706;
  }

  .pr-title {
    margin: 0 0 12px;
    font-size: 15px;
    font-weight: 500;
    color: #1d1d1f;
    line-height: 1.4;
  }

  .pr-number {
    color: #8b5cf6;
    font-weight: 600;
  }

  .pr-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-bottom: 8px;
    font-size: 12px;
    color: #6b7280;
    align-items: center;
  }

  .compact-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 0;
  }

  .compact-meta span {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 4px;
    background: #f5f5f7;
    color: #86868b;
  }

  .author {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .avatar {
    width: 20px;
    height: 20px;
    border-radius: 50%;
  }

  .branch-info {
    font-family: monospace;
    font-size: 11px;
    background: #f5f5f7;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .jira-link {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }

  .jira-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    background: #dbeafe;
    color: #1d4ed8;
  }

  .linked-label,
  .auto-label {
    font-size: 10px;
    color: #6b7280;
  }

  .no-jira {
    margin-bottom: 8px;
  }

  .no-jira-label {
    font-size: 12px;
    color: #9ca3af;
    font-style: italic;
  }

  .pr-actions {
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

  .action-btn.schedule {
    background: #8b5cf6;
    color: white;
  }

  .action-btn.schedule:hover {
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

  .synced-badge {
    background: #e8f8ec !important;
    color: #34c759 !important;
  }

  .author-compact,
  .updated-compact {
    background: transparent !important;
    color: #86868b !important;
  }
</style>
