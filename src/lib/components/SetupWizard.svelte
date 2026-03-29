<script lang="ts">
  import {
    setupStep,
    setupProgress,
    setupError,
    runSetup,
    checkWorkspaceStatus,
  } from "../stores/workspace";
  import type { SetupStep } from "../stores/workspace";

  let copied = $state(false);

  const steps: { id: string; label: string; needs: SetupStep }[] = [
    { id: "claude", label: "Claude Code CLI", needs: "NEED_CLAUDE" },
    { id: "node", label: "Node.js Runtime", needs: "NEED_NODE" },
    { id: "repo", label: "Mira Repository", needs: "NEED_REPO" },
    { id: "npm", label: "Dependencies", needs: "NEED_NPM" },
  ];

  function getStepStatus(stepNeeds: SetupStep, currentStep: SetupStep): "done" | "active" | "pending" | "error" {
    const order: SetupStep[] = ["NEED_CLAUDE", "NEED_NODE", "NEED_REPO", "NEED_NPM", "READY"];
    const stepIdx = order.indexOf(stepNeeds);
    const currentIdx = order.indexOf(currentStep);

    if (currentStep === "ERROR") {
      // Mark the step that was active when error occurred
      const progress = $setupProgress;
      if (progress) {
        const activeStep = steps.find(s => s.id === progress.step);
        if (activeStep && activeStep.needs === stepNeeds) return "error";
        if (activeStep) {
          const activeIdx = order.indexOf(activeStep.needs);
          return stepIdx < activeIdx ? "done" : "pending";
        }
      }
      return stepIdx < currentIdx ? "done" : "pending";
    }

    if (currentStep === "READY") return "done";
    if (currentStep === "CHECKING") return "pending";
    if (stepIdx < currentIdx) return "done";
    if (stepIdx === currentIdx) return "active";
    return "pending";
  }

  async function handleCopy() {
    await navigator.clipboard.writeText("npm install -g @anthropic-ai/claude-code");
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }

  async function handleRetry() {
    await checkWorkspaceStatus();
    if ($setupStep !== "NEED_CLAUDE") {
      await runSetup();
    }
  }

  async function handleRecheck() {
    await checkWorkspaceStatus();
    if ($setupStep !== "NEED_CLAUDE") {
      await runSetup();
    }
  }
</script>

<div class="wizard">
  <div class="wizard-header">
    <div class="wizard-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" />
      </svg>
    </div>
    <h2>Setting up Edit Mode</h2>
    <p>This only happens once. We'll set up everything you need to contribute.</p>
  </div>

  <div class="steps">
    {#each steps as step}
      {@const status = getStepStatus(step.needs, $setupStep)}
      <div class="step" class:done={status === "done"} class:active={status === "active"} class:error={status === "error"}>
        <div class="step-icon">
          {#if status === "done"}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          {:else if status === "active"}
            <span class="spinner"></span>
          {:else if status === "error"}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          {:else}
            <span class="dot"></span>
          {/if}
        </div>
        <div class="step-content">
          <span class="step-label">{step.label}</span>
          {#if status === "active" && $setupProgress?.step === step.id}
            <span class="step-detail">{$setupProgress.message}</span>
            {#if $setupProgress.percent > 0 && $setupProgress.percent < 100}
              <div class="progress-bar">
                <div class="progress-fill" style="width: {$setupProgress.percent}%"></div>
              </div>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if $setupStep === "NEED_CLAUDE"}
    <div class="claude-help">
      <p>Claude Code CLI is required. Install it by running:</p>
      <div class="code-block">
        <code>npm install -g @anthropic-ai/claude-code</code>
        <button class="copy-btn" onclick={handleCopy}>
          {copied ? "Copied!" : "Copy"}
        </button>
      </div>
      <button class="action-btn" onclick={handleRecheck}>
        I've installed it
      </button>
    </div>
  {:else if $setupStep === "ERROR"}
    <div class="error-block">
      <p>{$setupError}</p>
      <button class="action-btn" onclick={handleRetry}>Retry</button>
    </div>
  {:else if $setupStep === "CHECKING"}
    <div class="checking">
      <span class="spinner"></span>
      <span>Checking your setup...</span>
    </div>
  {/if}
</div>

<style>
  .wizard {
    max-width: 480px;
    margin: 0 auto;
    padding: 48px 24px;
    animation: fadeInUp 0.4s var(--ease-out);
  }

  .wizard-header {
    text-align: center;
    margin-bottom: 32px;
  }

  .wizard-icon {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 16px;
  }

  .wizard-header h2 {
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 8px;
  }

  .wizard-header p {
    font-size: 14px;
    color: var(--text-tertiary);
    margin: 0;
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 24px;
  }

  .step {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    transition: all 0.2s var(--ease-out);
  }

  .step.active {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
  }

  .step.done {
    opacity: 0.7;
  }

  .step.error {
    border-color: var(--accent-red);
    background: var(--accent-red-dim);
  }

  .step-icon {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .step.done .step-icon {
    color: var(--accent-green);
  }

  .step.error .step-icon {
    color: var(--accent-red);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--border-strong);
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .step-content {
    flex: 1;
    min-width: 0;
  }

  .step-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .step-detail {
    display: block;
    font-size: 11px;
    color: var(--text-tertiary);
    margin-top: 2px;
  }

  .progress-bar {
    height: 3px;
    background: var(--border-default);
    border-radius: 2px;
    margin-top: 6px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-blue);
    border-radius: 2px;
    transition: width 0.3s var(--ease-out);
  }

  .claude-help {
    text-align: center;
    padding: 20px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .claude-help p {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0 0 12px;
  }

  .code-block {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-base);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    margin-bottom: 16px;
  }

  .code-block code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent-purple);
    user-select: all;
  }

  .copy-btn {
    padding: 4px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-family: var(--font-body);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn {
    padding: 8px 20px;
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

  .action-btn:hover {
    opacity: 0.9;
    box-shadow: var(--shadow-glow-blue);
  }

  .error-block {
    text-align: center;
    padding: 16px;
    background: var(--accent-red-dim);
    border: 1px solid var(--accent-red);
    border-radius: var(--radius-md);
  }

  .error-block p {
    font-size: 13px;
    color: var(--accent-red);
    margin: 0 0 12px;
  }

  .checking {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 20px;
    font-size: 13px;
    color: var(--text-tertiary);
  }
</style>
