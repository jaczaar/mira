<script lang="ts">
  import StatusBadge from "./StatusBadge.svelte";
  import { syncState } from "../stores/sync";

  interface Props {
    currentRoute: "dashboard" | "settings" | "about";
    onNavigate: (route: "dashboard" | "settings" | "about") => void;
  }

  let { currentRoute, onNavigate }: Props = $props();
</script>

<header>
  <div class="logo">
    <h1>Mira</h1>
    <span class="subtitle">Auto-Scheduling</span>
  </div>

  <nav>
    <button
      class:active={currentRoute === "dashboard"}
      onclick={() => onNavigate("dashboard")}
    >
      Dashboard
    </button>
    <button
      class:active={currentRoute === "settings"}
      onclick={() => onNavigate("settings")}
    >
      Settings
    </button>
    <button
      class:active={currentRoute === "about"}
      onclick={() => onNavigate("about")}
    >
      About
    </button>
  </nav>

  <div class="status">
    <StatusBadge status={$syncState.status} message={$syncState.message} />
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    background: white;
    border-bottom: 1px solid #e5e5e5;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  }

  .logo {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
    color: #1d1d1f;
  }

  .subtitle {
    font-size: 12px;
    color: #86868b;
  }

  nav {
    display: flex;
    gap: 4px;
  }

  nav button {
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: #1d1d1f;
    font-size: 14px;
    cursor: pointer;
    border-radius: 6px;
    transition: background-color 0.2s;
  }

  nav button:hover {
    background: #f5f5f7;
  }

  nav button.active {
    background: #0071e3;
    color: white;
  }

  .status {
    min-width: 120px;
    text-align: right;
  }
</style>
