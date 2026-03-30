import { invoke } from "@tauri-apps/api/core";

// Config types
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

export interface ScheduleWindow {
  start_hour: number;
  end_hour: number;
  days: number[]; // 0=Sun, 1=Mon, ..., 6=Sat; empty = weekdays
}

// Jira types
export interface SimpleIssue {
  id: string;
  key: string;
  summary: string;
  status: string;
  status_category: string;
  priority: string | null;
  project_key: string;
  project_name: string;
  time_estimate_seconds: number | null;
  time_spent_seconds: number | null;
  due_date: string | null;
  issue_type: string | null;
  labels: string[];
  url: string;
  parent_key: string | null;
  parent_summary: string | null;
  is_epic: boolean;
}

// GitHub types
export interface SimplePullRequest {
  id: number;
  number: number;
  title: string;
  url: string;
  repo_name: string;
  repo_full_name: string;
  author: string;
  author_avatar: string | null;
  branch: string;
  target_branch: string;
  state: string;
  is_draft: boolean;
  created_at: string;
  updated_at: string;
  jira_key: string | null;
  pr_role: string;
}

export interface GitHubAccountInfo {
  login: string;
  name: string | null;
  avatar_url: string | null;
}

// Calendar types
export interface CalendarInfo {
  name: string;
  uid: string;
}

export interface GoogleAccountInfo {
  email: string;
  name?: string | null;
}

export interface CalendarEvent {
  uid: string;
  summary: string;
  start_date: string;
  end_date: string;
  description: string | null;
  url: string | null;
  calendar_name: string;
}

export interface CreateEventRequest {
  summary: string;
  start_date: string;
  end_date: string;
  description: string | null;
  url: string | null;
  calendar_name: string;
  is_focus_time: boolean;
  color_id: string | null;
}

export interface UpdateEventRequest {
  uid: string;
  summary: string | null;
  start_date: string | null;
  end_date: string | null;
  description: string | null;
  url: string | null;
  calendar_name: string;
  is_focus_time: boolean | null;
  color_id: string | null;
}

// Config commands
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

export async function saveJiraToken(token: string): Promise<void> {
  return invoke("save_jira_token", { token });
}

export async function getJiraToken(): Promise<string | null> {
  return invoke<string | null>("get_jira_token");
}

export async function deleteJiraToken(): Promise<void> {
  return invoke("delete_jira_token");
}

export async function hasJiraToken(): Promise<boolean> {
  return invoke<boolean>("has_jira_token");
}

// Jira commands
export async function getAssignedIssues(
  customJql?: string
): Promise<SimpleIssue[]> {
  return invoke<SimpleIssue[]>("get_assigned_issues", {
    customJql: customJql || null,
  });
}

export async function searchIssues(
  jql: string,
  maxResults?: number
): Promise<SimpleIssue[]> {
  return invoke<SimpleIssue[]>("search_issues", {
    jql,
    maxResults: maxResults || null,
  });
}

export async function createWorklog(
  issueKey: string,
  timeSpentSeconds: number,
  started: string,
  comment?: string
): Promise<void> {
  return invoke("create_worklog", {
    issueKey,
    timeSpentSeconds,
    started,
    comment: comment || null,
  });
}

export async function testJiraConnection(): Promise<string> {
  return invoke<string>("test_jira_connection");
}

export async function getIssueStatus(issueKey: string): Promise<SimpleIssue> {
  return invoke<SimpleIssue>("get_issue_status", { issueKey });
}

// Calendar commands
export async function getCalendars(accountEmail: string): Promise<CalendarInfo[]> {
  return invoke<CalendarInfo[]>("google_list_calendars", { accountEmail });
}

export async function createEvent(accountEmail: string, request: CreateEventRequest): Promise<string> {
  return invoke<string>("google_create_event", { accountEmail, request });
}

export async function updateEvent(accountEmail: string, request: UpdateEventRequest): Promise<void> {
  return invoke("google_update_event", { accountEmail, request });
}

export async function deleteEvent(
  accountEmail: string,
  uid: string,
  calendarName: string
): Promise<void> {
  return invoke("google_delete_event", { accountEmail, uid, calendarName });
}

export async function getEventsForDateRange(
  accountEmail: string,
  calendarName: string,
  startDate: string,
  endDate: string,
  searchText?: string
): Promise<CalendarEvent[]> {
  return invoke<CalendarEvent[]>("google_list_events", {
    accountEmail,
    calendarName,
    startDate,
    endDate,
    searchText: searchText ?? null,
  });
}

// Google auth commands
export async function googleAuthStart(): Promise<{ auth_url: string }> {
  return invoke<{ auth_url: string }>("google_auth_start");
}

export async function googleAuthWait(): Promise<GoogleAccountInfo> {
  return invoke<GoogleAccountInfo>("google_auth_wait");
}

export async function googleAuthStatus(): Promise<GoogleAccountInfo[]> {
  return invoke<GoogleAccountInfo[]>("google_auth_status");
}

export async function googleAuthSignOut(accountEmail: string): Promise<void> {
  return invoke("google_auth_sign_out", { accountEmail });
}

// GitHub commands
export async function getPullRequests(): Promise<SimplePullRequest[]> {
  return invoke<SimplePullRequest[]>("get_pull_requests");
}

export async function testGitHubConnection(): Promise<string> {
  return invoke<string>("test_github_connection");
}

export async function saveGitHubToken(token: string): Promise<void> {
  return invoke("save_github_token", { token });
}

export async function getGitHubToken(): Promise<string | null> {
  return invoke<string | null>("get_github_token");
}

export async function deleteGitHubToken(): Promise<void> {
  return invoke("delete_github_token");
}

export async function hasGitHubToken(): Promise<boolean> {
  return invoke<boolean>("has_github_token");
}

// GitHub Device Flow OAuth
export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export async function githubDeviceFlowStart(
  clientId: string
): Promise<DeviceCodeResponse> {
  return invoke<DeviceCodeResponse>("github_device_flow_start", { clientId });
}

export async function githubDeviceFlowPoll(
  clientId: string,
  deviceCode: string
): Promise<string> {
  return invoke<string>("github_device_flow_poll", { clientId, deviceCode });
}

// Jira OAuth 2.0 (3LO)
export interface JiraOAuthResult {
  display_name: string;
  email: string;
  site_url: string;
}

export async function jiraAuthStart(
  clientId: string,
  clientSecret: string
): Promise<{ auth_url: string }> {
  return invoke<{ auth_url: string }>("jira_auth_start", { clientId, clientSecret });
}

export async function jiraAuthWait(): Promise<JiraOAuthResult> {
  return invoke<JiraOAuthResult>("jira_auth_wait");
}

// Claude chat types
export interface ClaudeInfo {
  path: string;
  version: string;
}

export interface ChatStreamEvent {
  session_id: string;
  event_type: string;
  data: string;
}

export interface DiffFile {
  path: string;
  status: string;
  diff: string;
}

export interface ChangeDiff {
  files: DiffFile[];
  summary: string;
}

export interface PRResult {
  url: string;
  number: number;
  branch: string;
}

// Claude chat commands
export async function checkClaudeInstalled(): Promise<ClaudeInfo> {
  return invoke<ClaudeInfo>("check_claude_installed");
}

export async function startChatSession(repoPath: string): Promise<string> {
  return invoke<string>("start_chat_session", { repoPath });
}

export async function sendChatMessage(
  sessionId: string,
  message: string
): Promise<void> {
  return invoke("send_chat_message", { sessionId, message });
}

export async function cancelChatMessage(sessionId: string): Promise<void> {
  return invoke("cancel_chat_message", { sessionId });
}

export async function stopChatSession(sessionId: string): Promise<void> {
  return invoke("stop_chat_session", { sessionId });
}

export async function getChangesDiff(sessionId: string): Promise<ChangeDiff> {
  return invoke<ChangeDiff>("get_changes_diff", { sessionId });
}

export async function submitPR(
  sessionId: string,
  title: string,
  body: string
): Promise<PRResult> {
  return invoke<PRResult>("submit_pr", { sessionId, title, body });
}

export async function discardChanges(sessionId: string): Promise<void> {
  return invoke("discard_changes", { sessionId });
}

// Workspace types
export interface WorkspaceStatus {
  claude_available: boolean;
  node_installed: boolean;
  node_path: string | null;
  repo_cloned: boolean;
  npm_installed: boolean;
  workspace_path: string;
}

export interface WorkspaceProgressEvent {
  step: string;
  message: string;
  percent: number;
}

// Workspace commands
export async function workspaceCheckStatus(): Promise<WorkspaceStatus> {
  return invoke<WorkspaceStatus>("workspace_check_status");
}

export async function workspaceSetupNode(): Promise<void> {
  return invoke("workspace_setup_node");
}

export async function workspaceCloneRepo(): Promise<void> {
  return invoke("workspace_clone_repo");
}

export async function workspaceNpmInstall(): Promise<void> {
  return invoke("workspace_npm_install");
}

export async function workspaceStartVite(): Promise<number> {
  return invoke<number>("workspace_start_vite");
}

export async function workspaceStopVite(): Promise<void> {
  return invoke("workspace_stop_vite");
}

export async function workspaceGetPath(): Promise<string> {
  return invoke<string>("workspace_get_path");
}

export async function workspacePullLatest(): Promise<void> {
  return invoke("workspace_pull_latest");
}
