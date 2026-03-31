<script lang="ts">
  import { onMount } from "svelte";
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import {
    calendars,
    calendarEvents,
    eventsLoading,
    eventsError,
    accountCalendars,
    loadCalendarsForAllAccounts,
    getCachedEvents,
    setCachedEvents,
  } from "../lib/stores/calendar";
  import { loadConfig, config, saveConfig } from "../lib/stores/config";
  import {
    googleAccounts,
    loadGoogleAuthStatus,
  } from "../lib/stores/google";
  import * as api from "../lib/api";
  import { googleAuthStart, googleAuthWait } from "../lib/api";
  import { hasEmbeddedCredentials, EMBEDDED_GOOGLE_CLIENT_ID, EMBEDDED_GOOGLE_CLIENT_SECRET } from "../lib/google-oauth";
  import { themeMode, type ThemeMode } from "../lib/stores/theme";
  import type { CalendarEvent } from "../lib/api";

  import { onDestroy } from "svelte";

  let currentDate = $state(new Date());
  let showCalendarMenu = $state(false);
  let now = $state(new Date());
  let nowLineInterval: ReturnType<typeof setInterval> | null = null;

  onDestroy(() => { if (nowLineInterval) clearInterval(nowLineInterval); });

  function setTheme(mode: ThemeMode) {
    themeMode.set(mode);
  }
  let viewMode = $state<"week" | "day">("week");

  // Richer event colors: higher opacity bg, solid border, bright text
  const EVENT_COLORS = [
    { bg: "rgba(59, 130, 246, 0.40)", border: "#3b82f6", text: "#a8c8f8", title: "#e0edfe", wash: "rgba(59, 130, 246, 0.06)" },   // blue
    { bg: "rgba(139, 92, 246, 0.40)", border: "#8b5cf6", text: "#c4b0f4", title: "#e4d8fe", wash: "rgba(139, 92, 246, 0.06)" },   // violet
    { bg: "rgba(236, 72, 153, 0.40)", border: "#ec4899", text: "#f0a0c4", title: "#fcd8e8", wash: "rgba(236, 72, 153, 0.06)" },   // pink
    { bg: "rgba(245, 158, 11, 0.40)", border: "#f59e0b", text: "#f0c870", title: "#fef0d0", wash: "rgba(245, 158, 11, 0.06)" },   // amber
    { bg: "rgba(16, 185, 129, 0.40)", border: "#10b981", text: "#80d8b0", title: "#c8f0de", wash: "rgba(16, 185, 129, 0.06)" },   // emerald
    { bg: "rgba(6, 182, 212, 0.40)", border: "#06b6d4", text: "#70c8dc", title: "#c8eef6", wash: "rgba(6, 182, 212, 0.06)" },     // cyan
    { bg: "rgba(244, 63, 94, 0.40)", border: "#f43f5e", text: "#f4a0b0", title: "#fcd0d8", wash: "rgba(244, 63, 94, 0.06)" },     // rose
    { bg: "rgba(34, 197, 94, 0.40)", border: "#22c55e", text: "#88dca4", title: "#ccf4d8", wash: "rgba(34, 197, 94, 0.06)" },     // green
  ];

  function hashString(s: string): number {
    let hash = 0;
    for (let i = 0; i < s.length; i++) {
      hash = ((hash << 5) - hash + s.charCodeAt(i)) | 0;
    }
    return Math.abs(hash);
  }

  function getCalendarColorIndex(calendarName: string): number {
    const custom = $config.calendar_colors?.[calendarName];
    if (custom !== undefined && custom >= 0 && custom < EVENT_COLORS.length) return custom;
    return hashString(calendarName) % EVENT_COLORS.length;
  }

  function getEventColor(event: CalendarEvent) {
    return EVENT_COLORS[getCalendarColorIndex(event.calendar_name)];
  }

  let colorPickerCal = $state<string | null>(null);

  async function setCalendarColor(calUid: string, colorIndex: number) {
    const updated = { ...$config, calendar_colors: { ...($config.calendar_colors ?? {}), [calUid]: colorIndex } };
    await saveConfig(updated);
    colorPickerCal = null;
  }

  let connectLoading = $state(false);
  let connectError = $state<string | null>(null);
  let enabledCalendars = $state<Set<string>>(new Set());

  const hasAnyCreds = $derived(
    !!$config.google_client_id && !!$config.google_client_secret
  );

  const hasAnyAccounts = $derived($googleAccounts.length > 0);

  function getAccountForCalendar(calUid: string): string | undefined {
    for (const [email, cals] of $accountCalendars) {
      if (cals.some(c => c.uid === calUid)) return email;
    }
    return undefined;
  }

  async function handleSignIn() {
    connectLoading = true;
    connectError = null;
    try {
      if (!hasAnyCreds && hasEmbeddedCredentials()) {
        await saveConfig({
          ...$config,
          google_client_id: EMBEDDED_GOOGLE_CLIENT_ID,
          google_client_secret: EMBEDDED_GOOGLE_CLIENT_SECRET,
        });
      }
      const { auth_url } = await googleAuthStart();
      await openUrl(auth_url);
      await googleAuthWait();
      await loadGoogleAuthStatus();
      await loadCalendarsForAllAccounts($googleAccounts.map(a => a.email));
      // Enable all calendars from the new account
      enabledCalendars = new Set([...enabledCalendars, ...$calendars.map(c => c.uid)]);
      saveConfig({ ...$config, enabled_calendars: [...enabledCalendars] });
    } catch (err) {
      connectError = err instanceof Error ? err.message : String(err);
    } finally {
      connectLoading = false;
    }
  }


  const GRID_OFFSET = 15; // start 15 min before the first hour so labels aren't cut off
  const FIXED_ZOOM_STEPS = [44, 48, 52, 56, 60, 66, 72, 80, 90, 100, 112, 126, 144];

  // Day range from config (persisted)
  const startHour = $derived($config.day_start_hour ?? 6);
  const endHour = $derived($config.day_end_hour ?? 23);
  const HOURS = $derived(Array.from({ length: endHour - startHour }, (_, i) => i + startHour));
  const totalHours = $derived(endHour - startHour);

  // Auto-fit zoom: smallest level fills the container without scrolling
  let gridHeight = $state(600); // measured from DOM
  const autoFitZoom = $derived(Math.floor((gridHeight - GRID_OFFSET) / (totalHours + GRID_OFFSET / 60)));
  const ZOOM_STEPS = $derived([autoFitZoom, ...FIXED_ZOOM_STEPS.filter(s => s > autoFitZoom)]);

  let zoomIndex = $state(3); // will be corrected on mount
  const HOUR_HEIGHT = $derived(ZOOM_STEPS[Math.min(zoomIndex, ZOOM_STEPS.length - 1)]);
  const nowMinutesCurrent = $derived(now.getHours() * 60 + now.getMinutes());
  const nowTopPx = $derived(((nowMinutesCurrent - startHour * 60) / 60) * HOUR_HEIGHT + (HOUR_HEIGHT * GRID_OFFSET / 60));

  function zoomIn() {
    if (zoomIndex < ZOOM_STEPS.length - 1) {
      zoomIndex++;
      saveConfig({ ...$config, calendar_zoom: zoomIndex });
    }
  }
  function zoomOut() {
    if (zoomIndex > 0) {
      zoomIndex--;
      saveConfig({ ...$config, calendar_zoom: zoomIndex });
    }
  }

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

  function getEventDurationMinutes(event: CalendarEvent): number {
    const start = new Date(event.start_date);
    const end = new Date(event.end_date);
    return (end.getTime() - start.getTime()) / (1000 * 60);
  }

  function formatEventTime(event: CalendarEvent): string {
    const d = new Date(event.start_date);
    const h = d.getHours();
    const m = d.getMinutes();
    const period = h >= 12 ? "PM" : "AM";
    const hour = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return m === 0 ? `${hour} ${period}` : `${hour}:${m.toString().padStart(2, "0")} ${period}`;
  }

  function getEventCountForDay(day: Date): number {
    return getTimedEventsForDay(day).length + getAllDayEventsForDay(day).length;
  }

  function isLocationEvent(event: CalendarEvent): boolean {
    const s = event.summary?.toLowerCase() ?? "";
    return s === "office" || s === "wfh" || s === "work from home" || s === "remote" || s === "home";
  }

  function getEventStyle(event: CalendarEvent, dayDate: Date): { top: string; height: string } | null {
    const start = new Date(event.start_date);
    const end = new Date(event.end_date);
    const dayStr = formatDate(dayDate);
    const startStr = formatDate(start);
    const endStr = formatDate(end);

    if (startStr !== dayStr && endStr !== dayStr) return null;

    const startMinutes = startStr === dayStr ? start.getHours() * 60 + start.getMinutes() : startHour * 60;
    const endMinutes = endStr === dayStr ? end.getHours() * 60 + end.getMinutes() : endHour * 60;
    const clampedStart = Math.max(startMinutes, startHour * 60);
    const clampedEnd = Math.min(endMinutes, endHour * 60);
    const durationMinutes = Math.max(clampedEnd - clampedStart, 15);

    const top = ((clampedStart - startHour * 60) / 60) * HOUR_HEIGHT + (HOUR_HEIGHT * GRID_OFFSET / 60) + 1;
    const height = (durationMinutes / 60) * HOUR_HEIGHT - 2;

    return { top: `${top}px`, height: `${height}px` };
  }

  function isAllDay(event: CalendarEvent): boolean {
    const start = new Date(event.start_date);
    const end = new Date(event.end_date);
    const durationHours = (end.getTime() - start.getTime()) / (1000 * 60 * 60);
    if (durationHours >= 23) return true;
    if (start.getHours() === 0 && start.getMinutes() === 0 && end.getHours() === 0 && end.getMinutes() === 0) return true;
    const startDate = event.start_date.split("T")[0];
    const endDate = event.end_date.split("T")[0];
    if (startDate !== endDate && event.start_date.includes("T00:00:00")) return true;
    return false;
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
      return startStr <= dayStr && endStr > dayStr;
    });
  }

  interface EventLayout {
    event: CalendarEvent;
    column: number;
    totalColumns: number;
  }

  function layoutEventsForDay(day: Date): EventLayout[] {
    const events = getTimedEventsForDay(day);
    if (events.length === 0) return [];

    const sorted = [...events].sort((a, b) => new Date(a.start_date).getTime() - new Date(b.start_date).getTime());

    // Build connected overlap groups, then assign columns within each group
    const times = sorted.map(e => ({
      start: new Date(e.start_date).getTime(),
      end: new Date(e.end_date).getTime(),
    }));

    // Find connected components of overlapping events
    const groups: number[][] = [];
    let currentGroup: number[] = [];
    let groupEnd = 0;

    for (let i = 0; i < sorted.length; i++) {
      if (currentGroup.length === 0 || times[i].start < groupEnd) {
        currentGroup.push(i);
        groupEnd = Math.max(groupEnd, times[i].end);
      } else {
        groups.push(currentGroup);
        currentGroup = [i];
        groupEnd = times[i].end;
      }
    }
    if (currentGroup.length > 0) groups.push(currentGroup);

    // Assign columns within each group
    const layouts: EventLayout[] = [];
    for (const group of groups) {
      const columns: { end: number }[] = [];
      const groupLayouts: { idx: number; column: number }[] = [];

      for (const idx of group) {
        let placed = false;
        for (let col = 0; col < columns.length; col++) {
          if (times[idx].start >= columns[col].end) {
            columns[col].end = times[idx].end;
            groupLayouts.push({ idx, column: col });
            placed = true;
            break;
          }
        }
        if (!placed) {
          columns.push({ end: times[idx].end });
          groupLayouts.push({ idx, column: columns.length - 1 });
        }
      }

      const totalColumns = columns.length;
      for (const { idx, column } of groupLayouts) {
        layouts.push({ event: sorted[idx], column, totalColumns });
      }
    }

    return layouts;
  }

  const hasAnyAllDayEvents = $derived(
    $calendarEvents.some(e => isAllDay(e))
  );

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
    // Persist selection
    saveConfig({ ...$config, enabled_calendars: [...next] });
  }

  async function loadWeekEvents() {
    if (enabledCalendars.size === 0) {
      calendarEvents.set([]);
      return;
    }

    const start = formatDate(weekStart) + "T00:00:00";
    const end = formatDate(weekEnd) + "T23:59:59";

    // Check if all calendars are cached — if so, skip loading state
    let allCached = true;
    for (const calId of enabledCalendars) {
      if (!getCachedEvents(calId, start, end)) { allCached = false; break; }
    }

    if (!allCached) eventsLoading.set(true);
    eventsError.set(null);
    try {
      const allEvents: CalendarEvent[] = [];
      const seen = new Set<string>();
      for (const calId of enabledCalendars) {
        const accountEmail = getAccountForCalendar(calId);
        if (!accountEmail) continue;

        let events = getCachedEvents(calId, start, end);
        if (!events) {
          events = await api.getEventsForDateRange(accountEmail, calId, start, end);
          setCachedEvents(calId, start, end, events);
        }

        for (const event of events) {
          const normStart = event.start_date.replace(/[+-]\d{2}:\d{2}$/, "").split("T")[0];
          const normEnd = event.end_date.replace(/[+-]\d{2}:\d{2}$/, "").split("T")[0];
          const key = `${event.summary}|${normStart}|${normEnd}`;
          if (!seen.has(key)) {
            seen.add(key);
            allEvents.push(event);
          }
        }
      }
      calendarEvents.set(allEvents);
    } catch (error) {
      eventsError.set(error instanceof Error ? error.message : String(error));
    } finally {
      eventsLoading.set(false);
    }
  }

  onMount(async () => {
    // Restore zoom immediately from store
    if ($config.calendar_zoom != null && $config.calendar_zoom >= 0 && $config.calendar_zoom < ZOOM_STEPS.length) {
      zoomIndex = $config.calendar_zoom;
    }

    const alreadyLoaded = $calendars.length > 0;

    if (!alreadyLoaded) {
      // First mount: load everything from scratch
      await loadConfig();
      await loadGoogleAuthStatus();

      if ($config.calendar_zoom != null && $config.calendar_zoom >= 0 && $config.calendar_zoom < ZOOM_STEPS.length) {
        zoomIndex = $config.calendar_zoom;
      }

      if ($googleAccounts.length > 0) {
        await loadCalendarsForAllAccounts($googleAccounts.map(a => a.email));
      }
    }

    // Restore enabled calendars from config
    if ($calendars.length > 0) {
      if ($config.enabled_calendars && $config.enabled_calendars.length > 0) {
        const validUids = new Set($calendars.map(c => c.uid));
        const restored = $config.enabled_calendars.filter(uid => validUids.has(uid));
        enabledCalendars = new Set(restored.length > 0 ? restored : $calendars.map(c => c.uid));
      } else if ($config.selected_calendar) {
        enabledCalendars = new Set([$config.selected_calendar]);
      } else {
        enabledCalendars = new Set($calendars.map(c => c.uid));
      }
      await loadWeekEvents();
    }

    // Update now-line every minute
    nowLineInterval = setInterval(() => { now = new Date(); }, 60_000);
  });

  $effect(() => {
    weekStart;
    enabledCalendars;
    if (hasAnyAccounts && enabledCalendars.size > 0) {
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
      <h2 class="cal-title">
        {weekStart.toLocaleDateString("en-US", { month: "long", year: "numeric" })}
      </h2>
    </div>
    <div class="cal-right">
      <div class="view-toggle">
        <button class:active={viewMode === "week"} onclick={() => (viewMode = "week")}>Week</button>
        <button class:active={viewMode === "day"} onclick={() => (viewMode = "day")}>Day</button>
      </div>
      <div class="zoom-controls">
        <button class="zoom-btn" onclick={zoomOut} disabled={zoomIndex === 0} title="Zoom out">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <button class="zoom-btn" onclick={zoomIn} disabled={zoomIndex === ZOOM_STEPS.length - 1} title="Zoom in">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      </div>
      <div class="filter-anchor">
        <button class="nav-btn" onclick={() => showCalendarMenu = !showCalendarMenu} title="Calendars &amp; Settings">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="4" y1="21" x2="4" y2="14" /><line x1="4" y1="10" x2="4" y2="3" />
            <line x1="12" y1="21" x2="12" y2="12" /><line x1="12" y1="8" x2="12" y2="3" />
            <line x1="20" y1="21" x2="20" y2="16" /><line x1="20" y1="12" x2="20" y2="3" />
            <line x1="1" y1="14" x2="7" y2="14" /><line x1="9" y1="8" x2="15" y2="8" /><line x1="17" y1="16" x2="23" y2="16" />
          </svg>
        </button>
        {#if showCalendarMenu}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div class="filter-backdrop" role="presentation" onclick={() => showCalendarMenu = false}></div>
          <div class="filter-menu">
            {#each $googleAccounts as account}
              <div class="filter-account">
                <span class="filter-account-email">{account.email}</span>
              </div>
              {#each ($accountCalendars.get(account.email) ?? []) as cal}
                {@const calColor = EVENT_COLORS[getCalendarColorIndex(cal.uid)]}
                <div class="filter-cal-row">
                  <button class="filter-cal-item" onclick={() => toggleCalendar(cal.uid)}>
                    <span class="filter-check" class:checked={enabledCalendars.has(cal.uid)}>
                      {#if enabledCalendars.has(cal.uid)}
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      {/if}
                    </span>
                    <span class="filter-cal-name">{cal.name}</span>
                  </button>
                  <button
                    class="color-dot"
                    style="background: {calColor.border}"
                    title="Change color"
                    onclick={(e) => { e.stopPropagation(); colorPickerCal = colorPickerCal === cal.uid ? null : cal.uid; }}
                  ></button>
                  {#if colorPickerCal === cal.uid}
                    <div class="color-picker">
                      {#each EVENT_COLORS as c, i}
                        <button
                          class="color-swatch"
                          class:active={getCalendarColorIndex(cal.uid) === i}
                          style="background: {c.border}"
                          onclick={(e) => { e.stopPropagation(); setCalendarColor(cal.uid, i); }}
                        ></button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            {/each}
            <div class="filter-divider"></div>
            <button class="filter-add-btn" onclick={handleSignIn} disabled={connectLoading}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              {connectLoading ? "Connecting..." : "Add account"}
            </button>
            {#if connectError}
              <p class="connect-error">{connectError}</p>
            {/if}
            <div class="filter-divider"></div>
            <div class="day-range-row">
              <span class="day-range-label">Day range</span>
              <select class="day-range-select" value={startHour} onchange={(e) => saveConfig({ ...$config, day_start_hour: +e.currentTarget.value })}>
                {#each Array.from({ length: 12 }, (_, i) => i) as h}
                  <option value={h}>{formatHour(h)}</option>
                {/each}
              </select>
              <span class="day-range-sep">–</span>
              <select class="day-range-select" value={endHour} onchange={(e) => saveConfig({ ...$config, day_end_hour: +e.currentTarget.value })}>
                {#each Array.from({ length: 12 }, (_, i) => i + 13) as h}
                  <option value={h}>{formatHour(h)}</option>
                {/each}
              </select>
            </div>
            <div class="filter-divider"></div>
            <div class="theme-row">
              <button class="theme-pill" class:active={$themeMode === "light"} onclick={() => setTheme("light")} title="Light">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                </svg>
              </button>
              <button class="theme-pill" class:active={$themeMode === "dark"} onclick={() => setTheme("dark")} title="Dark">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
                </svg>
              </button>
              <button class="theme-pill" class:active={$themeMode === "system"} onclick={() => setTheme("system")} title="Auto">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
                </svg>
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#if !hasAnyAccounts}
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
      <p>Connect your Google account to see events.</p>
      <button class="connect-cta" onclick={handleSignIn} disabled={connectLoading}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        {connectLoading ? "Connecting..." : "Connect Google Account"}
      </button>
    </div>
  {:else if $eventsLoading}
    <div class="skeleton-calendar">
      <div class="skeleton-header-row">
        {#each Array(7) as _}
          <div class="skeleton-day-header">
            <div class="skeleton-line short"></div>
            <div class="skeleton-circle"></div>
          </div>
        {/each}
      </div>
      <div class="skeleton-grid">
        {#each Array(6) as _, i}
          <div class="skeleton-hour" style="animation-delay: {i * 80}ms">
            <div class="skeleton-line gutter"></div>
            <div class="skeleton-events">
              {#if i === 1 || i === 3}
                <div class="skeleton-event"></div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="week-grid">
      <div class="day-headers">
        <div class="time-gutter-header"></div>
        {#each viewMode === "week" ? weekDays : [currentDate] as day}
          {@const eventCount = getEventCountForDay(day)}
          <div class="day-header" class:today={isToday(day)}>
            <span class="day-name">{formatDayHeader(day)}</span>
            <span class="day-num" class:today={isToday(day)}>{formatDayNumber(day)}</span>
            {#if eventCount > 0}
              <span class="day-density">
                {#each Array(Math.min(eventCount, 5)) as _}<span class="density-dot"></span>{/each}
              </span>
            {/if}
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
                {@const color = getEventColor(event)}
                {#if isLocationEvent(event)}
                  <div class="allday-location" title={event.summary} style="color: {color.text}">
                    {event.summary}
                  </div>
                {:else}
                  <div class="allday-event" title={event.summary} style="background: {color.bg}; color: {color.title}; border-left-color: {color.border}">
                    {event.summary}
                  </div>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      {/if}

      <div class="grid-body" bind:clientHeight={gridHeight}>
        <div class="time-gutter" style="padding-top: {HOUR_HEIGHT * GRID_OFFSET / 60}px">
          {#each HOURS as hour}
            <div class="time-label" style="height: {HOUR_HEIGHT}px">
              <span>{formatHour(hour)}</span>
            </div>
          {/each}
        </div>

        <div class="days-container">
          {#each viewMode === "week" ? weekDays : [currentDate] as day, dayIndex}
            <div class="day-column" class:today={isToday(day)} style="padding-top: {HOUR_HEIGHT * GRID_OFFSET / 60}px">
              {#each HOURS as _hour}
                <div class="hour-slot" style="height: {HOUR_HEIGHT}px"></div>
              {/each}

              {#if nowMinutesCurrent >= startHour * 60 && nowMinutesCurrent <= endHour * 60}
                {#if isToday(day)}
                  <div class="now-line" style="top: {nowTopPx}px">
                    <span class="now-dot"></span>
                  </div>
                {:else}
                  <div class="now-line now-line--ghost" style="top: {nowTopPx}px"></div>
                {/if}
              {/if}

              {#each layoutEventsForDay(day) as { event, column, totalColumns }}
                {@const style = getEventStyle(event, day)}
                {@const color = getEventColor(event)}
                {@const duration = getEventDurationMinutes(event)}
                {@const isShort = duration <= 45}
                {#if style}
                  <div
                    class="event-block"
                    class:event-block--short={isShort}
                    style="top: {style.top}; height: {style.height}; background: {color.bg}; border-left-color: {color.border}; left: calc(3px + {column} * (100% - 6px) / {totalColumns}); width: calc((100% - 6px) / {totalColumns} - 2px)"
                    title={event.summary}
                  >
                    {#if isShort}
                      <span class="event-title" style="color: {color.title}">{formatEventTime(event)} · {event.summary}</span>
                    {:else}
                      <span class="event-time" style="color: {color.text}">{formatEventTime(event)}</span>
                      <span class="event-title" style="color: {color.title}">{event.summary}</span>
                    {/if}
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
    margin-bottom: 16px;
  }

  .cal-nav {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .cal-right {
    display: flex;
    align-items: center;
    gap: 4px;
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

  .day-range-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
  }

  .day-range-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    margin-right: auto;
  }

  .day-range-select {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: 6px;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 3px 6px;
    cursor: pointer;
  }

  .day-range-sep {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .theme-row {
    display: flex;
    gap: 2px;
    padding: 4px 4px 2px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    margin: 2px;
  }

  .theme-pill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border: none;
    background: transparent;
    border-radius: 4px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .theme-pill:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  .theme-pill.active {
    color: var(--accent-blue);
    background: var(--bg-hover);
  }

  .zoom-controls {
    display: flex;
    gap: 2px;
  }

  .zoom-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 1px solid var(--border-default);
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .zoom-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .zoom-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .filter-anchor {
    position: relative;
    z-index: 20;
  }

  .filter-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .filter-menu {
    position: absolute;
    top: 36px;
    right: 0;
    display: flex;
    flex-direction: column;
    min-width: 220px;
    max-height: 400px;
    overflow-y: auto;
    padding: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    animation: fadeInUp 0.12s var(--ease-out);
    z-index: 100;
  }

  .filter-account {
    padding: 6px 8px 2px;
  }

  .filter-account-email {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }

  .filter-cal-row {
    position: relative;
    display: flex;
    align-items: center;
  }

  .color-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    flex-shrink: 0;
    margin-right: 6px;
    transition: transform 0.12s var(--ease-out), border-color 0.12s var(--ease-out);
  }

  .color-dot:hover {
    transform: scale(1.2);
    border-color: var(--border-strong);
  }

  .color-picker {
    position: absolute;
    right: 0;
    top: 100%;
    display: flex;
    gap: 4px;
    padding: 6px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
    z-index: 20;
  }

  .color-swatch {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.1s var(--ease-out), border-color 0.1s var(--ease-out);
  }

  .color-swatch:hover {
    transform: scale(1.2);
  }

  .color-swatch.active {
    border-color: var(--text-primary);
    transform: scale(1.15);
  }

  .filter-cal-item {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
    text-align: left;
  }

  .filter-cal-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-check {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    border: 1.5px solid var(--border-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.12s var(--ease-out);
  }

  .filter-check.checked {
    background: var(--accent-blue);
    border-color: var(--accent-blue);
    color: white;
  }

  .filter-cal-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .filter-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 4px 0;
  }

  .filter-add-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .filter-add-btn svg {
    width: 16px;
    height: 16px;
  }

  .filter-add-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-add-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
    font-size: 19px;
    font-weight: 700;
    margin: 0 0 0 10px;
    color: var(--text-primary);
    letter-spacing: -0.03em;
    white-space: nowrap;
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

  .connect-cta {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
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

  .connect-cta:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: var(--shadow-glow-blue);
  }

  .connect-cta:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
    border: 1px solid var(--border-default);
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
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
    min-height: 28px;
  }

  .allday-label {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 4px;
  }

  .allday-label span {
    font-family: var(--font-mono);
    font-size: 8px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    white-space: nowrap;
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
    background: var(--today-tint);
  }

  .allday-event {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    border-left: 2px solid transparent;
    transition: filter 0.12s var(--ease-out);
  }

  .allday-event:hover {
    filter: brightness(1.15);
    cursor: pointer;
  }

  .allday-location {
    font-size: 9px;
    font-weight: 500;
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    opacity: 0.5;
    padding: 2px 4px;
  }

  .day-headers {
    display: flex;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--day-header-bg);
  }

  .time-gutter-header {
    width: 56px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-subtle);
  }

  .day-header {
    flex: 1;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 4px;
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
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }

  .day-num.today {
    background: var(--accent-blue);
    color: white;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
  }

  .day-header.today .day-name {
    color: var(--accent-blue);
    font-weight: 600;
  }

  .day-density {
    display: flex;
    gap: 2px;
    margin-left: 2px;
  }

  .density-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-tertiary);
    opacity: 0.5;
  }

  .day-header.today .density-dot {
    background: var(--accent-blue);
    opacity: 0.6;
  }

  .grid-body {
    display: flex;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    position: relative;
  }

  .time-gutter {
    width: 56px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-subtle);
  }

  .time-label {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding: 0 10px;
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
    background: var(--today-tint);
  }

  .now-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 0;
    border-top: 2px solid #e53935;
    z-index: 5;
    pointer-events: none;
  }

  .now-dot {
    position: absolute;
    top: -5px;
    left: -4px;
    width: 8px;
    height: 8px;
    background: #e53935;
    border-radius: 50%;
  }

  .now-line--ghost {
    border-top: 1px dashed rgba(229, 57, 53, 0.2);
  }

  .hour-slot {
    border-bottom: 1px solid var(--border-subtle);
    position: relative;
  }

  .hour-slot::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    border-bottom: 1px dashed color-mix(in srgb, var(--border-subtle) 40%, transparent);
  }

  .event-block {
    position: absolute;
    border-left: 3px solid var(--accent-blue);
    border-radius: 5px;
    padding: 4px 8px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.12s var(--ease-out), box-shadow 0.12s var(--ease-out), filter 0.12s var(--ease-out);
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .event-block:hover {
    filter: brightness(1.15);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.2);
    transform: scale(1.01);
    z-index: 5;
  }

  .event-block--short {
    flex-direction: row;
    align-items: center;
    padding: 2px 6px;
  }

  .event-block--short .event-title {
    font-size: 10px;
    -webkit-line-clamp: 1;
    white-space: nowrap;
  }

  .event-time {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    display: block;
    letter-spacing: 0.03em;
    line-height: 1.4;
    opacity: 0.85;
  }

  .event-title {
    font-size: 12px;
    font-weight: 600;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.35;
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

  .skeleton-calendar {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--bg-surface);
    flex: 1;
    animation: fadeIn 0.3s var(--ease-out);
  }

  .skeleton-header-row {
    display: flex;
    border-bottom: 1px solid var(--border-subtle);
    padding: 8px 0;
    padding-left: 48px;
  }

  .skeleton-day-header {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .skeleton-circle {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--bg-hover);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .skeleton-grid {
    padding: 8px 0;
  }

  .skeleton-hour {
    display: flex;
    align-items: flex-start;
    height: 56px;
    border-bottom: 1px solid var(--border-subtle);
    animation: fadeIn 0.4s var(--ease-out) both;
  }

  .skeleton-line {
    height: 10px;
    background: var(--bg-hover);
    border-radius: 4px;
    animation: pulse 1.4s ease-in-out infinite;
  }

  .skeleton-line.short {
    width: 28px;
  }

  .skeleton-line.gutter {
    width: 32px;
    margin: 4px 8px;
    flex-shrink: 0;
  }

  .skeleton-events {
    flex: 1;
    padding: 4px 8px;
  }

  .skeleton-event {
    height: 36px;
    width: 40%;
    background: var(--bg-hover);
    border-radius: 6px;
    border-left: 3px solid var(--border-strong);
    animation: pulse 1.4s ease-in-out infinite;
  }
</style>
