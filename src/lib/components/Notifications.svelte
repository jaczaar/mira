<script lang="ts">
  import { syncState } from "../stores/sync";
  import { configError } from "../stores/config";
  import { tasksError } from "../stores/tasks";

  let dismissed = $state<string[]>([]);

  interface Notification {
    id: string;
    type: "success" | "error" | "warning" | "info";
    message: string;
  }

  const notifications = $derived.by(() => {
    const items: Notification[] = [];

    if ($syncState.status === "success" && $syncState.message && !dismissed.includes("sync-success")) {
      items.push({
        id: "sync-success",
        type: "success",
        message: $syncState.message,
      });
    }

    if ($syncState.status === "error" && $syncState.message && !dismissed.includes("sync-error")) {
      items.push({
        id: "sync-error",
        type: "error",
        message: $syncState.message,
      });
    }

    if ($configError && !dismissed.includes("config-error")) {
      items.push({
        id: "config-error",
        type: "error",
        message: $configError,
      });
    }

    if ($tasksError && !dismissed.includes("tasks-error")) {
      items.push({
        id: "tasks-error",
        type: "error",
        message: $tasksError,
      });
    }

    return items;
  });

  function dismiss(id: string) {
    dismissed = [...dismissed, id];
    // Auto-clear after some time
    setTimeout(() => {
      dismissed = dismissed.filter((d) => d !== id);
    }, 30000);
  }
</script>

{#if notifications.length > 0}
  <div class="notifications">
    {#each notifications as notification (notification.id)}
      <div class="notification {notification.type}">
        <span class="message">{notification.message}</span>
        <button class="dismiss" onclick={() => dismiss(notification.id)}>×</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .notifications {
    position: fixed;
    top: 80px;
    right: 20px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 400px;
  }

  .notification {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .notification.success {
    background: #e8f8ec;
    color: #1a7f37;
    border: 1px solid #34c759;
  }

  .notification.error {
    background: #ffebea;
    color: #cf222e;
    border: 1px solid #ff3b30;
  }

  .notification.warning {
    background: #fff9e6;
    color: #9a6700;
    border: 1px solid #ff9500;
  }

  .notification.info {
    background: #e8f4fd;
    color: #0969da;
    border: 1px solid #0071e3;
  }

  .message {
    font-size: 13px;
    line-height: 1.4;
  }

  .dismiss {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    opacity: 0.6;
    padding: 0;
    line-height: 1;
  }

  .dismiss:hover {
    opacity: 1;
  }
</style>
