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
    background: #fafafa;
    border: 1px solid #e5e5e5;
    border-radius: 10px;
    overflow: hidden;
    font-size: 13px;
  }

  .form-header {
    padding: 10px 12px;
    background: #f5f5f7;
    border-bottom: 1px solid #e5e5e5;
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
    font-size: 12px;
    font-weight: 500;
    color: #86868b;
  }

  input, textarea {
    border: 1px solid #d2d2d7;
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }

  input:focus, textarea:focus {
    border-color: #0071e3;
  }

  textarea {
    resize: none;
  }

  .form-actions {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid #e5e5e5;
  }

  .btn-submit {
    flex: 1;
    padding: 6px 12px;
    background: #0071e3;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }

  .btn-submit:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-cancel {
    padding: 6px 12px;
    background: none;
    color: #86868b;
    border: 1px solid #d2d2d7;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
  }

  .btn-cancel:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
