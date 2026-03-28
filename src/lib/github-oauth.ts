export const GITHUB_CLIENT_ID = import.meta.env.VITE_GITHUB_CLIENT_ID ?? "";
export const GITHUB_CLIENT_SECRET = import.meta.env.VITE_GITHUB_CLIENT_SECRET ?? "";

export function hasGitHubOAuthCredentials(): boolean {
  return !!GITHUB_CLIENT_ID;
}
