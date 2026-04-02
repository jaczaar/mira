<script lang="ts">
  import { onMount } from "svelte";
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
    checkForChanges,
  } from "../stores/chat";
  import DiffViewer from "./DiffViewer.svelte";
  import PRSubmitForm from "./PRSubmitForm.svelte";

  interface Props {
    repoPath: string;
    embedded?: boolean;
  }

  let { repoPath, embedded = false }: Props = $props();

  let inputText = $state("");
  let messagesEl: HTMLDivElement | undefined = $state();
  let showPRForm = $state(false);

  onMount(async () => {
    if (embedded && !$claudeCheckDone) {
      await checkClaude();
      if ($claudeInstalled && !$chatSession) {
        await startSession(repoPath);
      }
    } else if (embedded && $claudeInstalled && !$chatSession) {
      await startSession(repoPath);
    }
  });

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

<div class="chat-page">
  <div class="chat-header">
    <div class="chat-title">
      <span>Claude Code</span>
      {#if $chatLoading}
        <span class="streaming-indicator"></span>
      {/if}
    </div>
    <div class="chat-actions">
      {#if $chatSession}
        <button class="icon-btn" onclick={() => checkForChanges($chatSession!.sessionId)} title="Check for changes">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12a9 9 0 11-6.22-8.56" />
            <polyline points="21 3 21 9 15 9" />
          </svg>
        </button>
        <button class="icon-btn" onclick={stopSession} title="End session">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

    <div class="chat-body" bind:this={messagesEl}>
      {#if !$claudeCheckDone}
        <div class="status-msg">Checking CLI...</div>
      {:else if !$claudeInstalled}
        <div class="onboarding">
          <p>Claude Code CLI required:</p>
          <code>npm install -g @anthropic-ai/claude-code</code>
          <button class="btn-inline" onclick={() => { claudeCheckDone.set(false); checkClaude(); }}>
            Retry
          </button>
        </div>
      {:else if !$chatSession}
        <div class="status-msg">Starting...</div>
      {:else}

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

<style>
  .chat-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    animation: fadeInUp 0.3s var(--ease-out);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
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
    font-size: 13px;
    text-align: center;
    color: var(--text-tertiary);
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
