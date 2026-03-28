import { writable, get } from "svelte/store";
import * as api from "../api";
import type { SimpleIssue, CalendarEvent } from "../api";
import { config } from "./config";
import { getAccountForCalendar } from "./calendar";
import { format, addDays, subDays } from "date-fns";

export interface SyncedTask extends SimpleIssue {
  calendar_event_uid?: string;
  last_synced?: string;
  allocated_time?: TaskAllocation[];
}

export interface TaskAllocation {
  date: string;
  duration_minutes: number;
  calendar_event_uid?: string;
}

export const tasks = writable<SyncedTask[]>([]);
export const tasksLoading = writable<boolean>(false);
export const tasksError = writable<string | null>(null);

// Match calendar events to tasks based on Jira key in event summary or description
function matchTasksToEvents(taskList: SyncedTask[], events: CalendarEvent[]): SyncedTask[] {
  return taskList.map((task) => {
    // Look for an event that contains the task key in its summary or description
    const matchingEvent = events.find((event) => {
      // Check if summary or description contains the task key
      if (event.summary?.includes(task.key)) {
        return true;
      }
      if (event.description?.includes(task.key)) {
        return true;
      }
      // Check if URL contains the task key
      if (event.url?.includes(task.key)) {
        return true;
      }
      return false;
    });

    if (matchingEvent) {
      return {
        ...task,
        calendar_event_uid: matchingEvent.uid,
        last_synced: matchingEvent.start_date,
      };
    }
    return task;
  });
}

export async function loadAssignedTasks(customJql?: string): Promise<void> {
  tasksLoading.set(true);
  tasksError.set(null);

  try {
    const issues = await api.getAssignedIssues(customJql);
    let syncedTasks: SyncedTask[] = issues.map((issue) => ({
      ...issue,
      calendar_event_uid: undefined,
      last_synced: undefined,
      allocated_time: undefined,
    }));

    // Try to match with calendar events if calendar is configured
    const currentConfig = get(config);
    if (currentConfig.selected_calendar && syncedTasks.length > 0) {
      try {
        const today = new Date();
        const pastDate = subDays(today, 30);
        const futureDate = addDays(today, 60);
        const startDate = format(pastDate, "yyyy-MM-dd");
        const endDate = format(futureDate, "yyyy-MM-dd");

        // Get all events for the date range - we'll match by task keys
        const events = await api.getEventsForDateRange(
          getAccountForCalendar(currentConfig.selected_calendar) ?? "",
          currentConfig.selected_calendar,
          startDate,
          endDate
        );

        syncedTasks = matchTasksToEvents(syncedTasks, events);
      } catch (calendarError) {
        console.warn("Failed to match tasks with calendar events:", calendarError);
        // Continue without calendar matching
      }
    }

    tasks.set(syncedTasks);
  } catch (error) {
    tasksError.set(error instanceof Error ? error.message : String(error));
  } finally {
    tasksLoading.set(false);
  }
}

export async function searchTasks(
  jql: string,
  maxResults?: number
): Promise<void> {
  tasksLoading.set(true);
  tasksError.set(null);

  try {
    const issues = await api.searchIssues(jql, maxResults);
    let syncedTasks: SyncedTask[] = issues.map((issue) => ({
      ...issue,
      calendar_event_uid: undefined,
      last_synced: undefined,
      allocated_time: undefined,
    }));

    // Try to match with calendar events if calendar is configured
    const currentConfig = get(config);
    if (currentConfig.selected_calendar && syncedTasks.length > 0) {
      try {
        const today = new Date();
        const pastDate = subDays(today, 30);
        const futureDate = addDays(today, 60);
        const startDate = format(pastDate, "yyyy-MM-dd");
        const endDate = format(futureDate, "yyyy-MM-dd");

        const events = await api.getEventsForDateRange(
          getAccountForCalendar(currentConfig.selected_calendar) ?? "",
          currentConfig.selected_calendar,
          startDate,
          endDate
        );

        syncedTasks = matchTasksToEvents(syncedTasks, events);
      } catch (calendarError) {
        console.warn("Failed to match tasks with calendar events:", calendarError);
      }
    }

    tasks.set(syncedTasks);
  } catch (error) {
    tasksError.set(error instanceof Error ? error.message : String(error));
  } finally {
    tasksLoading.set(false);
  }
}

export async function refreshTaskStatus(issueKey: string): Promise<void> {
  try {
    const updatedIssue = await api.getIssueStatus(issueKey);
    tasks.update((currentTasks) =>
      currentTasks.map((task) =>
        task.key === issueKey
          ? {
              ...task,
              status: updatedIssue.status,
              status_category: updatedIssue.status_category,
            }
          : task
      )
    );
  } catch (error) {
    console.error(`Failed to refresh status for ${issueKey}:`, error);
  }
}

export function updateTaskCalendarEvent(
  issueKey: string,
  eventUid: string
): void {
  tasks.update((currentTasks) =>
    currentTasks.map((task) =>
      task.key === issueKey
        ? {
            ...task,
            calendar_event_uid: eventUid,
            last_synced: new Date().toISOString(),
          }
        : task
    )
  );
}

export function updateTaskAllocation(
  issueKey: string,
  allocations: TaskAllocation[]
): void {
  tasks.update((currentTasks) =>
    currentTasks.map((task) =>
      task.key === issueKey
        ? {
            ...task,
            allocated_time: allocations,
          }
        : task
    )
  );
}

export function clearTaskCalendarEvent(issueKey: string): void {
  tasks.update((currentTasks) =>
    currentTasks.map((task) =>
      task.key === issueKey
        ? {
            ...task,
            calendar_event_uid: undefined,
            last_synced: undefined,
          }
        : task
    )
  );
}

export function getCompletedTasks(): SyncedTask[] {
  return get(tasks).filter((task) => task.status_category === "done");
}

export function getActiveTasks(): SyncedTask[] {
  return get(tasks).filter((task) => task.status_category !== "done");
}

export function getSyncedTasks(): SyncedTask[] {
  return get(tasks).filter((task) => task.calendar_event_uid !== undefined);
}
