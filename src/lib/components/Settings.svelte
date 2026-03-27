<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import {
    config,
    configLoading,
    configError,
    hasToken,
    loadConfig,
    saveConfig,
    saveJiraToken,
    deleteJiraToken,
    testConnection,
  } from "../stores/config";
  import {
    calendars,
    calendarsLoading,
    calendarsError,
    loadCalendars,
  } from "../stores/calendar";
  import {
    googleAccount,
    googleAuthLoading,
    googleAuthError,
    loadGoogleAuthStatus,
  } from "../stores/google";
  import { googleAuthStart, googleAuthWait, googleAuthSignOut } from "../api";
  import {
    saveGitHubToken as saveGitHubTokenStore,
    deleteGitHubToken as deleteGitHubTokenStore,
    testGitHubConnection,
  } from "../stores/github";
  import * as api from "../api";

  let jiraUrl = $state("");
  let jiraEmail = $state("");
  let jiraToken = $state("");
  let googleClientId = $state("");
  let googleClientSecret = $state("");
  let selectedCalendar = $state<string | null>(null);
  let syncFrequency = $state<"manual" | "hourly" | "daily">("manual");
  let autoSyncOnLaunch = $state(false);
  let jqlFilter = $state("");
  let eventTitleTemplate = $state("[{key}] {summary}");
  let defaultEventColor = $state<string | null>(null);
  let connectionStatus = $state<"idle" | "testing" | "success" | "error">(
    "idle",
  );
  let connectionMessage = $state("");
  let showToken = $state(false);
  let googleConnectionStatus = $state<
    "idle" | "connecting" | "connected" | "error"
  >("idle");
  let googleConnectionMessage = $state("");

  // GitHub state
  let githubToken = $state("");
  let githubUsername = $state("");
  let prEventTitleTemplate = $state("[PR Review] {repo}: {title}");
  let prDefaultEventColor = $state<string | null>(null);
  let showGitHubToken = $state(false);
  let githubConnectionStatus = $state<"idle" | "testing" | "success" | "error">(
    "idle",
  );
  let githubConnectionMessage = $state("");
  let hasGitHubTokenLocal = $state(false);

  // Google Calendar color palette (colorId 1-11)
  const calendarColors = [
    { id: null, name: "Default", color: "#4285f4" },
    { id: "1", name: "Lavender", color: "#7986cb" },
    { id: "2", name: "Sage", color: "#33b679" },
    { id: "3", name: "Grape", color: "#8e24aa" },
    { id: "4", name: "Flamingo", color: "#e67c73" },
    { id: "5", name: "Banana", color: "#f6bf26" },
    { id: "6", name: "Tangerine", color: "#f4511e" },
    { id: "7", name: "Peacock", color: "#039be5" },
    { id: "8", name: "Graphite", color: "#616161" },
    { id: "9", name: "Blueberry", color: "#3f51b5" },
    { id: "10", name: "Basil", color: "#0b8043" },
    { id: "11", name: "Tomato", color: "#d50000" },
  ];

  function colorHexFor(id: string | null): string {
    const match = calendarColors.find((color) => color.id === id);
    return match?.color ?? "#4285f4";
  }

  onMount(async () => {
    await loadConfig();
    await loadGoogleAuthStatus();
    try {
      hasGitHubTokenLocal = await api.hasGitHubToken();
    } catch {
      hasGitHubTokenLocal = false;
    }
    if ($googleAccount) {
      googleConnectionStatus = "connected";
      googleConnectionMessage = `Connected as ${$googleAccount.email}`;
      await loadCalendars();
    }

    // Initialize form from config
    jiraUrl = $config.jira_url;
    jiraEmail = $config.jira_email;
    googleClientId = $config.google_client_id;
    googleClientSecret = $config.google_client_secret;
    selectedCalendar = $config.selected_calendar;
    syncFrequency = $config.sync_frequency;
    autoSyncOnLaunch = $config.auto_sync_on_launch;
    jqlFilter = $config.jql_filter || "";
    eventTitleTemplate = $config.event_title_template;
    defaultEventColor = $config.default_event_color;
    githubUsername = $config.github_username || "";
    prEventTitleTemplate =
      $config.pr_event_title_template || "[PR Review] {repo}: {title}";
    prDefaultEventColor = $config.pr_default_event_color;
  });

  async function handleSaveConfig() {
    await saveConfig({
      jira_url: jiraUrl,
      jira_email: jiraEmail,
      google_client_id: googleClientId,
      google_client_secret: googleClientSecret,
      selected_calendar: selectedCalendar,
      sync_frequency: syncFrequency,
      auto_sync_on_launch: autoSyncOnLaunch,
      jql_filter: jqlFilter || null,
      event_title_template: eventTitleTemplate,
      timezone: $config.timezone,
      default_event_color: defaultEventColor,
      github_username: githubUsername,
      pr_event_title_template: prEventTitleTemplate,
      pr_default_event_color: prDefaultEventColor,
    });
  }

  async function handleSaveToken() {
    if (jiraToken) {
      try {
        console.log("Saving token...");
        await saveJiraToken(jiraToken);
        console.log("Token saved successfully");
        jiraToken = "";
        connectionMessage = "Token saved successfully";
        connectionStatus = "success";
      } catch (error) {
        console.error("Failed to save token:", error);
        connectionStatus = "error";
        connectionMessage = `Failed to save token: ${error instanceof Error ? error.message : String(error)}`;
      }
    }
  }

  async function handleGoogleConnect() {
    googleConnectionMessage = "";
    if (!googleClientId || !googleClientSecret) {
      googleConnectionStatus = "error";
      googleConnectionMessage =
        "Please enter both Google OAuth Client ID and Client Secret first.";
      return;
    }

    googleConnectionStatus = "connecting";
    googleAuthLoading.set(true);
    try {
      const { auth_url } = await googleAuthStart();
      await open(auth_url);
      const account = await googleAuthWait();
      googleAccount.set(account);
      googleConnectionStatus = "connected";
      googleConnectionMessage = `Connected as ${account.email}`;
      await loadCalendars();
    } catch (error) {
      googleConnectionStatus = "error";
      googleConnectionMessage =
        error instanceof Error ? error.message : String(error);
    } finally {
      googleAuthLoading.set(false);
    }
  }

  async function handleGoogleDisconnect() {
    googleConnectionMessage = "";
    googleAuthLoading.set(true);
    try {
      await googleAuthSignOut();
      googleAccount.set(null);
      selectedCalendar = null;
      await handleSaveConfig();
      googleConnectionStatus = "idle";
    } catch (error) {
      googleConnectionStatus = "error";
      googleConnectionMessage =
        error instanceof Error ? error.message : String(error);
    } finally {
      googleAuthLoading.set(false);
    }
  }

  async function handleDeleteToken() {
    await deleteJiraToken();
  }

  async function handleTestConnection() {
    connectionStatus = "testing";
    connectionMessage = "";

    try {
      const displayName = await testConnection();
      connectionStatus = "success";
      connectionMessage = `Connected as ${displayName}`;
    } catch (error) {
      connectionStatus = "error";
      connectionMessage =
        error instanceof Error ? error.message : "Connection failed";
    }
  }

  function previewTitle(): string {
    return eventTitleTemplate
      .replace("{key}", "PROJ-123")
      .replace("{summary}", "Example task summary")
      .replace("{project}", "PROJ")
      .replace("{priority}", "High")
      .replace("{status}", "In Progress")
      .replace("{type}", "Task");
  }

  function previewPRTitle(): string {
    return prEventTitleTemplate
      .replace("{repo}", "my-repo")
      .replace("{title}", "Fix important bug")
      .replace("{number}", "42")
      .replace("{author}", "developer");
  }

  async function handleSaveGitHubToken() {
    if (githubToken) {
      try {
        await saveGitHubTokenStore(githubToken);
        githubToken = "";
        hasGitHubTokenLocal = true;
        githubConnectionMessage = "Token saved successfully";
        githubConnectionStatus = "success";
      } catch (error) {
        githubConnectionStatus = "error";
        githubConnectionMessage = `Failed to save token: ${error instanceof Error ? error.message : String(error)}`;
      }
    }
  }

  async function handleDeleteGitHubToken() {
    try {
      await deleteGitHubTokenStore();
      hasGitHubTokenLocal = false;
      githubConnectionStatus = "idle";
      githubConnectionMessage = "";
    } catch (error) {
      githubConnectionStatus = "error";
      githubConnectionMessage = `Failed to delete token: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  async function handleTestGitHubConnection() {
    githubConnectionStatus = "testing";
    githubConnectionMessage = "";

    try {
      const displayName = await testGitHubConnection();
      githubConnectionStatus = "success";
      githubConnectionMessage = `Connected as ${displayName}`;
    } catch (error) {
      githubConnectionStatus = "error";
      githubConnectionMessage =
        error instanceof Error ? error.message : "Connection failed";
    }
  }
</script>

<div class="settings">
  <section>
    <h2>Jira Configuration</h2>

    <div class="form-group">
      <label for="jira-url">Jira URL</label>
      <input
        id="jira-url"
        type="url"
        bind:value={jiraUrl}
        placeholder="https://your-org.atlassian.net"
        onchange={handleSaveConfig}
      />
    </div>

    <div class="form-group">
      <label for="jira-email">Email</label>
      <input
        id="jira-email"
        type="email"
        bind:value={jiraEmail}
        placeholder="your-email@example.com"
        onchange={handleSaveConfig}
      />
    </div>

    <div class="form-group">
      <label for="jira-token">Personal Access Token</label>
      <div class="token-input">
        <input
          id="jira-token"
          type={showToken ? "text" : "password"}
          bind:value={jiraToken}
          placeholder={$hasToken ? "Token saved" : "Enter your Jira PAT"}
        />
        <button
          class="icon-button"
          onclick={() => (showToken = !showToken)}
          title={showToken ? "Hide" : "Show"}
        >
          {showToken ? "Hide" : "Show"}
        </button>
      </div>
      <div class="button-group">
        <button
          onclick={handleSaveToken}
          disabled={!jiraToken || $configLoading}
        >
          Save Token
        </button>
        {#if $hasToken}
          <button
            class="danger"
            onclick={handleDeleteToken}
            disabled={$configLoading}
          >
            Delete Token
          </button>
        {/if}
      </div>
    </div>

    <div class="form-group">
      <button
        class="test-button"
        onclick={handleTestConnection}
        disabled={connectionStatus === "testing" ||
          !jiraUrl ||
          !jiraEmail ||
          !$hasToken}
      >
        {connectionStatus === "testing" ? "Testing..." : "Test Connection"}
      </button>
      {#if connectionMessage}
        <p
          class="connection-status"
          class:success={connectionStatus === "success"}
          class:error={connectionStatus === "error"}
        >
          {connectionMessage}
        </p>
      {/if}
    </div>
  </section>

  <section>
    <h2>Google Calendar Connection</h2>

    <div class="form-group">
      <label for="google-client-id">Google OAuth Client ID</label>
      <input
        id="google-client-id"
        type="text"
        bind:value={googleClientId}
        placeholder="Your OAuth Client ID"
        onchange={handleSaveConfig}
      />
    </div>

    <div class="form-group">
      <label for="google-client-secret">Google OAuth Client Secret</label>
      <input
        id="google-client-secret"
        type="password"
        bind:value={googleClientSecret}
        placeholder="Your OAuth Client Secret"
        onchange={handleSaveConfig}
      />
      <p class="hint">
        Create an OAuth Client ID (Desktop app) in Google Cloud Console. You'll
        find both the Client ID and Client Secret in the downloaded JSON file.
      </p>
    </div>

    <div class="form-group">
      {#if $googleAccount}
        <p class="connection-status success">
          Connected as {$googleAccount.email}
        </p>
        <button
          class="danger"
          onclick={handleGoogleDisconnect}
          disabled={$googleAuthLoading}
        >
          Disconnect Google
        </button>
      {:else}
        <button onclick={handleGoogleConnect} disabled={$googleAuthLoading}>
          {googleConnectionStatus === "connecting"
            ? "Connecting..."
            : "Connect Google Calendar"}
        </button>
      {/if}
      {#if googleConnectionMessage}
        <p
          class="connection-status"
          class:success={googleConnectionStatus === "connected"}
          class:error={googleConnectionStatus === "error"}
        >
          {googleConnectionMessage}
        </p>
      {/if}
      {#if $googleAuthError}
        <p class="connection-status error">{$googleAuthError}</p>
      {/if}
    </div>
  </section>

  <section>
    <h2>GitHub Configuration</h2>

    <div class="form-group">
      <label for="github-token">Personal Access Token</label>
      <div class="token-input">
        <input
          id="github-token"
          type={showGitHubToken ? "text" : "password"}
          bind:value={githubToken}
          placeholder={hasGitHubTokenLocal
            ? "Token saved"
            : "Enter your GitHub PAT"}
        />
        <button
          class="icon-button"
          onclick={() => (showGitHubToken = !showGitHubToken)}
          title={showGitHubToken ? "Hide" : "Show"}
        >
          {showGitHubToken ? "Hide" : "Show"}
        </button>
      </div>
      <p class="hint">
        Create a PAT with <code>repo</code> scope at GitHub Settings &gt; Developer
        settings &gt; Personal access tokens
      </p>
      <div class="button-group">
        <button
          onclick={handleSaveGitHubToken}
          disabled={!githubToken || $configLoading}
        >
          Save Token
        </button>
        {#if hasGitHubTokenLocal}
          <button
            class="danger"
            onclick={handleDeleteGitHubToken}
            disabled={$configLoading}
          >
            Delete Token
          </button>
        {/if}
      </div>
    </div>

    <div class="form-group">
      <button
        class="test-button"
        onclick={handleTestGitHubConnection}
        disabled={githubConnectionStatus === "testing" || !hasGitHubTokenLocal}
      >
        {githubConnectionStatus === "testing"
          ? "Testing..."
          : "Test Connection"}
      </button>
      {#if githubConnectionMessage}
        <p
          class="connection-status"
          class:success={githubConnectionStatus === "success"}
          class:error={githubConnectionStatus === "error"}
        >
          {githubConnectionMessage}
        </p>
      {/if}
    </div>

    <div class="form-group">
      <label for="github-username">GitHub Username (optional)</label>
      <input
        id="github-username"
        type="text"
        bind:value={githubUsername}
        placeholder="Auto-detected from token"
        onchange={handleSaveConfig}
      />
      <p class="hint">Leave blank to auto-detect from your token</p>
    </div>

    <div class="form-group">
      <label for="pr-title-template">PR Event Title Template</label>
      <input
        id="pr-title-template"
        type="text"
        bind:value={prEventTitleTemplate}
        placeholder="[PR Review] repo: title"
        onchange={handleSaveConfig}
      />
      <p class="hint">
        Available placeholders: &#123;repo&#125;, &#123;title&#125;,
        &#123;number&#125;, &#123;author&#125;
      </p>
      <p class="preview">Preview: {previewPRTitle()}</p>
    </div>

    <div class="form-group">
      <label for="pr-default-color">PR Review Event Color</label>
      <div class="color-select">
        <span
          class="color-swatch"
          style={`background: ${colorHexFor(prDefaultEventColor)}`}
        ></span>
        <select
          id="pr-default-color"
          bind:value={prDefaultEventColor}
          onchange={handleSaveConfig}
        >
          {#each calendarColors as color}
            <option value={color.id}>{color.name}</option>
          {/each}
        </select>
      </div>
      <p class="hint">Default color for PR review events on your calendar.</p>
    </div>
  </section>

  <section>
    <h2>Calendar Settings</h2>

    <div class="form-group">
      <label for="calendar-select">Target Calendar</label>
      {#if !$googleAccount}
        <p>Connect Google Calendar to load your calendars.</p>
      {:else if $calendarsLoading}
        <p>Loading calendars...</p>
      {:else}
        <select
          id="calendar-select"
          bind:value={selectedCalendar}
          onchange={handleSaveConfig}
        >
          <option value={null}>Select a calendar</option>
          {#each $calendars as cal}
            <option value={cal.uid}>{cal.name}</option>
          {/each}
        </select>
      {/if}
      <button
        onclick={loadCalendars}
        disabled={$calendarsLoading || !$googleAccount}
      >
        Refresh Calendars
      </button>
      {#if $calendarsError}
        <p class="connection-status error">{$calendarsError}</p>
      {/if}
    </div>
  </section>

  <section>
    <h2>Sync Settings</h2>

    <div class="form-group">
      <label for="sync-frequency">Sync Frequency</label>
      <select
        id="sync-frequency"
        bind:value={syncFrequency}
        onchange={handleSaveConfig}
      >
        <option value="manual">Manual</option>
        <option value="hourly">Hourly</option>
        <option value="daily">Daily</option>
      </select>
    </div>

    <div class="form-group checkbox">
      <input
        id="auto-sync"
        type="checkbox"
        bind:checked={autoSyncOnLaunch}
        onchange={handleSaveConfig}
      />
      <label for="auto-sync">Auto-sync on launch</label>
    </div>

    <div class="form-group">
      <label for="jql-filter">Custom JQL Filter (optional)</label>
      <textarea
        id="jql-filter"
        bind:value={jqlFilter}
        placeholder="assignee = currentUser() AND resolution = Unresolved"
        rows="3"
        onchange={handleSaveConfig}
      ></textarea>
    </div>
  </section>

  <section>
    <h2>Event Customization</h2>

    <div class="form-group">
      <label for="title-template">Event Title Template</label>
      <input
        id="title-template"
        type="text"
        bind:value={eventTitleTemplate}
        placeholder="[&#123;key&#125;] &#123;summary&#125;"
        onchange={handleSaveConfig}
      />
      <p class="hint">
        Available placeholders: {"{key}"}, {"{summary}"}, {"{project}"}, {"{priority}"},
        {"{status}"}, {"{type}"}
      </p>
      <p class="preview">Preview: {previewTitle()}</p>
    </div>

    <div class="form-group">
      <label for="default-color">Default Event Color</label>
      <div class="color-select">
        <span
          class="color-swatch"
          style={`background: ${colorHexFor(defaultEventColor)}`}
        ></span>
        <select
          id="default-color"
          bind:value={defaultEventColor}
          onchange={handleSaveConfig}
        >
          {#each calendarColors as color}
            <option value={color.id}>{color.name}</option>
          {/each}
        </select>
      </div>
      <p class="hint">Used when no specific color is selected for a session.</p>
    </div>
  </section>

  {#if $configError}
    <div class="error-message">{$configError}</div>
  {/if}
</div>

<style>
  .settings {
    max-width: 600px;
  }

  section {
    background: white;
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 20px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin: 0 0 16px;
    font-size: 18px;
    color: #1d1d1f;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group.checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .form-group.checkbox label {
    margin-bottom: 0;
  }

  label {
    display: block;
    margin-bottom: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #1d1d1f;
  }

  input[type="text"],
  input[type="email"],
  input[type="url"],
  input[type="password"],
  select,
  textarea {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    font-size: 14px;
    transition: border-color 0.2s;
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: none;
    border-color: #0071e3;
  }

  .token-input {
    display: flex;
    gap: 8px;
  }

  .token-input input {
    flex: 1;
  }

  .icon-button {
    padding: 10px 12px;
    background: #f5f5f7;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
  }

  .button-group {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  button {
    padding: 10px 16px;
    background: #0071e3;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  button:hover:not(:disabled) {
    background: #0077ed;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.danger {
    background: #ff3b30;
  }

  button.danger:hover:not(:disabled) {
    background: #ff453a;
  }

  .test-button {
    background: #34c759;
  }

  .test-button:hover:not(:disabled) {
    background: #30d158;
  }

  .connection-status {
    margin-top: 8px;
    font-size: 13px;
    padding: 8px 12px;
    border-radius: 6px;
  }

  .connection-status.success {
    background: #e8f8ec;
    color: #34c759;
  }

  .connection-status.error {
    background: #ffebea;
    color: #ff3b30;
  }

  .hint {
    margin-top: 4px;
    font-size: 12px;
    color: #86868b;
  }

  .preview {
    margin-top: 8px;
    padding: 8px 12px;
    background: #f5f5f7;
    border-radius: 6px;
    font-size: 13px;
    color: #1d1d1f;
  }

  .color-select {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .color-swatch {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 1px solid #d2d2d7;
  }

  .error-message {
    padding: 12px;
    background: #ffebea;
    border-radius: 8px;
    color: #ff3b30;
    font-size: 14px;
  }
</style>
