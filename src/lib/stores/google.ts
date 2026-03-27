import { writable } from "svelte/store";
import * as api from "../api";
import type { GoogleAccountInfo } from "../api";

export const googleAccount = writable<GoogleAccountInfo | null>(null);
export const googleAuthLoading = writable<boolean>(false);
export const googleAuthError = writable<string | null>(null);

export async function loadGoogleAuthStatus(): Promise<void> {
  googleAuthLoading.set(true);
  googleAuthError.set(null);

  try {
    const account = await api.googleAuthStatus();
    googleAccount.set(account);
  } catch (error) {
    googleAuthError.set(error instanceof Error ? error.message : String(error));
  } finally {
    googleAuthLoading.set(false);
  }
}
