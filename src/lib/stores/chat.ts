import { writable, get } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  checkClaudeInstalled,
  startChatSession,
  sendChatMessage as apiSendMessage,
  cancelChatMessage as apiCancelMessage,
  stopChatSession as apiStopSession,
  getChangesDiff,
  submitPR as apiSubmitPR,
  discardChanges as apiDiscardChanges,
  type ClaudeInfo,
  type ChangeDiff,
  type PRResult,
  type ChatStreamEvent,
} from "../api";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: Date;
  isStreaming: boolean;
}

export interface ChatSession {
  sessionId: string;
  messages: ChatMessage[];
}

export const chatSession = writable<ChatSession | null>(null);
export const chatLoading = writable<boolean>(false);
export const chatError = writable<string | null>(null);
export const claudeInstalled = writable<ClaudeInfo | null>(null);
export const claudeCheckDone = writable<boolean>(false);
export const pendingDiff = writable<ChangeDiff | null>(null);
export const prSubmitting = writable<boolean>(false);

let unlisten: UnlistenFn | null = null;
let messageCounter = 0;
let sessionHasContext = false;

const SYSTEM_CONTEXT = `[CONTEXT — You are working inside the Mira desktop app's built-in Claude Code chatbot. The user is a community contributor making changes to the Mira codebase. Here is what you need to know:

## What Mira Is
Mira is a Motion-like auto-scheduling desktop app built with Tauri 2 (Rust backend) + Svelte 5 (TypeScript frontend). It syncs Jira tasks and GitHub pull requests with Google Calendar, letting users schedule work blocks, track PR reviews, and log time back to Jira.

## Tech Stack
- Frontend: Svelte 5 (runes: $state, $derived, $props, $effect), TypeScript strict, Vite, scoped CSS (no Tailwind, no component library)
- Backend: Tauri 2 (Rust), with modules for Jira API, GitHub API, Google Calendar OAuth/API, and this Claude chat integration
- Styling: Dark theme using CSS custom properties (--bg-base, --bg-surface, --accent-blue, --font-display 'Outfit', --font-body 'DM Sans', --font-mono 'JetBrains Mono', etc.)
- State management: Svelte writable stores in src/lib/stores/
- API layer: src/lib/api.ts wraps Tauri invoke() commands

## Key Directories
- src/lib/components/ — All UI components (Dashboard, TaskCard, PRCard, TaskScheduler, PRScheduler, ChatWidget, Settings, etc.)
- src/lib/stores/ — State stores (tasks.ts, github.ts, calendar.ts, config.ts, sync.ts, chat.ts, google.ts)
- src/routes/ — Page components (Dashboard.svelte, Calendar.svelte, SettingsPage.svelte, About.svelte)
- src-tauri/src/ — Rust backend (jira/, github/, google/, claude/, config/)

## Current Features
- Jira task sync with epic grouping, fuzzy search, priority/status badges
- GitHub PR tracking with role detection (author/reviewer), repo grouping
- Google Calendar integration: OAuth flow, event creation, weekly calendar view
- Task & PR scheduling modals with time slot detection and multi-slot booking
- Calendar-to-Jira worklog sync
- Floating bottom nav pill for route switching
- This chat widget for community contributions (changes → diff review → PR creation)

## Design System
The app uses a warm dark gray palette (#202025 base), desaturated pastel accents (blue #7cacf8, purple #b89eff, green #6ee7a0, amber #f5d06b), and avoids card-heavy layouts in favor of open spacing with subtle dividers. Buttons use transparent accent backgrounds on hover.

## What the User Can Ask You
They may ask you to add features, fix bugs, improve UI, refactor code, or anything else. You have full access to the codebase. After making changes, the user can review the diff and create a PR directly from this chat.

Always prefer editing existing files over creating new ones. Follow the existing patterns (Svelte 5 runes, scoped CSS with design system variables, TypeScript strict). Do not add comments or docstrings to code you didn't change.]

`;


function genId(): string {
  return `msg-${Date.now()}-${messageCounter++}`;
}

export async function checkClaude(): Promise<boolean> {
  try {
    const info = await checkClaudeInstalled();
    claudeInstalled.set(info);
    claudeCheckDone.set(true);
    return true;
  } catch {
    claudeInstalled.set(null);
    claudeCheckDone.set(true);
    return false;
  }
}

export async function startSession(repoPath: string): Promise<void> {
  try {
    const sessionId = await startChatSession(repoPath);

    chatSession.set({
      sessionId,
      messages: [],
    });
    sessionHasContext = false;

    // Set up event listener for streaming
    if (unlisten) {
      unlisten();
    }

    unlisten = await listen<ChatStreamEvent>("chat-stream", (event) => {
      const { session_id, event_type, data } = event.payload;

      chatSession.update((s) => {
        if (!s || s.sessionId !== session_id) return s;

        if (event_type === "content") {
          const lastMsg = s.messages[s.messages.length - 1];
          if (lastMsg && lastMsg.role === "assistant" && lastMsg.isStreaming) {
            // Try to parse stream-json format
            let text = "";
            try {
              const parsed = JSON.parse(data);
              if (parsed.type === "assistant" && parsed.message?.content) {
                const blocks = parsed.message.content;
                text = blocks
                  .filter((b: { type: string }) => b.type === "text")
                  .map((b: { text: string }) => b.text)
                  .join("");
              } else if (parsed.type === "content_block_delta") {
                text = parsed.delta?.text || "";
              }
              // All other JSON event types (result, message_start, etc.) are ignored
            } catch {
              // Not JSON — use raw text as-is
              text = data;
            }
            if (text) {
              lastMsg.content += text;
            }
          }
        } else if (event_type === "done") {
          const lastMsg = s.messages[s.messages.length - 1];
          if (lastMsg && lastMsg.role === "assistant") {
            lastMsg.isStreaming = false;
          }
          chatLoading.set(false);
          // Auto-check for changes after response completes
          checkForChanges(s.sessionId);
        } else if (event_type === "error") {
          const lastMsg = s.messages[s.messages.length - 1];
          if (lastMsg && lastMsg.role === "assistant" && lastMsg.isStreaming) {
            lastMsg.content += `\n\n**Error:** ${data}`;
          }
        }

        return { ...s, messages: [...s.messages] };
      });
    });

    chatError.set(null);
  } catch (e) {
    chatError.set(String(e));
  }
}

export async function sendMessage(message: string): Promise<void> {
  const session = get(chatSession);
  if (!session) return;

  // On first message, prepend system context so Claude knows the codebase
  let messageToSend = message;
  if (!sessionHasContext) {
    messageToSend = SYSTEM_CONTEXT + message;
    sessionHasContext = true;
  }

  // Add user message (show original, not the context-prefixed version)
  const userMsg: ChatMessage = {
    id: genId(),
    role: "user",
    content: message,
    timestamp: new Date(),
    isStreaming: false,
  };

  // Add placeholder assistant message for streaming
  const assistantMsg: ChatMessage = {
    id: genId(),
    role: "assistant",
    content: "",
    timestamp: new Date(),
    isStreaming: true,
  };

  chatSession.update((s) => {
    if (!s) return s;
    return {
      ...s,
      messages: [...s.messages, userMsg, assistantMsg],
    };
  });

  chatLoading.set(true);
  chatError.set(null);

  try {
    await apiSendMessage(session.sessionId, messageToSend);
  } catch (e) {
    chatLoading.set(false);
    chatSession.update((s) => {
      if (!s) return s;
      const msgs = [...s.messages];
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant" && last.isStreaming) {
        last.content = `**Error:** ${String(e)}`;
        last.isStreaming = false;
      }
      return { ...s, messages: msgs };
    });
  }
}

export async function cancelMessage(): Promise<void> {
  const session = get(chatSession);
  if (!session) return;

  try {
    await apiCancelMessage(session.sessionId);
  } catch {
    // Ignore cancel errors
  }

  chatLoading.set(false);
  chatSession.update((s) => {
    if (!s) return s;
    const msgs = [...s.messages];
    const last = msgs[msgs.length - 1];
    if (last && last.role === "assistant" && last.isStreaming) {
      last.content += "\n\n*(Cancelled)*";
      last.isStreaming = false;
    }
    return { ...s, messages: msgs };
  });
}

async function checkForChanges(sessionId: string): Promise<void> {
  try {
    const diff = await getChangesDiff(sessionId);
    if (diff.files.length > 0) {
      pendingDiff.set(diff);
    } else {
      pendingDiff.set(null);
    }
  } catch {
    // Ignore diff check errors
  }
}

export async function submitPR(
  title: string,
  body: string
): Promise<PRResult | null> {
  const session = get(chatSession);
  if (!session) return null;

  prSubmitting.set(true);
  try {
    const result = await apiSubmitPR(session.sessionId, title, body);
    pendingDiff.set(null);

    // Add system message about PR
    chatSession.update((s) => {
      if (!s) return s;
      return {
        ...s,
        messages: [
          ...s.messages,
          {
            id: genId(),
            role: "system" as const,
            content: `PR created: [#${result.number}](${result.url}) on branch \`${result.branch}\``,
            timestamp: new Date(),
            isStreaming: false,
          },
        ],
      };
    });

    return result;
  } catch (e) {
    chatError.set(String(e));
    return null;
  } finally {
    prSubmitting.set(false);
  }
}

export async function discardAllChanges(): Promise<void> {
  const session = get(chatSession);
  if (!session) return;

  try {
    await apiDiscardChanges(session.sessionId);
    pendingDiff.set(null);

    chatSession.update((s) => {
      if (!s) return s;
      return {
        ...s,
        messages: [
          ...s.messages,
          {
            id: genId(),
            role: "system" as const,
            content: "Changes discarded.",
            timestamp: new Date(),
            isStreaming: false,
          },
        ],
      };
    });
  } catch (e) {
    chatError.set(String(e));
  }
}

export async function stopSession(): Promise<void> {
  const session = get(chatSession);
  if (!session) return;

  try {
    await apiStopSession(session.sessionId);
  } catch {
    // Ignore
  }

  if (unlisten) {
    unlisten();
    unlisten = null;
  }

  chatSession.set(null);
  chatLoading.set(false);
  chatError.set(null);
  pendingDiff.set(null);
}
