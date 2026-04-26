<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Calendar from "./routes/Calendar.svelte";
  import About from "./routes/About.svelte";
  import EditMode from "./routes/EditMode.svelte";
  import SettingsPage from "./routes/SettingsPage.svelte";
  import "./lib/stores/theme";

  type Route = "calendar" | "about" | "edit" | "settings";
  let currentRoute = $state<Route>("calendar");

  function navigate(route: Route) {
    currentRoute = route;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.metaKey || e.ctrlKey) {
      if (e.key === "1") {
        e.preventDefault();
        navigate("calendar");
      } else if (e.key === "2") {
        e.preventDefault();
        navigate("edit");
      } else if (e.key === ",") {
        e.preventDefault();
        navigate("settings");
      }
    }
  }

  onMount(() => {
    const unlisten = listen<string>("menu-navigate", (event) => {
      const route = event.payload as Route;
      if (route === "calendar" || route === "about" || route === "edit" || route === "settings") {
        navigate(route);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<main>
  <div class="content">
    {#if currentRoute === "calendar"}
      <Calendar onNavigate={navigate} />
    {:else if currentRoute === "about"}
      <About />
    {:else if currentRoute === "edit"}
      <EditMode onNavigate={navigate} />
    {:else if currentRoute === "settings"}
      <SettingsPage onNavigate={navigate} />
    {/if}
  </div>
</main>

<style>
  :global(:root) {
    --bg-base: #19191b;
    --bg-surface: #212123;
    --bg-elevated: #28282b;
    --bg-hover: #2f2f33;
    --bg-active: #38383d;

    --border-subtle: rgba(232, 226, 213, 0.05);
    --border-default: rgba(232, 226, 213, 0.09);
    --border-strong: rgba(232, 226, 213, 0.16);

    --text-primary: #e3ded2;
    --text-secondary: #9b978c;
    --text-tertiary: #6d6a61;
    --text-inverse: #14130f;

    --accent-blue: #8aa89a;
    --accent-blue-dim: rgba(138, 168, 154, 0.10);
    --accent-blue-glow: rgba(138, 168, 154, 0.18);
    --accent-purple: #b39db0;
    --accent-purple-dim: rgba(179, 157, 176, 0.10);
    --accent-purple-glow: rgba(179, 157, 176, 0.18);
    --accent-green: #94b497;
    --accent-green-dim: rgba(148, 180, 151, 0.09);
    --accent-amber: #d4b888;
    --accent-amber-dim: rgba(212, 184, 136, 0.09);
    --accent-red: #cf9a8c;
    --accent-red-dim: rgba(207, 154, 140, 0.09);

    --gradient-brand: linear-gradient(135deg, #8aa89a 0%, #b39db0 100%);
    --gradient-surface: linear-gradient(180deg, rgba(232, 226, 213, 0.015) 0%, transparent 100%);

    --font-display: 'Outfit', sans-serif;
    --font-body: 'DM Sans', sans-serif;
    --font-mono: 'JetBrains Mono', monospace;

    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 14px;
    --radius-xl: 20px;
    --radius-full: 9999px;

    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.22);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.28);
    --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.38);
    --shadow-glow-blue: 0 0 24px rgba(138, 168, 154, 0.10);
    --shadow-glow-purple: 0 0 24px rgba(179, 157, 176, 0.10);

    --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
    --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);

    --header-shadow: 0 8px 40px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(232, 226, 213, 0.04);
    --header-hover: rgba(232, 226, 213, 0.04);
    --header-active: rgba(232, 226, 213, 0.07);
    --today-tint: rgba(138, 168, 154, 0.035);
    --today-tint-strong: rgba(138, 168, 154, 0.022);
    --day-header-bg: rgba(232, 226, 213, 0.008);
  }

  :global(:root.light) {
      --bg-base: #f4f1ea;
      --bg-surface: rgba(253, 250, 243, 0.82);
      --bg-elevated: #fdfaf3;
      --bg-hover: rgba(45, 40, 30, 0.04);
      --bg-active: rgba(45, 40, 30, 0.07);

      --border-subtle: rgba(45, 40, 30, 0.06);
      --border-default: rgba(45, 40, 30, 0.11);
      --border-strong: rgba(45, 40, 30, 0.17);

      --text-primary: #1f1d18;
      --text-secondary: #4d4a42;
      --text-tertiary: #7a766c;
      --text-inverse: #efece4;

      --accent-blue: #4f8473;
      --accent-blue-dim: rgba(79, 132, 115, 0.09);
      --accent-blue-glow: rgba(79, 132, 115, 0.14);
      --accent-purple: #8a6e84;
      --accent-purple-dim: rgba(138, 110, 132, 0.09);
      --accent-purple-glow: rgba(138, 110, 132, 0.14);
      --accent-green: #4f8a5e;
      --accent-green-dim: rgba(79, 138, 94, 0.09);
      --accent-amber: #a88440;
      --accent-amber-dim: rgba(168, 132, 64, 0.09);
      --accent-red: #b06450;
      --accent-red-dim: rgba(176, 100, 80, 0.08);

      --gradient-brand: linear-gradient(135deg, #4f8473 0%, #8a6e84 100%);
      --gradient-surface: linear-gradient(180deg, rgba(45, 40, 30, 0.012) 0%, transparent 100%);

      --shadow-sm: 0 1px 2px rgba(45, 40, 30, 0.06);
      --shadow-md: 0 4px 16px rgba(45, 40, 30, 0.08);
      --shadow-lg: 0 8px 32px rgba(45, 40, 30, 0.12);
      --shadow-glow-blue: 0 0 24px rgba(79, 132, 115, 0.08);
      --shadow-glow-purple: 0 0 24px rgba(138, 110, 132, 0.08);

      --header-shadow: 0 8px 40px rgba(45, 40, 30, 0.08), 0 0 0 1px rgba(45, 40, 30, 0.06);
      --header-hover: rgba(45, 40, 30, 0.05);
      --header-active: rgba(45, 40, 30, 0.08);
      --today-tint: rgba(79, 132, 115, 0.05);
      --today-tint-strong: rgba(79, 132, 115, 0.03);
      --day-header-bg: rgba(45, 40, 30, 0.012);
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
    padding: 0 24px 24px;
    width: 100%;
    overflow-y: auto;
    animation: fadeInUp 0.4s var(--ease-out);
  }
</style>
