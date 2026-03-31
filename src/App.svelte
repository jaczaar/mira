<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Header from "./lib/components/Header.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import Calendar from "./routes/Calendar.svelte";
  import About from "./routes/About.svelte";
  import EditMode from "./routes/EditMode.svelte";
  import "./lib/stores/theme";

  type Route = "dashboard" | "calendar" | "about" | "edit";
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
        navigate("dashboard");
      } else if (e.key === "3") {
        e.preventDefault();
        navigate("edit");
      }
    }
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

<svelte:window onkeydown={handleKeydown} />

<main>
  <div class="content">
    {#if currentRoute === "dashboard"}
      <Dashboard />
    {:else if currentRoute === "calendar"}
      <Calendar />
    {:else if currentRoute === "about"}
      <About />
    {:else if currentRoute === "edit"}
      <EditMode />
    {/if}
  </div>
  <Header {currentRoute} onNavigate={navigate} />
</main>

<style>
  :global(:root) {
    --bg-base: #2c2c36;
    --bg-surface: #34343f;
    --bg-elevated: #3d3d4a;
    --bg-hover: #474754;
    --bg-active: #51515f;

    --border-subtle: rgba(255, 255, 255, 0.08);
    --border-default: rgba(255, 255, 255, 0.12);
    --border-strong: rgba(255, 255, 255, 0.2);

    --text-primary: #e8e8ec;
    --text-secondary: #a8a8b0;
    --text-tertiary: #8e8e98;
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

    --header-shadow: 0 8px 40px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.04);
    --header-hover: rgba(255, 255, 255, 0.06);
    --header-active: rgba(255, 255, 255, 0.1);
    --today-tint: rgba(124, 172, 248, 0.03);
    --today-tint-strong: rgba(124, 172, 248, 0.02);
    --day-header-bg: rgba(255, 255, 255, 0.01);
  }

  :global(:root.light) {
      --bg-base: #f0f1f4;
      --bg-surface: rgba(255, 255, 255, 0.82);
      --bg-elevated: #ffffff;
      --bg-hover: rgba(0, 0, 0, 0.05);
      --bg-active: rgba(0, 0, 0, 0.08);

      --border-subtle: rgba(0, 0, 0, 0.07);
      --border-default: rgba(0, 0, 0, 0.12);
      --border-strong: rgba(0, 0, 0, 0.18);

      --text-primary: #1a1a1e;
      --text-secondary: #4a4a54;
      --text-tertiary: #71717a;
      --text-inverse: #f0f0f2;

      --accent-blue: #3b7de9;
      --accent-blue-dim: rgba(59, 125, 233, 0.1);
      --accent-blue-glow: rgba(59, 125, 233, 0.16);
      --accent-purple: #8b6cdf;
      --accent-purple-dim: rgba(139, 108, 223, 0.1);
      --accent-purple-glow: rgba(139, 108, 223, 0.16);
      --accent-green: #2da562;
      --accent-green-dim: rgba(45, 165, 98, 0.1);
      --accent-amber: #c49a20;
      --accent-amber-dim: rgba(196, 154, 32, 0.1);
      --accent-red: #d44848;
      --accent-red-dim: rgba(212, 72, 72, 0.08);

      --gradient-brand: linear-gradient(135deg, #3b7de9 0%, #8b6cdf 100%);
      --gradient-surface: linear-gradient(180deg, rgba(0, 0, 0, 0.01) 0%, transparent 100%);

      --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.06);
      --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.08);
      --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.12);
      --shadow-glow-blue: 0 0 24px rgba(59, 125, 233, 0.08);
      --shadow-glow-purple: 0 0 24px rgba(139, 108, 223, 0.08);

      --header-shadow: 0 8px 40px rgba(0, 0, 0, 0.08), 0 0 0 1px rgba(0, 0, 0, 0.06);
      --header-hover: rgba(0, 0, 0, 0.05);
      --header-active: rgba(0, 0, 0, 0.08);
      --today-tint: rgba(59, 125, 233, 0.05);
      --today-tint-strong: rgba(59, 125, 233, 0.03);
      --day-header-bg: rgba(0, 0, 0, 0.01);
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
    padding: 48px 4% 24px;
    width: 100%;
    overflow-y: auto;
    animation: fadeInUp 0.4s var(--ease-out);
  }
</style>
