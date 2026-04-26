<script lang="ts">
  import { prSubmitting } from "../stores/chat";

  interface Props {
    onSubmit: (title: string, body: string) => void;
    onCancel: () => void;
  }

  let { onSubmit, onCancel }: Props = $props();

  let title = $state("");
  let body = $state("");

  function handleSubmit() {
    if (!title.trim()) return;
    onSubmit(title.trim(), body.trim());
  }
</script>

<div class="pr-form">
  <div class="form-header">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-purple)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="18" cy="18" r="3" />
      <circle cx="6" cy="6" r="3" />
      <path d="M13 6h3a2 2 0 0 1 2 2v7" />
      <line x1="6" y1="9" x2="6" y2="21" />
    </svg>
    <strong>Create Pull Request</strong>
  </div>

  <div class="form-body">
    <label>
      <span>Title</span>
      <input
        type="text"
        bind:value={title}
        placeholder="Brief description of changes"
        disabled={$prSubmitting}
      />
    </label>

    <label>
      <span>Description</span>
      <textarea
        bind:value={body}
        placeholder="What does this PR do?"
        rows="3"
        disabled={$prSubmitting}
      ></textarea>
    </label>
  </div>

  <div class="form-actions">
    <button class="btn-submit" onclick={handleSubmit} disabled={!title.trim() || $prSubmitting}>
      {#if $prSubmitting}
        Creating PR...
      {:else}
        Submit PR
      {/if}
    </button>
    <button class="btn-cancel" onclick={onCancel} disabled={$prSubmitting}>
      Cancel
    </button>
  </div>
</div>

<style>
  .pr-form {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-size: 13px;
  }

  .form-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--bg-hover);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
  }

  .form-body {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label span {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  input, textarea {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 7px 10px;
    font-size: 13px;
    font-family: var(--font-body);
    color: var(--text-primary);
    background: var(--bg-surface);
    outline: none;
    transition: border-color 0.15s;
  }

  input::placeholder, textarea::placeholder {
    color: var(--text-tertiary);
  }

  input:focus, textarea:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  input:disabled, textarea:disabled {
    opacity: 0.5;
  }

  textarea {
    resize: none;
  }

  .form-actions {
    display: flex;
    gap: 6px;
    padding: 10px 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .btn-submit {
    flex: 1;
    padding: 6px 12px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid var(--accent-blue-dim);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 600;
    transition: all 0.15s;
  }

  .btn-submit:hover:not(:disabled) {
    background: var(--accent-blue-dim);
    box-shadow: var(--shadow-glow-blue);
  }

  .btn-submit:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-cancel {
    padding: 6px 12px;
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    transition: all 0.15s;
  }

  .btn-cancel:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .btn-cancel:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
