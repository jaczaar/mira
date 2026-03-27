import { writable, get } from "svelte/store";
import * as api from "../api";
import { config } from "./config";
import { tasks, updateTaskCalendarEvent, clearTaskCalendarEvent } from "./tasks";
import type { SyncedTask } from "./tasks";
import type { CreateEventRequest } from "../api";

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

function formatEventTitle(template: string, task: SyncedTask): string {
  return template
    .replace("{key}", task.key)
    .replace("{summary}", task.summary)
    .replace("{project}", task.project_key)
    .replace("{priority}", task.priority || "None")
    .replace("{status}", task.status)
    .replace("{type}", task.issue_type || "Task");
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

  const activeTasks = currentTasks.filter((t) => t.status_category !== "done");
  const total = activeTasks.length;

  syncState.set({
    status: "syncing",
    lastSync: null,
    message: "Starting sync...",
    progress: 0,
    total,
  });

  for (let i = 0; i < activeTasks.length; i++) {
    const task = activeTasks[i];

    syncState.update((s) => ({
      ...s,
      message: `Syncing ${task.key}...`,
      progress: i + 1,
    }));

    try {
      const title = formatEventTitle(currentConfig.event_title_template, task);
      const description = `Jira Issue: ${task.key}\n${task.summary}\n\nStatus: ${task.status}\nProject: ${task.project_name}`;

      // Default to 1 hour event starting now if no time estimate
      const now = new Date();
      const startDate = now.toISOString().slice(0, 19);
      const endDate = new Date(now.getTime() + 60 * 60 * 1000)
        .toISOString()
        .slice(0, 19);

      const eventRequest: CreateEventRequest = {
        summary: title,
        start_date: startDate,
        end_date: endDate,
        description,
        url: task.url,
        calendar_name: currentConfig.selected_calendar,
        is_focus_time: false,
        color_id: currentConfig.default_event_color,
      };

      // For now, always create new events
      // TODO: Check for existing events and update instead
      const eventUid = await api.createEvent(eventRequest);
      updateTaskCalendarEvent(task.key, eventUid);
      result.created++;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      result.errors.push(`Failed to sync ${task.key}: ${errorMessage}`);
    }
  }

  // Handle completed tasks - delete their calendar events
  const completedTasks = currentTasks.filter(
    (t) => t.status_category === "done" && t.calendar_event_uid
  );

  for (const task of completedTasks) {
    try {
      if (task.calendar_event_uid && currentConfig.selected_calendar) {
        await api.deleteEvent(task.calendar_event_uid, currentConfig.selected_calendar);
        clearTaskCalendarEvent(task.key);
        result.deleted++;
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      result.errors.push(`Failed to delete event for ${task.key}: ${errorMessage}`);
    }
  }

  const finalStatus: SyncStatus = result.errors.length > 0 ? "error" : "success";
  const message =
    result.errors.length > 0
      ? `Sync completed with ${result.errors.length} errors`
      : `Sync completed: ${result.created} created, ${result.updated} updated, ${result.deleted} deleted`;

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
  endDate: string
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
      currentConfig.selected_calendar,
      startDate,
      endDate
    );

    // Filter events that match Jira tasks
    const jiraEvents = events.filter((event) => {
      // Check if summary contains a Jira key pattern (e.g., [PROJ-123])
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
        // Calculate duration from event times
        const start = new Date(event.start_date);
        const end = new Date(event.end_date);
        const durationSeconds = Math.floor((end.getTime() - start.getTime()) / 1000);

        if (durationSeconds > 0) {
          await api.createWorklog(
            issueKey,
            durationSeconds,
            start.toISOString(),
            `Time logged from calendar event: ${event.summary}`
          );
          result.created++;
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        result.errors.push(`Failed to log time for ${issueKey}: ${errorMessage}`);
      }
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    result.errors.push(`Failed to fetch calendar events: ${errorMessage}`);
  }

  const finalStatus: SyncStatus = result.errors.length > 0 ? "error" : "success";
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
