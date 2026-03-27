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
      // Small delay to let DOM update
      setTimeout(scrollToBottom, 50);
    }
  });
</script>

{#if !isExpanded}
  <button class="chat-bubble" onclick={handleToggle} title="Claude Code Chat">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  </button>
{:else}
  <div class="chat-panel">
    <div class="chat-header">
      <div class="chat-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
        </svg>
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
          <button class="retry-btn" onclick={() => { claudeCheckDone.set(false); checkClaude(); }}>
            Retry
          </button>
        </div>
      {:else if !hasGhToken}
        <div class="onboarding">
          <p><strong>GitHub token not configured</strong></p>
          <p>Add a GitHub PAT in Settings to enable PR creation for community contributions.</p>
          <p>You can still use the chat without it.</p>
          <button class="retry-btn" onclick={async () => { hasGhToken = await hasGitHubToken(); if (!$chatSession) await startSession(repoPath); }}>
            Continue anyway
          </button>
        </div>
      {:else if !$chatSession}
        <div class="status-msg">Starting session...</div>
      {:else}
        {#if $chatSession.messages.length === 0}
          <div class="welcome">
            <p><strong>Ask Claude to make changes to Mira</strong></p>
            <p>Describe a feature, bug fix, or improvement. Claude will edit files in this repo. You can then review changes and create a PR.</p>
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
            <button class="send-btn cancel" onclick={cancelMessage}>Cancel</button>
          {:else}
            <button class="send-btn" onclick={handleSend} disabled={!inputText.trim()}>
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
    bottom: 24px;
    right: 24px;
    z-index: 1000;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: #0071e3;
    color: white;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 113, 227, 0.4);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .chat-bubble:hover {
    transform: scale(1.1);
    box-shadow: 0 6px 16px rgba(0, 113, 227, 0.5);
  }

  .chat-panel {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 1000;
    width: 420px;
    height: 600px;
    background: white;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: #1d1d1f;
    color: white;
  }

  .chat-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
  }

  .streaming-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #34c759;
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .chat-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    opacity: 0.7;
  }

  .icon-btn:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.1);
  }

  .chat-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .status-msg {
    color: #86868b;
    font-size: 13px;
    text-align: center;
    padding: 20px;
  }

  .onboarding {
    padding: 16px;
    background: #f5f5f7;
    border-radius: 12px;
    font-size: 13px;
    line-height: 1.5;
  }

  .onboarding code {
    display: block;
    background: #1d1d1f;
    color: #34c759;
    padding: 8px 12px;
    border-radius: 6px;
    margin: 8px 0;
    font-size: 12px;
  }

  .retry-btn {
    margin-top: 8px;
    padding: 6px 16px;
    background: #0071e3;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .welcome {
    padding: 16px;
    background: linear-gradient(135deg, #f0f4ff 0%, #f5f0ff 100%);
    border-radius: 12px;
    font-size: 13px;
    line-height: 1.5;
  }

  .message {
    max-width: 90%;
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
    background: #0071e3;
    color: white;
    border-bottom-right-radius: 4px;
  }

  .message.assistant .message-content {
    background: #f5f5f7;
    color: #1d1d1f;
    border-bottom-left-radius: 4px;
  }

  .message.system .message-content {
    background: #e8f5e9;
    color: #2e7d32;
    text-align: center;
    font-size: 12px;
    border-radius: 8px;
  }

  .cursor {
    animation: blink 0.8s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  .error-msg {
    background: #fff3f3;
    color: #ff3b30;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
  }

  .chat-input {
    border-top: 1px solid #e5e5e5;
    padding: 12px;
  }

  .chat-input textarea {
    width: 100%;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 13px;
    font-family: inherit;
    resize: none;
    outline: none;
    transition: border-color 0.2s;
  }

  .chat-input textarea:focus {
    border-color: #0071e3;
  }

  .chat-input textarea:disabled {
    background: #f5f5f7;
    color: #86868b;
  }

  .input-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .send-btn {
    padding: 6px 16px;
    background: #0071e3;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .send-btn.cancel {
    background: #86868b;
  }
</style>
