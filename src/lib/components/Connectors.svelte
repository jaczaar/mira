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

  let showDropdown = $state(false);

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
  const connectedCount = $derived(
    (isGoogleConnected ? 1 : 0) + (isJiraConnected ? 1 : 0) + (isGitHubConnected ? 1 : 0)
  );

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") showDropdown = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="integrations-anchor">
  <button class="integrations-trigger" onclick={() => showDropdown = !showDropdown}>
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
    </svg>
    Integrations
    {#if connectedCount > 0}
      <span class="badge">{connectedCount}</span>
    {/if}
  </button>

  {#if showDropdown}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="dropdown-backdrop" role="presentation" onclick={() => showDropdown = false}></div>
    <div class="dropdown">
      <!-- Google accounts — one row per account -->
      {#each $googleAccounts as account}
        <div class="row">
          <div class="row-icon google">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
            </svg>
          </div>
          <span class="row-label">{account.email}</span>
          <span class="row-status connected">Connected</span>
          <button class="row-action disconnect" onclick={() => disconnectGoogle(account.email)} disabled={googleLoading}>Disconnect</button>
        </div>
      {/each}

      {#if !isGoogleConnected}
        <div class="row">
          <div class="row-icon google">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
            </svg>
          </div>
          <span class="row-label">Google Calendar</span>
          <button class="row-action connect" onclick={connectGoogle} disabled={googleLoading}>
            {googleLoading ? "Connecting..." : "Connect"}
          </button>
        </div>
      {:else}
        <div class="row">
          <div class="row-icon google">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </div>
          <span class="row-label add">Add Google account</span>
          <button class="row-action connect" onclick={connectGoogle} disabled={googleLoading}>
            {googleLoading ? "..." : "Add"}
          </button>
        </div>
      {/if}
      {#if googleError}
        <div class="row-error">{googleError}</div>
      {/if}

      <div class="dropdown-divider"></div>

      <!-- Jira -->
      <div class="row">
        <div class="row-icon jira">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2L2 7l10 5 10-5-10-5z" /><path d="M2 17l10 5 10-5" /><path d="M2 12l10 5 10-5" />
          </svg>
        </div>
        <span class="row-label">Jira</span>
        {#if isJiraConnected}
          <span class="row-detail">{$config.jira_email}</span>
          <span class="row-status connected">Connected</span>
          <button class="row-action disconnect" onclick={disconnectJira}>Disconnect</button>
        {:else}
          <button class="row-action connect" onclick={connectJira} disabled={jiraLoading}>
            {jiraLoading ? "Connecting..." : "Connect"}
          </button>
        {/if}
      </div>
      {#if jiraMessage}
        <div class="row-error" class:success={jiraStatus === "success"}>{jiraMessage}</div>
      {/if}

      <div class="dropdown-divider"></div>

      <!-- GitHub -->
      <div class="row">
        <div class="row-icon github">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
          </svg>
        </div>
        <span class="row-label">GitHub</span>
        {#if isGitHubConnected}
          <span class="row-detail">{$config.github_username || ""}</span>
          <span class="row-status connected">Connected</span>
          <button class="row-action disconnect" onclick={disconnectGitHub}>Disconnect</button>
        {:else if githubStatus === "polling" && githubUserCode}
          <span class="row-detail">Code: <strong>{githubUserCode}</strong></span>
          <button class="row-action disconnect" onclick={() => { githubPolling = false; githubStatus = "idle"; githubUserCode = ""; }}>Cancel</button>
        {:else}
          <button class="row-action connect" onclick={connectGitHub} disabled={githubStatus === "waiting"}>
            {githubStatus === "waiting" ? "..." : "Connect"}
          </button>
        {/if}
      </div>
      {#if githubMessage}
        <div class="row-error" class:success={githubStatus === "success"}>{githubMessage}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .integrations-anchor {
    position: relative;
  }

  .integrations-trigger {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-secondary);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .integrations-trigger:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .badge {
    background: var(--accent-green-dim);
    color: var(--accent-green);
    font-size: 10px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: var(--radius-full);
  }

  .dropdown-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 100;
    width: 380px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 6px;
    animation: fadeInUp 0.15s var(--ease-out);
  }

  .dropdown-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 4px 0;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
  }

  .row:hover {
    background: var(--bg-hover);
  }

  .row-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .row-icon.google {
    background: rgba(52, 168, 83, 0.1);
    color: #34a853;
  }

  .row-icon.jira {
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .row-icon.github {
    background: var(--accent-purple-dim);
    color: var(--accent-purple);
  }

  .row-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .row-label.add {
    color: var(--text-tertiary);
    font-weight: 400;
  }

  .row-detail {
    font-size: 11px;
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .row-status {
    font-size: 10px;
    font-weight: 500;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    white-space: nowrap;
  }

  .row-status.connected {
    background: var(--accent-green-dim);
    color: var(--accent-green);
  }

  .row-action {
    margin-left: auto;
    padding: 3px 10px;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .row-action.connect {
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .row-action.connect:hover:not(:disabled) {
    background: var(--accent-blue-glow);
  }

  .row-action.disconnect {
    background: transparent;
    color: var(--text-tertiary);
  }

  .row-action.disconnect:hover {
    color: var(--accent-red);
    background: var(--accent-red-dim);
  }

  .row-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .row-error {
    padding: 4px 8px;
    font-size: 11px;
    color: var(--accent-red);
  }

  .row-error.success {
    color: var(--accent-green);
  }
</style>
