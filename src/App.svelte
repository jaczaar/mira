<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Header from "./lib/components/Header.svelte";
  import ChatWidget from "./lib/components/ChatWidget.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import Calendar from "./routes/Calendar.svelte";
  import SettingsPage from "./routes/SettingsPage.svelte";
  import About from "./routes/About.svelte";

  type Route = "dashboard" | "calendar" | "settings" | "about";
  let currentRoute = $state<Route>("calendar");

  function navigate(route: Route) {
    currentRoute = route;
  }

  onMount(() => {
    const unlisten = listen<string>("menu-navigate", (event) => {
      navigate(event.payload as Route);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<main>
  <div class="content">
    {#if currentRoute === "dashboard"}
      <Dashboard />
    {:else if currentRoute === "calendar"}
      <Calendar />
    {:else if currentRoute === "settings"}
      <SettingsPage />
    {:else if currentRoute === "about"}
      <About />
    {/if}
  </div>

  <Header {currentRoute} onNavigate={navigate} />
  <ChatWidget repoPath="." />
</main>

<style>
  :global(:root) {
    --bg-base: #202025;
    --bg-surface: #28282f;
    --bg-elevated: #31313a;
    --bg-hover: #3a3a44;
    --bg-active: #44444f;

    --border-subtle: rgba(255, 255, 255, 0.06);
    --border-default: rgba(255, 255, 255, 0.1);
    --border-strong: rgba(255, 255, 255, 0.16);

    --text-primary: #e0e0e4;
    --text-secondary: #9a9aa0;
    --text-tertiary: #6e6e76;
    --text-inverse: #121214;

    --accent-blue: #7cacf8;
    --accent-blue-dim: rgba(124, 172, 248, 0.12);
    --accent-blue-glow: rgba(124, 172, 248, 0.2);
    --accent-purple: #b89eff;
    --accent-purple-dim: rgba(184, 158, 255, 0.12);
    --accent-purple-glow: rgba(184, 158, 255, 0.2);
    --accent-green: #6ee7a0;
    --accent-green-dim: rgba(110, 231, 160, 0.1);
    --accent-amber: #f5d06b;
    --accent-amber-dim: rgba(245, 208, 107, 0.1);
    --accent-red: #f09090;
    --accent-red-dim: rgba(240, 144, 144, 0.1);

    --gradient-brand: linear-gradient(135deg, #7cacf8 0%, #b89eff 100%);
    --gradient-surface: linear-gradient(180deg, rgba(255,255,255,0.02) 0%, transparent 100%);

    --font-display: 'Outfit', sans-serif;
    --font-body: 'DM Sans', sans-serif;
    --font-mono: 'JetBrains Mono', monospace;

    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 14px;
    --radius-xl: 20px;
    --radius-full: 9999px;

    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.25);
    --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.35);
    --shadow-glow-blue: 0 0 24px rgba(124, 172, 248, 0.1);
    --shadow-glow-purple: 0 0 24px rgba(184, 158, 255, 0.1);

    --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
    --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    font-family: var(--font-body);
    background-color: var(--bg-base);
    color: var(--text-primary);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(::selection) {
    background: var(--accent-blue-dim);
    color: var(--text-primary);
  }

  :global(::-webkit-scrollbar) {
    width: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: var(--border-strong);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: var(--text-tertiary);
  }

  @keyframes -global-fadeInUp {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes -global-fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes -global-spin {
    to { transform: rotate(360deg); }
  }

  @keyframes -global-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @keyframes -global-slideInRight {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .content {
    flex: 1;
    padding: 16px 3% 60px;
    width: 100%;
    overflow-y: auto;
    animation: fadeInUp 0.4s var(--ease-out);
  }
</style>
