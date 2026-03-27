<script lang="ts">
  import Header from "./lib/components/Header.svelte";
  import ChatWidget from "./lib/components/ChatWidget.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import SettingsPage from "./routes/SettingsPage.svelte";
  import About from "./routes/About.svelte";

  let currentRoute = $state<"dashboard" | "settings" | "about">("dashboard");

  function navigate(route: "dashboard" | "settings" | "about") {
    currentRoute = route;
  }
</script>

<main>
  <Header {currentRoute} onNavigate={navigate} />

  <div class="content">
    {#if currentRoute === "dashboard"}
      <Dashboard />
    {:else if currentRoute === "settings"}
      <SettingsPage />
    {:else if currentRoute === "about"}
      <About />
    {/if}
  </div>

  <ChatWidget repoPath="." />
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif;
    background-color: #f5f5f7;
    color: #1d1d1f;
  }

  :global(*) {
    box-sizing: border-box;
  }

  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .content {
    flex: 1;
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }
</style>
