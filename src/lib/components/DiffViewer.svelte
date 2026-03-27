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
        return "#34c759";
      case "deleted":
        return "#ff3b30";
      case "modified":
        return "#ff9500";
      default:
        return "#86868b";
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
    background: #fafafa;
    border: 1px solid #e5e5e5;
    border-radius: 10px;
    overflow: hidden;
    font-size: 12px;
  }

  .diff-header {
    padding: 10px 12px;
    background: #f5f5f7;
    border-bottom: 1px solid #e5e5e5;
  }

  .diff-header strong {
    display: block;
    font-size: 13px;
    margin-bottom: 2px;
  }

  .diff-summary {
    color: #86868b;
    font-size: 11px;
  }

  .diff-files {
    max-height: 200px;
    overflow-y: auto;
  }

  .diff-file {
    border-bottom: 1px solid #f0f0f0;
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
  }

  .file-header:hover {
    background: #f0f0f2;
  }

  .status-icon {
    font-weight: 700;
    font-family: monospace;
    width: 14px;
    text-align: center;
  }

  .file-path {
    flex: 1;
    font-family: monospace;
    color: #1d1d1f;
  }

  .expand-icon {
    color: #86868b;
    font-size: 10px;
  }

  .diff-content {
    margin: 0;
    padding: 8px 12px;
    background: #1d1d1f;
    font-family: monospace;
    font-size: 11px;
    line-height: 1.4;
    overflow-x: auto;
    max-height: 150px;
    overflow-y: auto;
  }

  .diff-add {
    color: #34c759;
  }

  .diff-remove {
    color: #ff6b6b;
  }

  .diff-context {
    color: #86868b;
  }

  .diff-actions {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid #e5e5e5;
  }

  .btn-approve {
    flex: 1;
    padding: 6px 12px;
    background: #34c759;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }

  .btn-approve:hover {
    background: #2db84d;
  }

  .btn-discard {
    padding: 6px 12px;
    background: none;
    color: #ff3b30;
    border: 1px solid #ff3b30;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
  }

  .btn-discard:hover {
    background: #fff3f3;
  }
</style>
