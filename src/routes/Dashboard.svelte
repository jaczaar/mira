<script lang="ts">
  import { onMount } from "svelte";
  import Notifications from "../lib/components/Notifications.svelte";
  import StatusBadge from "../lib/components/StatusBadge.svelte";
  import Connectors from "../lib/components/Connectors.svelte";
  import TaskScheduler from "../lib/components/TaskScheduler.svelte";
  import { loadAssignedTasks, tasks, unscheduleTasks } from "../lib/stores/tasks";
  import { loadConfig, config, hasToken, saveConfig } from "../lib/stores/config";
  import { syncState, syncTasksToCalendar } from "../lib/stores/sync";
  import { googleAccounts } from "../lib/stores/google";
  import { accountCalendars } from "../lib/stores/calendar";
  import type { SyncedTask } from "../lib/stores/tasks";
  import type { ScheduleWindow } from "../lib/api";

  const calendarColors = [
    { id: null, name: "Default", color: "#4285f4" },
    { id: "1", name: "Lavender", color: "#7986cb" },
    { id: "2", name: "Sage", color: "#33b679" },
    { id: "3", name: "Grape", color: "#8e24aa" },
    { id: "4", name: "Flamingo", color: "#e67c73" },
    { id: "5", name: "Banana", color: "#f6bf26" },
    { id: "6", name: "Tangerine", color: "#f4511e" },
    { id: "7", name: "Peacock", color: "#039be5" },
    { id: "8", name: "Graphite", color: "#616161" },
    { id: "9", name: "Blueberry", color: "#3f51b5" },
    { id: "10", name: "Basil", color: "#0b8043" },
    { id: "11", name: "Tomato", color: "#d50000" },
  ];

  function colorHexFor(id: string | null): string {
    return calendarColors.find((c) => c.id === id)?.color ?? "#4285f4";
  }

  let taskToSchedule = $state<SyncedTask | null>(null);

  type ViewFilter = "all" | "unscheduled" | "scheduled";
  let viewFilter = $state<ViewFilter>("all");
  let searchQuery = $state("");

  // Multi-select
  let selectedKeys = $state<Set<string>>(new Set());
  let unschedulingBatch = $state(false);

  function toggleSelect(key: string) {
    const next = new Set(selectedKeys);
    if (next.has(key)) next.delete(key); else next.add(key);
    selectedKeys = next;
  }

  async function handleUnscheduleSelected() {
    const scheduledKeys = [...selectedKeys].filter(k => $tasks.find(t => t.key === k)?.calendar_event_uid);
    if (scheduledKeys.length === 0) return;
    unschedulingBatch = true;
    try {
      await unscheduleTasks(scheduledKeys);
      selectedKeys = new Set();
    } finally {
      unschedulingBatch = false;
    }
  }

  // Preferences popover
  let showPrefs = $state(false);
  let schedulingStrategy = $state<"earliest_available" | "priority_weighted">("earliest_available");
  let allowTaskSplitting = $state(true);
  let accountScheduleWindows = $state<Record<string, ScheduleWindow>>({});

  onMount(async () => {
    await loadConfig();
    if ($hasToken) {
      await loadAssignedTasks($config.jql_filter || undefined);
    }
    schedulingStrategy = $config.scheduling_strategy ?? "earliest_available";
    allowTaskSplitting = $config.allow_task_splitting ?? true;
    accountScheduleWindows = $config.account_schedule_windows ?? {};
  });

  const activeTasks = $derived($tasks.filter((t) => t.status_category !== "done"));
  const scheduledCount = $derived(activeTasks.filter((t) => t.calendar_event_uid).length);
  const unscheduledCount = $derived(activeTasks.filter((t) => !t.calendar_event_uid).length);

  const filteredTasks = $derived.by(() => {
    let list = activeTasks;
    if (viewFilter === "scheduled") list = list.filter((t) => t.calendar_event_uid);
    if (viewFilter === "unscheduled") list = list.filter((t) => !t.calendar_event_uid);

    if (!searchQuery.trim()) return list;
    const q = searchQuery.toLowerCase();
    return list.filter((t) =>
      t.key.toLowerCase().includes(q) ||
      t.summary.toLowerCase().includes(q) ||
      t.project_name.toLowerCase().includes(q) ||
      (t.priority || "").toLowerCase().includes(q)
    );
  });

  const canSync = $derived($hasToken && !!$config.selected_calendar && $googleAccounts.length > 0);

  async function handleSync() {
    if (!canSync) return;
    await loadAssignedTasks($config.jql_filter || undefined);
    try { await syncTasksToCalendar(); } catch (e) { console.error("Sync failed:", e); }
  }

  async function handleRefreshOnly() {
    if (!$hasToken) return;
    await loadAssignedTasks($config.jql_filter || undefined);
  }

  async function handleCalendarChange(calUid: string) {
    await saveConfig({ ...$config, selected_calendar: calUid });
  }

  async function handleColorChange(colorId: string | null) {
    await saveConfig({ ...$config, default_event_color: colorId });
  }

  async function handleSaveSchedulingConfig() {
    await saveConfig({
      ...$config,
      scheduling_strategy: schedulingStrategy,
      allow_task_splitting: allowTaskSplitting,
      account_schedule_windows: accountScheduleWindows,
    });
  }

  const dayNames = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  function ensureAccountWindow(email: string): void {
    if (!accountScheduleWindows[email]) {
      accountScheduleWindows[email] = { start_hour: 9, end_hour: 17, days: [1, 2, 3, 4, 5] };
      handleSaveSchedulingConfig();
    }
  }

  function toggleDay(email: string, day: number): void {
    ensureAccountWindow(email);
    const win = accountScheduleWindows[email];
    const idx = win.days.indexOf(day);
    if (idx >= 0) win.days.splice(idx, 1);
    else { win.days.push(day); win.days.sort(); }
    accountScheduleWindows = { ...accountScheduleWindows };
    handleSaveSchedulingConfig();
  }

  function updateWindowHour(email: string, field: "start_hour" | "end_hour", value: number): void {
    ensureAccountWindow(email);
    accountScheduleWindows[email][field] = value;
    accountScheduleWindows = { ...accountScheduleWindows };
    handleSaveSchedulingConfig();
  }

  function hourOptions(): { value: number; label: string }[] {
    const opts: { value: number; label: string }[] = [];
    for (let h = 0; h < 24; h++) {
      const ampm = h < 12 ? "AM" : "PM";
      const display = h === 0 ? 12 : h > 12 ? h - 12 : h;
      opts.push({ value: h, label: `${display}:00 ${ampm}` });
    }
    return opts;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") showPrefs = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="dashboard">
  <Notifications />

  <!-- Header: title + status + actions -->
  <div class="page-header">
    <div class="title-row">
      <h1>Mira</h1>
      <StatusBadge status={$syncState.status} message={$syncState.message} />

      <div class="header-actions">
        <button
          class="header-btn"
          onclick={handleRefreshOnly}
          disabled={!$hasToken || $syncState.status === "syncing"}
          title="Refresh tasks from Jira"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </button>
        <button
          class="header-btn primary"
          onclick={handleSync}
          disabled={!canSync || $syncState.status === "syncing"}
          title="Sync tasks to calendar"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
          </svg>
          {$syncState.status === "syncing" ? "Syncing..." : "Sync"}
        </button>

        <div class="prefs-anchor">
          <button
            class="header-btn"
            class:active={showPrefs}
            onclick={() => showPrefs = !showPrefs}
            title="Preferences"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </button>

          {#if showPrefs}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div class="prefs-backdrop" role="presentation" onclick={() => showPrefs = false}></div>
            <div class="prefs-panel">
              <div class="prefs-section">
                <div class="prefs-label">Calendar</div>
                <select
                  class="prefs-select"
                  value={$config.selected_calendar ?? ""}
                  onchange={(e) => handleCalendarChange((e.target as HTMLSelectElement).value)}
                >
                  <option value="" disabled>Select calendar</option>
                  {#each $googleAccounts as account}
                    {#each ($accountCalendars.get(account.email) ?? []) as cal}
                      <option value={cal.uid}>
                        {cal.name}{$googleAccounts.length > 1 ? ` (${account.email})` : ""}
                      </option>
                    {/each}
                  {/each}
                </select>
              </div>

              <div class="prefs-section">
                <div class="prefs-label">Event Color</div>
                <div class="color-row">
                  <span class="color-dot-preview" style="background: {colorHexFor($config.default_event_color ?? null)}"></span>
                  <select
                    class="prefs-select"
                    value={$config.default_event_color ?? ""}
                    onchange={(e) => { const v = (e.target as HTMLSelectElement).value; handleColorChange(v || null); }}
                  >
                    {#each calendarColors as color}
                      <option value={color.id ?? ""}>{color.name}</option>
                    {/each}
                  </select>
                </div>
              </div>

              <div class="prefs-divider"></div>

              <div class="prefs-section">
                <div class="prefs-label">Strategy</div>
                <select
                  class="prefs-select"
                  bind:value={schedulingStrategy}
                  onchange={handleSaveSchedulingConfig}
                >
                  <option value="earliest_available">Earliest Available</option>
                  <option value="priority_weighted">Priority Weighted</option>
                </select>
              </div>

              <label class="prefs-checkbox">
                <input type="checkbox" bind:checked={allowTaskSplitting} onchange={handleSaveSchedulingConfig} />
                Allow task splitting
              </label>

              {#if $googleAccounts.length > 0}
                <div class="prefs-divider"></div>
                <div class="prefs-section">
                  <div class="prefs-label">Work Hours</div>
                  {#each $googleAccounts as account}
                    {@const email = account.email}
                    {@const win = accountScheduleWindows[email] ?? { start_hour: 9, end_hour: 17, days: [1, 2, 3, 4, 5] }}
                    <div class="work-hours-block">
                      {#if $googleAccounts.length > 1}
                        <span class="work-hours-email">{email}</span>
                      {/if}
                      <div class="work-hours-times">
                        <select value={win.start_hour} onchange={(e) => updateWindowHour(email, "start_hour", Number((e.target as HTMLSelectElement).value))}>
                          {#each hourOptions() as opt}
                            <option value={opt.value}>{opt.label}</option>
                          {/each}
                        </select>
                        <span class="time-sep">-</span>
                        <select value={win.end_hour} onchange={(e) => updateWindowHour(email, "end_hour", Number((e.target as HTMLSelectElement).value))}>
                          {#each hourOptions() as opt}
                            <option value={opt.value}>{opt.label}</option>
                          {/each}
                        </select>
                      </div>
                      <div class="work-hours-days">
                        {#each dayNames as name, i}
                          <button class="day-chip" class:active={win.days.includes(i)} onclick={() => toggleDay(email, i)}>{name}</button>
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}

              <div class="prefs-divider"></div>

              <div class="prefs-section">
                <div class="prefs-label">Integrations</div>
                <Connectors />
              </div>

              <!-- batch unschedule is handled via multi-select in the task list -->
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>

  {#if $syncState.status === "syncing"}
    <div class="sync-progress">
      <div class="sync-progress-bar">
        <div class="sync-progress-fill" style="width: {($syncState.progress / Math.max($syncState.total, 1)) * 100}%"></div>
      </div>
      <span class="sync-progress-text">{$syncState.progress}/{$syncState.total}</span>
    </div>
  {/if}

  {#if !$hasToken}
    <div class="setup-prompt">
      <div class="setup-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
        </svg>
      </div>
      <h2>Welcome to Mira</h2>
      <p>Connect your tools to get started.</p>
      <p class="sub">Jira for tasks, Google Calendar for scheduling.</p>
      <div style="margin-top: 16px;"><Connectors /></div>
    </div>
  {:else}
    {#if !canSync}
      <div class="setup-hint">
        {#if $googleAccounts.length === 0}
          Connect Google Calendar to enable sync.
        {:else if !$config.selected_calendar}
          Select a calendar in preferences to enable sync.
        {/if}
        <button class="setup-hint-btn" onclick={() => showPrefs = true}>Open preferences</button>
      </div>
    {/if}

    <!-- Search + Filters -->
    <div class="toolbar">
      <div class="filter-pills">
        <button class="pill" class:active={viewFilter === "all"} onclick={() => viewFilter = "all"}>
          All <span class="pill-count">{activeTasks.length}</span>
        </button>
        <button class="pill" class:active={viewFilter === "unscheduled"} onclick={() => viewFilter = "unscheduled"}>
          Unscheduled <span class="pill-count">{unscheduledCount}</span>
        </button>
        <button class="pill" class:active={viewFilter === "scheduled"} onclick={() => viewFilter = "scheduled"}>
          Scheduled <span class="pill-count">{scheduledCount}</span>
        </button>
      </div>
      <div class="search-box">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input type="search" placeholder="Filter..." bind:value={searchQuery} />
        {#if searchQuery}
          <button class="search-clear" onclick={() => searchQuery = ""}>
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        {/if}
      </div>
    </div>

    <!-- Task List -->
    {#if filteredTasks.length === 0}
      <div class="empty">
        {#if searchQuery}
          <p>No tasks match "{searchQuery}"</p>
        {:else}
          <p>No tasks to show</p>
        {/if}
      </div>
    {:else}
      <div class="task-list">
        {#each filteredTasks as task, i (task.id)}
          <div
            class="task-row"
            class:selected={selectedKeys.has(task.key)}
            style="animation: fadeInUp 0.25s var(--ease-out) {Math.min(i, 12) * 25}ms both"
          >
            <button
              class="select-check"
              class:checked={selectedKeys.has(task.key)}
              onclick={() => toggleSelect(task.key)}
            >
              {#if selectedKeys.has(task.key)}
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              {/if}
            </button>
            <div class="task-body">
              <div class="task-top">
                <a href={task.url} target="_blank" rel="noopener" class="task-key">{task.key}</a>
                <span class="task-status">{task.status}</span>
                {#if task.priority}
                  <span class="task-priority">{task.priority}</span>
                {/if}
                {#if task.calendar_event_uid}
                  <span class="scheduled-dot" title="Scheduled"></span>
                {/if}
              </div>
              <div class="task-summary">{task.summary}</div>
              <div class="task-meta">
                <span>{task.project_name}</span>
                {#if task.time_estimate_seconds}
                  {@const h = Math.floor(task.time_estimate_seconds / 3600)}
                  {@const m = Math.floor((task.time_estimate_seconds % 3600) / 60)}
                  <span class="task-est">{h > 0 ? `${h}h ` : ""}{m > 0 ? `${m}m` : ""}</span>
                {/if}
                {#if task.due_date}
                  <span class="task-due">Due {task.due_date}</span>
                {/if}
              </div>
            </div>
            <div class="task-actions">
              <button class="task-action-btn primary" onclick={() => taskToSchedule = task}>
                {task.calendar_event_uid ? "Reschedule" : "Schedule"}
              </button>
            </div>
          </div>
        {/each}
      </div>

      {#if selectedKeys.size > 0}
        {@const selectedScheduled = [...selectedKeys].filter(k => activeTasks.find(t => t.key === k)?.calendar_event_uid).length}
        <div class="selection-bar">
          <span class="selection-count">{selectedKeys.size} selected</span>
          <button class="selection-action" onclick={() => selectedKeys = new Set()}>Clear</button>
          {#if selectedScheduled > 0}
            <button
              class="selection-action danger"
              onclick={handleUnscheduleSelected}
              disabled={unschedulingBatch}
            >
              {unschedulingBatch ? "Removing..." : `Unschedule${selectedScheduled < selectedKeys.size ? ` (${selectedScheduled})` : ""}`}
            </button>
          {/if}
        </div>
      {/if}
    {/if}
  {/if}
</div>

{#if taskToSchedule}
  <TaskScheduler
    task={taskToSchedule}
    onClose={() => taskToSchedule = null}
    onScheduled={() => console.log("Task scheduled")}
  />
{/if}

<style>
  .dashboard {
    max-width: 900px;
    margin: 0 auto;
  }

  /* Header */
  .page-header {
    margin-bottom: 20px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  h1 {
    font-family: var(--font-display);
    font-size: 26px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
    letter-spacing: -0.04em;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
  }

  .header-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 6px 10px;
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

  .header-btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--bg-elevated);
  }

  .header-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .header-btn.primary {
    background: var(--accent-blue-dim);
    border-color: rgba(124, 172, 248, 0.2);
    color: var(--accent-blue);
  }

  .header-btn.primary:hover:not(:disabled) {
    background: rgba(124, 172, 248, 0.18);
    border-color: rgba(124, 172, 248, 0.35);
    box-shadow: var(--shadow-glow-blue);
  }

  .header-btn.active {
    color: var(--accent-blue);
    border-color: rgba(124, 172, 248, 0.3);
    background: var(--accent-blue-dim);
  }

  /* Sync progress */
  .sync-progress {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
    animation: fadeIn 0.2s var(--ease-out);
  }

  .sync-progress-bar {
    flex: 1;
    height: 3px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }

  .sync-progress-fill {
    height: 100%;
    background: var(--gradient-brand);
    transition: width 0.4s var(--ease-out);
    border-radius: 2px;
  }

  .sync-progress-text {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    min-width: 36px;
    text-align: right;
  }

  /* Preferences popover */
  .prefs-anchor {
    position: relative;
  }

  .prefs-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .prefs-panel {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 100;
    width: 320px;
    max-height: 480px;
    overflow-y: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 12px;
    animation: fadeInUp 0.15s var(--ease-out);
  }

  .prefs-section {
    margin-bottom: 10px;
  }

  .prefs-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
    margin-bottom: 5px;
  }

  .prefs-select {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-primary);
    background: var(--bg-surface);
    color-scheme: dark;
  }

  .prefs-select:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .color-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .color-dot-preview {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .color-row .prefs-select {
    flex: 1;
  }

  .prefs-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 10px 0;
  }

  .prefs-checkbox {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 0;
  }

  .prefs-checkbox input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: var(--accent-blue);
  }

  /* Work hours in prefs */
  .work-hours-block {
    padding: 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    margin-bottom: 6px;
    background: var(--bg-surface);
  }

  .work-hours-email {
    display: block;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .work-hours-times {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-bottom: 6px;
  }

  .work-hours-times select {
    padding: 4px 6px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    color: var(--text-primary);
    background: var(--bg-elevated);
    color-scheme: dark;
  }

  .time-sep {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .work-hours-days {
    display: flex;
    gap: 2px;
  }

  .day-chip {
    padding: 2px 6px;
    font-size: 10px;
    font-weight: 500;
    font-family: var(--font-body);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s;
  }

  .day-chip:hover {
    color: var(--text-secondary);
    border-color: var(--border-strong);
  }

  .day-chip.active {
    background: var(--accent-blue-dim);
    border-color: rgba(124, 172, 248, 0.3);
    color: var(--accent-blue);
  }

  /* Setup */
  .setup-prompt {
    padding: 56px 40px;
    text-align: center;
    animation: fadeInUp 0.4s var(--ease-out);
  }

  .setup-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: var(--gradient-brand);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 20px;
    color: white;
  }

  .setup-prompt h2 {
    margin: 0 0 8px;
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .setup-prompt p {
    margin: 4px 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .setup-prompt .sub {
    color: var(--text-tertiary);
    font-size: 14px;
  }

  .setup-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    margin-bottom: 16px;
    background: var(--accent-amber-dim);
    border: 1px solid rgba(245, 208, 107, 0.15);
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--accent-amber);
    animation: fadeInUp 0.2s var(--ease-out);
  }

  .setup-hint-btn {
    margin-left: auto;
    padding: 3px 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: rgba(245, 208, 107, 0.15);
    color: var(--accent-amber);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }

  .setup-hint-btn:hover {
    background: rgba(245, 208, 107, 0.25);
  }

  /* Toolbar: filters + search */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }

  .filter-pills {
    display: flex;
    gap: 2px;
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 11px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .pill:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  .pill.active {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .pill-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .pill.active .pill-count {
    color: var(--text-secondary);
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    max-width: 220px;
    margin-left: auto;
    padding: 0 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    transition: border-color 0.15s;
  }

  .search-box:focus-within {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .search-box svg {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-box input {
    flex: 1;
    padding: 6px 0;
    border: none;
    background: transparent;
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-primary);
    outline: none;
  }

  .search-box input::placeholder {
    color: var(--text-tertiary);
  }

  .search-clear {
    padding: 2px;
    background: var(--bg-hover);
    border: none;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    color: var(--text-tertiary);
  }

  .search-clear:hover {
    color: var(--text-primary);
  }

  /* Empty state */
  .empty {
    text-align: center;
    padding: 48px 24px;
    color: var(--text-tertiary);
    font-size: 13px;
  }

  /* Task list */
  .task-list {
    display: flex;
    flex-direction: column;
  }

  .task-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 2px;
    border-bottom: 1px solid var(--border-subtle);
    transition: background 0.12s var(--ease-out);
  }

  .task-row:first-child {
    border-top: 1px solid var(--border-subtle);
  }

  .task-row:hover {
    background: var(--bg-hover);
  }

  .task-indicator {
    width: 3px;
    min-height: 32px;
    border-radius: 2px;
    flex-shrink: 0;
    align-self: stretch;
    background: var(--accent-blue);
    opacity: 0.4;
    transition: opacity 0.15s;
  }

  .task-body {
    flex: 1;
    min-width: 0;
  }

  .task-top {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 3px;
    flex-wrap: wrap;
  }

  .task-key {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--accent-blue);
    text-decoration: none;
    letter-spacing: 0.01em;
  }

  .task-key:hover {
    text-decoration: underline;
  }

  .task-status {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: var(--bg-hover);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .task-priority {
    font-size: 10px;
    color: var(--accent-amber);
  }

  .scheduled-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-green);
  }

  .task-summary {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
    margin-bottom: 4px;
  }

  .task-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .task-est {
    font-family: var(--font-mono);
  }

  .task-due {
    color: var(--accent-amber);
  }

  .task-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
    align-self: center;
  }

  .task-action-btn {
    padding: 5px 10px;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
    background: transparent;
    color: var(--text-tertiary);
  }

  .task-action-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .task-action-btn.primary {
    color: var(--accent-blue);
  }

  .task-action-btn.primary:hover {
    background: var(--accent-blue-dim);
  }

  .task-action-btn.ghost:hover {
    color: var(--accent-red);
    background: var(--accent-red-dim);
  }

  /* Multi-select checkboxes */
  .select-check {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: 1.5px solid var(--border-strong);
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    align-self: center;
    transition: all 0.12s var(--ease-out);
    padding: 0;
    color: white;
  }

  .select-check:hover {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
  }

  .select-check.checked {
    background: var(--accent-blue);
    border-color: var(--accent-blue);
  }

  .task-row.selected {
    background: var(--accent-blue-dim);
  }

  /* Floating selection bar */
  .selection-bar {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 90;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: fadeInUp 0.2s var(--ease-out);
  }

  .selection-count {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .selection-action {
    padding: 5px 12px;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    background: transparent;
    color: var(--text-secondary);
    transition: all 0.12s var(--ease-out);
  }

  .selection-action:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .selection-action.danger {
    background: var(--accent-red-dim);
    color: var(--accent-red);
  }

  .selection-action.danger:hover:not(:disabled) {
    background: rgba(240, 144, 144, 0.2);
  }

  .selection-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
