<script lang="ts">
  import { onMount } from "svelte";
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import {
    calendars,
    calendarEvents,
    eventsLoading,
    eventsError,
    loadCalendars,
  } from "../lib/stores/calendar";
  import { loadConfig, config, saveConfig } from "../lib/stores/config";
  import {
    googleAccount,
    loadGoogleAuthStatus,
  } from "../lib/stores/google";
  import * as api from "../lib/api";
  import { googleAuthStart, googleAuthWait } from "../lib/api";
  import { hasEmbeddedCredentials, EMBEDDED_GOOGLE_CLIENT_ID, EMBEDDED_GOOGLE_CLIENT_SECRET } from "../lib/google-oauth";
  import type { CalendarEvent } from "../lib/api";

  let currentDate = $state(new Date());
  let viewMode = $state<"week" | "day">("week");

  let showConnect = $state(false);
  let connectLoading = $state(false);
  let connectError = $state<string | null>(null);
  let enabledCalendars = $state<Set<string>>(new Set());

  const hasAnyCreds = $derived(
    !!$config.google_client_id && !!$config.google_client_secret
  );

  async function handleAddClick() {
    showConnect = !showConnect;
  }


  async function handleSignIn() {
    connectLoading = true;
    connectError = null;
    showConnect = false;
    try {
      // If no user creds in config, seed from embedded
      if (!hasAnyCreds && hasEmbeddedCredentials()) {
        await saveConfig({
          ...$config,
          google_client_id: EMBEDDED_GOOGLE_CLIENT_ID,
          google_client_secret: EMBEDDED_GOOGLE_CLIENT_SECRET,
        });
      }
      const { auth_url } = await googleAuthStart();
      await openUrl(auth_url);
      const account = await googleAuthWait();
      googleAccount.set(account);
      await loadCalendars();
    } catch (err) {
      connectError = err instanceof Error ? err.message : String(err);
      showConnect = true;
    } finally {
      connectLoading = false;
    }
  }


  const START_HOUR = 7;
  const END_HOUR = 21;
  const HOURS = Array.from({ length: END_HOUR - START_HOUR }, (_, i) => i + START_HOUR);
  const HOUR_HEIGHT = 48;

  const weekStart = $derived.by(() => {
    const d = new Date(currentDate);
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    d.setDate(diff);
    d.setHours(0, 0, 0, 0);
    return d;
  });

  const weekDays = $derived.by(() => {
    const days: Date[] = [];
    for (let i = 0; i < 7; i++) {
      const d = new Date(weekStart);
      d.setDate(d.getDate() + i);
      days.push(d);
    }
    return days;
  });

  const weekEnd = $derived.by(() => {
    const d = new Date(weekStart);
    d.setDate(d.getDate() + 7);
    return d;
  });

  function formatDate(d: Date): string {
    return d.toISOString().split("T")[0];
  }

  function formatDayHeader(d: Date): string {
    return d.toLocaleDateString("en-US", { weekday: "short" });
  }

  function formatDayNumber(d: Date): string {
    return d.getDate().toString();
  }

  function isToday(d: Date): boolean {
    const today = new Date();
    return formatDate(d) === formatDate(today);
  }

  function formatHour(h: number): string {
    if (h === 0) return "12 AM";
    if (h < 12) return `${h} AM`;
    if (h === 12) return "12 PM";
    return `${h - 12} PM`;
  }

  function getEventStyle(event: CalendarEvent, dayDate: Date): { top: string; height: string } | null {
    const start = new Date(event.start_date);
    const end = new Date(event.end_date);
    const dayStr = formatDate(dayDate);
    const startStr = formatDate(start);
    const endStr = formatDate(end);

    if (startStr !== dayStr && endStr !== dayStr) return null;

    const startMinutes = startStr === dayStr ? start.getHours() * 60 + start.getMinutes() : START_HOUR * 60;
    const endMinutes = endStr === dayStr ? end.getHours() * 60 + end.getMinutes() : END_HOUR * 60;
    const clampedStart = Math.max(startMinutes, START_HOUR * 60);
    const clampedEnd = Math.min(endMinutes, END_HOUR * 60);
    const durationMinutes = Math.max(clampedEnd - clampedStart, 15);

    const top = ((clampedStart - START_HOUR * 60) / 60) * HOUR_HEIGHT;
    const height = (durationMinutes / 60) * HOUR_HEIGHT;

    return { top: `${top}px`, height: `${Math.max(height, 20)}px` };
  }

  function isAllDay(event: CalendarEvent): boolean {
    const start = new Date(event.start_date);
    const end = new Date(event.end_date);
    const durationHours = (end.getTime() - start.getTime()) / (1000 * 60 * 60);
    return durationHours >= 23 || (start.getHours() === 0 && start.getMinutes() === 0 && end.getHours() === 0 && end.getMinutes() === 0);
  }

  function getTimedEventsForDay(day: Date): CalendarEvent[] {
    const dayStr = formatDate(day);
    return $calendarEvents.filter((event) => {
      if (isAllDay(event)) return false;
      const startStr = event.start_date.split("T")[0];
      const endStr = event.end_date.split("T")[0];
      return startStr === dayStr || endStr === dayStr;
    });
  }

  function getAllDayEventsForDay(day: Date): CalendarEvent[] {
    const dayStr = formatDate(day);
    return $calendarEvents.filter((event) => {
      if (!isAllDay(event)) return false;
      const startStr = event.start_date.split("T")[0];
      const endStr = event.end_date.split("T")[0];
      return startStr <= dayStr && endStr >= dayStr;
    });
  }

  const hasAnyAllDayEvents = $derived(
    $calendarEvents.some(e => isAllDay(e))
  );

  function formatEventTime(event: CalendarEvent): string {
    const start = new Date(event.start_date);
    return start.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit", hour12: true });
  }

  function navigateWeek(direction: number) {
    const d = new Date(currentDate);
    d.setDate(d.getDate() + direction * 7);
    currentDate = d;
  }

  function goToToday() {
    currentDate = new Date();
  }

  function toggleCalendar(uid: string) {
    const next = new Set(enabledCalendars);
    if (next.has(uid)) {
      next.delete(uid);
    } else {
      next.add(uid);
    }
    enabledCalendars = next;
  }

  async function loadWeekEvents() {
    if (enabledCalendars.size === 0) {
      calendarEvents.set([]);
      return;
    }

    const start = formatDate(weekStart) + "T00:00:00";
    const end = formatDate(weekEnd) + "T23:59:59";

    eventsLoading.set(true);
    eventsError.set(null);
    try {
      const allEvents: CalendarEvent[] = [];
      for (const calId of enabledCalendars) {
        const events = await api.getEventsForDateRange(calId, start, end);
        allEvents.push(...events);
      }
      calendarEvents.set(allEvents);
    } catch (error) {
      eventsError.set(error instanceof Error ? error.message : String(error));
    } finally {
      eventsLoading.set(false);
    }
  }

  onMount(async () => {
    await loadConfig();
    await loadGoogleAuthStatus();

    if ($googleAccount) {
      await loadCalendars();
      // Enable the selected calendar by default, or all if none selected
      if ($config.selected_calendar) {
        enabledCalendars = new Set([$config.selected_calendar]);
      } else if ($calendars.length > 0) {
        enabledCalendars = new Set($calendars.map(c => c.uid));
      }
      await loadWeekEvents();
    }
  });

  $effect(() => {
    weekStart;
    enabledCalendars;
    if ($googleAccount && enabledCalendars.size > 0) {
      loadWeekEvents();
    }
  });
</script>

<div class="calendar-view">
  <div class="cal-header">
    <div class="cal-nav">
      <button class="nav-btn" onclick={() => navigateWeek(-1)}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
      </button>
      <button class="today-btn" onclick={goToToday}>Today</button>
      <button class="nav-btn" onclick={() => navigateWeek(1)}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>
    </div>
    <div class="cal-center">
      <h2 class="cal-title">
        {weekStart.toLocaleDateString("en-US", { month: "long", year: "numeric" })}
      </h2>
      {#if $googleAccount && $calendars.length > 0}
        <div class="cal-pills">
          {#each $calendars as cal}
            <button
              class="cal-pill"
              class:active={enabledCalendars.has(cal.uid)}
              onclick={() => toggleCalendar(cal.uid)}
              title={cal.name}
            >
              <span class="cal-dot" class:on={enabledCalendars.has(cal.uid)}></span>
              {cal.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>
    <div class="cal-actions">
      <div class="view-toggle">
        <button class:active={viewMode === "week"} onclick={() => (viewMode = "week")}>Week</button>
        <button class:active={viewMode === "day"} onclick={() => (viewMode = "day")}>Day</button>
      </div>
      <div class="add-anchor">
        <button
          class="add-btn"
          class:loading={connectLoading}
          onclick={handleAddClick}
          disabled={connectLoading}
          title="Add calendar account"
        >
          {#if connectLoading}
            <span class="btn-spinner"></span>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          {/if}
        </button>
    {#if showConnect}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="connect-backdrop" role="presentation" onclick={() => { showConnect = false; }}></div>
      <div class="connect-popover">
        <button class="google-btn" onclick={handleSignIn} disabled={connectLoading}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
            <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
            <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
            <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
          </svg>
          {connectLoading ? "Connecting..." : "Sign in with Google"}
        </button>
        {#if connectError}
          <p class="connect-error">{connectError}</p>
        {/if}
      </div>
    {/if}
      </div>
    </div>
  </div>

  {#if !$googleAccount}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
          <line x1="16" y1="2" x2="16" y2="6" />
          <line x1="8" y1="2" x2="8" y2="6" />
          <line x1="3" y1="10" x2="21" y2="10" />
        </svg>
      </div>
      <h3>Your calendar will show here</h3>
      <p>Press + to connect your Google account.</p>
    </div>
  {:else if $eventsLoading}
    <div class="loading">
      <div class="spinner"></div>
      <span>Loading events...</span>
    </div>
  {:else}
    <div class="week-grid">
      <div class="day-headers">
        <div class="time-gutter-header"></div>
        {#each viewMode === "week" ? weekDays : [currentDate] as day}
          <div class="day-header" class:today={isToday(day)}>
            <span class="day-name">{formatDayHeader(day)}</span>
            <span class="day-num" class:today={isToday(day)}>{formatDayNumber(day)}</span>
          </div>
        {/each}
      </div>

      {#if hasAnyAllDayEvents}
        <div class="allday-row">
          <div class="time-gutter-header allday-label">
            <span>all day</span>
          </div>
          {#each viewMode === "week" ? weekDays : [currentDate] as day}
            <div class="allday-cell" class:today={isToday(day)}>
              {#each getAllDayEventsForDay(day) as event}
                <div class="allday-event" title={event.summary}>
                  {event.summary}
                </div>
              {/each}
            </div>
          {/each}
        </div>
      {/if}

      <div class="grid-body">
        <div class="time-gutter">
          {#each HOURS as hour}
            <div class="time-label" style="height: {HOUR_HEIGHT}px">
              <span>{formatHour(hour)}</span>
            </div>
          {/each}
        </div>

        <div class="days-container">
          {#each viewMode === "week" ? weekDays : [currentDate] as day}
            <div class="day-column" class:today={isToday(day)}>
              {#each HOURS as _hour}
                <div class="hour-slot" style="height: {HOUR_HEIGHT}px"></div>
              {/each}

              {#each getTimedEventsForDay(day) as event}
                {@const style = getEventStyle(event, day)}
                {#if style}
                  <div
                    class="event-block"
                    style="top: {style.top}; height: {style.height}"
                    title={event.summary}
                  >
                    <span class="event-time">{formatEventTime(event)}</span>
                    <span class="event-title">{event.summary}</span>
                  </div>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>

    {#if $eventsError}
      <div class="error-banner">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        {$eventsError}
      </div>
    {/if}
  {/if}
</div>

<style>
  .calendar-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .cal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .cal-nav {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .cal-center {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .cal-pills {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .cal-pill {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border: none;
    background: transparent;
    border-radius: var(--radius-full);
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .cal-pill:hover {
    color: var(--text-secondary);
  }

  .cal-pill.active {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .cal-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-tertiary);
    transition: background 0.15s;
  }

  .cal-dot.on {
    background: var(--accent-blue);
  }

  .cal-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .add-anchor {
    position: relative;
    margin-left: 4px;
    z-index: 20;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .nav-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .today-btn {
    padding: 6px 14px;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .today-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .cal-title {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
    letter-spacing: -0.03em;
  }


  .view-toggle {
    display: flex;
    gap: 2px;
    background: var(--bg-surface);
    border-radius: 7px;
    padding: 2px;
    border: 1px solid var(--border-strong);
  }

  .view-toggle button {
    padding: 5px 12px;
    border: none;
    background: transparent;
    border-radius: 5px;
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .view-toggle button:hover {
    color: var(--text-primary);
  }

  .view-toggle button.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .empty-state {
    text-align: center;
    padding: 64px 24px;
    animation: fadeInUp 0.4s var(--ease-out);
  }

  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: var(--bg-elevated);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 20px;
    color: var(--text-tertiary);
  }

  .empty-state h3 {
    margin: 0 0 8px;
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-state p {
    margin: 0;
    color: var(--text-tertiary);
    font-size: 14px;
  }


  .add-btn {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    border: 1px dashed var(--border-strong);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s var(--ease-out);
    flex-shrink: 0;
  }

  .add-btn:hover:not(:disabled) {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
    background: var(--bg-hover);
    border-style: solid;
  }

  .add-btn.loading {
    border-style: solid;
    border-color: var(--accent-blue);
    color: var(--accent-blue);
  }

  .btn-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .connect-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .connect-popover {
    position: absolute;
    top: 38px;
    right: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 260px;
    padding: 14px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    animation: fadeInUp 0.15s var(--ease-out);
  }

  .connect-label {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .connect-inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .connect-inputs input {
    padding: 7px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
  }

  .connect-inputs input::placeholder {
    color: var(--text-tertiary);
  }

  .connect-inputs input:focus {
    border-color: var(--accent-blue);
  }

  .google-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 9px 14px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .google-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--text-tertiary);
  }

  .google-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .google-btn.small {
    padding: 7px 12px;
    font-size: 12px;
  }

  .advanced-toggle {
    display: flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    color: var(--text-tertiary);
    font-family: var(--font-body);
    font-size: 12px;
    cursor: pointer;
    padding: 0;
    transition: color 0.15s;
  }

  .advanced-toggle:hover {
    color: var(--text-secondary);
  }

  .connect-error {
    font-size: 12px;
    color: var(--accent-red);
    margin: 0;
  }

  .connect-hint {
    font-size: 11px;
    color: var(--text-tertiary);
    margin: 0;
  }

  .inline-link {
    background: none;
    border: none;
    color: var(--accent-blue);
    cursor: pointer;
    font: inherit;
    padding: 0;
  }

  .inline-link:hover {
    text-decoration: underline;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 64px 24px;
    color: var(--text-tertiary);
    font-size: 14px;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .week-grid {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--bg-surface);
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .allday-row {
    display: flex;
    border-bottom: 1px solid var(--border-subtle);
    min-height: 28px;
  }

  .allday-label {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 6px;
  }

  .allday-label span {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .allday-cell {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    padding: 3px 2px;
    border-right: 1px solid var(--border-subtle);
  }

  .allday-cell:last-child {
    border-right: none;
  }

  .allday-cell.today {
    background: rgba(124, 172, 248, 0.03);
  }

  .allday-event {
    font-size: 11px;
    font-weight: 500;
    color: var(--accent-blue);
    background: var(--accent-blue-dim);
    padding: 1px 6px;
    border-radius: 3px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .day-headers {
    display: flex;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(255, 255, 255, 0.01);
  }

  .time-gutter-header {
    width: 48px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-subtle);
  }

  .day-header {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 6px 4px;
    border-right: 1px solid var(--border-subtle);
  }

  .day-header:last-child {
    border-right: none;
  }

  .day-name {
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .day-num {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }

  .day-num.today {
    background: var(--accent-blue);
    color: white;
  }

  .day-header.today .day-name {
    color: var(--accent-blue);
  }

  .grid-body {
    display: flex;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    position: relative;
  }

  .time-gutter {
    width: 48px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-subtle);
  }

  .time-label {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding: 0 8px;
  }

  .time-label span {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    transform: translateY(-6px);
  }

  .days-container {
    display: flex;
    flex: 1;
  }

  .day-column {
    flex: 1;
    position: relative;
    border-right: 1px solid var(--border-subtle);
  }

  .day-column:last-child {
    border-right: none;
  }

  .day-column.today {
    background: rgba(124, 172, 248, 0.02);
  }

  .hour-slot {
    border-bottom: 1px solid var(--border-subtle);
  }

  .event-block {
    position: absolute;
    left: 2px;
    right: 2px;
    background: var(--accent-blue-dim);
    border-left: 3px solid var(--accent-blue);
    border-radius: 4px;
    padding: 3px 6px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    z-index: 1;
  }

  .event-block:hover {
    background: var(--accent-blue-glow);
  }

  .event-time {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--accent-blue);
    display: block;
  }

  .event-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--accent-red-dim);
    border-radius: var(--radius-md);
    color: var(--accent-red);
    font-size: 13px;
    margin-top: 16px;
    border: 1px solid rgba(248, 113, 113, 0.15);
  }
</style>
