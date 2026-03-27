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
        return "#34c759";
      case "indeterminate":
        return "#0071e3";
      default:
        return "#86868b";
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
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: white;
    border-radius: 16px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid #e5e5e5;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #86868b;
    line-height: 1;
    padding: 0;
  }

  .close-btn:hover {
    color: #1d1d1f;
  }

  .pr-info {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: #f5f3ff;
    font-size: 13px;
  }

  .pr-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: #8b5cf6;
    color: white;
  }

  .pr-number {
    font-weight: 600;
    color: #8b5cf6;
  }

  .pr-title {
    color: #1d1d1f;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .auto-detected,
  .current-link {
    padding: 12px 24px;
    background: #f0f9ff;
    border-bottom: 1px solid #e5e5e5;
  }

  .current-link {
    background: #fef3c7;
  }

  .auto-label,
  .current-label {
    font-size: 12px;
    color: #6b7280;
    margin-bottom: 6px;
  }

  .auto-key,
  .current-key {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .jira-key {
    font-size: 14px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 4px;
    background: #dbeafe;
    color: #1d4ed8;
  }

  .use-btn,
  .clear-btn {
    padding: 4px 12px;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .use-btn {
    background: #0071e3;
    color: white;
  }

  .use-btn:hover {
    background: #0077ed;
  }

  .clear-btn {
    background: #ffebea;
    color: #ff3b30;
  }

  .clear-btn:hover {
    background: #ffd5d2;
  }

  .search-section {
    padding: 16px 24px;
    border-bottom: 1px solid #e5e5e5;
  }

  .search-section label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 8px;
    color: #1d1d1f;
  }

  .search-section input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    font-size: 14px;
  }

  .search-section input:focus {
    outline: none;
    border-color: #0071e3;
  }

  .loading,
  .no-results {
    padding: 24px;
    text-align: center;
    color: #86868b;
    font-size: 14px;
  }

  .error {
    padding: 12px 24px;
    background: #ffebea;
    color: #ff3b30;
    font-size: 13px;
  }

  .results {
    max-height: 300px;
    overflow-y: auto;
  }

  .result-item {
    display: block;
    width: 100%;
    padding: 12px 24px;
    border: none;
    border-bottom: 1px solid #f0f0f0;
    background: white;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .result-item:hover {
    background: #f9f9f9;
  }

  .result-item.selected {
    background: #dbeafe;
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .result-key {
    font-size: 13px;
    font-weight: 600;
    color: #0071e3;
  }

  .result-status {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--status-color) 15%, white);
    color: var(--status-color);
    font-weight: 500;
  }

  .result-summary {
    font-size: 14px;
    color: #1d1d1f;
    margin-bottom: 4px;
    line-height: 1.4;
  }

  .result-meta {
    display: flex;
    gap: 8px;
    font-size: 12px;
    color: #86868b;
  }

  .modal-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 24px;
    border-top: 1px solid #e5e5e5;
  }

  .cancel-btn,
  .link-btn {
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .cancel-btn {
    background: #f5f5f7;
    color: #1d1d1f;
  }

  .cancel-btn:hover {
    background: #e8e8ed;
  }

  .link-btn {
    background: #0071e3;
    color: white;
  }

  .link-btn:hover:not(:disabled) {
    background: #0077ed;
  }

  .link-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
