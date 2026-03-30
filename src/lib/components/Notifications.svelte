<script lang="ts">
  import { syncState } from "../stores/sync";
  import { configError } from "../stores/config";
  import { tasksError } from "../stores/tasks";

  let dismissed = $state<string[]>([]);
  let expandedIds = $state<string[]>([]);

  interface Notification {
    id: string;
    type: "success" | "error" | "warning" | "info";
    message: string;
    details?: string[];
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
        details: $syncState.errors.length > 0 ? $syncState.errors : undefined,
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
    expandedIds = expandedIds.filter((e) => e !== id);
    setTimeout(() => {
      dismissed = dismissed.filter((d) => d !== id);
    }, 30000);
  }

  function toggleExpand(id: string) {
    if (expandedIds.includes(id)) {
      expandedIds = expandedIds.filter((e) => e !== id);
    } else {
      expandedIds = [...expandedIds, id];
    }
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
        <div class="notification-main">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d={getIcon(notification.type)} />
          </svg>
          <span class="message">{notification.message}</span>
          {#if notification.details && notification.details.length > 0}
            <button class="expand-btn" onclick={() => toggleExpand(notification.id)} title="Show details">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" style="transition: transform 0.15s; transform: rotate({expandedIds.includes(notification.id) ? '180deg' : '0deg'})">
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
          {/if}
          <button class="dismiss" onclick={() => dismiss(notification.id)} title="Dismiss">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        {#if notification.details && expandedIds.includes(notification.id)}
          <div class="details">
            {#each notification.details as detail}
              <div class="detail-line">{detail}</div>
            {/each}
          </div>
        {/if}
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
    max-width: 420px;
  }

  .notification {
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    box-shadow: var(--shadow-lg);
    animation: slideInRight 0.4s var(--ease-out) both;
    backdrop-filter: blur(12px);
    overflow: hidden;
  }

  .notification-main {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
  }

  .notification-main svg {
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

  .expand-btn {
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

  .expand-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
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

  .details {
    padding: 0 14px 10px;
    border-top: 1px solid var(--border-subtle);
    margin-top: 0;
    animation: fadeIn 0.15s var(--ease-out);
  }

  .detail-line {
    padding: 4px 0;
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-subtle);
  }

  .detail-line:last-child {
    border-bottom: none;
  }
</style>
