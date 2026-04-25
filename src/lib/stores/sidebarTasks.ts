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
  createdAt: string;
}

const STORAGE_KEY = "mira.sidebarTasks.v1";

function load(): SidebarTask[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (t): t is SidebarTask =>
        t &&
        typeof t.id === "string" &&
        typeof t.title === "string" &&
        (t.lane === "backlog" || t.lane === "active") &&
        (t.source === "manual" || t.source === "jira")
    );
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

export function addManualTask(title: string): void {
  const trimmed = title.trim();
  if (!trimmed) return;
  sidebarTasks.update((list) => [
    ...list,
    {
      id: newId(),
      title: trimmed,
      lane: "backlog",
      source: "manual",
      createdAt: new Date().toISOString(),
    },
  ]);
}

export function importJiraTasks(
  jiraTasks: { key: string; summary: string; url: string }[]
): number {
  let added = 0;
  sidebarTasks.update((list) => {
    const seen = new Set(list.filter((t) => t.jiraKey).map((t) => t.jiraKey));
    const next = [...list];
    for (const jt of jiraTasks) {
      if (seen.has(jt.key)) continue;
      seen.add(jt.key);
      next.push({
        id: newId(),
        title: jt.summary,
        lane: "backlog",
        source: "jira",
        jiraKey: jt.key,
        jiraUrl: jt.url,
        createdAt: new Date().toISOString(),
      });
      added++;
    }
    return next;
  });
  return added;
}

export function moveTask(id: string, lane: SidebarLane): void {
  sidebarTasks.update((list) =>
    list.map((t) => (t.id === id ? { ...t, lane } : t))
  );
}

export function removeTask(id: string): void {
  sidebarTasks.update((list) => list.filter((t) => t.id !== id));
}
