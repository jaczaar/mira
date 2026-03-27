<script lang="ts">
  import Settings from "../lib/components/Settings.svelte";
  import Notifications from "../lib/components/Notifications.svelte";
</script>

<div class="settings-page">
  <Notifications />

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
