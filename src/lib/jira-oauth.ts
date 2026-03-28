export const JIRA_CLIENT_ID = import.meta.env.VITE_JIRA_CLIENT_ID ?? "";
export const JIRA_CLIENT_SECRET = import.meta.env.VITE_JIRA_CLIENT_SECRET ?? "";

export function hasJiraOAuthCredentials(): boolean {
  return !!JIRA_CLIENT_ID && !!JIRA_CLIENT_SECRET;
}
