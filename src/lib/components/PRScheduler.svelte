<script lang="ts">
  import { onMount } from "svelte";
  import type { ScheduledPR } from "../stores/github";
  import { config } from "../stores/config";
  import { updatePRCalendarEvent } from "../stores/github";
  import * as api from "../api";
  import type { CalendarEvent } from "../api";
  import { getAccountForCalendar } from "../stores/calendar";
  import {
    format,
    parseISO,
    addDays,
    addMonths,
    addMinutes,
    startOfWeek,
    startOfMonth,
    startOfDay,
    getDate,
    getMonth,
    getHours,
    getMinutes,
    isSameDay,
    isBefore,
    isWeekend as dateFnsIsWeekend,
    isToday as dateFnsIsToday,
    differenceInMinutes,
  } from "date-fns";

  interface Props {
    pr: ScheduledPR;
    onClose: () => void;
    onScheduled: () => void;
  }

  let { pr, onClose, onScheduled }: Props = $props();

  interface ScheduledSlot {
    id: string;
    date: string;
    time: string;
    durationHours: number;
    durationMinutes: number;
    isFocusTime: boolean;
    colorId: string | null;
  }

  let existingEvents = $state<CalendarEvent[]>([]);
  let loadingEvents = $state(true);
  let editingEvent = $state<CalendarEvent | null>(null);
  let scheduledSlots = $state<ScheduledSlot[]>([]);
  let selectedDate = $state<string | null>(null);
  let dayEvents = $state<CalendarEvent[]>([]);
  let loadingDayEvents = $state(false);
  let isFocusTime = $state(false);
  let selectedColorId = $state<string | null>(null);
  let hasUserSelectedColor = $state(false);

  let editTime = $state("09:00");
  let editDurationHours = $state(1);
  let editDurationMinutes = $state(0);
  let editColorId = $state<string | null>(null);

  interface TimeSlot {
    start: string;
    end: string;
    durationMinutes: number;
  }

  const MIN_SLOT_MINUTES = 30;

  const accountWorkHours = $derived.by(() => {
    const cal = $config.selected_calendar;
    if (!cal) return { start: 8, end: 18 };
    const email = getAccountForCalendar(cal) ?? "";
    const win = ($config.account_schedule_windows ?? {})[email];
    return win ? { start: win.start_hour, end: win.end_hour } : { start: 8, end: 18 };
  });

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

  let isScheduling = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (selectedDate && $config.selected_calendar) {
      loadDayEvents(selectedDate);
    }
  });

  // Use PR-specific color, falling back to general default color
  const prDefaultColor = $derived($config.pr_default_event_color ?? $config.default_event_color ?? null);

  $effect(() => {
    if (!hasUserSelectedColor) {
      selectedColorId = prDefaultColor;
    }
  });

  const resolvedSelectedColorId = $derived.by(() => {
    return hasUserSelectedColor
      ? selectedColorId
      : (selectedColorId ?? prDefaultColor);
  });

  async function loadDayEvents(dateStr: string) {
    loadingDayEvents = true;
    try {
      const selectedDay = parseISO(dateStr);
      const nextDay = addDays(selectedDay, 1);
      const endDateStr = format(nextDay, "yyyy-MM-dd");

      const events = await api.getEventsForDateRange(
        getAccountForCalendar($config.selected_calendar!) ?? "",
        $config.selected_calendar!,
        dateStr,
        endDateStr
      );

      dayEvents = events.filter((e) => {
        try {
          const eventDate = parseEventDate(e.start_date);
          return format(eventDate, "yyyy-MM-dd") === dateStr;
        } catch {
          return false;
        }
      });
    } catch (err) {
      console.error("Failed to load day events:", err);
      dayEvents = [];
    } finally {
      loadingDayEvents = false;
    }
  }

  const availableSlots = $derived.by((): TimeSlot[] => {
    if (!selectedDate) return [];

    const slots: TimeSlot[] = [];
    const today = new Date();
    const selectedDay = parseISO(selectedDate);
    const isSelectedToday = isSameDay(selectedDay, today);

    const busyPeriods: { start: number; end: number }[] = dayEvents
      .map((event) => {
        try {
          const start = parseEventDate(event.start_date);
          const end = parseEventDate(event.end_date);
          return {
            start: getHours(start) * 60 + getMinutes(start),
            end: getHours(end) * 60 + getMinutes(end),
          };
        } catch {
          return { start: 0, end: 0 };
        }
      })
      .filter((p) => p.start !== p.end);

    const pendingBusy = scheduledSlots
      .filter((s) => s.date === selectedDate)
      .map((s) => {
        const [hours, mins] = s.time.split(":").map(Number);
        const start = hours * 60 + mins;
        return {
          start,
          end: start + s.durationHours * 60 + s.durationMinutes,
        };
      });

    const allBusy = [...busyPeriods, ...pendingBusy].sort(
      (a, b) => a.start - b.start
    );

    let currentMinute = accountWorkHours.start * 60;
    if (isSelectedToday) {
      const nowMinutes = getHours(today) * 60 + getMinutes(today);
      currentMinute = Math.max(currentMinute, Math.ceil(nowMinutes / 30) * 30);
    }

    const endMinute = accountWorkHours.end * 60;

    for (const busy of allBusy) {
      if (busy.start > currentMinute && busy.start <= endMinute) {
        const gapDuration = busy.start - currentMinute;
        if (gapDuration >= MIN_SLOT_MINUTES) {
          slots.push({
            start: formatMinutesToTime(currentMinute),
            end: formatMinutesToTime(busy.start),
            durationMinutes: gapDuration,
          });
        }
      }
      currentMinute = Math.max(currentMinute, busy.end);
    }

    if (currentMinute < endMinute) {
      const gapDuration = endMinute - currentMinute;
      if (gapDuration >= MIN_SLOT_MINUTES) {
        slots.push({
          start: formatMinutesToTime(currentMinute),
          end: formatMinutesToTime(endMinute),
          durationMinutes: gapDuration,
        });
      }
    }

    return slots;
  });

  function formatMinutesToTime(minutes: number): string {
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`;
  }

  function formatTimeRange(start: string, end: string): string {
    const formatTime = (t: string) => {
      const [h, m] = t.split(":").map(Number);
      const period = h >= 12 ? "PM" : "AM";
      const hour = h > 12 ? h - 12 : h === 0 ? 12 : h;
      return m === 0 ? `${hour}${period}` : `${hour}:${m.toString().padStart(2, "0")}${period}`;
    };
    return `${formatTime(start)} - ${formatTime(end)}`;
  }

  function addSlotFromAvailable(slot: TimeSlot, duration: number) {
    if (!selectedDate) return;
    const resolvedColorId = resolvedSelectedColorId;

    const newSlot: ScheduledSlot = {
      id: crypto.randomUUID(),
      date: selectedDate,
      time: slot.start,
      durationHours: Math.floor(duration / 60),
      durationMinutes: duration % 60,
      isFocusTime,
      colorId: resolvedColorId,
    };

    scheduledSlots = [...scheduledSlots, newSlot];
  }

  type ViewMode = "day" | "week" | "month";
  let viewMode = $state<ViewMode>("week");
  let currentDate = $state(new Date());

  const calendarDays = $derived.by(() => {
    const days: {
      date: string;
      dayNum: number;
      dayName: string;
      monthName: string;
      isCurrentMonth: boolean;
      isWeekend: boolean;
      isToday: boolean;
      isPast: boolean;
    }[] = [];

    const today = startOfDay(new Date());

    if (viewMode === "day") {
      const dateStr = format(currentDate, "yyyy-MM-dd");
      days.push({
        date: dateStr,
        dayNum: getDate(currentDate),
        dayName: format(currentDate, "EEEE"),
        monthName: format(currentDate, "MMM"),
        isCurrentMonth: true,
        isWeekend: dateFnsIsWeekend(currentDate),
        isToday: dateFnsIsToday(currentDate),
        isPast: isBefore(startOfDay(currentDate), today),
      });
    } else if (viewMode === "week") {
      const weekStart = startOfWeek(currentDate, { weekStartsOn: 1 });

      for (let i = 0; i < 7; i++) {
        const date = addDays(weekStart, i);
        const dateStr = format(date, "yyyy-MM-dd");

        days.push({
          date: dateStr,
          dayNum: getDate(date),
          dayName: format(date, "EEE"),
          monthName: format(date, "MMM"),
          isCurrentMonth: getMonth(date) === getMonth(currentDate),
          isWeekend: dateFnsIsWeekend(date),
          isToday: dateFnsIsToday(date),
          isPast: isBefore(startOfDay(date), today),
        });
      }
    } else {
      const month = getMonth(currentDate);
      const firstDay = startOfMonth(currentDate);
      const monthStart = startOfWeek(firstDay, { weekStartsOn: 1 });

      for (let i = 0; i < 42; i++) {
        const date = addDays(monthStart, i);
        const dateStr = format(date, "yyyy-MM-dd");

        days.push({
          date: dateStr,
          dayNum: getDate(date),
          dayName: format(date, "EEE"),
          monthName: format(date, "MMM"),
          isCurrentMonth: getMonth(date) === month,
          isWeekend: dateFnsIsWeekend(date),
          isToday: dateFnsIsToday(date),
          isPast: isBefore(startOfDay(date), today),
        });
      }
    }

    return days;
  });

  const navDisplay = $derived.by(() => {
    if (viewMode === "day") {
      return format(currentDate, "EEEE, MMMM d, yyyy");
    } else if (viewMode === "week") {
      const weekStart = startOfWeek(currentDate, { weekStartsOn: 1 });
      const weekEnd = addDays(weekStart, 6);
      const startStr = format(weekStart, "MMM d");
      const endStr = format(weekEnd, "MMM d, yyyy");
      return `${startStr} - ${endStr}`;
    } else {
      return format(currentDate, "MMMM yyyy");
    }
  });

  function navigate(delta: number) {
    if (viewMode === "day") {
      currentDate = addDays(currentDate, delta);
    } else if (viewMode === "week") {
      currentDate = addDays(currentDate, delta * 7);
    } else {
      currentDate = addMonths(currentDate, delta);
    }
  }

  function goToToday() {
    currentDate = new Date();
  }

  function dayHasSlot(dateStr: string): boolean {
    return scheduledSlots.some((slot) => slot.date === dateStr);
  }

  function dayHasFocusSlot(dateStr: string): boolean {
    return scheduledSlots.some((slot) => slot.date === dateStr && slot.isFocusTime);
  }

  function dayHasEvent(dateStr: string): boolean {
    return existingEvents.some((e) => {
      try {
        return format(parseEventDate(e.start_date), "yyyy-MM-dd") === dateStr;
      } catch {
        return false;
      }
    });
  }

  let loadAttempted = $state(false);

  onMount(() => {
    loadExistingEventsWithTimeout();
  });

  async function loadExistingEventsWithTimeout() {
    const calendarName = $config.selected_calendar;
    if (!calendarName) {
      loadingEvents = false;
      loadAttempted = true;
      return;
    }

    loadingEvents = true;
    error = null;

    const timeout = new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error("Timeout loading events")), 5000);
    });

    try {
      const today = new Date();
      const endDate = addDays(today, 60);

      const searchKey = pr.repo_name;
      const events = await Promise.race([
        api.getEventsForDateRange(
          getAccountForCalendar(calendarName) ?? "",
          calendarName,
          format(today, "yyyy-MM-dd"),
          format(endDate, "yyyy-MM-dd"),
          searchKey
        ),
        timeout,
      ]);

      existingEvents = events.filter(
        (e) =>
          e.summary.includes(pr.repo_name) &&
          (e.summary.includes(`#${pr.number}`) || e.description?.includes(pr.url))
      );
    } catch (err) {
      console.error("Failed to load existing events:", err);
      existingEvents = [];
    } finally {
      loadingEvents = false;
      loadAttempted = true;
    }
  }

  async function loadExistingEvents() {
    await loadExistingEventsWithTimeout();
  }

  function formatSlotDate(dateStr: string): string {
    const date = parseISO(dateStr);
    return format(date, "MMM d");
  }

  function formatSlotDuration(hours: number, minutes: number): string {
    if (hours > 0 && minutes > 0) return `${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h`;
    return `${minutes}m`;
  }

  function colorHexFor(id: string | null): string {
    const match = calendarColors.find((color) => color.id === id);
    return match?.color ?? "#4285f4";
  }

  function parseEventDate(dateStr: string): Date {
    const isoMatch = dateStr.match(/^\d{4}-\d{2}-\d{2}/);
    if (isoMatch) {
      return new Date(dateStr);
    }
    return new Date(dateStr);
  }

  function formatEventTime(event: CalendarEvent): string {
    try {
      const start = parseEventDate(event.start_date);
      const end = parseEventDate(event.end_date);
      const dateStr = format(start, "EEE, MMM d");
      const startTime = format(start, "h:mm a");
      const durationMins = differenceInMinutes(end, start);
      const durationStr =
        durationMins >= 60
          ? `${Math.floor(durationMins / 60)}h${durationMins % 60 > 0 ? ` ${durationMins % 60}m` : ""}`
          : `${durationMins}m`;
      return `${dateStr} @ ${startTime} - ${durationStr}`;
    } catch {
      return event.start_date;
    }
  }

  function formatEventTitle(template: string): string {
    return template
      .replace("{repo}", pr.repo_name)
      .replace("{title}", pr.title)
      .replace("{number}", String(pr.number))
      .replace("{author}", pr.author);
  }

  function selectDate(dateStr: string) {
    selectedDate = dateStr;
  }

  function removeSlot(slotId: string) {
    scheduledSlots = scheduledSlots.filter((s) => s.id !== slotId);
  }

  function toggleSlotFocusTime(slotId: string) {
    scheduledSlots = scheduledSlots.map((s) =>
      s.id === slotId ? { ...s, isFocusTime: !s.isFocusTime } : s
    );
  }

  async function handleDeleteEvent(calendarEvent: CalendarEvent) {
    try {
      await api.deleteEvent(getAccountForCalendar(calendarEvent.calendar_name) ?? "", calendarEvent.uid, calendarEvent.calendar_name);
      existingEvents = existingEvents.filter((e) => e.uid !== calendarEvent.uid);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function startEditEvent(event: CalendarEvent) {
    editingEvent = event;
    editColorId = null;
    try {
      const start = parseEventDate(event.start_date);
      const end = parseEventDate(event.end_date);
      selectedDate = format(start, "yyyy-MM-dd");
      editTime = format(start, "HH:mm");
      const totalMins = differenceInMinutes(end, start);
      editDurationHours = Math.floor(totalMins / 60);
      editDurationMinutes = totalMins % 60;
      isFocusTime = false;
    } catch {
      // Keep defaults
    }
  }

  function cancelEdit() {
    editingEvent = null;
    selectedDate = null;
  }

  async function handleUpdateEvent() {
    if (!editingEvent || !selectedDate) return;

    const totalMinutes = editDurationHours * 60 + editDurationMinutes;
    if (totalMinutes <= 0) {
      error = "Please set a duration greater than 0.";
      return;
    }

    isScheduling = true;
    error = null;

    try {
      const startDateTime = parseISO(`${selectedDate}T${editTime}:00`);
      const endDateTime = addMinutes(startDateTime, totalMinutes);

      await api.updateEvent(getAccountForCalendar(editingEvent.calendar_name) ?? "", {
        uid: editingEvent.uid,
        summary: formatEventTitle($config.pr_event_title_template || "[PR Review] {repo}: {title}"),
        start_date: format(startDateTime, "yyyy-MM-dd'T'HH:mm:ss"),
        end_date: format(endDateTime, "yyyy-MM-dd'T'HH:mm:ss"),
        description: null,
        url: null,
        calendar_name: editingEvent.calendar_name,
        is_focus_time: isFocusTime,
        color_id: editColorId,
      });

      await loadExistingEvents();
      editingEvent = null;
      selectedDate = null;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isScheduling = false;
    }
  }

  async function handleSchedule() {
    if (!$config.selected_calendar) {
      error = "Please select a calendar in Settings first.";
      return;
    }

    if (scheduledSlots.length === 0) {
      error = "Please add at least one slot.";
      return;
    }

    isScheduling = true;
    error = null;

    try {
      const title = formatEventTitle($config.pr_event_title_template || "[PR Review] {repo}: {title}");
      const jiraKey = pr.linked_jira_key || pr.jira_key;
      const description = `PR Review: ${pr.repo_full_name}#${pr.number}\n${pr.title}\n\nAuthor: ${pr.author}\nBranch: ${pr.branch} → ${pr.target_branch}${jiraKey ? `\n\nJira: ${jiraKey}` : ""}\n\nURL: ${pr.url}`;

      const sortedSlots = [...scheduledSlots].sort((a, b) =>
        a.date.localeCompare(b.date)
      );
      let lastEventUid = "";

      for (const slot of sortedSlots) {
        const totalMinutes = slot.durationHours * 60 + slot.durationMinutes;
        const startDateTime = parseISO(`${slot.date}T${slot.time}:00`);
        const endDateTime = addMinutes(startDateTime, totalMinutes);

        const eventUid = await api.createEvent(getAccountForCalendar($config.selected_calendar!) ?? "", {
          summary: title,
          start_date: format(startDateTime, "yyyy-MM-dd'T'HH:mm:ss"),
          end_date: format(endDateTime, "yyyy-MM-dd'T'HH:mm:ss"),
          description,
          url: pr.url,
          calendar_name: $config.selected_calendar,
          is_focus_time: slot.isFocusTime,
          color_id: slot.colorId,
        });

        lastEventUid = eventUid;
      }

      updatePRCalendarEvent(pr.id, lastEventUid);
      await loadExistingEvents();
      scheduledSlots = [];
      onScheduled();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isScheduling = false;
    }
  }

  const totalScheduledMinutes = $derived.by(() => {
    let total = 0;
    for (const event of existingEvents) {
      try {
        const start = parseEventDate(event.start_date);
        const end = parseEventDate(event.end_date);
        total += differenceInMinutes(end, start);
      } catch {
        // Skip
      }
    }
    return total;
  });

  const totalScheduledFormatted = $derived.by(() => {
    const hours = Math.floor(totalScheduledMinutes / 60);
    const mins = totalScheduledMinutes % 60;
    if (hours > 0 && mins > 0) return `${hours}h ${mins}m`;
    if (hours > 0) return `${hours}h`;
    return `${mins}m`;
  });

  function getJiraKey(): string | null {
    return pr.linked_jira_key || pr.jira_key;
  }
</script>

<div
  class="scheduler-overlay"
  onclick={onClose}
  onkeydown={(e) => e.key === "Escape" && onClose()}
  role="dialog"
  aria-modal="true"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="scheduler-modal" role="presentation" onclick={(e) => e.stopPropagation()}>
    <div class="scheduler-header">
      <h2>{editingEvent ? "Edit Session" : "Schedule PR Review"}</h2>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    <div class="pr-info">
      <div class="pr-badges">
        <span class="pr-badge">PR</span>
        <span class="repo-badge">{pr.repo_name}</span>
        {#if pr.is_draft}
          <span class="draft-badge">Draft</span>
        {/if}
      </div>
      <h3>
        <span class="pr-number">#{pr.number}</span>
        {pr.title}
      </h3>
      <div class="pr-stats">
        <span class="stat">
          <span class="stat-label">Author:</span>
          {pr.author}
        </span>
        {#if getJiraKey()}
          <span class="stat jira">
            <span class="stat-label">Jira:</span>
            {getJiraKey()}
          </span>
        {:else}
          <span class="stat no-jira">No Jira linked</span>
        {/if}
        {#if totalScheduledMinutes > 0}
          <span class="stat scheduled">
            <span class="stat-label">Scheduled:</span>
            {totalScheduledFormatted}
          </span>
        {/if}
      </div>
    </div>

    {#if !editingEvent}
      <div class="existing-sessions">
        <div class="sessions-header">
          <h4>Scheduled Sessions</h4>
          {#if loadAttempted && !loadingEvents}
            <button class="refresh-btn" onclick={loadExistingEvents}>Refresh</button>
          {/if}
        </div>
        {#if loadingEvents}
          <div class="loading-sessions">
            <span>Loading calendar events...</span>
            <button
              class="skip-btn"
              onclick={() => {
                loadingEvents = false;
                loadAttempted = true;
              }}>Skip</button
            >
          </div>
        {:else if existingEvents.length === 0}
          <div class="no-sessions">No sessions scheduled yet</div>
        {:else}
          <div class="sessions-list">
            {#each existingEvents as event (event.uid)}
              <div class="session-item">
                <div class="session-info">
                  <span class="session-time">{formatEventTime(event)}</span>
                </div>
                <div class="session-actions">
                  <button
                    type="button"
                    class="edit-btn"
                    onclick={() => startEditEvent(event)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    class="delete-btn"
                    onclick={() => handleDeleteEvent(event)}
                  >
                    Delete
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="scheduler-form">
      <div class="view-toggle">
        <button
          type="button"
          class="view-btn"
          class:active={viewMode === "day"}
          onclick={() => (viewMode = "day")}>Day</button
        >
        <button
          type="button"
          class="view-btn"
          class:active={viewMode === "week"}
          onclick={() => (viewMode = "week")}>Week</button
        >
        <button
          type="button"
          class="view-btn"
          class:active={viewMode === "month"}
          onclick={() => (viewMode = "month")}>Month</button
        >
      </div>

      <div class="calendar-nav">
        <button type="button" class="nav-btn" onclick={() => navigate(-1)}>&lt;</button>
        <div class="nav-center">
          <span class="nav-display">{navDisplay}</span>
          <button type="button" class="today-btn" onclick={goToToday}>Today</button>
        </div>
        <button type="button" class="nav-btn" onclick={() => navigate(1)}>&gt;</button>
      </div>

      <div class="form-section">
        {#if viewMode === "week"}
          <div class="week-view">
            {#each calendarDays as day}
              <button
                type="button"
                class="week-day-cell"
                class:selected={selectedDate === day.date}
                class:weekend={day.isWeekend}
                class:today={day.isToday}
                class:past={day.isPast}
                class:has-slot={dayHasSlot(day.date)}
                class:has-focus-slot={dayHasFocusSlot(day.date)}
                class:has-event={dayHasEvent(day.date)}
                onclick={() => !day.isPast && selectDate(day.date)}
                disabled={day.isPast}
              >
                <span class="week-day-name">{day.dayName}</span>
                <span class="week-day-num">{day.dayNum}</span>
                <span class="week-day-month">{day.monthName}</span>
                {#if dayHasSlot(day.date) || dayHasEvent(day.date)}
                  <span class="day-indicator" class:focus={dayHasFocusSlot(day.date)}></span>
                {/if}
              </button>
            {/each}
          </div>
        {:else if viewMode === "month"}
          <div class="calendar-header">
            <span>Mon</span>
            <span>Tue</span>
            <span>Wed</span>
            <span>Thu</span>
            <span>Fri</span>
            <span>Sat</span>
            <span>Sun</span>
          </div>
          <div class="calendar-grid">
            {#each calendarDays as day}
              <button
                type="button"
                class="day-cell"
                class:selected={selectedDate === day.date}
                class:other-month={!day.isCurrentMonth}
                class:weekend={day.isWeekend}
                class:today={day.isToday}
                class:past={day.isPast}
                class:has-slot={dayHasSlot(day.date)}
                class:has-focus-slot={dayHasFocusSlot(day.date)}
                class:has-event={dayHasEvent(day.date)}
                onclick={() => !day.isPast && selectDate(day.date)}
                disabled={day.isPast}
              >
                <span class="day-num">{day.dayNum}</span>
                {#if dayHasSlot(day.date) || dayHasEvent(day.date)}
                  <span class="day-indicator" class:focus={dayHasFocusSlot(day.date)}></span>
                {/if}
              </button>
            {/each}
          </div>
        {:else}
          <div class="day-view">
            {#each calendarDays as day}
              <button
                type="button"
                class="day-view-cell"
                class:selected={selectedDate === day.date}
                class:today={day.isToday}
                class:past={day.isPast}
                class:has-slot={dayHasSlot(day.date)}
                class:has-event={dayHasEvent(day.date)}
                onclick={() => !day.isPast && selectDate(day.date)}
                disabled={day.isPast}
              >
                <span class="day-view-name">{day.dayName}</span>
                <span class="day-view-date">{day.monthName} {day.dayNum}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      {#if !editingEvent && scheduledSlots.length > 0}
        <div class="pending-slots">
          <div class="slots-header">
            <h4>Pending Slots ({scheduledSlots.length})</h4>
          </div>
          <div class="slots-list">
            {#each scheduledSlots as slot (slot.id)}
              <div class="slot-item">
                <div class="slot-info">
                  <span class="slot-date">{formatSlotDate(slot.date)}</span>
                  <span class="slot-color" style={`background: ${colorHexFor(slot.colorId)}`}></span>
                  <span class="slot-time">@ {slot.time}</span>
                  <span class="slot-duration">- {formatSlotDuration(slot.durationHours, slot.durationMinutes)}</span>
                  {#if slot.isFocusTime}
                    <span class="focus-badge">Focus</span>
                  {/if}
                </div>
                <div class="slot-actions">
                  <button
                    class="focus-toggle"
                    class:active={slot.isFocusTime}
                    onclick={() => toggleSlotFocusTime(slot.id)}
                    title="Toggle focus time"
                  >F</button>
                  <button class="remove-slot" onclick={() => removeSlot(slot.id)}>&times;</button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if selectedDate && !editingEvent}
        <div class="available-slots">
          <div class="available-header">
            <h4>Available on {formatSlotDate(selectedDate)}</h4>
            <div class="availability-controls">
              <label class="focus-checkbox-inline">
                <input type="checkbox" bind:checked={isFocusTime} />
                <span>Focus Time</span>
              </label>
              <div class="color-select-inline">
                <span>Color</span>
                <div class="color-select">
                  <span class="color-swatch" style={`background: ${colorHexFor(resolvedSelectedColorId)}`}></span>
                  <select bind:value={selectedColorId} onchange={() => (hasUserSelectedColor = true)}>
                    {#each calendarColors as color}
                      <option value={color.id}>{color.name}</option>
                    {/each}
                  </select>
                </div>
              </div>
            </div>
          </div>

          {#if loadingDayEvents}
            <div class="loading-slots">Loading calendar...</div>
          {:else if availableSlots.length === 0}
            <div class="no-slots">No available time slots</div>
          {:else}
            <div class="slots-grid">
              {#each availableSlots as slot}
                <div class="available-slot">
                  <div class="slot-time-range">{formatTimeRange(slot.start, slot.end)}</div>
                  <div class="slot-duration-label">
                    {Math.floor(slot.durationMinutes / 60)}h {slot.durationMinutes % 60}m available
                  </div>
                  <div class="slot-book-options">
                    {#if slot.durationMinutes >= 15}
                      <button type="button" class="book-btn" onclick={() => addSlotFromAvailable(slot, 15)}>15m</button>
                    {/if}
                    {#if slot.durationMinutes >= 30}
                      <button type="button" class="book-btn" onclick={() => addSlotFromAvailable(slot, 30)}>30m</button>
                    {/if}
                    {#if slot.durationMinutes >= 60}
                      <button type="button" class="book-btn" onclick={() => addSlotFromAvailable(slot, 60)}>1h</button>
                    {/if}
                    {#if slot.durationMinutes >= 120}
                      <button type="button" class="book-btn" onclick={() => addSlotFromAvailable(slot, 120)}>2h</button>
                    {/if}
                    <button
                      type="button"
                      class="book-btn book-all"
                      onclick={() => addSlotFromAvailable(slot, slot.durationMinutes)}
                    >All</button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if !editingEvent}
        <div class="select-day-prompt">Select a day to see available time slots</div>
      {/if}

      {#if editingEvent}
        <div class="edit-slot-form">
          <h4>Edit Session</h4>
          <div class="edit-row">
            <div class="edit-group">
              <label for="edit-time">Time</label>
              <input type="time" id="edit-time" bind:value={editTime} />
            </div>
            <div class="edit-group">
              <label for="edit-duration-hours">Duration</label>
              <div class="edit-duration">
                <input type="number" id="edit-duration-hours" min="0" max="12" bind:value={editDurationHours} />
                <span>h</span>
                <input type="number" min="0" max="59" step="15" bind:value={editDurationMinutes} />
                <span>m</span>
              </div>
            </div>
          </div>
          <div class="edit-options">
            <label class="focus-checkbox-inline">
              <input type="checkbox" bind:checked={isFocusTime} />
              <span>Focus Time</span>
            </label>
            <div class="color-select-inline">
              <span>Color</span>
              <div class="color-select">
                <span class="color-swatch" style={`background: ${editColorId ? colorHexFor(editColorId) : "#9aa0a6"}`}></span>
                <select bind:value={editColorId}>
                  <option value={null}>Keep current</option>
                  {#each calendarColors as color}
                    {#if color.id}
                      <option value={color.id}>{color.name}</option>
                    {/if}
                  {/each}
                </select>
              </div>
            </div>
          </div>
        </div>
      {/if}

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <div class="form-actions">
        {#if editingEvent}
          <button class="cancel-btn" onclick={cancelEdit} disabled={isScheduling}>Cancel</button>
          <button
            class="schedule-btn"
            onclick={handleUpdateEvent}
            disabled={isScheduling || !selectedDate}
          >
            {isScheduling ? "Updating..." : "Update Session"}
          </button>
        {:else}
          <button class="cancel-btn" onclick={onClose} disabled={isScheduling}>Cancel</button>
          <button
            class="schedule-btn pr-schedule"
            onclick={handleSchedule}
            disabled={isScheduling || scheduledSlots.length === 0}
          >
            {isScheduling ? "Scheduling..." : `Schedule ${scheduledSlots.length} slot${scheduledSlots.length !== 1 ? "s" : ""}`}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .scheduler-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .scheduler-modal {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-subtle);
  }

  .scheduler-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .scheduler-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    font-family: var(--font-display);
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 28px;
    cursor: pointer;
    color: var(--text-tertiary);
    line-height: 1;
    padding: 0;
    transition: color 0.15s var(--ease-out);
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .pr-info {
    padding: 16px 24px;
    background: linear-gradient(135deg, var(--accent-purple-dim) 0%, rgba(167, 139, 250, 0.1) 100%);
    border-bottom: 1px solid var(--border-subtle);
  }

  .pr-badges {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .pr-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--accent-purple);
    color: var(--bg-base);
  }

  .repo-badge {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .draft-badge {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--accent-amber-dim);
    color: var(--accent-amber);
  }

  .pr-info h3 {
    margin: 6px 0 8px;
    font-size: 15px;
    font-weight: 500;
    line-height: 1.3;
    color: var(--text-primary);
  }

  .pr-number {
    color: var(--accent-purple);
    font-weight: 600;
  }

  .pr-stats {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .stat {
    font-size: 13px;
    color: var(--text-primary);
  }

  .stat-label {
    color: var(--text-secondary);
  }

  .stat.jira {
    color: var(--accent-blue);
    font-weight: 500;
  }

  .stat.no-jira {
    color: var(--text-tertiary);
    font-style: italic;
  }

  .stat.scheduled {
    color: var(--accent-green);
    font-weight: 500;
  }

  .existing-sessions {
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .sessions-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .sessions-header h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .refresh-btn {
    padding: 4px 10px;
    border: 1px solid var(--border-default);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-size: 12px;
    cursor: pointer;
    color: var(--accent-purple);
    transition: all 0.15s var(--ease-out);
  }

  .refresh-btn:hover {
    background: var(--bg-hover);
  }

  .loading-sessions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    color: var(--text-secondary);
    padding: 8px 0;
  }

  .skip-btn {
    padding: 4px 10px;
    border: none;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-size: 12px;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.15s var(--ease-out);
  }

  .skip-btn:hover {
    background: var(--bg-active);
    color: var(--text-primary);
  }

  .no-sessions {
    font-size: 13px;
    color: var(--text-secondary);
    padding: 8px 0;
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .session-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .session-time {
    font-size: 13px;
    color: var(--text-primary);
  }

  .session-actions {
    display: flex;
    gap: 8px;
  }

  .edit-btn,
  .delete-btn {
    padding: 4px 10px;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .edit-btn {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .edit-btn:hover {
    background: var(--bg-active);
  }

  .delete-btn {
    background: var(--accent-red-dim);
    color: var(--accent-red);
  }

  .delete-btn:hover {
    background: rgba(248, 113, 113, 0.2);
  }

  .scheduler-form {
    padding: 20px 24px;
  }

  .view-toggle {
    display: flex;
    gap: 4px;
    margin-bottom: 12px;
    background: var(--bg-elevated);
    padding: 4px;
    border-radius: var(--radius-md);
  }

  .view-btn {
    flex: 1;
    padding: 8px 12px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.15s var(--ease-out);
  }

  .view-btn:hover {
    color: var(--text-primary);
  }

  .view-btn.active {
    background: var(--bg-active);
    color: var(--text-primary);
    box-shadow: var(--shadow-lg);
  }

  .calendar-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .nav-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .nav-display {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    font-family: var(--font-display);
  }

  .today-btn {
    padding: 4px 10px;
    border: 1px solid var(--border-default);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    color: var(--accent-purple);
    transition: all 0.15s var(--ease-out);
  }

  .today-btn:hover {
    background: var(--bg-hover);
  }

  .nav-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border-default);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    transition: all 0.15s var(--ease-out);
  }

  .nav-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .form-section {
    margin-bottom: 20px;
  }

  .week-view {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
  }

  .week-day-cell {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 4px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    color: var(--text-primary);
  }

  .week-day-cell:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-default);
  }

  .week-day-cell.selected {
    background: var(--accent-purple);
    border-color: var(--accent-purple);
    color: var(--bg-base);
  }

  .week-day-cell.weekend {
    background: var(--bg-base);
  }

  .week-day-cell.weekend.selected {
    background: var(--accent-purple);
  }

  .week-day-cell.today {
    border-color: var(--accent-purple);
    border-width: 2px;
  }

  .week-day-cell.past {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .week-day-cell.has-slot {
    background: var(--accent-purple-dim);
  }

  .week-day-cell.has-slot.selected {
    background: var(--accent-purple);
  }

  .week-day-name {
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    opacity: 0.7;
  }

  .week-day-num {
    font-size: 20px;
    font-weight: 600;
    line-height: 1.2;
    margin: 4px 0;
  }

  .week-day-month {
    font-size: 10px;
    opacity: 0.6;
  }

  .calendar-header {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
    margin-bottom: 8px;
  }

  .calendar-header span {
    text-align: center;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }

  .calendar-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
  }

  .day-cell {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    aspect-ratio: 1;
    border: 1px solid transparent;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    padding: 4px;
    color: var(--text-primary);
  }

  .day-cell:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .day-cell.selected {
    background: var(--accent-purple);
    color: var(--bg-base);
  }

  .day-cell.other-month {
    opacity: 0.4;
  }

  .day-cell.weekend {
    background: var(--bg-base);
  }

  .day-cell.weekend.selected {
    background: var(--accent-purple);
  }

  .day-cell.today {
    border-color: var(--accent-purple);
    border-width: 2px;
  }

  .day-cell.past {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .day-cell.has-slot {
    background: var(--accent-purple-dim);
  }

  .day-cell.has-slot.selected {
    background: var(--accent-purple);
  }

  .day-cell.has-event::after {
    content: "";
    position: absolute;
    bottom: 4px;
    width: 4px;
    height: 4px;
    background: var(--accent-green);
    border-radius: 50%;
  }

  .day-cell.selected.has-event::after {
    background: var(--bg-base);
  }

  .day-num {
    font-size: 14px;
    font-weight: 500;
    line-height: 1;
  }

  .day-indicator {
    position: absolute;
    bottom: 4px;
    width: 6px;
    height: 6px;
    background: var(--accent-purple);
    border-radius: 50%;
  }

  .day-indicator.focus {
    background: var(--accent-amber);
  }

  .day-cell.selected .day-indicator {
    background: var(--bg-base);
  }

  .day-view {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .day-view-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    border: 2px solid var(--border-subtle);
    background: var(--bg-elevated);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    color: var(--text-primary);
  }

  .day-view-cell:hover:not(:disabled) {
    border-color: var(--accent-purple);
    background: var(--bg-hover);
  }

  .day-view-cell.selected {
    background: var(--accent-purple);
    border-color: var(--accent-purple);
    color: var(--bg-base);
  }

  .day-view-cell.today {
    border-color: var(--accent-purple);
  }

  .day-view-cell.past {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .day-view-name {
    font-size: 18px;
    font-weight: 600;
  }

  .day-view-date {
    font-size: 14px;
    opacity: 0.7;
    margin-top: 4px;
  }

  .pending-slots {
    margin-bottom: 16px;
    padding: 12px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .slots-header h4 {
    margin: 0 0 10px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .slots-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .slot-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .slot-info {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }

  .slot-date {
    font-weight: 500;
    color: var(--text-primary);
  }

  .slot-time {
    color: var(--accent-purple);
  }

  .slot-duration {
    color: var(--text-secondary);
  }

  .slot-color {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1px solid var(--border-default);
  }

  .focus-badge {
    padding: 2px 6px;
    background: var(--accent-amber);
    color: var(--bg-base);
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .slot-actions {
    display: flex;
    gap: 6px;
  }

  .focus-toggle {
    width: 24px;
    height: 24px;
    border: 1px solid var(--border-default);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.15s var(--ease-out);
  }

  .focus-toggle.active {
    background: var(--accent-amber);
    border-color: var(--accent-amber);
    color: var(--bg-base);
  }

  .focus-toggle:hover {
    background: var(--bg-hover);
  }

  .focus-toggle.active:hover {
    background: var(--accent-amber);
    filter: brightness(0.9);
  }

  .remove-slot {
    width: 24px;
    height: 24px;
    border: none;
    background: var(--accent-red-dim);
    border-radius: var(--radius-sm);
    font-size: 16px;
    cursor: pointer;
    color: var(--accent-red);
    line-height: 1;
    transition: all 0.15s var(--ease-out);
  }

  .remove-slot:hover {
    background: rgba(248, 113, 113, 0.2);
  }

  .available-slots {
    padding: 16px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    margin-bottom: 16px;
    border: 1px solid var(--border-subtle);
  }

  .available-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    gap: 12px;
    flex-wrap: wrap;
  }

  .available-header h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .availability-controls,
  .edit-options {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .focus-checkbox-inline {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .focus-checkbox-inline input {
    width: 14px;
    height: 14px;
    accent-color: var(--accent-amber);
    color-scheme: dark;
  }

  .color-select-inline {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .color-select {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .color-select select {
    padding: 6px 8px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: 12px;
    background: var(--bg-elevated);
    color: var(--text-primary);
    color-scheme: dark;
  }

  .color-swatch {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1px solid var(--border-default);
  }

  .loading-slots,
  .no-slots {
    text-align: center;
    padding: 20px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .select-day-prompt {
    text-align: center;
    padding: 24px;
    color: var(--text-secondary);
    font-size: 14px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    margin-bottom: 16px;
    border: 1px solid var(--border-subtle);
  }

  .slots-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .available-slot {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 12px;
  }

  .slot-time-range {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .slot-duration-label {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
    margin-bottom: 10px;
  }

  .slot-book-options {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .book-btn {
    padding: 6px 12px;
    border: 1px solid var(--accent-purple);
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    color: var(--accent-purple);
    transition: all 0.15s var(--ease-out);
  }

  .book-btn:hover {
    background: var(--accent-purple);
    color: var(--bg-base);
  }

  .book-btn.book-all {
    background: var(--accent-purple);
    color: var(--bg-base);
  }

  .book-btn.book-all:hover {
    background: var(--accent-purple);
    filter: brightness(0.85);
  }

  .edit-slot-form {
    padding: 16px;
    background: var(--accent-amber-dim);
    border: 1px solid var(--accent-amber);
    border-radius: var(--radius-md);
    margin-bottom: 16px;
  }

  .edit-slot-form h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .edit-row {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
  }

  .edit-group {
    flex: 1;
  }

  .edit-group label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    margin-bottom: 4px;
    color: var(--text-secondary);
  }

  .edit-group input[type="time"] {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: 14px;
    background: var(--bg-elevated);
    color: var(--text-primary);
    color-scheme: dark;
  }

  .edit-duration {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .edit-duration input {
    width: 45px;
    padding: 8px 6px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: 14px;
    text-align: center;
    background: var(--bg-elevated);
    color: var(--text-primary);
    color-scheme: dark;
  }

  .edit-duration span {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .error-message {
    padding: 12px;
    background: var(--accent-red-dim);
    border-radius: var(--radius-md);
    color: var(--accent-red);
    font-size: 14px;
    margin-bottom: 16px;
  }

  .form-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  .cancel-btn,
  .schedule-btn {
    padding: 12px 24px;
    border: none;
    border-radius: var(--radius-md);
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .cancel-btn {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .cancel-btn:hover:not(:disabled) {
    background: var(--bg-active);
  }

  .schedule-btn {
    background: var(--accent-purple);
    color: var(--bg-base);
  }

  .schedule-btn.pr-schedule {
    background: var(--accent-purple);
  }

  .schedule-btn:hover:not(:disabled) {
    background: var(--accent-purple);
    filter: brightness(0.85);
  }

  .schedule-btn.pr-schedule:hover:not(:disabled) {
    background: var(--accent-purple);
    filter: brightness(0.85);
  }

  .schedule-btn:disabled,
  .cancel-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
