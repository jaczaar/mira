<script lang="ts">
  import { onMount } from "svelte";
  import type { ScheduledPR } from "../stores/github";
  import { updatePRJiraLink } from "../stores/github";
  import * as api from "../api";
  import type { SimpleIssue } from "../api";

  interface Props {
    pr: ScheduledPR;
    onClose: () => void;
    onLinked: () => void;
  }

  let { pr, onClose, onLinked }: Props = $props();

  let searchQuery = $state("");
  let searchResults = $state<SimpleIssue[]>([]);
  let isSearching = $state(false);
  let error = $state<string | null>(null);
  let selectedIssue = $state<SimpleIssue | null>(null);

  // Debounce timer
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    // If PR has auto-detected Jira key, search for it
    if (pr.jira_key && !pr.linked_jira_key) {
      searchQuery = pr.jira_key;
      handleSearch();
    }
  });

  async function handleSearch() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    isSearching = true;
    error = null;

    try {
      // Try exact key match first, then summary search
      const jql = `key = "${searchQuery}" OR summary ~ "${searchQuery}" ORDER BY updated DESC`;
      const results = await api.searchIssues(jql, 10);
      searchResults = results;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      searchResults = [];
    } finally {
      isSearching = false;
    }
  }

  function onSearchInput() {
    // Debounce search
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    searchTimeout = setTimeout(() => {
      handleSearch();
    }, 300);
  }

  function selectIssue(issue: SimpleIssue) {
    selectedIssue = issue;
  }

  function handleLink() {
    if (selectedIssue) {
      updatePRJiraLink(pr.id, selectedIssue.key);
      onLinked();
      onClose();
    }
  }

  function handleClearLink() {
    updatePRJiraLink(pr.id, null);
    onLinked();
    onClose();
  }

  function handleUseAutoDetected() {
    if (pr.jira_key) {
      updatePRJiraLink(pr.id, pr.jira_key);
      onLinked();
      onClose();
    }
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
</script>

<div
  class="modal-overlay"
  onclick={onClose}
  onkeydown={(e) => e.key === "Escape" && onClose()}
  role="dialog"
  aria-modal="true"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="modal" role="presentation" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2>Link to Jira Issue</h2>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    <div class="pr-info">
      <span class="pr-badge">PR</span>
      <span class="pr-number">#{pr.number}</span>
      <span class="pr-title">{pr.title}</span>
    </div>

    {#if pr.jira_key}
      <div class="auto-detected">
        <div class="auto-label">Auto-detected from PR title/branch:</div>
        <div class="auto-key">
          <span class="jira-key">{pr.jira_key}</span>
          <button class="use-btn" onclick={handleUseAutoDetected}>Use this</button>
        </div>
      </div>
    {/if}

    {#if pr.linked_jira_key}
      <div class="current-link">
        <div class="current-label">Currently linked to:</div>
        <div class="current-key">
          <span class="jira-key">{pr.linked_jira_key}</span>
          <button class="clear-btn" onclick={handleClearLink}>Clear link</button>
        </div>
      </div>
    {/if}

    <div class="search-section">
      <label for="jira-search">Search Jira Issues</label>
      <input
        id="jira-search"
        type="text"
        placeholder="Search by key (PROJ-123) or summary..."
        bind:value={searchQuery}
        oninput={onSearchInput}
        onkeydown={(e) => e.key === "Enter" && handleSearch()}
      />
    </div>

    {#if isSearching}
      <div class="loading">Searching...</div>
    {:else if error}
      <div class="error">{error}</div>
    {:else if searchResults.length > 0}
      <div class="results">
        {#each searchResults as issue (issue.id)}
          <button
            class="result-item"
            class:selected={selectedIssue?.id === issue.id}
            onclick={() => selectIssue(issue)}
          >
            <div class="result-header">
              <span class="result-key">{issue.key}</span>
              <span
                class="result-status"
                style="--status-color: {getStatusColor(issue.status_category)}"
              >
                {issue.status}
              </span>
            </div>
            <div class="result-summary">{issue.summary}</div>
            <div class="result-meta">
              <span>{issue.project_name}</span>
              {#if issue.issue_type}
                <span>{issue.issue_type}</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {:else if searchQuery}
      <div class="no-results">No issues found</div>
    {/if}

    <div class="modal-actions">
      <button class="cancel-btn" onclick={onClose}>Cancel</button>
      <button
        class="link-btn"
        onclick={handleLink}
        disabled={!selectedIssue}
      >
        Link to {selectedIssue?.key || "Issue"}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-default);
    animation: fadeInUp 0.3s var(--ease-out);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .modal-header h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 22px;
    cursor: pointer;
    color: var(--text-tertiary);
    line-height: 1;
    padding: 2px;
    border-radius: 4px;
    transition: all 0.15s;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .pr-info {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: var(--bg-elevated);
    font-size: 13px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .pr-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--accent-purple-dim);
    color: var(--accent-purple);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .pr-number {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--accent-purple);
  }

  .pr-title {
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .auto-detected,
  .current-link {
    padding: 10px 20px;
    background: var(--accent-blue-dim);
    border-bottom: 1px solid var(--border-subtle);
  }

  .current-link {
    background: var(--accent-amber-dim);
  }

  .auto-label,
  .current-label {
    font-size: 11px;
    color: var(--text-tertiary);
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .auto-key,
  .current-key {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .jira-key {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 4px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid var(--accent-blue-dim);
  }

  .use-btn,
  .clear-btn {
    padding: 4px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .use-btn {
    background: var(--accent-blue-dim);
    border-color: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .use-btn:hover {
    background: var(--accent-blue-glow);
  }

  .clear-btn {
    background: var(--accent-red-dim);
    border-color: rgba(248, 113, 113, 0.15);
    color: var(--accent-red);
  }

  .clear-btn:hover {
    background: rgba(248, 113, 113, 0.2);
  }

  .search-section {
    padding: 14px 20px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .search-section label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    margin-bottom: 6px;
    color: var(--text-secondary);
  }

  .search-section input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    background: var(--bg-elevated);
    color: var(--text-primary);
    transition: border-color 0.15s;
  }

  .search-section input::placeholder {
    color: var(--text-tertiary);
  }

  .search-section input:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .loading,
  .no-results {
    padding: 24px;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 13px;
  }

  .error {
    padding: 10px 20px;
    background: var(--accent-red-dim);
    color: var(--accent-red);
    font-size: 13px;
  }

  .results {
    max-height: 300px;
    overflow-y: auto;
  }

  .result-item {
    display: block;
    width: 100%;
    padding: 10px 20px;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
    color: var(--text-primary);
  }

  .result-item:hover {
    background: var(--bg-elevated);
  }

  .result-item.selected {
    background: var(--accent-blue-dim);
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .result-key {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-blue);
  }

  .result-status {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: color-mix(in srgb, var(--status-color) 12%, transparent);
    color: var(--status-color);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .result-summary {
    font-size: 13px;
    color: var(--text-primary);
    margin-bottom: 4px;
    line-height: 1.4;
  }

  .result-meta {
    display: flex;
    gap: 8px;
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .modal-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding: 14px 20px;
    border-top: 1px solid var(--border-subtle);
  }

  .cancel-btn,
  .link-btn {
    padding: 8px 18px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .cancel-btn {
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .link-btn {
    background: var(--accent-blue-dim);
    border-color: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .link-btn:hover:not(:disabled) {
    background: var(--accent-blue-glow);
    box-shadow: var(--shadow-glow-blue);
  }

  .link-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
</style>
