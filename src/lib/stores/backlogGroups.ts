import { writable, get } from "svelte/store";

export interface BacklogGroup {
  id: string;
  name: string;
  jql?: string;
}

const STORAGE_KEY = "mira.backlogGroups.v1";
export const DEFAULT_GROUP_ID = "default";

function newId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID();
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function load(): BacklogGroup[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (g): g is BacklogGroup =>
        g && typeof g.id === "string" && typeof g.name === "string"
    );
  } catch {
    return [];
  }
}

export const backlogGroups = writable<BacklogGroup[]>(load());

backlogGroups.subscribe((value) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  } catch {
    /* ignore */
  }
});

export function addBacklogGroup(name: string, jql?: string): string {
  const id = newId();
  backlogGroups.update((list) => [...list, { id, name: name.trim() || "Untitled", jql }]);
  return id;
}

export function removeBacklogGroup(id: string): void {
  backlogGroups.update((list) => list.filter((g) => g.id !== id));
}

export function renameBacklogGroup(id: string, name: string): void {
  const trimmed = name.trim();
  if (!trimmed) return;
  backlogGroups.update((list) =>
    list.map((g) => (g.id === id ? { ...g, name: trimmed } : g))
  );
}

export function setGroupJql(id: string, jql: string): void {
  backlogGroups.update((list) =>
    list.map((g) => (g.id === id ? { ...g, jql: jql.trim() || undefined } : g))
  );
}

export function getDefaultGroupId(): string {
  const list = get(backlogGroups);
  return list[0]?.id ?? DEFAULT_GROUP_ID;
}
