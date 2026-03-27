<script lang="ts">
  import type { SyncStatus } from "../stores/sync";

  interface Props {
    status: SyncStatus;
    message?: string | null;
  }

  let { status, message = null }: Props = $props();

  const statusConfig = $derived({
    idle: { label: "Ready", color: "var(--text-tertiary)", bg: "var(--bg-elevated)", dot: "var(--text-tertiary)" },
    syncing: { label: "Syncing", color: "var(--accent-blue)", bg: "var(--accent-blue-dim)", dot: "var(--accent-blue)" },
    success: { label: "Synced", color: "var(--accent-green)", bg: "var(--accent-green-dim)", dot: "var(--accent-green)" },
    error: { label: "Error", color: "var(--accent-red)", bg: "var(--accent-red-dim)", dot: "var(--accent-red)" },
  }[status]);
</script>

<div
  class="badge"
  class:syncing={status === "syncing"}
  style="--badge-color: {statusConfig.color}; --badge-bg: {statusConfig.bg}; --badge-dot: {statusConfig.dot}"
  title={message || statusConfig.label}
>
  <span class="dot"></span>
  <span class="label">{statusConfig.label}</span>
</div>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: var(--radius-full);
    background: var(--badge-bg);
    color: var(--badge-color);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
    border: 1px solid rgba(255, 255, 255, 0.04);
    transition: all 0.3s var(--ease-out);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--badge-dot);
    transition: background 0.3s;
  }

  .badge.syncing .dot {
    animation: pulse 1.2s ease-in-out infinite;
  }
</style>
