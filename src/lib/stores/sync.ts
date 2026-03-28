import { writable, get } from "svelte/store";
import * as api from "../api";
import { config } from "./config";
import { getAccountForCalendar } from "./calendar";
import { tasks, updateTaskCalendarEvent, clearTaskCalendarEvent } from "./tasks";
import type { SyncedTask } from "./tasks";
import type { CalendarEvent, CreateEventRequest } from "../api";
import { format, addDays, parseISO, isSameDay } from "date-fns";

export type SyncStatus = "idle" | "syncing" | "success" | "error";

export interface SyncState {
  status: SyncStatus;
  lastSync: string | null;
  message: string | null;
  progress: number;
  total: number;
}

export const syncState = writable<SyncState>({
  status: "idle",
  lastSync: null,
  message: null,
  progress: 0,
  total: 0,
});

export interface SyncResult {
  created: number;
  updated: number;
  deleted: number;
  errors: string[];
}

const WORK_START = 8; // 8 AM
const WORK_END = 18; // 6 PM
const MIN_SLOT_MINUTES = 30;
const DEFAULT_TASK_MINUTES = 60;
const BUFFER_MINUTES = 0;
const SCHEDULE_DAYS = 14; // Look ahead 2 weeks

interface FreeSlot {
  date: string; // YYYY-MM-DD
  startMinute: number;
  endMinute: number;
  durationMinutes: number;
}

function formatEventTitle(template: string, task: SyncedTask): string {
  return template
    .replace("{key}", task.key)
    .replace("{summary}", task.summary)
    .replace("{project}", task.project_key)
    .replace("{priority}", task.priority || "None")
    .replace("{status}", task.status)
    .replace("{type}", task.issue_type || "Task");
}

function minutesToTimeStr(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:00`;
}

function priorityScore(priority: string | null): number {
  switch (priority?.toLowerCase()) {
    case "highest":
      return 0;
    case "high":
      return 1;
    case "medium":
      return 2;
    case "low":
      return 3;
    case "lowest":
      return 4;
    default:
      return 2;
  }
}

function getTaskDuration(task: SyncedTask): number {
  if (task.time_estimate_seconds && task.time_estimate_seconds > 0) {
    const minutes = Math.round(task.time_estimate_seconds / 60);
    return Math.max(MIN_SLOT_MINUTES, minutes);
  }
  return DEFAULT_TASK_MINUTES;
}

function buildBusyPeriods(events: CalendarEvent[], date: string): { start: number; end: number }[] {
  return events
    .filter((event) => {
      try {
        const eventStart = parseISO(event.start_date);
        return isSameDay(eventStart, parseISO(date));
      } catch {
        return false;
      }
    })
    .map((event) => {
      try {
        const start = parseISO(event.start_date);
        const end = parseISO(event.end_date);
        return {
          start: start.getHours() * 60 + start.getMinutes(),
          end: end.getHours() * 60 + end.getMinutes(),
        };
      } catch {
        return { start: 0, end: 0 };
      }
    })
    .filter((p) => p.start !== p.end)
    .sort((a, b) => a.start - b.start);
}

function findFreeSlots(
  busyPeriods: { start: number; end: number }[],
  date: string,
  isToday: boolean,
  now: Date,
): FreeSlot[] {
  const slots: FreeSlot[] = [];

  let currentMinute = WORK_START * 60;
  if (isToday) {
    const nowMinutes = now.getHours() * 60 + now.getMinutes();
    currentMinute = Math.max(currentMinute, Math.ceil(nowMinutes / 30) * 30);
  }

  const endMinute = WORK_END * 60;

  for (const busy of busyPeriods) {
    if (busy.start > currentMinute && busy.start <= endMinute) {
      const gapDuration = busy.start - currentMinute;
      if (gapDuration >= MIN_SLOT_MINUTES) {
        slots.push({
          date,
          startMinute: currentMinute,
          endMinute: busy.start,
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
        date,
        startMinute: currentMinute,
        endMinute: endMinute,
        durationMinutes: gapDuration,
      });
    }
  }

  return slots;
}

async function getFreeSlotsByDay(
  accountEmail: string,
  calendarName: string,
  days: number,
): Promise<FreeSlot[]> {
  const now = new Date();
  const today = format(now, "yyyy-MM-dd");
  const endDate = format(addDays(now, days), "yyyy-MM-dd");

  const events = await api.getEventsForDateRange(
    accountEmail,
    calendarName,
    today,
    endDate,
  );

  const allSlots: FreeSlot[] = [];

  for (let d = 0; d < days; d++) {
    const date = format(addDays(now, d), "yyyy-MM-dd");
    const dayOfWeek = addDays(now, d).getDay();

    // Skip weekends
    if (dayOfWeek === 0 || dayOfWeek === 6) continue;

    const busyPeriods = buildBusyPeriods(events, date);
    const daySlots = findFreeSlots(busyPeriods, date, d === 0, now);
    allSlots.push(...daySlots);
  }

  return allSlots;
}

function allocateTaskToSlots(
  taskDuration: number,
  freeSlots: FreeSlot[],
): { date: string; startMinute: number; endMinute: number }[] {
  const allocations: { date: string; startMinute: number; endMinute: number }[] = [];
  let remaining = taskDuration;

  for (let i = 0; i < freeSlots.length && remaining > 0; i++) {
    const slot = freeSlots[i];
    const allocatable = Math.min(remaining, slot.durationMinutes);

    if (allocatable >= MIN_SLOT_MINUTES) {
      allocations.push({
        date: slot.date,
        startMinute: slot.startMinute,
        endMinute: slot.startMinute + allocatable,
      });

      // Shrink the slot for future tasks
      freeSlots[i] = {
        ...slot,
        startMinute: slot.startMinute + allocatable + BUFFER_MINUTES,
        durationMinutes: slot.durationMinutes - allocatable - BUFFER_MINUTES,
      };

      // Remove slot if too small
      if (freeSlots[i].durationMinutes < MIN_SLOT_MINUTES) {
        freeSlots.splice(i, 1);
        i--;
      }

      remaining -= allocatable;
    }
  }

  // If we couldn't fit everything but have some, return what we got
  // If we couldn't fit anything, force into the first available slot
  if (allocations.length === 0 && freeSlots.length > 0) {
    const slot = freeSlots[0];
    const duration = Math.min(DEFAULT_TASK_MINUTES, slot.durationMinutes);
    allocations.push({
      date: slot.date,
      startMinute: slot.startMinute,
      endMinute: slot.startMinute + duration,
    });
    freeSlots[0] = {
      ...slot,
      startMinute: slot.startMinute + duration,
      durationMinutes: slot.durationMinutes - duration,
    };
    if (freeSlots[0].durationMinutes < MIN_SLOT_MINUTES) {
      freeSlots.splice(0, 1);
    }
  }

  return allocations;
}

export async function syncTasksToCalendar(): Promise<SyncResult> {
  const currentConfig = get(config);
  const currentTasks = get(tasks);

  if (!currentConfig.selected_calendar) {
    throw new Error("No calendar selected. Please configure a calendar first.");
  }

  const result: SyncResult = {
    created: 0,
    updated: 0,
    deleted: 0,
    errors: [],
  };

  // Sort tasks: already-synced tasks skip scheduling, then by priority, then by due date
  const activeTasks = currentTasks
    .filter((t) => t.status_category !== "done" && !t.calendar_event_uid)
    .sort((a, b) => {
      // Priority first
      const pDiff = priorityScore(a.priority) - priorityScore(b.priority);
      if (pDiff !== 0) return pDiff;
      // Then by due date (earlier first, no due date last)
      if (a.due_date && b.due_date) return a.due_date.localeCompare(b.due_date);
      if (a.due_date) return -1;
      if (b.due_date) return 1;
      return 0;
    });

  const total = activeTasks.length;

  if (total === 0) {
    syncState.set({
      status: "success",
      lastSync: new Date().toISOString(),
      message: "All tasks are already synced or completed.",
      progress: 0,
      total: 0,
    });
    return result;
  }

  syncState.set({
    status: "syncing",
    lastSync: null,
    message: "Analyzing calendar availability...",
    progress: 0,
    total,
  });

  const accountEmail =
    getAccountForCalendar(currentConfig.selected_calendar) ?? "";

  // Fetch free slots across the next 2 weeks
  const freeSlots = await getFreeSlotsByDay(
    accountEmail,
    currentConfig.selected_calendar,
    SCHEDULE_DAYS,
  );

  // Schedule each task into available slots
  for (let i = 0; i < activeTasks.length; i++) {
    const task = activeTasks[i];
    const taskDuration = getTaskDuration(task);

    syncState.update((s) => ({
      ...s,
      message: `Scheduling ${task.key} (${Math.round(taskDuration / 60)}h ${taskDuration % 60}m)...`,
      progress: i + 1,
    }));

    const allocations = allocateTaskToSlots(taskDuration, freeSlots);

    if (allocations.length === 0) {
      result.errors.push(
        `No available time for ${task.key} in the next ${SCHEDULE_DAYS} days`,
      );
      continue;
    }

    // Create a calendar event for each allocation block
    for (const alloc of allocations) {
      try {
        const title = formatEventTitle(
          currentConfig.event_title_template,
          task,
        );
        const description = `Jira: ${task.key}\n${task.summary}\n\nStatus: ${task.status}\nPriority: ${task.priority || "None"}\nProject: ${task.project_name}`;

        const startDate = `${alloc.date}T${minutesToTimeStr(alloc.startMinute)}`;
        const endDate = `${alloc.date}T${minutesToTimeStr(alloc.endMinute)}`;

        const eventRequest: CreateEventRequest = {
          summary: title,
          start_date: startDate,
          end_date: endDate,
          description,
          url: task.url,
          calendar_name: currentConfig.selected_calendar!,
          is_focus_time: false,
          color_id: currentConfig.default_event_color,
        };

        const eventUid = await api.createEvent(accountEmail, eventRequest);
        updateTaskCalendarEvent(task.key, eventUid);
        result.created++;
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        result.errors.push(`Failed to sync ${task.key}: ${errorMessage}`);
      }
    }
  }

  // Handle completed tasks — delete their calendar events
  const completedTasks = currentTasks.filter(
    (t) => t.status_category === "done" && t.calendar_event_uid,
  );

  for (const task of completedTasks) {
    try {
      if (task.calendar_event_uid && currentConfig.selected_calendar) {
        await api.deleteEvent(
          accountEmail,
          task.calendar_event_uid,
          currentConfig.selected_calendar,
        );
        clearTaskCalendarEvent(task.key);
        result.deleted++;
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      result.errors.push(
        `Failed to delete event for ${task.key}: ${errorMessage}`,
      );
    }
  }

  const finalStatus: SyncStatus =
    result.errors.length > 0 ? "error" : "success";
  const message =
    result.errors.length > 0
      ? `Sync completed with ${result.errors.length} errors`
      : `Scheduled ${result.created} events across ${SCHEDULE_DAYS} days (${result.deleted} completed removed)`;

  syncState.set({
    status: finalStatus,
    lastSync: new Date().toISOString(),
    message,
    progress: total,
    total,
  });

  return result;
}

export async function syncCalendarToWorklogs(
  startDate: string,
  endDate: string,
): Promise<SyncResult> {
  const currentConfig = get(config);

  if (!currentConfig.selected_calendar) {
    throw new Error("No calendar selected. Please configure a calendar first.");
  }

  const result: SyncResult = {
    created: 0,
    updated: 0,
    deleted: 0,
    errors: [],
  };

  syncState.set({
    status: "syncing",
    lastSync: null,
    message: "Fetching calendar events...",
    progress: 0,
    total: 0,
  });

  try {
    const events = await api.getEventsForDateRange(
      getAccountForCalendar(currentConfig.selected_calendar) ?? "",
      currentConfig.selected_calendar,
      startDate,
      endDate,
    );

    // Filter events that match Jira tasks
    const jiraEvents = events.filter((event) => {
      const keyMatch = event.summary.match(/\[([A-Z]+-\d+)\]/);
      return keyMatch !== null;
    });

    syncState.update((s) => ({
      ...s,
      message: `Found ${jiraEvents.length} Jira-related events`,
      total: jiraEvents.length,
    }));

    for (let i = 0; i < jiraEvents.length; i++) {
      const event = jiraEvents[i];
      const keyMatch = event.summary.match(/\[([A-Z]+-\d+)\]/);

      if (!keyMatch) continue;

      const issueKey = keyMatch[1];

      syncState.update((s) => ({
        ...s,
        message: `Logging time for ${issueKey}...`,
        progress: i + 1,
      }));

      try {
        const start = new Date(event.start_date);
        const end = new Date(event.end_date);
        const durationSeconds = Math.floor(
          (end.getTime() - start.getTime()) / 1000,
        );

        if (durationSeconds > 0) {
          await api.createWorklog(
            issueKey,
            durationSeconds,
            start.toISOString(),
            `Time logged from calendar event: ${event.summary}`,
          );
          result.created++;
        }
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        result.errors.push(
          `Failed to log time for ${issueKey}: ${errorMessage}`,
        );
      }
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    result.errors.push(`Failed to fetch calendar events: ${errorMessage}`);
  }

  const finalStatus: SyncStatus =
    result.errors.length > 0 ? "error" : "success";
  const message =
    result.errors.length > 0
      ? `Worklog sync completed with ${result.errors.length} errors`
      : `Worklog sync completed: ${result.created} worklogs created`;

  syncState.set({
    status: finalStatus,
    lastSync: new Date().toISOString(),
    message,
    progress: result.created,
    total: result.created,
  });

  return result;
}

export function resetSyncState(): void {
  syncState.set({
    status: "idle",
    lastSync: null,
    message: null,
    progress: 0,
    total: 0,
  });
}
