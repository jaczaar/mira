// Embedded Google OAuth credentials for Mira.
// Read from environment variables at build time (Vite injects VITE_ prefixed vars).
// Locally: set in .env (gitignored)
// CI/CD: set as GitHub secrets → VITE_GOOGLE_CLIENT_ID, VITE_GOOGLE_CLIENT_SECRET

export const EMBEDDED_GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID ?? "";
export const EMBEDDED_GOOGLE_CLIENT_SECRET = import.meta.env.VITE_GOOGLE_CLIENT_SECRET ?? "";

export function hasEmbeddedCredentials(): boolean {
  return !!EMBEDDED_GOOGLE_CLIENT_ID && !!EMBEDDED_GOOGLE_CLIENT_SECRET;
}
