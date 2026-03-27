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
      <div class="author-info">
        {#if pr.author_avatar}
          <img src={pr.author_avatar} alt={pr.author} class="avatar" />
        {/if}
        <span>{pr.author}</span>
      </div>
      <span class="branch-info">{pr.branch} → {pr.target_branch}</span>
      <span class="updated">{formatTimeAgo(pr.updated_at)}</span>
    </div>

    {#if getJiraKey()}
      <div class="jira-link">
        <span class="jira-badge">{getJiraKey()}</span>
        {#if pr.linked_jira_key && pr.linked_jira_key !== pr.jira_key}
          <span class="link-label">Linked</span>
        {:else}
          <span class="link-label">Auto-detected</span>
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
        <span class="synced-badge">
          <span class="sync-dot"></span>
          Scheduled
        </span>
      {/if}
    </div>
  {/if}

  <div class="pr-actions">
    {#if onSchedule}
      <button class="act-btn accent" onclick={() => onSchedule(pr)}>
        {pr.calendar_event_uid ? "Reschedule" : "Schedule"}
      </button>
    {/if}
    {#if onLinkJira && !compact}
      <button class="act-btn link" onclick={() => onLinkJira(pr)}>
        {getJiraKey() ? "Change Jira" : "Link Jira"}
      </button>
    {/if}
    <a href={pr.url} target="_blank" rel="noopener" class="act-btn ghost">
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
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    padding: 16px;
    border: 1px solid var(--border-subtle);
    transition: all 0.25s var(--ease-out);
    border-left: 2px solid var(--accent-purple);
  }

  .pr-card.compact {
    border-radius: 0;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    border-left: none;
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
    font-size: 13px;
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

  .pr-card:not(.compact):hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-glow-purple);
    transform: translateY(-1px);
  }

  .pr-card.compact:hover {
    background: var(--bg-elevated);
  }

  .pr-card.synced:not(.compact) {
    border-left-color: var(--accent-green);
  }

  .pr-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }

  .pr-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--accent-purple-dim);
    color: var(--accent-purple);
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  .role-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
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

  .pr-repo {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .draft-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--accent-amber-dim);
    color: var(--accent-amber);
  }

  .pr-title {
    margin: 0 0 10px;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }

  .pr-number {
    font-family: var(--font-mono);
    color: var(--accent-purple);
    font-weight: 600;
  }

  .pr-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 8px;
    font-size: 12px;
    color: var(--text-tertiary);
    align-items: center;
  }

  .compact-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin-bottom: 0;
  }

  .compact-meta span {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .author-info {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .avatar {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid var(--border-default);
  }

  .branch-info {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-elevated);
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .jira-link {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }

  .jira-badge {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid rgba(91, 141, 239, 0.15);
  }

  .link-label {
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .no-jira {
    margin-bottom: 8px;
  }

  .no-jira-label {
    font-size: 12px;
    color: var(--text-tertiary);
    font-style: italic;
  }

  .pr-actions {
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

  .act-btn.accent {
    background: var(--accent-purple-dim);
    border-color: rgba(167, 139, 250, 0.2);
    color: var(--accent-purple);
  }

  .act-btn.accent:hover {
    background: rgba(167, 139, 250, 0.2);
    box-shadow: 0 0 12px var(--accent-purple-dim);
  }

  .act-btn.link {
    background: var(--accent-blue-dim);
    border-color: rgba(91, 141, 239, 0.15);
    color: var(--accent-blue);
  }

  .act-btn.link:hover {
    background: rgba(91, 141, 239, 0.2);
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

  .synced-badge {
    display: inline-flex !important;
    align-items: center;
    gap: 4px;
    background: var(--accent-green-dim) !important;
    color: var(--accent-green) !important;
  }

  .author-compact,
  .updated-compact {
    background: transparent !important;
    color: var(--text-tertiary) !important;
  }
</style>
