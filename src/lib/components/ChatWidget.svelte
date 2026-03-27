<script lang="ts">
  import {
    chatSession,
    chatLoading,
    chatError,
    claudeInstalled,
    claudeCheckDone,
    pendingDiff,
    checkClaude,
    startSession,
    sendMessage,
    cancelMessage,
    stopSession,
    discardAllChanges,
    submitPR,
  } from "../stores/chat";
  import { hasGitHubToken } from "../api";
  import DiffViewer from "./DiffViewer.svelte";
  import PRSubmitForm from "./PRSubmitForm.svelte";

  interface Props {
    repoPath: string;
  }

  let { repoPath }: Props = $props();

  let isExpanded = $state(false);
  let inputText = $state("");
  let messagesEl: HTMLDivElement | undefined = $state();
  let hasGhToken = $state(false);
  let showPRForm = $state(false);

  async function handleToggle() {
    isExpanded = !isExpanded;
    if (isExpanded && !$claudeCheckDone) {
      await checkClaude();
      hasGhToken = await hasGitHubToken();
      if ($claudeInstalled && !$chatSession) {
        await startSession(repoPath);
      }
    }
  }

  async function handleSend() {
    const msg = inputText.trim();
    if (!msg || $chatLoading) return;
    inputText = "";
    await sendMessage(msg);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  function scrollToBottom() {
    if (messagesEl) {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }
  }

  $effect(() => {
    if ($chatSession?.messages) {
      setTimeout(scrollToBottom, 50);
    }
  });
</script>

{#if !isExpanded}
  <button class="chat-bubble" onclick={handleToggle} title="Claude Code Chat">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  </button>
{:else}
  <div class="chat-panel">
    <div class="chat-header">
      <div class="chat-title">
        <div class="chat-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <span>Claude Code</span>
        {#if $chatLoading}
          <span class="streaming-indicator"></span>
        {/if}
      </div>
      <div class="chat-actions">
        {#if $chatSession}
          <button class="icon-btn" onclick={stopSession} title="End session">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" />
            </svg>
          </button>
        {/if}
        <button class="icon-btn" onclick={handleToggle} title="Minimize">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      </div>
    </div>

    <div class="chat-body" bind:this={messagesEl}>
      {#if !$claudeCheckDone}
        <div class="status-msg">Checking Claude Code CLI...</div>
      {:else if !$claudeInstalled}
        <div class="onboarding">
          <p><strong>Claude Code CLI not found</strong></p>
          <p>Install it to use the chat feature:</p>
          <code>npm install -g @anthropic-ai/claude-code</code>
          <p>Then reload the app.</p>
          <button class="btn-inline" onclick={() => { claudeCheckDone.set(false); checkClaude(); }}>
            Retry
          </button>
        </div>
      {:else if !hasGhToken}
        <div class="onboarding">
          <p><strong>GitHub token not configured</strong></p>
          <p>Add a GitHub PAT in Settings to enable PR creation for community contributions.</p>
          <p>You can still use the chat without it.</p>
          <button class="btn-inline" onclick={async () => { hasGhToken = await hasGitHubToken(); if (!$chatSession) await startSession(repoPath); }}>
            Continue anyway
          </button>
        </div>
      {:else if !$chatSession}
        <div class="status-msg">Starting session...</div>
      {:else}
        {#if $chatSession.messages.length === 0}
          <div class="welcome">
            <div class="welcome-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
              </svg>
            </div>
            <p><strong>Ask Claude to make changes to Mira</strong></p>
            <p class="sub">Describe a feature, bug fix, or improvement. Claude will edit files in this repo.</p>
          </div>
        {/if}

        {#each $chatSession.messages as msg (msg.id)}
          <div class="message {msg.role}">
            <div class="message-content">
              {#if msg.role === "system"}
                <em>{@html msg.content}</em>
              {:else}
                {msg.content}
              {/if}
              {#if msg.isStreaming}
                <span class="cursor">|</span>
              {/if}
            </div>
          </div>
        {/each}

        {#if $pendingDiff && !showPRForm}
          <DiffViewer
            diff={$pendingDiff}
            onCreatePR={() => { showPRForm = true; }}
            onDiscard={() => discardAllChanges()}
          />
        {/if}

        {#if showPRForm}
          <PRSubmitForm
            onSubmit={async (title, body) => {
              await submitPR(title, body);
              showPRForm = false;
            }}
            onCancel={() => { showPRForm = false; }}
          />
        {/if}
      {/if}

      {#if $chatError}
        <div class="error-msg">{$chatError}</div>
      {/if}
    </div>

    {#if $chatSession}
      <div class="chat-input">
        <textarea
          bind:value={inputText}
          onkeydown={handleKeydown}
          placeholder="Ask Claude to make changes..."
          rows="2"
          disabled={$chatLoading}
        ></textarea>
        <div class="input-actions">
          {#if $chatLoading}
            <button class="send-btn cancel" onclick={cancelMessage}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <rect x="3" y="3" width="18" height="18" rx="2" />
              </svg>
              Cancel
            </button>
          {:else}
            <button class="send-btn" onclick={handleSend} disabled={!inputText.trim()}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
              </svg>
              Send
            </button>
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .chat-bubble {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 1000;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    background: var(--bg-elevated);
    color: var(--accent-blue);
    border: 1px solid var(--border-default);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow-lg);
    transition: all 0.25s var(--ease-out);
  }

  .chat-bubble:hover {
    transform: translateY(-2px);
    border-color: var(--accent-blue);
    box-shadow: var(--shadow-lg), var(--shadow-glow-blue);
    background: var(--accent-blue-dim);
  }

  .chat-panel {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 1000;
    width: 400px;
    height: 560px;
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-default);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: fadeInUp 0.3s var(--ease-out);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border-subtle);
  }

  .chat-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .chat-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: var(--gradient-brand);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .streaming-indicator {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-green);
    animation: pulse 1.2s ease-in-out infinite;
  }

  .chat-actions {
    display: flex;
    gap: 2px;
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .chat-body {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .status-msg {
    color: var(--text-tertiary);
    font-size: 13px;
    text-align: center;
    padding: 20px;
  }

  .onboarding {
    padding: 14px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .onboarding strong {
    color: var(--text-primary);
  }

  .onboarding code {
    display: block;
    background: var(--bg-base);
    color: var(--accent-green);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    margin: 8px 0;
    font-family: var(--font-mono);
    font-size: 11px;
    border: 1px solid var(--border-subtle);
  }

  .btn-inline {
    margin-top: 8px;
    padding: 5px 14px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid rgba(91, 141, 239, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
  }

  .btn-inline:hover {
    background: rgba(91, 141, 239, 0.2);
  }

  .welcome {
    padding: 16px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    font-size: 13px;
    line-height: 1.5;
    text-align: center;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .welcome strong {
    color: var(--text-primary);
  }

  .welcome .sub {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .welcome-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--gradient-brand);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: white;
    margin-bottom: 8px;
  }

  .message {
    max-width: 88%;
    font-size: 13px;
    line-height: 1.5;
  }

  .message.user {
    align-self: flex-end;
  }

  .message.assistant {
    align-self: flex-start;
  }

  .message.system {
    align-self: center;
    max-width: 100%;
  }

  .message-content {
    padding: 8px 12px;
    border-radius: 12px;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .message.user .message-content {
    background: var(--accent-blue);
    color: white;
    border-bottom-right-radius: 4px;
  }

  .message.assistant .message-content {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    border-bottom-left-radius: 4px;
  }

  .message.system .message-content {
    background: var(--accent-green-dim);
    color: var(--accent-green);
    text-align: center;
    font-size: 12px;
    border-radius: var(--radius-sm);
  }

  .cursor {
    color: var(--accent-blue);
    animation: pulse 0.8s ease-in-out infinite;
  }

  .error-msg {
    background: var(--accent-red-dim);
    color: var(--accent-red);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    border: 1px solid rgba(248, 113, 113, 0.15);
  }

  .chat-input {
    border-top: 1px solid var(--border-subtle);
    padding: 10px;
    background: var(--bg-elevated);
  }

  .chat-input textarea {
    width: 100%;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-primary);
    background: var(--bg-surface);
    resize: none;
    outline: none;
    transition: border-color 0.15s;
  }

  .chat-input textarea::placeholder {
    color: var(--text-tertiary);
  }

  .chat-input textarea:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .chat-input textarea:disabled {
    background: var(--bg-base);
    color: var(--text-tertiary);
  }

  .input-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
  }

  .send-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 14px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid rgba(91, 141, 239, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    transition: all 0.15s;
  }

  .send-btn:hover:not(:disabled) {
    background: rgba(91, 141, 239, 0.2);
    box-shadow: var(--shadow-glow-blue);
  }

  .send-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .send-btn.cancel {
    background: var(--bg-hover);
    border-color: var(--border-default);
    color: var(--text-secondary);
  }

  .send-btn.cancel:hover {
    color: var(--text-primary);
    box-shadow: none;
  }
</style>
