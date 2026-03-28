import { writable, get } from "svelte/store";
import * as api from "../api";
import type { CalendarInfo, CalendarEvent } from "../api";

export const calendars = writable<CalendarInfo[]>([]);
export const calendarsLoading = writable<boolean>(false);
export const calendarsError = writable<string | null>(null);

export const calendarEvents = writable<CalendarEvent[]>([]);
export const eventsLoading = writable<boolean>(false);
export const eventsError = writable<string | null>(null);

// Per-account calendar map: email -> CalendarInfo[]
export const accountCalendars = writable<Map<string, CalendarInfo[]>>(new Map());

export async function loadCalendarsForAccount(email: string): Promise<CalendarInfo[]> {
  try {
    const cals = await api.getCalendars(email);
    accountCalendars.update((m) => {
      const next = new Map(m);
      next.set(email, cals);
      return next;
    });
    // Also update the flat list for backward compat
    const allCals: CalendarInfo[] = [];
    const map = get(accountCalendars);
    for (const [, list] of map) {
      allCals.push(...list);
    }
    calendars.set(allCals);
    return cals;
  } catch (error) {
    calendarsError.set(error instanceof Error ? error.message : String(error));
    return [];
  }
}

export async function loadCalendarsForAllAccounts(emails: string[]): Promise<void> {
  calendarsLoading.set(true);
  calendarsError.set(null);
  try {
    const newMap = new Map<string, CalendarInfo[]>();
    const allCals: CalendarInfo[] = [];
    for (const email of emails) {
      const cals = await api.getCalendars(email);
      newMap.set(email, cals);
      allCals.push(...cals);
    }
    accountCalendars.set(newMap);
    calendars.set(allCals);
  } catch (error) {
    calendarsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    calendarsLoading.set(false);
  }
}

// Legacy compat
export async function loadCalendars(): Promise<void> {
  // No-op — use loadCalendarsForAllAccounts instead
}

export async function loadEventsForDateRange(
  calendarName: string,
  startDate: string,
  endDate: string
): Promise<void> {
  eventsLoading.set(true);
  eventsError.set(null);

  try {
    // Find which account owns this calendar
    const map = get(accountCalendars);
    let accountEmail = "";
    for (const [email, cals] of map) {
      if (cals.some((c) => c.uid === calendarName)) {
        accountEmail = email;
        break;
      }
    }
    if (!accountEmail) {
      throw new Error(`No account found for calendar ${calendarName}`);
    }
    const events = await api.getEventsForDateRange(accountEmail, calendarName, startDate, endDate);
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

export function getAccountForCalendar(calendarUid: string): string | undefined {
  const map = get(accountCalendars);
  for (const [email, cals] of map) {
    if (cals.some((c) => c.uid === calendarUid)) return email;
  }
  return undefined;
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
    return newStart < eventEnd && newEnd > eventStart;
  });
}
