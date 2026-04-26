<script lang="ts">
  import Settings from "../lib/components/Settings.svelte";
  import Notifications from "../lib/components/Notifications.svelte";

  type AppRoute = "calendar" | "about" | "edit" | "settings";
  interface Props {
    onNavigate?: (route: AppRoute) => void;
  }
  let { onNavigate }: Props = $props();
</script>

<div class="settings-page">
  <Notifications />

  <button class="back-btn" onclick={() => onNavigate?.("calendar")} title="Back to Calendar">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="15 18 9 12 15 6" />
    </svg>
    <span>Calendar</span>
  </button>

  <div class="page-header">
    <h1>Settings</h1>
    <p class="description">Configure your integrations and sync preferences.</p>
  </div>

  <Settings />
</div>

<style>
  .settings-page {
    max-width: 720px;
    margin: 0 auto;
  }

  .back-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px 0 8px;
    margin-bottom: 20px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-secondary);
    border-radius: var(--radius-md);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s var(--ease-out), color 0.15s var(--ease-out), border-color 0.15s var(--ease-out);
  }

  .back-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-default);
  }

  .page-header {
    margin-bottom: 24px;
  }

  h1 {
    font-family: var(--font-display);
    font-size: 26px;
    font-weight: 700;
    margin: 0 0 4px;
    color: var(--text-primary);
    letter-spacing: -0.03em;
  }

  .description {
    color: var(--text-tertiary);
    margin: 0;
    font-size: 14px;
  }
</style>
