<script lang="ts">
  interface Props {
    currentRoute: "dashboard" | "calendar" | "about" | "edit" | "settings";
    onNavigate: (route: "dashboard" | "calendar" | "about" | "edit" | "settings") => void;
  }

  let { currentRoute, onNavigate }: Props = $props();

  const navItems = [
    { route: "calendar" as const, label: "Calendar", icon: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" },
    { route: "dashboard" as const, label: "Dashboard", icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" },
    { route: "edit" as const, label: "Edit", icon: "M16 18L22 12 16 6M8 6L2 12 8 18" },
    { route: "settings" as const, label: "Settings", icon: "M12 15a3 3 0 100-6 3 3 0 000 6zM19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" },
  ];
</script>

<nav>
  {#each navItems as item}
    <button
      class:active={currentRoute === item.route}
      onclick={() => onNavigate(item.route)}
      title={item.label}
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d={item.icon} />
      </svg>
      {#if currentRoute === item.route}
        <span class="label">{item.label}</span>
      {/if}
    </button>
  {/each}
</nav>

<style>
  nav {
    position: fixed;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px;
    background: color-mix(in srgb, var(--bg-base) 85%, transparent);
    backdrop-filter: blur(24px) saturate(180%);
    -webkit-backdrop-filter: blur(24px) saturate(180%);
    border: 1px solid var(--border-default);
    border-radius: 12px;
    box-shadow: var(--header-shadow);
  }

  button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border-radius: 9px;
    transition: all 0.2s var(--ease-out);
    white-space: nowrap;
  }

  button svg {
    transition: all 0.2s var(--ease-out);
    flex-shrink: 0;
  }

  button:hover {
    color: var(--text-secondary);
    background: var(--header-hover);
  }

  button.active {
    color: var(--text-primary);
    background: var(--header-active);
  }

  button.active svg {
    stroke: var(--accent-blue);
  }

  .label {
    animation: fadeIn 0.2s var(--ease-out);
  }
</style>
