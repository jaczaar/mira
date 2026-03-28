<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import { config, hasToken, loadConfig, saveConfig, deleteJiraToken } from "../stores/config";
  import { googleAccounts, loadGoogleAuthStatus } from "../stores/google";
  import { googleAuthStart, googleAuthWait, googleAuthSignOut, githubDeviceFlowStart, githubDeviceFlowPoll, jiraAuthStart, jiraAuthWait } from "../api";
  import { hasEmbeddedCredentials, EMBEDDED_GOOGLE_CLIENT_ID, EMBEDDED_GOOGLE_CLIENT_SECRET } from "../google-oauth";
  import {
    deleteGitHubToken as deleteGitHubTokenStore,
    hasGitHubToken as hasGitHubTokenDerived,
    checkGitHubToken,
  } from "../stores/github";
  import { GITHUB_CLIENT_ID, hasGitHubOAuthCredentials } from "../github-oauth";
  import { JIRA_CLIENT_ID, JIRA_CLIENT_SECRET, hasJiraOAuthCredentials } from "../jira-oauth";

  type ConnectorId = "google" | "jira" | "github";
  let expandedConnector = $state<ConnectorId | null>(null);

  // Google state
  let googleLoading = $state(false);
  let googleError = $state<string | null>(null);

  // Jira state
  let jiraLoading = $state(false);
  let jiraStatus = $state<"idle" | "success" | "error">("idle");
  let jiraMessage = $state("");

  // GitHub Device Flow state
  let githubStatus = $state<"idle" | "waiting" | "polling" | "success" | "error">("idle");
  let githubMessage = $state("");
  let githubUserCode = $state("");
  let githubPolling = $state(false);

  const isGoogleConnected = $derived($googleAccounts.length > 0);
  const isJiraConnected = $derived($hasToken && !!$config.jira_url);
  const isGitHubConnected = $derived($hasGitHubTokenDerived);

  onMount(async () => {
    await loadConfig();
    await loadGoogleAuthStatus();
    await checkGitHubToken();
  });

  async function connectGoogle() {
    googleLoading = true;
    googleError = null;
    try {
      if (!$config.google_client_id && hasEmbeddedCredentials()) {
        await saveConfig({
          ...$config,
          google_client_id: EMBEDDED_GOOGLE_CLIENT_ID,
          google_client_secret: EMBEDDED_GOOGLE_CLIENT_SECRET,
        });
      }
      const { auth_url } = await googleAuthStart();
      await open(auth_url);
      await googleAuthWait();
      await loadGoogleAuthStatus();
      expandedConnector = null;
    } catch (err) {
      googleError = err instanceof Error ? err.message : String(err);
    } finally {
      googleLoading = false;
    }
  }

  async function disconnectGoogle(email: string) {
    googleLoading = true;
    try {
      await googleAuthSignOut(email);
      await loadGoogleAuthStatus();
    } finally {
      googleLoading = false;
    }
  }

  async function connectJira() {
    if (!hasJiraOAuthCredentials()) return;
    jiraLoading = true;
    jiraStatus = "idle";
    jiraMessage = "";
    try {
      const { auth_url } = await jiraAuthStart(JIRA_CLIENT_ID, JIRA_CLIENT_SECRET);
      await open(auth_url);
      const result = await jiraAuthWait();
      await loadConfig();
      jiraStatus = "success";
      jiraMessage = `Connected as ${result.display_name}`;
      setTimeout(() => { expandedConnector = null; }, 1200);
    } catch (err) {
      jiraStatus = "error";
      jiraMessage = err instanceof Error ? err.message : String(err);
    } finally {
      jiraLoading = false;
    }
  }

  async function disconnectJira() {
    await deleteJiraToken();
    jiraStatus = "idle";
    jiraMessage = "";
  }

  async function connectGitHub() {
    if (!hasGitHubOAuthCredentials()) return;
    githubStatus = "waiting";
    githubMessage = "";
    githubUserCode = "";
    try {
      const deviceCode = await githubDeviceFlowStart(GITHUB_CLIENT_ID);
      githubUserCode = deviceCode.user_code;
      githubStatus = "polling";

      await open(deviceCode.verification_uri);

      // Poll for token
      githubPolling = true;
      const interval = (deviceCode.interval || 5) * 1000;
      const expiresAt = Date.now() + deviceCode.expires_in * 1000;

      while (githubPolling && Date.now() < expiresAt) {
        await new Promise((r) => setTimeout(r, interval));
        if (!githubPolling) break;
        try {
          const displayName = await githubDeviceFlowPoll(GITHUB_CLIENT_ID, deviceCode.device_code);
          await checkGitHubToken();
          githubStatus = "success";
          githubMessage = `Connected as ${displayName}`;
          githubPolling = false;
          setTimeout(() => { expandedConnector = null; }, 1200);
          return;
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          if (msg.includes("authorization_pending") || msg.includes("slow_down")) {
            continue;
          }
          throw err;
        }
      }

      if (githubPolling) {
        githubStatus = "error";
        githubMessage = "Device code expired. Please try again.";
        githubPolling = false;
      }
    } catch (err) {
      githubStatus = "error";
      githubMessage = err instanceof Error ? err.message : String(err);
      githubPolling = false;
    }
  }

  async function disconnectGitHub() {
    githubPolling = false;
    await deleteGitHubTokenStore();
    await checkGitHubToken();
    githubStatus = "idle";
    githubMessage = "";
    githubUserCode = "";
  }

  function toggleConnector(id: ConnectorId) {
    expandedConnector = expandedConnector === id ? null : id;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") expandedConnector = null;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="connectors">
  <div class="connectors-header">
    <h3>Integrations</h3>
  </div>

  <div class="connector-grid">
    <!-- Google Calendar -->
    <div class="connector-card" class:connected={isGoogleConnected} class:expanded={expandedConnector === "google"}>
      <button class="connector-row" onclick={() => !isGoogleConnected && toggleConnector("google")}>
        <div class="connector-icon google">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
          </svg>
        </div>
        <div class="connector-info">
          <span class="connector-name">Google Calendar</span>
          {#if isGoogleConnected}
            <span class="connector-detail">{$googleAccounts[0].email}</span>
          {:else}
            <span class="connector-detail muted">Sync events & schedule tasks</span>
          {/if}
        </div>
        {#if isGoogleConnected}
          <span class="status-pill connected">
            <span class="status-dot"></span>
            Connected
          </span>
        {:else}
          <span class="connect-arrow">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="9 18 15 12 9 6" /></svg>
          </span>
        {/if}
      </button>

      {#if expandedConnector === "google" && !isGoogleConnected}
        <div class="connector-body">
          <p class="body-hint">Sign in with your Google account to sync calendar events.</p>
          <button class="action-btn google-btn" onclick={connectGoogle} disabled={googleLoading}>
            {#if googleLoading}
              <span class="btn-spinner"></span>
              Waiting for sign-in...
            {:else}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" /><polyline points="10 17 15 12 10 7" /><line x1="15" y1="12" x2="3" y2="12" />
              </svg>
              Sign in with Google
            {/if}
          </button>
          {#if googleError}
            <p class="error-text">{googleError}</p>
          {/if}
        </div>
      {/if}

      {#if isGoogleConnected}
        <div class="connector-manage">
          {#each $googleAccounts as account}
            <button class="manage-btn disconnect" onclick={() => disconnectGoogle(account.email)}>Disconnect</button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Jira -->
    <div class="connector-card" class:connected={isJiraConnected} class:expanded={expandedConnector === "jira"}>
      <button class="connector-row" onclick={() => !isJiraConnected && toggleConnector("jira")}>
        <div class="connector-icon jira">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2L2 7l10 5 10-5-10-5z" /><path d="M2 17l10 5 10-5" /><path d="M2 12l10 5 10-5" />
          </svg>
        </div>
        <div class="connector-info">
          <span class="connector-name">Jira</span>
          {#if isJiraConnected}
            <span class="connector-detail">{$config.jira_email}</span>
          {:else}
            <span class="connector-detail muted">Import tasks & track time</span>
          {/if}
        </div>
        {#if isJiraConnected}
          <span class="status-pill connected">
            <span class="status-dot"></span>
            Connected
          </span>
        {:else}
          <span class="connect-arrow">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="9 18 15 12 9 6" /></svg>
          </span>
        {/if}
      </button>

      {#if expandedConnector === "jira" && !isJiraConnected}
        <div class="connector-body">
          <p class="body-hint">Sign in with your Atlassian account to import tasks.</p>
          <button class="action-btn" onclick={connectJira} disabled={jiraLoading}>
            {#if jiraLoading}
              <span class="btn-spinner"></span>
              Waiting for sign-in...
            {:else}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" /><polyline points="10 17 15 12 10 7" /><line x1="15" y1="12" x2="3" y2="12" />
              </svg>
              Sign in with Atlassian
            {/if}
          </button>
          {#if jiraMessage}
            <p class="status-text" class:success={jiraStatus === "success"} class:error={jiraStatus === "error"}>{jiraMessage}</p>
          {/if}
        </div>
      {/if}

      {#if isJiraConnected}
        <div class="connector-manage">
          <button class="manage-btn disconnect" onclick={disconnectJira}>Disconnect</button>
        </div>
      {/if}
    </div>

    <!-- GitHub -->
    <div class="connector-card" class:connected={isGitHubConnected} class:expanded={expandedConnector === "github"}>
      <button class="connector-row" onclick={() => !isGitHubConnected && toggleConnector("github")}>
        <div class="connector-icon github">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
          </svg>
        </div>
        <div class="connector-info">
          <span class="connector-name">GitHub</span>
          {#if isGitHubConnected}
            <span class="connector-detail">{$config.github_username || "Connected"}</span>
          {:else}
            <span class="connector-detail muted">Track PRs & code reviews</span>
          {/if}
        </div>
        {#if isGitHubConnected}
          <span class="status-pill connected">
            <span class="status-dot"></span>
            Connected
          </span>
        {:else}
          <span class="connect-arrow">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="9 18 15 12 9 6" /></svg>
          </span>
        {/if}
      </button>

      {#if expandedConnector === "github" && !isGitHubConnected}
        <div class="connector-body">
          {#if githubStatus === "polling" && githubUserCode}
            <div class="device-flow">
              <p class="body-hint">Enter this code on GitHub:</p>
              <div class="device-code">{githubUserCode}</div>
              <p class="body-hint">A browser window has opened. Paste the code above and authorize Mira.</p>
              <div class="polling-indicator">
                <span class="btn-spinner dark"></span>
                <span>Waiting for authorization...</span>
              </div>
              <button class="cancel-link" onclick={() => { githubPolling = false; githubStatus = "idle"; githubUserCode = ""; }}>Cancel</button>
            </div>
          {:else}
            <p class="body-hint">Click below to authorize Mira with your GitHub account.</p>
            <button class="action-btn" onclick={connectGitHub} disabled={githubStatus === "waiting"}>
              {#if githubStatus === "waiting"}
                <span class="btn-spinner"></span>
                Starting...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
                </svg>
                Sign in with GitHub
              {/if}
            </button>
          {/if}
          {#if githubMessage}
            <p class="status-text" class:success={githubStatus === "success"} class:error={githubStatus === "error"}>{githubMessage}</p>
          {/if}
        </div>
      {/if}

      {#if isGitHubConnected}
        <div class="connector-manage">
          <button class="manage-btn disconnect" onclick={disconnectGitHub}>Disconnect</button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .connectors {
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid var(--border-subtle);
    animation: fadeInUp 0.4s var(--ease-out) 0.1s both;
  }

  .connectors-header {
    margin-bottom: 14px;
  }

  .connectors-header h3 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
  }

  .connector-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .connector-card {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    overflow: hidden;
    transition: all 0.2s var(--ease-out);
  }

  .connector-card:hover {
    border-color: var(--border-strong);
  }

  .connector-card.connected {
    border-color: var(--border-subtle);
  }

  .connector-card.expanded {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 1px var(--accent-blue-dim);
  }

  .connector-row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 14px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    font-family: var(--font-body);
    transition: background 0.12s var(--ease-out);
  }

  .connector-card.connected .connector-row {
    cursor: default;
  }

  .connector-row:hover {
    background: var(--bg-hover);
  }

  .connector-card.connected .connector-row:hover {
    background: transparent;
  }

  .connector-icon {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .connector-icon.google {
    background: rgba(52, 168, 83, 0.1);
    color: #34a853;
  }

  .connector-icon.jira {
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .connector-icon.github {
    background: var(--accent-purple-dim);
    color: var(--accent-purple);
  }

  .connector-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .connector-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .connector-detail {
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .connector-detail.muted {
    color: var(--text-tertiary);
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 500;
    font-family: var(--font-body);
  }

  .status-pill.connected {
    background: var(--accent-green-dim);
    color: var(--accent-green);
  }

  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-green);
  }

  .connect-arrow {
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    transition: color 0.12s, transform 0.2s var(--ease-out);
  }

  .connector-card.expanded .connect-arrow {
    transform: rotate(90deg);
    color: var(--accent-blue);
  }

  /* Expanded body */
  .connector-body {
    padding: 0 14px 14px;
    animation: fadeInUp 0.2s var(--ease-out);
  }

  .body-hint {
    margin: 0 0 12px;
    font-size: 13px;
    color: var(--text-tertiary);
    line-height: 1.5;
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
  }

  .form-stack input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-primary);
    background: var(--bg-elevated);
    outline: none;
    transition: border-color 0.15s;
  }

  .form-stack input:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .form-stack input::placeholder {
    color: var(--text-tertiary);
  }

  .token-row {
    display: flex;
    gap: 6px;
  }

  .token-row input {
    flex: 1;
  }

  .toggle-vis {
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

  .toggle-vis:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .form-hint {
    margin: 0;
    font-size: 11px;
    color: var(--text-tertiary);
    line-height: 1.4;
  }

  .form-hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 10px;
    color: var(--accent-purple);
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 18px;
    background: var(--gradient-brand);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .action-btn:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: var(--shadow-glow-blue);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.google-btn {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-strong);
  }

  .action-btn.google-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    box-shadow: var(--shadow-sm);
    opacity: 1;
  }

  .btn-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .action-btn.google-btn .btn-spinner {
    border-color: var(--border-default);
    border-top-color: var(--accent-blue);
  }

  .status-text {
    margin: 8px 0 0;
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
  }

  .status-text.success {
    background: var(--accent-green-dim);
    color: var(--accent-green);
  }

  .status-text.error {
    background: var(--accent-red-dim);
    color: var(--accent-red);
  }

  .error-text {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--accent-red);
  }

  /* Connected state manage row */
  .connector-manage {
    display: flex;
    justify-content: flex-end;
    padding: 0 14px 10px;
  }

  .manage-btn {
    padding: 4px 10px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .manage-btn.disconnect {
    color: var(--text-tertiary);
  }

  .manage-btn.disconnect:hover {
    color: var(--accent-red);
    background: var(--accent-red-dim);
  }

  .device-flow {
    text-align: center;
  }

  .device-code {
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: 700;
    letter-spacing: 0.12em;
    color: var(--text-primary);
    padding: 14px 20px;
    background: var(--bg-base);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    margin: 10px 0 12px;
    user-select: all;
  }

  .polling-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-tertiary);
    margin-top: 8px;
  }

  .btn-spinner.dark {
    border-color: var(--border-default);
    border-top-color: var(--accent-blue);
  }

  .cancel-link {
    margin-top: 10px;
    background: none;
    border: none;
    color: var(--text-tertiary);
    font-family: var(--font-body);
    font-size: 12px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: all 0.12s;
  }

  .cancel-link:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }
</style>
