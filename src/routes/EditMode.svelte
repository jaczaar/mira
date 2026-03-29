<script lang="ts">
  import { onMount } from "svelte";
  import ChatWidget from "../lib/components/ChatWidget.svelte";
  import SetupWizard from "../lib/components/SetupWizard.svelte";
  import {
    setupStep,
    workspacePath,
    vitePort,
    checkWorkspaceStatus,
    runSetup,
    startVite,
  } from "../lib/stores/workspace";

  let splitPercent = $state(50);
  let dragging = $state(false);
  let containerEl: HTMLDivElement | undefined = $state();

  onMount(async () => {
    if ($vitePort && $setupStep === "READY") {
      return;
    }

    await checkWorkspaceStatus();

    if ($setupStep !== "NEED_CLAUDE" && $setupStep !== "READY" && $setupStep !== "CHECKING") {
      await runSetup();
    }

    if ($setupStep === "READY" && !$vitePort) {
      await startVite();
    }
  });

  $effect(() => {
    if ($setupStep === "READY" && !$vitePort) {
      startVite();
    }
  });

  function onDragStart(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
    document.addEventListener("mousemove", onDragMove);
    document.addEventListener("mouseup", onDragEnd);
  }

  function onDragMove(e: MouseEvent) {
    if (!dragging || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    const pct = ((e.clientX - rect.left) / rect.width) * 100;
    splitPercent = Math.min(80, Math.max(20, pct));
  }

  function onDragEnd() {
    dragging = false;
    document.removeEventListener("mousemove", onDragMove);
    document.removeEventListener("mouseup", onDragEnd);
  }

  function openExternal() {
    if ($vitePort) {
      window.open(`http://localhost:${$vitePort}`, "_blank");
    }
  }
</script>

<div class="edit-mode">
  {#if $setupStep !== "READY"}
    <SetupWizard />
  {:else}
    <div class="split" class:dragging bind:this={containerEl} style="--split: {splitPercent}%">
      <div class="chat-pane">
        {#if $workspacePath}
          <ChatWidget repoPath={$workspacePath} embedded />
        {/if}
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="divider" onmousedown={onDragStart}>
        <div class="divider-handle"></div>
      </div>
      <div class="preview-pane">
        {#if $vitePort}
          <div class="preview-header">
            <span class="preview-label">Preview</span>
            <button class="external-btn" onclick={openExternal} title="Open in browser">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" />
              </svg>
            </button>
          </div>
          <iframe
            src={`http://localhost:${$vitePort}`}
            title="Preview"
            class="preview-frame"
          ></iframe>
        {:else}
          <div class="preview-placeholder">
            Starting preview server...
          </div>
        {/if}
      </div>
    </div>
    {#if dragging}
      <div class="drag-overlay"></div>
    {/if}
  {/if}
</div>

<style>
  .edit-mode {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .split {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .chat-pane {
    width: var(--split);
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .divider {
    width: 5px;
    flex-shrink: 0;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    transition: background 0.12s;
  }

  .divider:hover,
  .dragging .divider {
    background: var(--border-subtle);
  }

  .divider-handle {
    width: 3px;
    height: 32px;
    border-radius: 2px;
    background: var(--border-strong);
    opacity: 0;
    transition: opacity 0.12s;
  }

  .divider:hover .divider-handle,
  .dragging .divider-handle {
    opacity: 1;
  }

  .preview-pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 10px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .preview-label {
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .external-btn {
    display: inline-flex;
    align-items: center;
    padding: 3px 5px;
    background: none;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .external-btn:hover {
    color: var(--text-secondary);
    border-color: var(--border-default);
  }

  .preview-frame {
    width: 100%;
    flex: 1;
    border: none;
    background: var(--bg-base);
  }

  .preview-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 100%;
    color: var(--text-tertiary);
    font-family: var(--font-body);
    font-size: 13px;
  }

  .drag-overlay {
    position: absolute;
    inset: 0;
    z-index: 100;
    cursor: col-resize;
  }
</style>
