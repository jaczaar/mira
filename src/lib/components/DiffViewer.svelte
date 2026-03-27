<script lang="ts">
  import type { ChangeDiff } from "../api";

  interface Props {
    diff: ChangeDiff;
    onCreatePR: () => void;
    onDiscard: () => void;
  }

  let { diff, onCreatePR, onDiscard }: Props = $props();

  let expandedFiles = $state<Set<string>>(new Set());

  function toggleFile(path: string) {
    const next = new Set(expandedFiles);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedFiles = next;
  }

  function statusIcon(status: string): string {
    switch (status) {
      case "added":
      case "untracked":
        return "+";
      case "deleted":
        return "-";
      case "modified":
        return "~";
      default:
        return "?";
    }
  }

  function statusColor(status: string): string {
    switch (status) {
      case "added":
      case "untracked":
        return "var(--accent-green)";
      case "deleted":
        return "var(--accent-red)";
      case "modified":
        return "var(--accent-amber)";
      default:
        return "var(--text-tertiary)";
    }
  }
</script>

<div class="diff-viewer">
  <div class="diff-header">
    <strong>Changes detected</strong>
    <span class="diff-summary">{diff.summary}</span>
  </div>

  <div class="diff-files">
    {#each diff.files as file (file.path)}
      <div class="diff-file">
        <button class="file-header" onclick={() => toggleFile(file.path)}>
          <span class="status-icon" style="color: {statusColor(file.status)}">{statusIcon(file.status)}</span>
          <span class="file-path">{file.path}</span>
          <span class="expand-icon">{expandedFiles.has(file.path) ? "▼" : "▶"}</span>
        </button>
        {#if expandedFiles.has(file.path) && file.diff}
          <pre class="diff-content">{#each file.diff.split('\n') as line}{#if line.startsWith('+') && !line.startsWith('+++')}<span class="diff-add">{line}</span>
{:else if line.startsWith('-') && !line.startsWith('---')}<span class="diff-remove">{line}</span>
{:else}<span class="diff-context">{line}</span>
{/if}{/each}</pre>
        {/if}
      </div>
    {/each}
  </div>

  <div class="diff-actions">
    <button class="btn-approve" onclick={onCreatePR}>Create PR</button>
    <button class="btn-discard" onclick={onDiscard}>Discard</button>
  </div>
</div>

<style>
  .diff-viewer {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-size: 12px;
  }

  .diff-header {
    padding: 10px 12px;
    background: var(--bg-hover);
    border-bottom: 1px solid var(--border-subtle);
  }

  .diff-header strong {
    display: block;
    font-size: 13px;
    margin-bottom: 2px;
    color: var(--text-primary);
  }

  .diff-summary {
    color: var(--text-tertiary);
    font-size: 11px;
  }

  .diff-files {
    max-height: 200px;
    overflow-y: auto;
  }

  .diff-file {
    border-bottom: 1px solid var(--border-subtle);
  }

  .file-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    color: var(--text-primary);
    transition: background 0.1s;
  }

  .file-header:hover {
    background: var(--bg-hover);
  }

  .status-icon {
    font-weight: 700;
    font-family: var(--font-mono);
    width: 14px;
    text-align: center;
  }

  .file-path {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .expand-icon {
    color: var(--text-tertiary);
    font-size: 10px;
  }

  .diff-content {
    margin: 0;
    padding: 8px 12px;
    background: var(--bg-base);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.5;
    overflow-x: auto;
    max-height: 150px;
    overflow-y: auto;
  }

  .diff-add {
    color: var(--accent-green);
  }

  .diff-remove {
    color: var(--accent-red);
  }

  .diff-context {
    color: var(--text-tertiary);
  }

  .diff-actions {
    display: flex;
    gap: 6px;
    padding: 10px 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .btn-approve {
    flex: 1;
    padding: 6px 12px;
    background: var(--accent-green-dim);
    color: var(--accent-green);
    border: 1px solid rgba(74, 222, 128, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 600;
    transition: all 0.15s;
  }

  .btn-approve:hover {
    background: rgba(74, 222, 128, 0.2);
  }

  .btn-discard {
    padding: 6px 12px;
    background: transparent;
    color: var(--accent-red);
    border: 1px solid rgba(248, 113, 113, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    transition: all 0.15s;
  }

  .btn-discard:hover {
    background: var(--accent-red-dim);
  }
</style>
