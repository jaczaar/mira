<script lang="ts">
  import type { SyncStatus } from "../stores/sync";

  interface Props {
    status: SyncStatus;
    message?: string | null;
  }

  let { status, message = null }: Props = $props();

  const statusConfig = $derived({
    idle: { label: "Ready", color: "#86868b", bg: "#f5f5f7" },
    syncing: { label: "Syncing...", color: "#0071e3", bg: "#e8f4fd" },
    success: { label: "Synced", color: "#34c759", bg: "#e8f8ec" },
    error: { label: "Error", color: "#ff3b30", bg: "#ffebea" },
  }[status]);
</script>

<div
  class="badge"
  style="--badge-color: {statusConfig.color}; --badge-bg: {statusConfig.bg}"
  title={message || statusConfig.label}
>
  {#if status === "syncing"}
    <span class="spinner"></span>
  {/if}
  <span class="label">{statusConfig.label}</span>
</div>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 12px;
    background: var(--badge-bg);
    color: var(--badge-color);
    font-size: 12px;
    font-weight: 500;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--badge-color);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
