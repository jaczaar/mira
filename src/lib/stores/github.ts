import { writable, get } from "svelte/store";
import * as api from "../api";
import type { SimplePullRequest, CalendarEvent } from "../api";
import { config } from "./config";
import { format, addDays, subDays } from "date-fns";

// Extended PR type with scheduling state
export interface ScheduledPR extends SimplePullRequest {
  calendar_event_uid?: string;
  linked_jira_key?: string; // Can override auto-detected
  last_synced?: string;
}

// Stores
export const pullRequests = writable<ScheduledPR[]>([]);
export const prsLoading = writable<boolean>(false);
export const prsError = writable<string | null>(null);
export const hasGitHubToken = writable<boolean>(false);

// Match calendar events to PRs based on PR URL in event description
function matchPRsToEvents(prs: ScheduledPR[], events: CalendarEvent[]): ScheduledPR[] {
  return prs.map((pr) => {
    // Look for an event that contains the PR URL in its description or summary
    const matchingEvent = events.find((event) => {
      // Check if description contains the PR URL
      if (event.description?.includes(pr.url)) {
        return true;
      }
      // Check if summary contains repo name and PR number
      if (event.summary?.includes(pr.repo_name) && event.summary?.includes(`#${pr.number}`)) {
        return true;
      }
      return false;
    });

    if (matchingEvent) {
      return {
        ...pr,
        calendar_event_uid: matchingEvent.uid,
        last_synced: matchingEvent.start_date,
      };
    }
    return pr;
  });
}

// Load pull requests (review requests + authored) from GitHub and match with calendar events
export async function loadPullRequests(): Promise<void> {
  prsLoading.set(true);
  prsError.set(null);

  try {
    const prs = await api.getPullRequests();
    let scheduledPRs: ScheduledPR[] = prs.map((pr) => ({
      ...pr,
      calendar_event_uid: undefined,
      linked_jira_key: pr.jira_key || undefined,
      last_synced: undefined,
    }));

    // Try to match with calendar events if calendar is configured
    const currentConfig = get(config);
    if (currentConfig.selected_calendar && scheduledPRs.length > 0) {
      try {
        const today = new Date();
        const pastDate = subDays(today, 30);
        const futureDate = addDays(today, 60);
        const startDate = format(pastDate, "yyyy-MM-dd");
        const endDate = format(futureDate, "yyyy-MM-dd");

        // Search for PR review events - look for "[PR Review]" or repo names
        const events = await api.getEventsForDateRange(
          currentConfig.selected_calendar,
          startDate,
          endDate,
          "PR Review"
        );

        scheduledPRs = matchPRsToEvents(scheduledPRs, events);
      } catch (calendarError) {
        console.warn("Failed to match PRs with calendar events:", calendarError);
        // Continue without calendar matching
      }
    }

    pullRequests.set(scheduledPRs);
  } catch (error) {
    prsError.set(error instanceof Error ? error.message : String(error));
  } finally {
    prsLoading.set(false);
  }
}

// Update PR with calendar event UID
export function updatePRCalendarEvent(prId: number, eventUid: string): void {
  pullRequests.update((prs) =>
    prs.map((pr) =>
      pr.id === prId
        ? {
            ...pr,
            calendar_event_uid: eventUid,
            last_synced: new Date().toISOString(),
          }
        : pr
    )
  );
}

// Update PR with linked Jira key
export function updatePRJiraLink(prId: number, jiraKey: string | null): void {
  pullRequests.update((prs) =>
    prs.map((pr) =>
      pr.id === prId
        ? {
            ...pr,
            linked_jira_key: jiraKey || undefined,
          }
        : pr
    )
  );
}

// Clear calendar event for a PR
export function clearPRCalendarEvent(prId: number): void {
  pullRequests.update((prs) =>
    prs.map((pr) =>
      pr.id === prId
        ? {
            ...pr,
            calendar_event_uid: undefined,
            last_synced: undefined,
          }
        : pr
    )
  );
}

// Get PRs with linked Jira issues
export function getPRsWithJiraLink(): ScheduledPR[] {
  return get(pullRequests).filter(
    (pr) => pr.linked_jira_key || pr.jira_key
  );
}

// Get PRs with scheduled calendar events
export function getScheduledPRs(): ScheduledPR[] {
  return get(pullRequests).filter((pr) => pr.calendar_event_uid !== undefined);
}

// Check if GitHub token exists
export async function checkGitHubToken(): Promise<boolean> {
  try {
    const exists = await api.hasGitHubToken();
    hasGitHubToken.set(exists);
    return exists;
  } catch {
    hasGitHubToken.set(false);
    return false;
  }
}

// Save GitHub token
export async function saveGitHubToken(token: string): Promise<void> {
  await api.saveGitHubToken(token);
  hasGitHubToken.set(true);
}

// Delete GitHub token
export async function deleteGitHubToken(): Promise<void> {
  await api.deleteGitHubToken();
  hasGitHubToken.set(false);
  pullRequests.set([]);
}

// Test GitHub connection
export async function testGitHubConnection(): Promise<string> {
  return api.testGitHubConnection();
}
