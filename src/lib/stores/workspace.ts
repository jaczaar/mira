import { writable, get } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  workspaceCheckStatus,
  workspaceSetupNode,
  workspaceCloneRepo,
  workspaceNpmInstall,
  workspaceStartVite,
  workspaceStopVite,
  workspacePullLatest,
  type WorkspaceStatus,
  type WorkspaceProgressEvent,
} from "../api";

export type SetupStep =
  | "CHECKING"
  | "NEED_CLAUDE"
  | "NEED_NODE"
  | "NEED_REPO"
  | "NEED_NPM"
  | "READY"
  | "ERROR";

export const workspaceStatus = writable<WorkspaceStatus | null>(null);
export const setupStep = writable<SetupStep>("CHECKING");
export const setupProgress = writable<WorkspaceProgressEvent | null>(null);
export const setupError = writable<string | null>(null);
export const vitePort = writable<number | null>(null);
export const workspacePath = writable<string | null>(null);

let progressUnlisten: UnlistenFn | null = null;

async function listenProgress() {
  if (progressUnlisten) return;
  progressUnlisten = await listen<WorkspaceProgressEvent>(
    "workspace-progress",
    (event) => {
      setupProgress.set(event.payload);
    }
  );
}

function stopListeningProgress() {
  if (progressUnlisten) {
    progressUnlisten();
    progressUnlisten = null;
  }
}

export async function checkWorkspaceStatus(): Promise<WorkspaceStatus> {
  setupStep.set("CHECKING");
  const status = await workspaceCheckStatus();
  workspaceStatus.set(status);
  workspacePath.set(status.workspace_path);

  if (!status.claude_available) {
    setupStep.set("NEED_CLAUDE");
  } else if (!status.node_installed) {
    setupStep.set("NEED_NODE");
  } else if (!status.repo_cloned) {
    setupStep.set("NEED_REPO");
  } else if (!status.npm_installed) {
    setupStep.set("NEED_NPM");
  } else {
    setupStep.set("READY");
  }

  return status;
}

export async function runSetup(): Promise<void> {
  await listenProgress();
  setupError.set(null);

  try {
    const status = get(workspaceStatus);
    if (!status) return;

    // Step 1: Node.js
    if (!status.node_installed) {
      setupStep.set("NEED_NODE");
      await workspaceSetupNode();
    }

    // Step 2: Clone repo
    if (!status.repo_cloned) {
      setupStep.set("NEED_REPO");
      await workspaceCloneRepo();
    }

    // Step 3: npm install
    if (!status.npm_installed) {
      setupStep.set("NEED_NPM");
      await workspaceNpmInstall();
    }

    // Re-check status
    const updated = await workspaceCheckStatus();
    workspaceStatus.set(updated);

    if (updated.claude_available && updated.node_installed && updated.repo_cloned && updated.npm_installed) {
      setupStep.set("READY");
    }
  } catch (e) {
    setupError.set(e instanceof Error ? e.message : String(e));
    setupStep.set("ERROR");
  } finally {
    stopListeningProgress();
  }
}

export async function startVite(): Promise<number> {
  await listenProgress();
  try {
    const port = await workspaceStartVite();
    vitePort.set(port);
    return port;
  } finally {
    stopListeningProgress();
  }
}

export async function stopVite(): Promise<void> {
  await workspaceStopVite();
  vitePort.set(null);
}

export async function pullLatest(): Promise<void> {
  await workspacePullLatest();
}
