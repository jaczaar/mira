<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
</script>

<div class="about">
  <div class="card">
    <h1>Mira</h1>
    <p class="version">Version 0.1.0</p>

    <p class="description">
      Mira is a Motion-like auto-scheduling desktop app that syncs Jira tasks
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
          <kbd>⌘</kbd> + <kbd>R</kbd>
          <span>Refresh tasks</span>
        </div>
        <div class="shortcut">
          <kbd>⌘</kbd> + <kbd>S</kbd>
          <span>Sync to calendar</span>
        </div>
        <div class="shortcut">
          <kbd>⌘</kbd> + <kbd>,</kbd>
          <span>Open settings</span>
        </div>
      </div>
    </section>

    <footer>
      <p>Built with Tauri + Svelte</p>
    </footer>
  </div>
</div>

<style>
  .about {
    max-width: 600px;
    margin: 0 auto;
  }

  .card {
    background: white;
    border-radius: 12px;
    padding: 32px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  h1 {
    font-size: 32px;
    font-weight: 600;
    margin: 0;
    color: #1d1d1f;
  }

  .version {
    color: #86868b;
    margin: 4px 0 24px;
  }

  .description {
    font-size: 16px;
    line-height: 1.6;
    color: #1d1d1f;
    margin-bottom: 32px;
  }

  section {
    margin-bottom: 28px;
  }

  h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 12px;
    color: #1d1d1f;
  }

  ul,
  ol {
    margin: 0;
    padding-left: 24px;
  }

  li {
    margin: 8px 0;
    line-height: 1.5;
    color: #1d1d1f;
  }

  .link-btn {
    background: none;
    border: none;
    color: #0071e3;
    cursor: pointer;
    font-size: inherit;
    padding: 0;
    text-decoration: underline;
  }

  .link-btn:hover {
    color: #0077ed;
  }

  .shortcuts {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  kbd {
    display: inline-block;
    padding: 4px 8px;
    background: #f5f5f7;
    border: 1px solid #d2d2d7;
    border-radius: 4px;
    font-family: monospace;
    font-size: 12px;
  }

  .shortcut span {
    margin-left: 12px;
    color: #86868b;
  }

  footer {
    margin-top: 32px;
    padding-top: 20px;
    border-top: 1px solid #e5e5e5;
    text-align: center;
    color: #86868b;
    font-size: 13px;
  }
</style>
