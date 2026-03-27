import { writable, get } from "svelte/store";
import * as api from "../api";
import type { CalendarInfo, CalendarEvent } from "../api";

export const calendars = writable<CalendarInfo[]>([]);
export const calendarsLoading = writable<boolean>(false);
export const calendarsError = writable<string | null>(null);

export const calendarEvents = writable<CalendarEvent[]>([]);
export const eventsLoading = writable<boolean>(false);
export const eventsError = writable<string | null>(null);

export async function loadCalendars(): Promise<void> {
  calendarsLoading.set(true);
  calendarsError.set(null);

  try {
    const loadedCalendars = await api.getCalendars();
    calendars.set(loadedCalendars);
  } catch (error) {
    calendarsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    calendarsLoading.set(false);
  }
}

export async function loadEventsForDateRange(
  calendarName: string,
  startDate: string,
  endDate: string
): Promise<void> {
  eventsLoading.set(true);
  eventsError.set(null);

  try {
    const events = await api.getEventsForDateRange(calendarName, startDate, endDate);
    calendarEvents.set(events);
  } catch (error) {
    eventsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    eventsLoading.set(false);
  }
}

export function clearEvents(): void {
  calendarEvents.set([]);
  eventsError.set(null);
}

export function getEventsByDate(date: string): CalendarEvent[] {
  const events = get(calendarEvents);
  return events.filter((event) => {
    const eventDate = event.start_date.split("T")[0];
    return eventDate === date;
  });
}

export function hasConflict(
  startDate: string,
  endDate: string
): CalendarEvent[] {
  const events = get(calendarEvents);
  const newStart = new Date(startDate).getTime();
  const newEnd = new Date(endDate).getTime();

  return events.filter((event) => {
    const eventStart = new Date(event.start_date).getTime();
    const eventEnd = new Date(event.end_date).getTime();

    // Check for overlap
    return newStart < eventEnd && newEnd > eventStart;
  });
}
