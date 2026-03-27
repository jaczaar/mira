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
    setTimeout(() => {
      dismissed = dismissed.filter((d) => d !== id);
    }, 30000);
  }

  function getIcon(type: string): string {
    switch (type) {
      case "success": return "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z";
      case "error": return "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
      case "warning": return "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z";
      default: return "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
    }
  }
</script>

{#if notifications.length > 0}
  <div class="notifications">
    {#each notifications as notification, i (notification.id)}
      <div
        class="notification {notification.type}"
        style="animation-delay: {i * 60}ms"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d={getIcon(notification.type)} />
        </svg>
        <span class="message">{notification.message}</span>
        <button class="dismiss" onclick={() => dismiss(notification.id)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .notifications {
    position: fixed;
    top: 64px;
    right: 20px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 380px;
  }

  .notification {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    box-shadow: var(--shadow-lg);
    animation: slideInRight 0.4s var(--ease-out) both;
    backdrop-filter: blur(12px);
  }

  .notification svg {
    flex-shrink: 0;
  }

  .notification.success {
    border-color: rgba(74, 222, 128, 0.2);
    color: var(--accent-green);
  }

  .notification.error {
    border-color: rgba(248, 113, 113, 0.2);
    color: var(--accent-red);
  }

  .notification.warning {
    border-color: rgba(251, 191, 36, 0.2);
    color: var(--accent-amber);
  }

  .notification.info {
    border-color: rgba(91, 141, 239, 0.2);
    color: var(--accent-blue);
  }

  .message {
    flex: 1;
    font-size: 13px;
    line-height: 1.4;
    color: var(--text-primary);
  }

  .dismiss {
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    color: var(--text-tertiary);
    transition: all 0.15s;
  }

  .dismiss:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }
</style>
