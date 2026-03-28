import { writable } from "svelte/store";
import * as api from "../api";
import type { GoogleAccountInfo } from "../api";

export const googleAccounts = writable<GoogleAccountInfo[]>([]);
export const googleAuthLoading = writable<boolean>(false);
export const googleAuthError = writable<string | null>(null);

// Compat alias — true if at least one account is connected
export const googleAccount = {
  subscribe(fn: (value: GoogleAccountInfo | null) => void) {
    return googleAccounts.subscribe((accounts) => {
      fn(accounts.length > 0 ? accounts[0] : null);
    });
  },
  set(_value: GoogleAccountInfo | null) {
    // no-op for compat — use googleAccounts instead
  },
};

export async function loadGoogleAuthStatus(): Promise<void> {
  googleAuthLoading.set(true);
  googleAuthError.set(null);

  try {
    const accounts = await api.googleAuthStatus();
    googleAccounts.set(accounts);
  } catch (error) {
    googleAuthError.set(error instanceof Error ? error.message : String(error));
  } finally {
    googleAuthLoading.set(false);
  }
}
