import { writable } from "svelte/store";
import type { ScheduleWindow } from "../api";

const WINDOWS_KEY = "mira.calendarBoxes.windows.v1";
const ORDER_KEY = "mira.calendarBoxes.order.v1";

const DEFAULT_WINDOW: ScheduleWindow = {
  start_hour: 9,
  end_hour: 17,
  days: [1, 2, 3, 4, 5],
};

function loadWindows(): Record<string, ScheduleWindow> {
  if (typeof localStorage === "undefined") return {};
  try {
    const raw = localStorage.getItem(WINDOWS_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object") return parsed;
    return {};
  } catch {
    return {};
  }
}

function loadOrder(): string[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(ORDER_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter((x) => typeof x === "string") : [];
  } catch {
    return [];
  }
}

export const calendarWindows = writable<Record<string, ScheduleWindow>>(loadWindows());
export const calendarOrder = writable<string[]>(loadOrder());

calendarWindows.subscribe((value) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(WINDOWS_KEY, JSON.stringify(value));
  } catch {
    /* ignore */
  }
});

calendarOrder.subscribe((value) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(ORDER_KEY, JSON.stringify(value));
  } catch {
    /* ignore */
  }
});

export function getCalendarWindow(uid: string, fallback?: ScheduleWindow): ScheduleWindow {
  let result = fallback ?? DEFAULT_WINDOW;
  const unsubscribe = calendarWindows.subscribe((map) => {
    if (map[uid]) result = map[uid];
  });
  unsubscribe();
  return result;
}

export function setCalendarWindow(uid: string, win: ScheduleWindow): void {
  calendarWindows.update((map) => ({ ...map, [uid]: win }));
}

export function reorderCalendar(uid: string, delta: number, visibleUids: string[]): void {
  calendarOrder.update((current) => {
    // Materialize current ordering for the visible set.
    const ordered = orderedVisibleUids(current, visibleUids);
    const idx = ordered.indexOf(uid);
    if (idx < 0) return current;
    const target = idx + delta;
    if (target < 0 || target >= ordered.length) return current;
    [ordered[idx], ordered[target]] = [ordered[target], ordered[idx]];
    return ordered;
  });
}

export function orderedVisibleUids(orderList: string[], visibleUids: string[]): string[] {
  const visibleSet = new Set(visibleUids);
  const result: string[] = [];
  const added = new Set<string>();
  for (const uid of orderList) {
    if (visibleSet.has(uid) && !added.has(uid)) {
      result.push(uid);
      added.add(uid);
    }
  }
  for (const uid of visibleUids) {
    if (!added.has(uid)) {
      result.push(uid);
      added.add(uid);
    }
  }
  return result;
}

export const DEFAULT_CALENDAR_WINDOW = DEFAULT_WINDOW;
