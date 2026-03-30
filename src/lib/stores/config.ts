import { writable } from "svelte/store";
import * as api from "../api";
import type { ScheduleWindow } from "../api";

export interface AppConfig {
  jira_url: string;
  jira_email: string;
  google_client_id: string;
  google_client_secret: string;
  selected_calendar: string | null;
  sync_frequency: "manual" | "hourly" | "daily";
  auto_sync_on_launch: boolean;
  jql_filter: string | null;
  event_title_template: string;
  timezone: string | null;
  default_event_color: string | null;
  // GitHub configuration
  github_username: string;
  pr_event_title_template: string;
  pr_default_event_color: string | null;
  calendar_colors: Record<string, number>;
  account_colors: Record<string, number>;
  scheduling_strategy: "earliest_available" | "priority_weighted";
  allow_task_splitting: boolean;
  account_schedule_windows: Record<string, ScheduleWindow>;
}

const defaultConfig: AppConfig = {
  jira_url: "",
  jira_email: "",
  google_client_id: "",
  google_client_secret: "",
  selected_calendar: null,
  sync_frequency: "manual",
  auto_sync_on_launch: false,
  jql_filter: null,
  event_title_template: "[{key}] {summary}",
  timezone: null,
  default_event_color: null,
  github_username: "",
  pr_event_title_template: "[PR Review] {repo}: {title}",
  pr_default_event_color: null,
  calendar_colors: {},
  account_colors: {},
  scheduling_strategy: "earliest_available",
  allow_task_splitting: true,
  account_schedule_windows: {},
};

export const config = writable<AppConfig>(defaultConfig);
export const configLoading = writable<boolean>(false);
export const configError = writable<string | null>(null);
export const hasToken = writable<boolean>(false);

export async function loadConfig(): Promise<void> {
  configLoading.set(true);
  configError.set(null);

  try {
    const loadedConfig = await api.getConfig();
    config.set(loadedConfig);

    const tokenExists = await api.hasJiraToken();
    hasToken.set(tokenExists);
  } catch (error) {
    configError.set(error instanceof Error ? error.message : String(error));
  } finally {
    configLoading.set(false);
  }
}

export async function saveConfig(newConfig: AppConfig): Promise<void> {
  configLoading.set(true);
  configError.set(null);

  try {
    await api.saveConfig(newConfig);
    config.set(newConfig);
  } catch (error) {
    configError.set(error instanceof Error ? error.message : String(error));
    throw error;
  } finally {
    configLoading.set(false);
  }
}

export async function saveJiraToken(token: string): Promise<void> {
  configLoading.set(true);
  configError.set(null);

  try {
    await api.saveJiraToken(token);
    hasToken.set(true);
  } catch (error) {
    configError.set(error instanceof Error ? error.message : String(error));
    throw error;
  } finally {
    configLoading.set(false);
  }
}

export async function deleteJiraToken(): Promise<void> {
  configLoading.set(true);
  configError.set(null);

  try {
    await api.deleteJiraToken();
    hasToken.set(false);
  } catch (error) {
    configError.set(error instanceof Error ? error.message : String(error));
    throw error;
  } finally {
    configLoading.set(false);
  }
}

export async function testConnection(): Promise<string> {
  configLoading.set(true);
  configError.set(null);

  try {
    const displayName = await api.testJiraConnection();
    return displayName;
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : String(error);
    configError.set(errorMessage);
    throw error;
  } finally {
    configLoading.set(false);
  }
}
