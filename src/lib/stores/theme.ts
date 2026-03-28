import { writable } from "svelte/store";

export type ThemeMode = "dark" | "light" | "system";

const STORAGE_KEY = "mira-theme";

function getStored(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "dark" || v === "light" || v === "system") return v;
  } catch {}
  return "system";
}

function resolveEffective(mode: ThemeMode): "dark" | "light" {
  if (mode !== "system") return mode;
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export const themeMode = writable<ThemeMode>(getStored());
export const effectiveTheme = writable<"dark" | "light">(resolveEffective(getStored()));

function applyTheme(effective: "dark" | "light") {
  document.documentElement.classList.toggle("light", effective === "light");
}

let currentMode: ThemeMode = getStored();

themeMode.subscribe((mode) => {
  currentMode = mode;
  try { localStorage.setItem(STORAGE_KEY, mode); } catch {}
  const effective = resolveEffective(mode);
  effectiveTheme.set(effective);
  applyTheme(effective);
});

if (typeof window !== "undefined") {
  const mq = window.matchMedia("(prefers-color-scheme: light)");
  mq.addEventListener("change", () => {
    if (currentMode === "system") {
      const effective = resolveEffective("system");
      effectiveTheme.set(effective);
      applyTheme(effective);
    }
  });

  applyTheme(resolveEffective(currentMode));
}
