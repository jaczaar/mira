import { writable } from "svelte/store";

export type SidebarLane = "backlog" | "active";
export type SidebarSource = "manual" | "jira";

export interface SidebarTask {
  id: string;
  title: string;
  lane: SidebarLane;
  source: SidebarSource;
  jiraKey?: string;
  jiraUrl?: string;
  projectKey?: string;
  groupId?: string;
  durationMinutes: number;
  calendarUid?: string;
  createdAt: string;
}

const STORAGE_KEY = "mira.sidebarTasks.v1";
const DEFAULT_DURATION_MINUTES = 60;

function deriveProjectKey(jiraKey?: string): string | undefined {
  if (!jiraKey) return undefined;
  const dash = jiraKey.indexOf("-");
  return dash > 0 ? jiraKey.slice(0, dash) : jiraKey;
}

function load(): SidebarTask[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter(
        (t): t is SidebarTask =>
          t &&
          typeof t.id === "string" &&
          typeof t.title === "string" &&
          (t.lane === "backlog" || t.lane === "active") &&
          (t.source === "manual" || t.source === "jira")
      )
      .map((t) => ({
        ...t,
        durationMinutes:
          typeof t.durationMinutes === "number" && t.durationMinutes > 0
            ? t.durationMinutes
            : DEFAULT_DURATION_MINUTES,
        projectKey: t.projectKey ?? deriveProjectKey(t.jiraKey),
        // Active tasks now require a calendarUid; legacy ones get cleared.
        ...(t.lane === "active" && !t.calendarUid
          ? { lane: "backlog" as const, calendarUid: undefined }
          : {}),
      }));
  } catch {
    return [];
  }
}

function persist(list: SidebarTask[]): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  } catch {
    // Quota or privacy mode — silently drop.
  }
}

export const sidebarTasks = writable<SidebarTask[]>(load());
sidebarTasks.subscribe(persist);

function newId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export function addManualTask(
  title: string,
  durationMinutes?: number,
  groupId?: string
): void {
  const trimmed = title.trim();
  if (!trimmed) return;
  const dur =
    Number.isFinite(durationMinutes) && (durationMinutes ?? 0) > 0
      ? Math.min(Math.max(Math.round(durationMinutes!), 15), 8 * 60)
      : DEFAULT_DURATION_MINUTES;
  sidebarTasks.update((list) => [
    ...list,
    {
      id: newId(),
      title: trimmed,
      lane: "backlog",
      source: "manual",
      groupId,
      durationMinutes: dur,
      createdAt: new Date().toISOString(),
    },
  ]);
}

export function importJiraTasks(
  jiraTasks: {
    key: string;
    summary: string;
    url: string;
    estimateSeconds?: number | null;
  }[],
  groupId?: string
): number {
  let added = 0;
  sidebarTasks.update((list) => {
    const seen = new Set(list.filter((t) => t.jiraKey).map((t) => t.jiraKey));
    const next = [...list];
    for (const jt of jiraTasks) {
      if (seen.has(jt.key)) continue;
      seen.add(jt.key);
      const minutes =
        jt.estimateSeconds && jt.estimateSeconds > 0
          ? Math.max(15, Math.round(jt.estimateSeconds / 60))
          : DEFAULT_DURATION_MINUTES;
      next.push({
        id: newId(),
        title: jt.summary,
        lane: "backlog",
        source: "jira",
        jiraKey: jt.key,
        jiraUrl: jt.url,
        projectKey: deriveProjectKey(jt.key),
        groupId,
        durationMinutes: minutes,
        createdAt: new Date().toISOString(),
      });
      added++;
    }
    return next;
  });
  return added;
}

export function moveTask(id: string, lane: SidebarLane, calendarUid?: string): void {
  sidebarTasks.update((list) =>
    list.map((t) =>
      t.id === id
        ? {
            ...t,
            lane,
            calendarUid: lane === "active" ? calendarUid ?? t.calendarUid : undefined,
          }
        : t
    )
  );
}

export function moveTaskToCalendar(id: string, calendarUid: string): void {
  sidebarTasks.update((list) =>
    list.map((t) => (t.id === id ? { ...t, lane: "active", calendarUid } : t))
  );
}

export function moveTaskWithinLane(id: string, delta: number): void {
  if (delta === 0) return;
  sidebarTasks.update((list) => {
    const idx = list.findIndex((t) => t.id === id);
    if (idx < 0) return list;
    const task = list[idx];
    const sameLaneIndices = list
      .map((t, i) => ({ t, i }))
      .filter((x) => x.t.lane === task.lane)
      .map((x) => x.i);
    const pos = sameLaneIndices.indexOf(idx);
    const targetPos = pos + delta;
    if (targetPos < 0 || targetPos >= sameLaneIndices.length) return list;
    const targetIdx = sameLaneIndices[targetPos];
    const next = [...list];
    [next[idx], next[targetIdx]] = [next[targetIdx], next[idx]];
    return next;
  });
}

export function setTaskDuration(id: string, minutes: number): void {
  if (!Number.isFinite(minutes) || minutes <= 0) return;
  const clamped = Math.min(Math.max(Math.round(minutes), 15), 8 * 60);
  sidebarTasks.update((list) =>
    list.map((t) => (t.id === id ? { ...t, durationMinutes: clamped } : t))
  );
}

export function removeTask(id: string): void {
  sidebarTasks.update((list) => list.filter((t) => t.id !== id));
}

export function removeTasks(ids: string[]): void {
  if (ids.length === 0) return;
  const set = new Set(ids);
  sidebarTasks.update((list) => list.filter((t) => !set.has(t.id)));
}

export function removeTasksInGroup(groupId: string): void {
  sidebarTasks.update((list) => list.filter((t) => t.groupId !== groupId));
}
