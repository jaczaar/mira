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
        Grouped
      </button>
      <button class:active={viewMode === "flat"} onclick={() => (viewMode = "flat")}>
        Flat
      </button>
    </div>
  </div>

  {#if $prsLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading PR reviews...</p>
    </div>
  {:else if $prsError}
    <div class="error">
      <p>Failed to load PRs</p>
      <p class="error-detail">{$prsError}</p>
    </div>
  {:else if filteredPRs.length === 0}
    <div class="empty">
      <p>{roleFilter === "reviewer" ? "No PRs waiting for your review" : roleFilter === "author" ? "No PRs authored by you" : "No pull requests found"}</p>
    </div>
  {:else if viewMode === "flat"}
    <div class="pr-grid">
      {#each filteredPRs as pr (pr.id)}
        <PRCard {pr} {onSchedule} {onLinkJira} />
      {/each}
    </div>
  {:else}
    <div class="repo-groups">
      {#each groupedByRepo as group (group.repoName)}
        <div class="repo-group">
          <div class="repo-header">
            <span class="repo-name">{group.repoName}</span>
            <span class="repo-full-name">{group.repoFullName}</span>
            <span class="pr-count">{group.prs.length} {group.prs.length === 1 ? "PR" : "PRs"}</span>
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
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e5e5e5;
  }

  .role-filter {
    display: flex;
    gap: 4px;
  }

  .role-filter button {
    padding: 8px 16px;
    border: 1px solid #d2d2d7;
    background: white;
    border-radius: 20px;
    font-size: 13px;
    color: #1d1d1f;
    cursor: pointer;
    transition: all 0.2s;
  }

  .role-filter button:hover {
    background: #f5f5f7;
  }

  .role-filter button.active {
    background: #1d1d1f;
    color: white;
    border-color: #1d1d1f;
  }

  .view-toggle {
    display: flex;
    gap: 4px;
  }

  .view-toggle button {
    padding: 8px 16px;
    border: 1px solid #d2d2d7;
    background: white;
    border-radius: 20px;
    font-size: 13px;
    color: #1d1d1f;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-toggle button:hover {
    background: #f5f5f7;
  }

  .view-toggle button.active {
    background: #1d1d1f;
    color: white;
    border-color: #1d1d1f;
  }

  .pr-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 16px;
  }

  .repo-groups {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .repo-group {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .repo-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    background: linear-gradient(135deg, #8b5cf6 0%, #6d28d9 100%);
    color: white;
  }

  .repo-name {
    font-weight: 600;
    font-size: 14px;
    background: rgba(255, 255, 255, 0.2);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .repo-full-name {
    flex: 1;
    font-size: 13px;
    opacity: 0.8;
  }

  .pr-count {
    font-size: 12px;
    opacity: 0.8;
  }

  .repo-prs {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: #f0f0f0;
  }

  .loading,
  .error,
  .empty {
    text-align: center;
    padding: 40px;
    background: white;
    border-radius: 12px;
    color: #86868b;
  }

  .loading .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #e5e5e5;
    border-top-color: #8b5cf6;
    border-radius: 50%;
    margin: 0 auto 16px;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    background: #ffebea;
    color: #ff3b30;
  }

  .error-detail {
    font-size: 13px;
    opacity: 0.8;
  }
</style>
