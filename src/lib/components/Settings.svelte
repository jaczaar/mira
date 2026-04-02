<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import { check } from "@tauri-apps/plugin-updater";
  import { getVersion } from "@tauri-apps/api/app";
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


  let currentVersion = $state("");
  let updateStatus = $state<"idle" | "checking" | "available" | "downloading" | "up-to-date" | "error">("idle");
  let updateError = $state("");
  let updateVersion = $state("");
  let updateNotes = $state("");
  let updateObj: Awaited<ReturnType<typeof check>> = $state(null);
  let downloadedBytes = $state(0);
  let totalBytes = $state(0);

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
    currentVersion = await getVersion();
    await loadConfig();
    await loadGoogleAuthStatus();
    if ($googleAccount) {
      googleConnectionStatus = "connected";
      googleConnectionMessage = `Connected as ${$googleAccount.email}`;
      await loadCalendars();
    }

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
      github_username: $config.github_username ?? "",
      pr_event_title_template: $config.pr_event_title_template ?? "[PR Review] {repo}: {title}",
      pr_default_event_color: $config.pr_default_event_color ?? null,
      calendar_colors: $config.calendar_colors ?? {},
      account_colors: $config.account_colors ?? {},
      scheduling_strategy: $config.scheduling_strategy ?? "earliest_available",
      allow_task_splitting: $config.allow_task_splitting ?? true,
      account_schedule_windows: $config.account_schedule_windows ?? {},
    });
  }

  async function handleSaveToken() {
    if (jiraToken) {
      try {
        await saveJiraToken(jiraToken);
        jiraToken = "";
        connectionMessage = "Token saved successfully";
        connectionStatus = "success";
      } catch (error) {
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
      await googleAuthSignOut($googleAccount?.email ?? "");
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


  async function handleCheckForUpdate() {
    updateStatus = "checking";
    updateError = "";
    try {
      const update = await check();
      if (update) {
        updateVersion = update.version;
        updateNotes = update.body ?? "";
        updateStatus = "available";
        updateObj = update;
      } else {
        updateStatus = "up-to-date";
      }
    } catch (error) {
      updateStatus = "error";
      const msg = error instanceof Error ? error.message : String(error);
      if (msg.includes("release JSON") || msg.includes("404") || msg.includes("Not Found")) {
        updateError = "No published updates found. Updates will appear here when a new release is available.";
      } else {
        updateError = msg;
      }
    }
  }

  async function handleDownloadAndInstall() {
    if (!updateObj) return;
    updateStatus = "downloading";
    downloadedBytes = 0;
    totalBytes = 0;
    try {
      await updateObj.downloadAndInstall((event) => {
        if (event.event === "Started" && event.data.contentLength) {
          totalBytes = event.data.contentLength;
        } else if (event.event === "Progress") {
          downloadedBytes += event.data.chunkLength;
        }
      });
    } catch (error) {
      updateStatus = "error";
      updateError = error instanceof Error ? error.message : String(error);
    }
  }


</script>

<div class="settings">
  <section>
    <div class="section-header">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-blue)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5z" />
        <path d="M2 17l10 5 10-5" />
        <path d="M2 12l10 5 10-5" />
      </svg>
      <h2>Jira</h2>
    </div>

    <div class="form-group">
      <label for="jira-url">Instance URL</label>
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
          placeholder={$hasToken ? "Token saved securely" : "Enter your Jira PAT"}
        />
        <button
          class="icon-button"
          onclick={() => (showToken = !showToken)}
          title={showToken ? "Hide" : "Show"}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            {#if showToken}
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
              <line x1="1" y1="1" x2="23" y2="23" />
            {:else}
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
              <circle cx="12" cy="12" r="3" />
            {/if}
          </svg>
        </button>
      </div>
      <div class="button-group">
        <button
          class="btn primary"
          onclick={handleSaveToken}
          disabled={!jiraToken || $configLoading}
        >
          Save Token
        </button>
        {#if $hasToken}
          <button
            class="btn danger"
            onclick={handleDeleteToken}
            disabled={$configLoading}
          >
            Delete
          </button>
        {/if}
      </div>
    </div>

    <div class="form-group">
      <button
        class="btn test"
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
    <div class="section-header">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-green)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
        <line x1="16" y1="2" x2="16" y2="6" />
        <line x1="8" y1="2" x2="8" y2="6" />
        <line x1="3" y1="10" x2="21" y2="10" />
      </svg>
      <h2>Google Calendar</h2>
    </div>

    {#if $googleAccount}
      <!-- Connected state: compact summary -->
      <div class="gcal-connected">
        <div class="gcal-connected-info">
          <span class="gcal-dot"></span>
          <span class="gcal-email">{$googleAccount.email}</span>
          {#if selectedCalendar && $calendars.length > 0}
            <span class="gcal-sep">&middot;</span>
            <span class="gcal-cal">{$calendars.find(c => c.uid === selectedCalendar)?.name || 'Calendar selected'}</span>
          {/if}
        </div>
        <button class="btn ghost" onclick={handleGoogleDisconnect} disabled={$googleAuthLoading}>
          Disconnect
        </button>
      </div>

      <!-- Calendar picker (inline, only if connected but no calendar chosen) -->
      {#if !selectedCalendar}
        <div class="gcal-step">
          <div class="gcal-step-marker">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
          </div>
          <div class="gcal-step-content">
            <p class="gcal-step-title">Pick a calendar</p>
            <p class="gcal-step-hint">Where should Mira schedule your tasks?</p>
            {#if $calendarsLoading}
              <p class="gcal-step-hint">Loading your calendars...</p>
            {:else}
              <select
                id="calendar-select"
                bind:value={selectedCalendar}
                onchange={handleSaveConfig}
              >
                <option value={null}>Choose a calendar...</option>
                {#each $calendars as cal}
                  <option value={cal.uid}>{cal.name}</option>
                {/each}
              </select>
            {/if}
          </div>
        </div>
      {:else}
        <!-- Calendar change option, tucked away -->
        <div class="form-group">
          <label for="calendar-select">Target Calendar</label>
          <div class="gcal-cal-row">
            <select
              id="calendar-select"
              bind:value={selectedCalendar}
              onchange={handleSaveConfig}
            >
              {#each $calendars as cal}
                <option value={cal.uid}>{cal.name}</option>
              {/each}
            </select>
            <button
              class="btn ghost"
              onclick={loadCalendars}
              disabled={$calendarsLoading}
            >
              Refresh
            </button>
          </div>
        </div>
      {/if}

      {#if $calendarsError}
        <p class="connection-status error">{$calendarsError}</p>
      {/if}
    {:else}
      <!-- Not connected: guided stepper -->
      <div class="gcal-guide">
        <div class="gcal-step" class:done={!!googleClientId && !!googleClientSecret}>
          <div class="gcal-step-marker">
            {#if googleClientId && googleClientSecret}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-green)" stroke-width="2.5" stroke-linecap="round"><polyline points="20 6 9 17 4 12" /></svg>
            {:else}
              <span class="gcal-step-num">1</span>
            {/if}
          </div>
          <div class="gcal-step-content">
            <p class="gcal-step-title">Add OAuth credentials</p>
            <p class="gcal-step-hint">Create a <strong>Desktop app</strong> OAuth client in <button class="inline-link" onclick={() => open('https://console.cloud.google.com/apis/credentials')}>Google Cloud Console</button>, then paste the ID and secret below.</p>
            <div class="gcal-inputs">
              <input
                id="google-client-id"
                type="text"
                bind:value={googleClientId}
                placeholder="Client ID"
                onchange={handleSaveConfig}
              />
              <input
                id="google-client-secret"
                type="password"
                bind:value={googleClientSecret}
                placeholder="Client Secret"
                onchange={handleSaveConfig}
              />
            </div>
          </div>
        </div>

        <div class="gcal-step" class:disabled={!googleClientId || !googleClientSecret}>
          <div class="gcal-step-marker">
            <span class="gcal-step-num">2</span>
          </div>
          <div class="gcal-step-content">
            <p class="gcal-step-title">Sign in with Google</p>
            <p class="gcal-step-hint">A browser window will open for you to authorize Mira.</p>
            <button
              class="btn primary gcal-connect-btn"
              onclick={handleGoogleConnect}
              disabled={$googleAuthLoading || !googleClientId || !googleClientSecret}
            >
              {#if googleConnectionStatus === "connecting"}
                <span class="gcal-spinner"></span>
                Waiting for sign-in...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                  <polyline points="10 17 15 12 10 7" />
                  <line x1="15" y1="12" x2="3" y2="12" />
                </svg>
                Connect Google Account
              {/if}
            </button>
          </div>
        </div>
      </div>

      {#if googleConnectionMessage}
        <p
          class="connection-status"
          class:success={googleConnectionStatus === "connected"}
          class:error={googleConnectionStatus === "error"}
          style="margin-top: 12px;"
        >
          {googleConnectionMessage}
        </p>
      {/if}
      {#if $googleAuthError}
        <p class="connection-status error" style="margin-top: 8px;">{$googleAuthError}</p>
      {/if}
    {/if}
  </section>


  <section>
    <div class="section-header">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-amber)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
      <h2>Sync &amp; Calendar</h2>
    </div>

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
      <label for="jql-filter">Custom JQL Filter</label>
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
    <div class="section-header">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-blue)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
      </svg>
      <h2>Event Customization</h2>
    </div>

    <div class="form-group">
      <label for="title-template">Task Event Title Template</label>
      <input
        id="title-template"
        type="text"
        bind:value={eventTitleTemplate}
        placeholder="[&#123;key&#125;] &#123;summary&#125;"
        onchange={handleSaveConfig}
      />
      <p class="hint">
        Placeholders: {"{key}"}, {"{summary}"}, {"{project}"}, {"{priority}"},
        {"{status}"}, {"{type}"}
      </p>
      <div class="preview">{previewTitle()}</div>
    </div>

    <div class="form-group">
      <label for="default-color">Task Event Color</label>
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
    </div>
  </section>

  <section>
    <div class="section-header">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-blue)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>
      <h2>Updates</h2>
    </div>

    <div class="form-group">
      <label>Current Version</label>
      <p class="version-display">v{currentVersion}</p>
    </div>

    <div class="button-group">
      <button
        class="btn primary"
        onclick={handleCheckForUpdate}
        disabled={updateStatus === "checking" || updateStatus === "downloading"}
      >
        {#if updateStatus === "checking"}
          Checking...
        {:else}
          Check for updates
        {/if}
      </button>
    </div>

    {#if updateStatus === "up-to-date"}
      <div class="connection-status success">You're on the latest version.</div>
    {/if}

    {#if updateStatus === "available"}
      <div class="update-available">
        <p class="update-version">v{updateVersion} is available</p>
        {#if updateNotes}
          <p class="update-notes">{updateNotes}</p>
        {/if}
        <div class="button-group">
          <button class="btn primary" onclick={handleDownloadAndInstall}>
            Download & Install
          </button>
        </div>
      </div>
    {/if}

    {#if updateStatus === "downloading"}
      <div class="update-progress">
        <p class="hint">Downloading update...</p>
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {totalBytes > 0 ? Math.round((downloadedBytes / totalBytes) * 100) : 0}%"
          ></div>
        </div>
        {#if totalBytes > 0}
          <p class="hint">{Math.round(downloadedBytes / 1024 / 1024)}MB / {Math.round(totalBytes / 1024 / 1024)}MB</p>
        {/if}
      </div>
    {/if}

    {#if updateStatus === "error"}
      <div class="connection-status error">{updateError}</div>
    {/if}
  </section>

  {#if $configError}
    <div class="error-message">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      {$configError}
    </div>
  {/if}
</div>

<style>
  .settings {
    max-width: 600px;
  }

  section {
    padding: 0 0 24px;
    margin-bottom: 24px;
    border-bottom: 1px solid var(--border-subtle);
    animation: fadeInUp 0.3s var(--ease-out) both;
  }

  section:last-of-type {
    border-bottom: none;
  }

  section:nth-child(2) { animation-delay: 40ms; }
  section:nth-child(3) { animation-delay: 80ms; }
  section:nth-child(4) { animation-delay: 120ms; }
  section:nth-child(5) { animation-delay: 160ms; }
  section:nth-child(6) { animation-delay: 200ms; }
  section:nth-child(7) { animation-delay: 240ms; }

  .section-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .form-group {
    margin-bottom: 14px;
  }

  .form-group.checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .form-group.checkbox label {
    margin-bottom: 0;
    font-size: 13px;
  }

  .form-group.checkbox input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-blue);
  }

  label {
    display: block;
    margin-bottom: 5px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  input[type="text"],
  input[type="email"],
  input[type="url"],
  input[type="password"],
  select,
  textarea {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-primary);
    background: var(--bg-elevated);
    transition: all 0.15s;
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  input::placeholder,
  textarea::placeholder {
    color: var(--text-tertiary);
  }

  select {
    cursor: pointer;
    color-scheme: dark;
  }

  textarea {
    resize: vertical;
    min-height: 60px;
  }

  .token-input {
    display: flex;
    gap: 6px;
  }

  .token-input input {
    flex: 1;
  }

  .icon-button {
    padding: 8px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }

  .icon-button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .button-group {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }

  .btn {
    padding: 7px 14px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .btn.primary {
    background: var(--accent-blue-dim);
    border-color: rgba(91, 141, 239, 0.2);
    color: var(--accent-blue);
  }

  .btn.primary:hover:not(:disabled) {
    background: rgba(91, 141, 239, 0.2);
    box-shadow: var(--shadow-glow-blue);
  }

  .btn.danger {
    background: var(--accent-red-dim);
    border-color: rgba(248, 113, 113, 0.15);
    color: var(--accent-red);
  }

  .btn.danger:hover:not(:disabled) {
    background: rgba(248, 113, 113, 0.18);
  }

  .btn.test {
    background: var(--accent-green-dim);
    border-color: rgba(74, 222, 128, 0.15);
    color: var(--accent-green);
  }

  .btn.test:hover:not(:disabled) {
    background: rgba(74, 222, 128, 0.18);
  }

  .btn.ghost {
    background: transparent;
    margin-top: 6px;
  }

  .connection-status {
    margin-top: 8px;
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .connection-status.success {
    background: var(--accent-green-dim);
    color: var(--accent-green);
    border-color: rgba(74, 222, 128, 0.15);
  }

  .connection-status.error {
    background: var(--accent-red-dim);
    color: var(--accent-red);
    border-color: rgba(248, 113, 113, 0.15);
  }

  .hint {
    margin-top: 4px;
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 11px;
    color: var(--accent-purple);
  }

  .preview {
    margin-top: 6px;
    padding: 6px 10px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
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
    border: 2px solid var(--border-default);
    flex-shrink: 0;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--accent-red-dim);
    border-radius: var(--radius-md);
    color: var(--accent-red);
    font-size: 13px;
    border: 1px solid rgba(248, 113, 113, 0.15);
  }

  /* Google Calendar guided flow */
  .gcal-connected {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 14px;
  }

  .gcal-connected-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .gcal-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-green);
    flex-shrink: 0;
  }

  .gcal-email {
    color: var(--text-primary);
    font-weight: 500;
  }

  .gcal-sep {
    color: var(--text-tertiary);
  }

  .gcal-cal {
    color: var(--text-tertiary);
  }

  .gcal-cal-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .gcal-cal-row select {
    flex: 1;
  }

  .gcal-guide {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .gcal-step {
    display: flex;
    gap: 14px;
    padding: 14px 0;
    position: relative;
    transition: opacity 0.2s;
  }

  .gcal-step + .gcal-step {
    border-top: 1px solid var(--border-subtle);
  }

  .gcal-step.disabled {
    opacity: 0.35;
    pointer-events: none;
  }

  .gcal-step-marker {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .gcal-step.done .gcal-step-marker {
    border-color: rgba(110, 231, 160, 0.25);
    background: var(--accent-green-dim);
  }

  .gcal-step-num {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .gcal-step-content {
    flex: 1;
    min-width: 0;
  }

  .gcal-step-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 3px;
  }

  .gcal-step-hint {
    font-size: 12px;
    color: var(--text-tertiary);
    margin: 0 0 10px;
    line-height: 1.5;
  }

  .gcal-step-hint strong {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .inline-link {
    background: none;
    border: none;
    color: var(--accent-blue);
    cursor: pointer;
    font: inherit;
    font-size: inherit;
    padding: 0;
    text-decoration: none;
  }

  .inline-link:hover {
    text-decoration: underline;
  }

  .gcal-inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .gcal-connect-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .gcal-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  .version-display {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-secondary);
  }

  .update-available {
    margin-top: 10px;
  }

  .update-version {
    margin: 0 0 4px;
    font-size: 14px;
    font-weight: 600;
    color: var(--accent-green);
  }

  .update-notes {
    margin: 0 0 10px;
    font-size: 12px;
    color: var(--text-tertiary);
    line-height: 1.5;
  }

  .update-progress {
    margin-top: 10px;
  }

  .progress-bar {
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
    margin: 8px 0;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-blue);
    border-radius: 2px;
    transition: width 0.2s;
  }


</style>
