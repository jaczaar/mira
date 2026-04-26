import { get } from "svelte/store";
import * as api from "../api";
import type { ScheduleWindow } from "../api";
import { config } from "../stores/config";
import {
  accountCalendars,
  createCalendarEvent,
} from "../stores/calendar";
import {
  calendarOrder,
  calendarWindows,
  orderedVisibleUids,
  DEFAULT_CALENDAR_WINDOW,
} from "../stores/calendarBoxes";
import type { SidebarTask } from "../stores/sidebarTasks";

export type ScheduleMode = "smart" | "chunk";

export interface AutoScheduleResult {
  scheduled: { taskId: string; placements: { date: string; start: string; end: string }[] }[];
  failed: { taskId: string; reason: string }[];
}

const HORIZON_DAYS = 14;
const SLOT_GRANULARITY_MIN = 15;
const MIN_CHUNK_MIN = 30;

interface Busy {
  start: number;
  end: number;
}

interface Gap {
  ts: number;
  date: string;
  start: number;
  end: number;
}

interface CalendarSource {
  calendarUid: string;
  accountEmail: string;
}

function getEnabledCalendars(): CalendarSource[] {
  const cfg = get(config);
  const map = get(accountCalendars);
  const enabledSet = new Set(cfg.enabled_calendars ?? []);
  const sources: CalendarSource[] = [];
  for (const [email, cals] of map) {
    for (const cal of cals) {
      if (enabledSet.size === 0 || enabledSet.has(cal.uid)) {
        sources.push({ calendarUid: cal.uid, accountEmail: email });
      }
    }
  }
  return sources;
}

function getAccountWindow(email: string): ScheduleWindow {
  const cfg = get(config);
  const win = (cfg.account_schedule_windows ?? {})[email];
  if (!win) return DEFAULT_CALENDAR_WINDOW;
  return {
    start_hour: win.start_hour,
    end_hour: win.end_hour,
    days: win.days?.length ? win.days : DEFAULT_CALENDAR_WINDOW.days,
  };
}

function windowFor(uid: string, accountEmail: string): ScheduleWindow {
  const winsMap = get(calendarWindows);
  if (winsMap[uid]) {
    const w = winsMap[uid];
    return {
      start_hour: w.start_hour,
      end_hour: w.end_hour,
      days: w.days?.length ? w.days : DEFAULT_CALENDAR_WINDOW.days,
    };
  }
  return getAccountWindow(accountEmail);
}

function ymd(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function hm(minutes: number): string {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

function eventBusy(ev: api.CalendarEvent, dayStart: Date, dayEnd: Date): Busy | null {
  try {
    const s = new Date(ev.start_date).getTime();
    const e = new Date(ev.end_date).getTime();
    if (e <= dayStart.getTime() || s >= dayEnd.getTime()) return null;
    const clampedS = Math.max(s, dayStart.getTime());
    const clampedE = Math.min(e, dayEnd.getTime());
    return {
      start: Math.floor((clampedS - dayStart.getTime()) / 60000),
      end: Math.ceil((clampedE - dayStart.getTime()) / 60000),
    };
  } catch {
    return null;
  }
}

function mergeBusy(busies: Busy[]): Busy[] {
  if (busies.length === 0) return [];
  const sorted = [...busies].sort((a, b) => a.start - b.start);
  const out: Busy[] = [{ ...sorted[0] }];
  for (let i = 1; i < sorted.length; i++) {
    const last = out[out.length - 1];
    const cur = sorted[i];
    if (cur.start <= last.end) last.end = Math.max(last.end, cur.end);
    else out.push({ ...cur });
  }
  return out;
}

function buildGapsForDay(
  dayDate: Date,
  busy: Busy[],
  windowStart: number,
  windowEnd: number,
  earliestStart: number
): Gap[] {
  const dayStartMs = dayDate.getTime();
  const gaps: Gap[] = [];
  let cursor = Math.max(windowStart, earliestStart);
  cursor = Math.ceil(cursor / SLOT_GRANULARITY_MIN) * SLOT_GRANULARITY_MIN;
  for (const b of busy) {
    if (cursor + SLOT_GRANULARITY_MIN <= b.start && cursor < windowEnd) {
      const end = Math.min(b.start, windowEnd);
      if (end - cursor >= SLOT_GRANULARITY_MIN) {
        gaps.push({ ts: dayStartMs + cursor * 60000, date: ymd(dayDate), start: cursor, end });
      }
    }
    cursor = Math.max(cursor, b.end);
    cursor = Math.ceil(cursor / SLOT_GRANULARITY_MIN) * SLOT_GRANULARITY_MIN;
  }
  if (cursor < windowEnd) {
    gaps.push({ ts: dayStartMs + cursor * 60000, date: ymd(dayDate), start: cursor, end: windowEnd });
  }
  return gaps;
}

async function placeEvent(
  accountEmail: string,
  calendarUid: string,
  task: SidebarTask,
  date: string,
  startMin: number,
  endMin: number
) {
  const summary = task.jiraKey ? `[${task.jiraKey}] ${task.title}` : task.title;
  await createCalendarEvent(accountEmail, {
    summary,
    start_date: `${date}T${hm(startMin)}:00`,
    end_date: `${date}T${hm(endMin)}:00`,
    description: task.jiraUrl ?? null,
    url: task.jiraUrl ?? null,
    calendar_name: calendarUid,
    is_focus_time: false,
    color_id: null,
  });
}

interface DayBusy {
  date: Date;
  busies: Busy[]; // accumulated busy ranges for this day, merged.
}

function placeBusyFromEvents(daysBusy: Map<string, DayBusy>, events: api.CalendarEvent[]) {
  for (const ev of events) {
    for (const [, db] of daysBusy) {
      const dayEnd = new Date(db.date);
      dayEnd.setDate(db.date.getDate() + 1);
      const b = eventBusy(ev, db.date, dayEnd);
      if (b) db.busies.push(b);
    }
  }
}

export async function autoScheduleActive(
  tasks: SidebarTask[],
  mode: ScheduleMode = "smart"
): Promise<AutoScheduleResult> {
  const result: AutoScheduleResult = { scheduled: [], failed: [] };
  if (tasks.length === 0) return result;

  const sources = getEnabledCalendars();
  if (sources.length === 0) {
    for (const t of tasks)
      result.failed.push({ taskId: t.id, reason: "No visible calendar" });
    return result;
  }
  const sourceByUid = new Map(sources.map((s) => [s.calendarUid, s]));
  const visibleUids = sources.map((s) => s.calendarUid);
  const orderedUids = orderedVisibleUids(get(calendarOrder), visibleUids);

  // Group tasks by their calendarUid; preserve order within each bucket.
  const tasksByCal = new Map<string, SidebarTask[]>();
  for (const t of tasks) {
    if (!t.calendarUid) {
      result.failed.push({ taskId: t.id, reason: "Task is not assigned to a calendar" });
      continue;
    }
    if (!sourceByUid.has(t.calendarUid)) {
      result.failed.push({
        taskId: t.id,
        reason: "Calendar is not visible",
      });
      continue;
    }
    const arr = tasksByCal.get(t.calendarUid) ?? [];
    arr.push(t);
    tasksByCal.set(t.calendarUid, arr);
  }
  if (tasksByCal.size === 0) return result;

  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const horizonEnd = new Date(today);
  horizonEnd.setDate(today.getDate() + HORIZON_DAYS);

  // Pull busy events from every visible calendar in parallel — this is the
  // baseline busy map shared across all calendar boxes.
  const fetched = await Promise.all(
    sources.map((src) =>
      api
        .getEventsForDateRange(src.accountEmail, src.calendarUid, ymd(today), ymd(horizonEnd))
        .catch(() => [] as api.CalendarEvent[])
    )
  );
  const allEvents: api.CalendarEvent[] = fetched.flat();

  // Pre-build one entry per calendar day in the horizon. We populate per box.
  const horizonDates: Date[] = [];
  for (let i = 0; i < HORIZON_DAYS; i++) {
    const d = new Date(today);
    d.setDate(today.getDate() + i);
    horizonDates.push(d);
  }

  // Iterate calendar boxes in user-priority order.
  for (const uid of orderedUids) {
    const queue = tasksByCal.get(uid);
    if (!queue || queue.length === 0) continue;
    const src = sourceByUid.get(uid);
    if (!src) continue;
    const win = windowFor(uid, src.accountEmail);
    const allowedDays = new Set(win.days);
    const windowStart = win.start_hour * 60;
    const windowEnd = win.end_hour * 60;

    // Build busy map for this box's relevant days.
    const daysBusy = new Map<string, DayBusy>();
    for (const d of horizonDates) {
      if (!allowedDays.has(d.getDay())) continue;
      daysBusy.set(ymd(d), { date: new Date(d), busies: [] });
    }
    placeBusyFromEvents(daysBusy, allEvents);

    // Sort and merge busy per day.
    let gaps: Gap[] = [];
    const orderedDays = [...daysBusy.values()].sort((a, b) => a.date.getTime() - b.date.getTime());
    for (const db of orderedDays) {
      const merged = mergeBusy(db.busies);
      const earliest =
        ymd(db.date) === ymd(today)
          ? Math.max(windowStart, now.getHours() * 60 + now.getMinutes())
          : windowStart;
      gaps.push(...buildGapsForDay(db.date, merged, windowStart, windowEnd, earliest));
    }
    gaps.sort((a, b) => a.ts - b.ts);

    if (mode === "chunk") {
      for (const task of queue) {
        let remaining = task.durationMinutes;
        const placements: { date: string; start: string; end: string }[] = [];
        for (let gi = 0; gi < gaps.length && remaining > 0; gi++) {
          const g = gaps[gi];
          const avail = g.end - g.start;
          if (avail < SLOT_GRANULARITY_MIN) continue;
          const wantThisGap = Math.min(remaining, avail);
          const leftover = remaining - wantThisGap;
          const take =
            leftover > 0 && leftover < MIN_CHUNK_MIN
              ? Math.max(SLOT_GRANULARITY_MIN, wantThisGap - MIN_CHUNK_MIN > 0 ? wantThisGap - MIN_CHUNK_MIN : wantThisGap)
              : wantThisGap;
          const startMin = g.start;
          const endMin = startMin + take;
          try {
            await placeEvent(src.accountEmail, src.calendarUid, task, g.date, startMin, endMin);
            placements.push({ date: g.date, start: hm(startMin), end: hm(endMin) });
            remaining -= take;
            gaps[gi] = { ...g, start: endMin, ts: g.ts + take * 60000 };
          } catch (err) {
            result.failed.push({
              taskId: task.id,
              reason: err instanceof Error ? err.message : String(err),
            });
            remaining = -1;
            break;
          }
        }
        if (placements.length > 0) result.scheduled.push({ taskId: task.id, placements });
        if (remaining > 0) {
          result.failed.push({
            taskId: task.id,
            reason: `Only fit ${task.durationMinutes - remaining} of ${task.durationMinutes} minutes`,
          });
        }
      }
    } else {
      const placedIds = new Set<string>();
      const placementsByTask = new Map<string, { date: string; start: string; end: string }[]>();
      for (let gi = 0; gi < gaps.length; gi++) {
        let g = gaps[gi];
        let cursor = g.start;
        let progressed = true;
        while (progressed && cursor < g.end) {
          progressed = false;
          for (const task of queue) {
            if (placedIds.has(task.id)) continue;
            const need = task.durationMinutes;
            if (cursor + need > g.end) continue;
            try {
              await placeEvent(src.accountEmail, src.calendarUid, task, g.date, cursor, cursor + need);
              const list = placementsByTask.get(task.id) ?? [];
              list.push({ date: g.date, start: hm(cursor), end: hm(cursor + need) });
              placementsByTask.set(task.id, list);
              placedIds.add(task.id);
              cursor += need;
              progressed = true;
              break;
            } catch (err) {
              placedIds.add(task.id);
              result.failed.push({
                taskId: task.id,
                reason: err instanceof Error ? err.message : String(err),
              });
              progressed = true;
              break;
            }
          }
        }
        gaps[gi] = { ...g, start: cursor };
      }
      for (const task of queue) {
        if (placedIds.has(task.id)) {
          const placements = placementsByTask.get(task.id);
          if (placements && placements.length > 0) {
            result.scheduled.push({ taskId: task.id, placements });
          }
        } else {
          const windowLabel = `${win.start_hour}:00–${win.end_hour}:00`;
          const largestGapMin = gaps.reduce((max, g) => Math.max(max, g.end - g.start), 0);
          const detail =
            largestGapMin > 0
              ? `largest gap is ${largestGapMin} min`
              : `window ${windowLabel} fully booked`;
          result.failed.push({
            taskId: task.id,
            reason: `No ${task.durationMinutes}-min slot in next ${HORIZON_DAYS} days (${detail})`,
          });
        }
      }
    }

    // Once placed in this box, the events are real on the calendar; subsequent
    // boxes will see them as busy when we re-fetch on the *next* run. Within
    // this single run, we also tag those minutes as busy so a lower-priority
    // box that overlaps in time won't double-book.
    const newlyPlaced = result.scheduled
      .filter((s) => queue.some((q) => q.id === s.taskId))
      .flatMap((s) => s.placements);
    if (newlyPlaced.length > 0) {
      const synthetic: api.CalendarEvent[] = newlyPlaced.map((p) => ({
        uid: `synthetic-${Math.random()}`,
        summary: "",
        start_date: `${p.date}T${p.start}:00`,
        end_date: `${p.date}T${p.end}:00`,
        description: null,
        url: null,
        calendar_name: src.calendarUid,
      }));
      allEvents.push(...synthetic);
    }
  }

  return result;
}
