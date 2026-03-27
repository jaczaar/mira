<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
</script>

<div class="about">
  <div class="card">
    <div class="hero">
      <div class="logo-mark">M</div>
      <h1>Mira</h1>
      <p class="version">v0.1.0</p>
    </div>

    <p class="description">
      A Motion-like auto-scheduling desktop app that syncs Jira tasks
      and GitHub PRs with Google Calendar, with community-driven development
      powered by Claude Code.
    </p>

    <section>
      <h2>Features</h2>
      <ul>
        <li>Auto-schedule tasks based on priority, deadlines, and availability</li>
        <li>Sync Jira issues to Google Calendar events</li>
        <li>Track and schedule GitHub PR reviews</li>
        <li>Log calendar time blocks as Jira worklogs</li>
        <li>Community-driven development via built-in Claude Code chatbot</li>
      </ul>
    </section>

    <section>
      <h2>Getting Started</h2>
      <ol>
        <li>Go to <strong>Settings</strong></li>
        <li>Enter your Jira URL and email</li>
        <li>
          Create a Personal Access Token (PAT) in Jira and save it
          <button
            class="link-btn"
            onclick={() =>
              open(
                "https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/"
              )
            }
          >
            Learn how
          </button>
        </li>
        <li>Connect Google Calendar and select a calendar to sync to</li>
        <li>Return to Dashboard and click "Sync to Calendar"</li>
      </ol>
    </section>

    <section>
      <h2>Keyboard Shortcuts</h2>
      <div class="shortcuts">
        <div class="shortcut">
          <div class="keys">
            <kbd>&#8984;</kbd><span class="plus">+</span><kbd>R</kbd>
          </div>
          <span class="action">Refresh tasks</span>
        </div>
        <div class="shortcut">
          <div class="keys">
            <kbd>&#8984;</kbd><span class="plus">+</span><kbd>S</kbd>
          </div>
          <span class="action">Sync to calendar</span>
        </div>
        <div class="shortcut">
          <div class="keys">
            <kbd>&#8984;</kbd><span class="plus">+</span><kbd>,</kbd>
          </div>
          <span class="action">Open settings</span>
        </div>
      </div>
    </section>

    <footer>
      <span>Built with Tauri + Svelte</span>
    </footer>
  </div>
</div>

<style>
  .about {
    max-width: 560px;
    margin: 0 auto;
    animation: fadeInUp 0.4s var(--ease-out);
  }

  .card {
    padding: 8px 0;
  }

  .hero {
    text-align: center;
    margin-bottom: 28px;
  }

  .logo-mark {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: var(--gradient-brand);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 22px;
    color: white;
    margin-bottom: 12px;
  }

  h1 {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
    letter-spacing: -0.04em;
  }

  .version {
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    font-size: 12px;
    margin: 4px 0 0;
  }

  .description {
    font-size: 15px;
    line-height: 1.6;
    color: var(--text-secondary);
    margin-bottom: 28px;
  }

  section {
    margin-bottom: 24px;
  }

  h2 {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 12px;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  ul,
  ol {
    margin: 0;
    padding-left: 20px;
  }

  li {
    margin: 8px 0;
    line-height: 1.5;
    color: var(--text-secondary);
    font-size: 14px;
  }

  li::marker {
    color: var(--text-tertiary);
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--accent-blue);
    cursor: pointer;
    font-size: inherit;
    padding: 0;
    text-decoration: none;
    font-weight: 500;
  }

  .link-btn:hover {
    text-decoration: underline;
  }

  .shortcuts {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .shortcut {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .plus {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    padding: 3px 7px;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: 5px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    box-shadow: 0 1px 0 var(--border-subtle);
  }

  .action {
    font-size: 13px;
    color: var(--text-secondary);
  }

  footer {
    margin-top: 28px;
    padding-top: 16px;
    border-top: 1px solid var(--border-subtle);
    text-align: center;
    color: var(--text-tertiary);
    font-size: 12px;
  }
</style>
