<script lang="ts">
  import Settings from "../lib/components/Settings.svelte";
  import Notifications from "../lib/components/Notifications.svelte";
</script>

<div class="settings-page">
  <Notifications />

  <h1>Settings</h1>
  <p class="description">Configure your Jira connection and sync preferences.</p>

  <Settings />
</div>

<style>
  .settings-page {
    max-width: 800px;
    margin: 0 auto;
  }

  h1 {
    font-size: 28px;
    font-weight: 600;
    margin: 0 0 8px;
    color: #1d1d1f;
  }

  .description {
    color: #86868b;
    margin: 0 0 24px;
  }
</style>
