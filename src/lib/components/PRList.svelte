<script lang="ts">
  import PRCard from "./PRCard.svelte";
  import { pullRequests, prsLoading, prsError } from "../stores/github";
  import type { ScheduledPR } from "../stores/github";

  interface Props {
    onSchedule?: (pr: ScheduledPR) => void;
    onLinkJira?: (pr: ScheduledPR) => void;
  }

  let { onSchedule, onLinkJira }: Props = $props();

  let viewMode = $state<"flat" | "grouped">("grouped");
  let roleFilter = $state<"all" | "reviewer" | "author">("all");

  const filteredPRs = $derived.by((): ScheduledPR[] => {
    if (roleFilter === "all") return $pullRequests;
    return $pullRequests.filter((pr) => pr.pr_role === roleFilter);
  });

  interface RepoGroup {
    repoName: string;
    repoFullName: string;
    prs: ScheduledPR[];
  }

  const groupedByRepo = $derived.by((): RepoGroup[] => {
    const groups = new Map<string, RepoGroup>();

    for (const pr of filteredPRs) {
      const key = pr.repo_name;
      if (!groups.has(key)) {
        groups.set(key, {
          repoName: pr.repo_name,
          repoFullName: pr.repo_full_name,
          prs: [],
        });
      }
      groups.get(key)!.prs.push(pr);
    }

    return Array.from(groups.values()).sort((a, b) =>
      a.repoName.localeCompare(b.repoName)
    );
  });
</script>

<div class="pr-list">
  <div class="controls-bar">
    <div class="role-filter">
      <button class:active={roleFilter === "all"} onclick={() => (roleFilter = "all")}>
        All
      </button>
      <button class:active={roleFilter === "reviewer"} onclick={() => (roleFilter = "reviewer")}>
        To Review
      </button>
      <button class:active={roleFilter === "author"} onclick={() => (roleFilter = "author")}>
        Authored
      </button>
    </div>
    <div class="view-toggle">
      <button class:active={viewMode === "grouped"} onclick={() => (viewMode = "grouped")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="8" y1="6" x2="21" y2="6" />
          <line x1="8" y1="12" x2="21" y2="12" />
          <line x1="8" y1="18" x2="21" y2="18" />
          <line x1="3" y1="6" x2="3.01" y2="6" />
          <line x1="3" y1="12" x2="3.01" y2="12" />
          <line x1="3" y1="18" x2="3.01" y2="18" />
        </svg>
      </button>
      <button class:active={viewMode === "flat"} onclick={() => (viewMode = "flat")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <rect x="3" y="3" width="7" height="7" />
          <rect x="14" y="3" width="7" height="7" />
          <rect x="14" y="14" width="7" height="7" />
          <rect x="3" y="14" width="7" height="7" />
        </svg>
      </button>
    </div>
  </div>

  {#if $prsLoading}
    <div class="state-panel">
      <div class="spinner"></div>
      <p>Loading PR reviews...</p>
    </div>
  {:else if $prsError}
    <div class="state-panel error">
      <p>Failed to load PRs</p>
      <p class="detail">{$prsError}</p>
    </div>
  {:else if filteredPRs.length === 0}
    <div class="state-panel">
      <p>{roleFilter === "reviewer" ? "No PRs waiting for your review" : roleFilter === "author" ? "No PRs authored by you" : "No pull requests found"}</p>
    </div>
  {:else if viewMode === "flat"}
    <div class="pr-grid">
      {#each filteredPRs as pr, i (pr.id)}
        <div style="animation: fadeInUp 0.3s var(--ease-out) {i * 40}ms both">
          <PRCard {pr} {onSchedule} {onLinkJira} />
        </div>
      {/each}
    </div>
  {:else}
    <div class="repo-groups">
      {#each groupedByRepo as group, gi (group.repoName)}
        <div class="repo-group" style="animation: fadeInUp 0.3s var(--ease-out) {gi * 60}ms both">
          <div class="repo-header">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
            </svg>
            <span class="repo-name">{group.repoName}</span>
            <span class="repo-full-name">{group.repoFullName}</span>
            <span class="pr-count">{group.prs.length}</span>
          </div>
          <div class="repo-prs">
            {#each group.prs as pr (pr.id)}
              <PRCard {pr} {onSchedule} {onLinkJira} compact={true} />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pr-list {
    width: 100%;
  }

  .controls-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .role-filter {
    display: flex;
    gap: 2px;
  }

  .role-filter button {
    padding: 6px 12px;
    border: none;
    background: transparent;
    border-radius: 7px;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .role-filter button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .role-filter button.active {
    color: var(--text-primary);
  }

  .view-toggle {
    display: flex;
    gap: 2px;
  }

  .view-toggle button {
    padding: 6px 10px;
    border: none;
    background: transparent;
    border-radius: 7px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    display: flex;
    align-items: center;
  }

  .view-toggle button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .view-toggle button.active {
    color: var(--text-primary);
  }

  .pr-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 12px;
  }

  .repo-groups {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .repo-group {
    overflow: hidden;
  }

  .repo-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 4px;
    color: var(--accent-purple);
  }

  .repo-name {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .repo-full-name {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .pr-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    background: var(--bg-hover);
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }

  .repo-prs {
    display: flex;
    flex-direction: column;
  }

  .state-panel {
    text-align: center;
    padding: 48px 24px;
    color: var(--text-secondary);
  }

  .state-panel.error {
    color: var(--accent-red);
  }

  .state-panel .detail {
    font-size: 13px;
    color: var(--text-tertiary);
    margin-top: 4px;
  }

  .state-panel .spinner {
    width: 28px;
    height: 28px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-purple);
    border-radius: 50%;
    margin: 0 auto 14px;
    animation: spin 0.8s linear infinite;
  }
</style>
