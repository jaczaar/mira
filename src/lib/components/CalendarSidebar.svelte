<script lang="ts">
  import {
    sidebarTasks,
    addManualTask,
    importJiraTasks,
    moveTask,
    moveTaskToCalendar,
    moveTaskWithinLane,
    removeTask,
    removeTasks,
    removeTasksInGroup,
    setTaskDuration,
    type SidebarTask,
    type SidebarLane,
  } from "../stores/sidebarTasks";
  import { loadAssignedTasks, tasks } from "../stores/tasks";
  import { config, hasToken } from "../stores/config";
  import { accountCalendars } from "../stores/calendar";
  import {
    calendarOrder,
    calendarWindows,
    orderedVisibleUids,
    setCalendarWindow,
    reorderCalendar,
    DEFAULT_CALENDAR_WINDOW,
  } from "../stores/calendarBoxes";
  import {
    backlogGroups as bgStore,
    addBacklogGroup,
    removeBacklogGroup,
    renameBacklogGroup,
  } from "../stores/backlogGroups";
  import { autoScheduleActive } from "../scheduling/autoSchedule";
  import type { ScheduleWindow } from "../api";

  interface Props {
    onAfterSchedule?: () => void | Promise<void>;
  }

  let { onAfterSchedule }: Props = $props();

  const COLLAPSE_KEY = "mira.sidebarCollapse.v1";

  let dragId = $state<string | null>(null);
  let dragOverLane = $state<SidebarLane | null>(null);
  let dragOverCalendarUid = $state<string | null>(null);
  let scheduling = $state(false);
  let scheduleMessage = $state<string | null>(null);
  let scheduleErrors = $state<Record<string, string>>({});

  const CHUNK_PREF_KEY = "mira.autoSchedule.chunk.v1";
  let chunkMode = $state<boolean>(
    typeof localStorage !== "undefined" && localStorage.getItem(CHUNK_PREF_KEY) === "1"
  );
  function toggleChunkMode() {
    chunkMode = !chunkMode;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(CHUNK_PREF_KEY, chunkMode ? "1" : "0");
    }
  }

  // Tasks we've attempted in this session — prevents retry-loops on failure.
  const attempted = new Set<string>();

  let expandedId = $state<string | null>(null);
  function toggleExpanded(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function loadCollapsed(): Record<string, boolean> {
    if (typeof localStorage === "undefined") return {};
    try {
      const raw = localStorage.getItem(COLLAPSE_KEY);
      return raw ? JSON.parse(raw) : {};
    } catch {
      return {};
    }
  }

  let collapsed = $state<Record<string, boolean>>(loadCollapsed());

  function persistCollapsed() {
    if (typeof localStorage === "undefined") return;
    try {
      localStorage.setItem(COLLAPSE_KEY, JSON.stringify(collapsed));
    } catch {
      /* ignore */
    }
  }

  function toggle(key: string) {
    collapsed = { ...collapsed, [key]: !collapsed[key] };
    persistCollapsed();
  }

  const backlog = $derived($sidebarTasks.filter((t) => t.lane === "backlog"));
  const active = $derived($sidebarTasks.filter((t) => t.lane === "active"));

  interface VisibleCalendar {
    uid: string;
    name: string;
    accountEmail: string;
  }

  const visibleCalendars = $derived.by<VisibleCalendar[]>(() => {
    const enabled = new Set($config.enabled_calendars ?? []);
    const list: VisibleCalendar[] = [];
    for (const [email, cals] of $accountCalendars) {
      for (const cal of cals) {
        if (enabled.size === 0 || enabled.has(cal.uid)) {
          list.push({ uid: cal.uid, name: cal.name, accountEmail: email });
        }
      }
    }
    return list;
  });

  const orderedCalendarBoxes = $derived.by<VisibleCalendar[]>(() => {
    const order = orderedVisibleUids($calendarOrder, visibleCalendars.map((c) => c.uid));
    const byUid = new Map(visibleCalendars.map((c) => [c.uid, c]));
    return order.map((uid) => byUid.get(uid)!).filter(Boolean);
  });

  const activeByCalendar = $derived.by(() => {
    const map = new Map<string, SidebarTask[]>();
    for (const t of active) {
      if (!t.calendarUid) continue;
      const arr = map.get(t.calendarUid) ?? [];
      arr.push(t);
      map.set(t.calendarUid, arr);
    }
    return map;
  });

  function windowFor(uid: string): ScheduleWindow {
    return $calendarWindows[uid] ?? DEFAULT_CALENDAR_WINDOW;
  }

  function formatWindow(w: ScheduleWindow): string {
    const days = w.days?.length ? w.days : DEFAULT_CALENDAR_WINDOW.days;
    const dayNames = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    const isWeekdays = days.length === 5 && [1, 2, 3, 4, 5].every((d) => days.includes(d));
    const isAll = days.length === 7;
    let dayLabel = isWeekdays ? "Mon–Fri" : isAll ? "Daily" : days.map((d) => dayNames[d]).join(" ");
    return `${w.start_hour}–${w.end_hour} · ${dayLabel}`;
  }

  let editingWindowUid = $state<string | null>(null);
  function toggleWindowEditor(uid: string) {
    editingWindowUid = editingWindowUid === uid ? null : uid;
  }
  function updateWindow(uid: string, patch: Partial<ScheduleWindow>) {
    const cur = windowFor(uid);
    setCalendarWindow(uid, { ...cur, ...patch });
  }
  function toggleWindowDay(uid: string, day: number) {
    const cur = windowFor(uid);
    const days = cur.days?.length ? [...cur.days] : [...DEFAULT_CALENDAR_WINDOW.days];
    const idx = days.indexOf(day);
    if (idx >= 0) days.splice(idx, 1);
    else days.push(day);
    days.sort();
    setCalendarWindow(uid, { ...cur, days });
  }

  // Always auto-fire scheduling for any new task in the active lane.
  let autoTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const pending = active.filter((t) => !attempted.has(t.id));
    if (pending.length === 0 || scheduling) return;
    if (autoTimer) clearTimeout(autoTimer);
    autoTimer = setTimeout(() => {
      autoTimer = null;
      handleAutoSchedule();
    }, 1500);
  });

  // Map: groupId -> tasks. Orphan tasks (groupId not in $bgStore) are dropped
  // because every task must belong to a user-created group.
  const backlogByGroup = $derived.by<Map<string, SidebarTask[]>>(() => {
    const map = new Map<string, SidebarTask[]>();
    const validIds = new Set($bgStore.map((g) => g.id));
    for (const t of backlog) {
      if (!t.groupId || !validIds.has(t.groupId)) continue;
      const arr = map.get(t.groupId) ?? [];
      arr.push(t);
      map.set(t.groupId, arr);
    }
    return map;
  });

  let pendingDeleteGroup = $state<string | null>(null);

  // Per-group UI state.
  let groupAddOpen = $state<Record<string, boolean>>({});
  let groupAddTitle = $state<Record<string, string>>({});
  let groupAddDuration = $state<Record<string, number>>({});
  let groupImporting = $state<Record<string, boolean>>({});
  let groupImportMsg = $state<Record<string, string | null>>({});
  let groupRenaming = $state<string | null>(null);
  let groupRenameDraft = $state<string>("");
  let newGroupName = $state<string>("");
  let creatingGroup = $state<boolean>(false);

  function toggleGroupAdd(id: string) {
    groupAddOpen = { ...groupAddOpen, [id]: !groupAddOpen[id] };
    if (groupAddDuration[id] == null) {
      groupAddDuration = { ...groupAddDuration, [id]: 60 };
    }
  }

  function addToGroup(id: string) {
    const title = (groupAddTitle[id] ?? "").trim();
    if (!title) return;
    addManualTask(title, groupAddDuration[id] ?? 60, id);
    groupAddTitle = { ...groupAddTitle, [id]: "" };
  }

  async function importJiraToGroup(id: string) {
    if (!$hasToken || groupImporting[id]) return;
    const group = $bgStore.find((g) => g.id === id);
    if (!group) return;
    groupImporting = { ...groupImporting, [id]: true };
    groupImportMsg = { ...groupImportMsg, [id]: null };
    try {
      await loadAssignedTasks($config.jql_filter ?? undefined);
      const added = importJiraTasks(
        $tasks
          .filter((t) => t.status_category !== "done")
          .map((t) => ({
            key: t.key,
            summary: t.summary,
            url: t.url,
            estimateSeconds: t.time_estimate_seconds,
          })),
        id
      );
      groupImportMsg = {
        ...groupImportMsg,
        [id]: added === 0 ? "Already up to date" : `Added ${added}`,
      };
    } catch (err) {
      groupImportMsg = {
        ...groupImportMsg,
        [id]: err instanceof Error ? err.message : String(err),
      };
    } finally {
      groupImporting = { ...groupImporting, [id]: false };
    }
  }

  function startRenameGroup(id: string, currentName: string) {
    groupRenaming = id;
    groupRenameDraft = currentName;
  }

  function commitRenameGroup() {
    if (groupRenaming && groupRenameDraft.trim()) {
      renameBacklogGroup(groupRenaming, groupRenameDraft);
    }
    groupRenaming = null;
  }

  function handleNewGroupKey(e: KeyboardEvent) {
    if (e.key === "Enter" && newGroupName.trim()) {
      addBacklogGroup(newGroupName);
      newGroupName = "";
      creatingGroup = false;
    } else if (e.key === "Escape") {
      newGroupName = "";
      creatingGroup = false;
    }
  }

  async function handleAutoSchedule() {
    if (scheduling) return;
    const toSchedule = active.filter((t) => !attempted.has(t.id));
    if (toSchedule.length === 0) return;
    for (const t of toSchedule) attempted.add(t.id);
    scheduling = true;
    scheduleMessage = null;
    // Clear stale errors for the tasks we're about to retry
    const cleared = { ...scheduleErrors };
    for (const t of toSchedule) delete cleared[t.id];
    scheduleErrors = cleared;
    try {
      const result = await autoScheduleActive(toSchedule, chunkMode ? "chunk" : "smart");
      if (result.scheduled.length > 0) {
        removeTasks(result.scheduled.map((s) => s.taskId));
      }
      if (result.failed.length > 0) {
        const errs = { ...scheduleErrors };
        for (const f of result.failed) errs[f.taskId] = f.reason;
        scheduleErrors = errs;
      }
      if (result.failed.length === 0 && result.scheduled.length > 0) {
        scheduleMessage = `Scheduled ${result.scheduled.length}`;
      } else if (result.scheduled.length > 0) {
        scheduleMessage = `Scheduled ${result.scheduled.length}, ${result.failed.length} failed`;
      } else {
        scheduleMessage = null; // per-task error pills will speak for themselves
      }
      if (onAfterSchedule) await onAfterSchedule();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      const errs = { ...scheduleErrors };
      for (const t of toSchedule) errs[t.id] = msg;
      scheduleErrors = errs;
    } finally {
      scheduling = false;
    }
  }

  function retryTask(taskId: string) {
    attempted.delete(taskId);
    const next = { ...scheduleErrors };
    delete next[taskId];
    scheduleErrors = next;
    setTimeout(() => handleAutoSchedule(), 0);
  }

  const DRAG_MIME = "application/x-mira-sidebar-task";

  function onDragStart(e: DragEvent, id: string) {
    dragId = id;
    if (e.dataTransfer) {
      e.dataTransfer.setData("text/plain", id);
      e.dataTransfer.setData(DRAG_MIME, id);
      e.dataTransfer.effectAllowed = "move";
    }
  }

  function onDragEnd() {
    dragId = null;
    dragOverLane = null;
  }

  function onDragOver(e: DragEvent, lane: SidebarLane) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverLane = lane;
  }

  function onDragLeave(lane: SidebarLane) {
    if (dragOverLane === lane) dragOverLane = null;
  }

  function onDrop(e: DragEvent, lane: SidebarLane) {
    e.preventDefault();
    const id = dragId ?? e.dataTransfer?.getData("text/plain");
    if (id) moveTask(id, lane);
    dragId = null;
    dragOverLane = null;
  }

  function onCalendarBoxDragOver(e: DragEvent, uid: string) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverCalendarUid = uid;
  }

  function onCalendarBoxDragLeave(uid: string) {
    if (dragOverCalendarUid === uid) dragOverCalendarUid = null;
  }

  function onCalendarBoxDrop(e: DragEvent, uid: string) {
    e.preventDefault();
    const id = dragId ?? e.dataTransfer?.getData("text/plain");
    if (id) moveTaskToCalendar(id, uid);
    dragId = null;
    dragOverCalendarUid = null;
    dragOverLane = null;
  }

  function formatDuration(min: number): string {
    if (min % 60 === 0) return `${min / 60}h`;
    if (min < 60) return `${min}m`;
    const h = Math.floor(min / 60);
    const m = min % 60;
    return `${h}h ${m}m`;
  }

  const DURATION_OPTIONS = [15, 30, 45, 60, 90, 120, 180, 240];
</script>

<aside class="cal-sidebar">
  <div class="sidebar-scroll">
  <section
    class="lane"
    class:drop-target={dragOverLane === "backlog"}
    ondragover={(e) => onDragOver(e, "backlog")}
    ondragleave={() => onDragLeave("backlog")}
    ondrop={(e) => onDrop(e, "backlog")}
    role="list"
  >
    <button
      type="button"
      class="lane-header collapsible"
      onclick={() => toggle("lane:backlog")}
      aria-expanded={!collapsed["lane:backlog"]}
    >
      <span class="chev" class:open={!collapsed["lane:backlog"]}>
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </span>
      <span class="lane-title">Backlog</span>
      <span class="lane-count">{backlog.length}</span>
    </button>

    {#if !collapsed["lane:backlog"]}
      <div class="new-group-row">
        {#if creatingGroup}
          <input
            type="text"
            class="add-input"
            placeholder="Group name"
            bind:value={newGroupName}
            onkeydown={handleNewGroupKey}
            onblur={() => { if (newGroupName.trim()) { addBacklogGroup(newGroupName); newGroupName = ""; } creatingGroup = false; }}
          />
        {:else}
          <button class="new-group-btn" onclick={() => { creatingGroup = true; setTimeout(() => { const el = document.querySelector(".new-group-row .add-input") as HTMLInputElement | null; el?.focus(); }, 0); }}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            New group
          </button>
        {/if}
      </div>

      {#if $bgStore.length === 0}
        <p class="empty-hint">No groups yet — create one above to start adding tasks.</p>
      {/if}

      <div class="groups">
        {#each $bgStore as group (group.id)}
          {@const items = backlogByGroup.get(group.id) ?? []}
          {@const isOpen = !collapsed[`group:${group.id}`]}
          {@const addOpen = !!groupAddOpen[group.id]}
          <div class="group">
            <div class="group-header-row">
              <button
                type="button"
                class="group-header"
                onclick={() => toggle(`group:${group.id}`)}
                aria-expanded={isOpen}
              >
                <span class="chev" class:open={isOpen}>
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </span>
                {#if groupRenaming === group.id}
                  <input
                    type="text"
                    class="group-rename-input"
                    bind:value={groupRenameDraft}
                    onclick={(e) => e.stopPropagation()}
                    onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitRenameGroup(); } else if (e.key === "Escape") { groupRenaming = null; } }}
                    onblur={commitRenameGroup}
                  />
                {:else}
                  <span
                    class="group-label"
                    ondblclick={(e) => { e.stopPropagation(); startRenameGroup(group.id, group.name); }}
                    title="Double-click to rename"
                  >
                    {group.name}
                  </span>
                {/if}
                <span class="group-count">{items.length}</span>
              </button>
              <button
                class="group-action"
                onclick={() => toggleGroupAdd(group.id)}
                title="Add or import"
                aria-label="Add or import"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                  <line x1="12" y1="5" x2="12" y2="19" />
                  <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
              </button>
              <button
                class="group-action"
                onclick={() => startRenameGroup(group.id, group.name)}
                title="Rename group"
                aria-label="Rename group"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 20h9" />
                  <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                </svg>
              </button>
              {#if pendingDeleteGroup === group.id}
                <button
                  class="group-action danger confirm"
                  onclick={() => { removeTasksInGroup(group.id); removeBacklogGroup(group.id); pendingDeleteGroup = null; }}
                  title="Confirm delete (tasks will be removed)"
                >
                  Delete?
                </button>
                <button
                  class="group-action"
                  onclick={() => pendingDeleteGroup = null}
                  title="Cancel"
                  aria-label="Cancel delete"
                >
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              {:else}
                <button
                  class="group-action danger"
                  onclick={() => pendingDeleteGroup = group.id}
                  title="Delete group"
                  aria-label="Delete group"
                >
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                  </svg>
                </button>
              {/if}
            </div>

            {#if addOpen}
              <div class="group-actions-panel">
                <div class="add-row">
                  <input
                    type="text"
                    class="add-input"
                    placeholder="Add a task…"
                    value={groupAddTitle[group.id] ?? ""}
                    oninput={(e) => groupAddTitle = { ...groupAddTitle, [group.id]: e.currentTarget.value }}
                    onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); addToGroup(group.id); } }}
                  />
                  <select
                    class="duration-select add-duration"
                    value={groupAddDuration[group.id] ?? 60}
                    onchange={(e) => groupAddDuration = { ...groupAddDuration, [group.id]: +e.currentTarget.value }}
                    title="Estimated duration"
                  >
                    {#each DURATION_OPTIONS as opt}
                      <option value={opt}>{formatDuration(opt)}</option>
                    {/each}
                  </select>
                  <button
                    class="add-btn"
                    onclick={() => addToGroup(group.id)}
                    disabled={!(groupAddTitle[group.id] ?? "").trim()}
                    title="Add task"
                    aria-label="Add task"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                      <line x1="12" y1="5" x2="12" y2="19" />
                      <line x1="5" y1="12" x2="19" y2="12" />
                    </svg>
                  </button>
                </div>
                <button
                  class="import-btn"
                  onclick={() => importJiraToGroup(group.id)}
                  disabled={!$hasToken || !!groupImporting[group.id]}
                  title={$hasToken ? "Import assigned issues from Jira" : "Connect Jira to import"}
                >
                  {#if groupImporting[group.id]}
                    <span class="spinner"></span> Importing…
                  {:else}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                      <polyline points="7 10 12 15 17 10" />
                      <line x1="12" y1="15" x2="12" y2="3" />
                    </svg>
                    Import from Jira
                  {/if}
                </button>
                {#if groupImportMsg[group.id]}
                  <span class="status-line">{groupImportMsg[group.id]}</span>
                {/if}
              </div>
            {/if}

            {#if isOpen}
              {#if items.length === 0}
                <p class="empty-hint cal-empty-hint">Empty</p>
              {:else}
                <ul class="task-list">
                  {#each items as task (task.id)}
                    <li
                      class="task-item"
                      class:dragging={dragId === task.id}
                      draggable="true"
                      ondragstart={(e) => onDragStart(e, task.id)}
                      ondragend={onDragEnd}
                    >
                      <div class="task-main">
                        {#if task.jiraKey}
                          <a
                            class="jira-key"
                            href={task.jiraUrl ?? "#"}
                            target="_blank"
                            rel="noopener"
                            onmousedown={(e) => e.stopPropagation()}
                          >
                            {task.jiraKey}
                          </a>
                        {/if}
                        <span class="task-title" title={task.title}>{task.title}</span>
                      </div>
                      <button
                        class="remove-btn"
                        onclick={() => removeTask(task.id)}
                        title="Remove"
                        aria-label="Remove task"
                      >
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                          <line x1="18" y1="6" x2="6" y2="18" />
                          <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="lane" role="list">
    <button
      type="button"
      class="lane-header collapsible"
      onclick={() => toggle("lane:active")}
      aria-expanded={!collapsed["lane:active"]}
    >
      <span class="chev" class:open={!collapsed["lane:active"]}>
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </span>
      <span class="lane-title">Active</span>
      <span class="lane-count">{active.length}</span>
    </button>

    {#if !collapsed["lane:active"]}
      <div class="toggle-row">
        <label class="opt-toggle" title="Split tasks across short gaps instead of waiting for a contiguous block">
          <input type="checkbox" checked={chunkMode} onchange={toggleChunkMode} />
          <span>Break down if needed</span>
        </label>
        {#if scheduling}
          <span class="status-line"><span class="spinner"></span> Scheduling…</span>
        {:else if scheduleMessage}
          <span class="status-line ok">{scheduleMessage}</span>
        {/if}
      </div>

      {#if orderedCalendarBoxes.length === 0}
        <p class="empty-hint">Enable a calendar in the filter menu to see boxes here.</p>
      {/if}

      <div class="calendar-boxes">
        {#each orderedCalendarBoxes as box, boxIdx (box.uid)}
          {@const queue = activeByCalendar.get(box.uid) ?? []}
          {@const win = windowFor(box.uid)}
          {@const isEditing = editingWindowUid === box.uid}
          {@const allUids = orderedCalendarBoxes.map((c) => c.uid)}
          <div
            class="cal-box"
            class:drop-target={dragOverCalendarUid === box.uid}
            ondragover={(e) => onCalendarBoxDragOver(e, box.uid)}
            ondragleave={() => onCalendarBoxDragLeave(box.uid)}
            ondrop={(e) => onCalendarBoxDrop(e, box.uid)}
          >
            <div class="cal-box-header">
              <div class="cal-box-reorder">
                <button
                  class="reorder-btn"
                  onclick={() => reorderCalendar(box.uid, -1, allUids)}
                  disabled={boxIdx === 0}
                  title="Move calendar up"
                  aria-label="Move calendar up"
                >
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="18 15 12 9 6 15" />
                  </svg>
                </button>
                <button
                  class="reorder-btn"
                  onclick={() => reorderCalendar(box.uid, 1, allUids)}
                  disabled={boxIdx === orderedCalendarBoxes.length - 1}
                  title="Move calendar down"
                  aria-label="Move calendar down"
                >
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                </button>
              </div>
              <div class="cal-box-meta">
                <span class="cal-box-name" title={box.name}>{box.name}</span>
                <button
                  type="button"
                  class="cal-window-chip"
                  onclick={() => toggleWindowEditor(box.uid)}
                  title="Edit work hours"
                >
                  {formatWindow(win)}
                </button>
              </div>
              <span class="lane-count">{queue.length}</span>
            </div>

            {#if isEditing}
              <div class="window-editor">
                <div class="window-row">
                  <span class="window-label">Hours</span>
                  <select
                    class="hour-select"
                    value={win.start_hour}
                    onchange={(e) => updateWindow(box.uid, { start_hour: +e.currentTarget.value })}
                  >
                    {#each Array.from({ length: 24 }, (_, h) => h) as h}
                      <option value={h}>{h}:00</option>
                    {/each}
                  </select>
                  <span class="window-sep">–</span>
                  <select
                    class="hour-select"
                    value={win.end_hour}
                    onchange={(e) => updateWindow(box.uid, { end_hour: +e.currentTarget.value })}
                  >
                    {#each Array.from({ length: 24 }, (_, h) => h + 1) as h}
                      <option value={h}>{h}:00</option>
                    {/each}
                  </select>
                </div>
                <div class="window-row">
                  <span class="window-label">Days</span>
                  <div class="day-pills">
                    {#each [["Mo",1],["Tu",2],["We",3],["Th",4],["Fr",5],["Sa",6],["Su",0]] as [label, dayNum]}
                      {@const isOn = (win.days ?? DEFAULT_CALENDAR_WINDOW.days).includes(dayNum as number)}
                      <button
                        type="button"
                        class="day-pill"
                        class:on={isOn}
                        onclick={() => toggleWindowDay(box.uid, dayNum as number)}
                      >
                        {label}
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}

            <ul class="task-list">
              {#each queue as task, idx (task.id)}
                {@const opts = DURATION_OPTIONS.includes(task.durationMinutes)
                  ? DURATION_OPTIONS
                  : [...DURATION_OPTIONS, task.durationMinutes].sort((a, b) => a - b)}
                {@const isExpanded = expandedId === task.id}
                <li
                  class="task-item active"
                  class:dragging={dragId === task.id}
                  class:expanded={isExpanded}
                  draggable="true"
                  ondragstart={(e) => onDragStart(e, task.id)}
                  ondragend={onDragEnd}
                >
                  <div
                    class="task-row"
                    role="button"
                    tabindex="0"
                    onclick={() => toggleExpanded(task.id)}
                    onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleExpanded(task.id); } }}
                  >
                    <div class="reorder-col">
                      <button
                        class="reorder-btn"
                        onclick={(e) => { e.stopPropagation(); moveTaskWithinLane(task.id, -1); }}
                        disabled={idx === 0}
                        title="Move up"
                        aria-label="Move up"
                      >
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="18 15 12 9 6 15" />
                        </svg>
                      </button>
                      <button
                        class="reorder-btn"
                        onclick={(e) => { e.stopPropagation(); moveTaskWithinLane(task.id, 1); }}
                        disabled={idx === queue.length - 1}
                        title="Move down"
                        aria-label="Move down"
                      >
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="6 9 12 15 18 9" />
                        </svg>
                      </button>
                    </div>
                    <div class="task-main">
                      {#if task.jiraKey}
                        <a
                          class="jira-key"
                          href={task.jiraUrl ?? "#"}
                          target="_blank"
                          rel="noopener"
                          onclick={(e) => e.stopPropagation()}
                          onmousedown={(e) => e.stopPropagation()}
                        >
                          {task.jiraKey}
                        </a>
                      {/if}
                      <span class="task-title" title={task.title}>{task.title}</span>
                    </div>
                    <button
                      class="remove-btn"
                      onclick={(e) => { e.stopPropagation(); removeTask(task.id); }}
                      title="Remove"
                      aria-label="Remove task"
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  </div>
                  {#if scheduleErrors[task.id]}
                    <div class="task-error" role="alert">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <circle cx="12" cy="12" r="10" />
                        <line x1="12" y1="8" x2="12" y2="12" />
                        <line x1="12" y1="16" x2="12.01" y2="16" />
                      </svg>
                      <span class="task-error-msg" title={scheduleErrors[task.id]}>{scheduleErrors[task.id]}</span>
                      <button
                        class="task-error-retry"
                        onclick={(e) => { e.stopPropagation(); retryTask(task.id); }}
                        title="Try scheduling again"
                      >Retry</button>
                    </div>
                  {/if}
                  {#if isExpanded}
                    <div class="task-detail">
                      <label class="detail-row">
                        <span class="detail-label">Duration</span>
                        <select
                          class="duration-select"
                          value={task.durationMinutes}
                          onchange={(e) => setTaskDuration(task.id, +e.currentTarget.value)}
                          onmousedown={(e) => e.stopPropagation()}
                        >
                          {#each opts as opt}
                            <option value={opt}>{formatDuration(opt)}</option>
                          {/each}
                        </select>
                      </label>
                      {#if task.jiraUrl}
                        <a class="detail-link" href={task.jiraUrl} target="_blank" rel="noopener">Open in Jira</a>
                      {/if}
                    </div>
                  {/if}
                </li>
              {/each}
              {#if queue.length === 0}
                <li class="empty-hint cal-empty-hint">Drop tasks here</li>
              {/if}
            </ul>
          </div>
        {/each}
      </div>
    {/if}
  </section>
  </div>
</aside>

<style>
  .cal-sidebar {
    flex: 0 0 var(--sidebar-width, 260px);
    width: var(--sidebar-width, 260px);
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
    padding-right: 4px;
    /* Match the cal-header height so lanes align with the calendar grid */
    padding-top: 56px;
  }

  .sidebar-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .lane {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition: border-color 0.15s var(--ease-out), background 0.15s var(--ease-out);
  }

  .lane.drop-target {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
  }

  .lane-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 2px;
  }

  .lane-header.collapsible {
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    color: inherit;
  }

  .chev {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 12px;
    height: 12px;
    color: var(--text-tertiary);
    transition: transform 0.15s var(--ease-out);
  }

  .chev.open {
    transform: rotate(90deg);
  }

  .lane-title {
    flex: 1;
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .lane-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    background: var(--bg-hover);
    padding: 1px 7px;
    border-radius: var(--radius-full);
  }

  .add-row {
    display: flex;
    gap: 6px;
  }

  .add-input {
    flex: 1;
    min-width: 0;
    padding: 6px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s var(--ease-out);
  }

  .add-input::placeholder {
    color: var(--text-tertiary);
  }

  .add-input:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border: 1px solid rgba(124, 172, 248, 0.2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
    flex-shrink: 0;
  }

  .add-btn:hover:not(:disabled) {
    background: rgba(124, 172, 248, 0.2);
  }

  .add-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .import-row,
  .auto-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .import-btn,
  .auto-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    background: transparent;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .import-btn:hover:not(:disabled),
  .auto-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-style: solid;
  }

  .auto-btn {
    color: var(--accent-blue);
    border-color: rgba(124, 172, 248, 0.4);
  }

  .auto-btn:hover:not(:disabled) {
    background: var(--accent-blue-dim);
  }

  .import-btn:disabled,
  .auto-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .status-line {
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .status-line.ok {
    color: var(--accent-green);
  }

  .status-line.err {
    color: var(--accent-red);
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border-default);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  .groups {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 4px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: inherit;
    text-align: left;
    transition: background 0.12s var(--ease-out);
  }

  .group-header:hover {
    background: var(--bg-hover);
  }

  .group-label {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .group-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .task-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 4px;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: grab;
    transition: border-color 0.12s var(--ease-out), background 0.12s var(--ease-out), opacity 0.12s var(--ease-out);
  }

  .task-item:hover {
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .task-item.dragging {
    opacity: 0.4;
    cursor: grabbing;
  }

  .task-item.active {
    border-left: 2px solid var(--accent-blue);
    flex-direction: column;
    align-items: stretch;
    padding: 7px 8px;
  }

  .task-main {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .jira-key {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    color: var(--accent-blue);
    background: var(--accent-blue-dim);
    padding: 1px 6px;
    border-radius: 3px;
    text-decoration: none;
    flex-shrink: 0;
    letter-spacing: 0.01em;
  }

  .jira-key:hover {
    background: rgba(124, 172, 248, 0.2);
  }

  .task-title {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .task-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .duration-select {
    padding: 2px 4px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .add-duration {
    width: 56px;
    flex-shrink: 0;
    padding: 6px 4px;
    font-size: 11px;
  }

  .schedule-btn {
    padding: 3px 8px;
    background: var(--accent-blue-dim);
    border: 1px solid rgba(124, 172, 248, 0.2);
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 10px;
    font-weight: 500;
    color: var(--accent-blue);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .schedule-btn:hover {
    background: rgba(124, 172, 248, 0.2);
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
    flex-shrink: 0;
  }

  .remove-btn:hover {
    background: var(--bg-hover);
    color: var(--accent-red);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    padding: 2px 2px 0;
  }

  .opt-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
  }

  .opt-toggle input {
    margin: 0;
    accent-color: var(--accent-blue);
    width: 13px;
    height: 13px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .task-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    background: transparent;
    border: none;
    padding: 0;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }

  .task-error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    padding: 5px 8px;
    background: var(--accent-red-dim);
    border: 1px solid color-mix(in srgb, var(--accent-red) 24%, transparent);
    border-radius: 6px;
    color: var(--accent-red);
    font-size: 11px;
    line-height: 1.3;
  }

  .task-error svg {
    flex-shrink: 0;
    opacity: 0.85;
  }

  .task-error-msg {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .task-error-retry {
    flex-shrink: 0;
    padding: 2px 8px;
    border: 1px solid color-mix(in srgb, var(--accent-red) 30%, transparent);
    background: transparent;
    color: var(--accent-red);
    border-radius: 4px;
    font-family: var(--font-body);
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s var(--ease-out), color 0.12s var(--ease-out);
  }

  .task-error-retry:hover {
    background: color-mix(in srgb, var(--accent-red) 18%, transparent);
    color: var(--text-primary);
  }

  .task-item.expanded {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  .task-detail {
    margin-top: 8px;
    padding: 8px 6px 4px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .detail-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .detail-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .detail-link {
    font-size: 11px;
    color: var(--accent-blue);
    text-decoration: none;
  }

  .detail-link:hover {
    text-decoration: underline;
  }

  .reorder-col {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex-shrink: 0;
  }

  .reorder-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 12px;
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: 2px;
    padding: 0;
    transition: all 0.12s var(--ease-out);
  }

  .reorder-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .reorder-btn:disabled {
    opacity: 0.25;
    cursor: not-allowed;
  }

  .new-group-row {
    display: flex;
    padding: 2px;
  }

  .new-group-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 500;
    padding: 5px 10px;
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .new-group-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-style: solid;
  }

  .group-header-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .group-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.12s var(--ease-out);
  }

  .group-action:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .group-action.danger:hover {
    color: var(--accent-red);
  }

  .group-action.confirm {
    width: auto;
    padding: 0 8px;
    background: var(--accent-red);
    color: white;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .group-action.confirm:hover {
    background: var(--accent-red);
    opacity: 0.9;
  }

  .group-rename-input {
    flex: 1;
    background: var(--bg-elevated);
    border: 1px solid var(--accent-blue);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font: inherit;
    font-size: 11px;
    color: var(--text-primary);
    outline: none;
  }

  .group-actions-panel {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 4px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
  }

  .jira-row {
    display: flex;
    gap: 6px;
  }

  .jql-input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .calendar-boxes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cal-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    transition: border-color 0.15s var(--ease-out), background 0.15s var(--ease-out);
  }

  .cal-box.drop-target {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
  }

  .cal-box-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .cal-box-reorder {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex-shrink: 0;
  }

  .cal-box-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .cal-box-name {
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cal-window-chip {
    align-self: flex-start;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 7px;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .cal-window-chip:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .window-editor {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 4px;
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
  }

  .window-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .window-label {
    font-family: var(--font-mono);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-tertiary);
    width: 38px;
    flex-shrink: 0;
  }

  .hour-select {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: 2px 4px;
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
  }

  .window-sep {
    color: var(--text-tertiary);
  }

  .day-pills {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
  }

  .day-pill {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: 10px;
    width: 24px;
    height: 22px;
    cursor: pointer;
    padding: 0;
    transition: all 0.12s var(--ease-out);
  }

  .day-pill.on {
    background: var(--accent-blue-dim);
    border-color: rgba(124, 172, 248, 0.4);
    color: var(--accent-blue);
  }

  .cal-empty-hint {
    padding: 6px 4px;
    font-style: italic;
    color: var(--text-tertiary);
    font-size: 11px;
    text-align: center;
  }

  .empty-hint {
    margin: 0;
    padding: 10px 4px;
    font-size: 11px;
    color: var(--text-tertiary);
    text-align: center;
    font-style: italic;
  }
</style>
