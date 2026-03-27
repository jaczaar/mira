# Mira

Mira is a Motion-like auto-scheduling desktop app that syncs Jira tasks and GitHub PRs with Google Calendar. It features community-driven development via a built-in Claude Code chatbot.

**Features**

- Auto-schedule tasks based on priority, deadlines, and calendar availability.
- Pull assigned Jira issues into a focused dashboard.
- Track GitHub PRs requiring review and schedule review time.
- Filter by status, group by epic, and search with fuzzy matching.
- Schedule tasks into Google Calendar with multi-slot scheduling, focus time, and per-event colors.
- Sync calendar events back to Jira as worklogs.
- Built-in Claude Code chatbot for community contributions via PR workflow.

**Tech Stack**

- Tauri 2 (Rust) backend
- Svelte 5 + TypeScript frontend
- Vite build tooling

**Requirements**

- Node.js 18+ and npm
- Rust toolchain (stable)
- Tauri prerequisites for your OS (WebView, system build tools)
- Jira Cloud account with a Personal Access Token (PAT)
- Google Cloud OAuth Client ID/Secret (Desktop) with Google Calendar API enabled
- Claude Code CLI (optional, for community contribution features)

**Setup**

1. Install dependencies.

```
npm install
```

2. Run the desktop app in development.

```
npm run tauri dev
```

3. Build the desktop app.

```
npm run tauri build
```

**Configuration**

Open Settings and configure:

- Jira URL, email, and PAT.
- Google OAuth Client ID and Client Secret, then connect your Google account.
- Target Google Calendar.
- GitHub PAT and username for PR tracking.
- Optional JQL filter, event title template, default event color, and sync preferences.

**Community Contributions**

Mira includes a built-in Claude Code chatbot. If you have the Claude Code CLI installed:

1. Clone this repo and run the app.
2. Open the chat widget (bottom-right corner).
3. Describe the feature or fix you want.
4. Claude Code implements the changes in your local clone.
5. Review the diff and approve to create a PR.
6. The maintainer reviews and merges your contribution.

**Local Data**

Configuration is stored locally in your system config directory under `mira/`. Tokens are stored in files with restricted permissions.

**License**

ISC
