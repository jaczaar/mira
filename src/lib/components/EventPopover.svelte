<script lang="ts">
  import type { CalendarEvent } from "../api";

  interface Props {
    event: CalendarEvent;
    position: { x: number; y: number };
    onEdit: (event: CalendarEvent, updates: { summary?: string; start_date?: string; end_date?: string; description?: string | null }) => Promise<void>;
    onDelete: (event: CalendarEvent) => void;
    onClose: () => void;
  }

  let { event, position, onEdit, onDelete, onClose }: Props = $props();

  // View / Edit mode
  let editing = $state(false);
  let saving = $state(false);

  // Edit fields
  let editSummary = $state(event.summary);
  let editDescription = $state(event.description ?? "");
  let editStartDate = $state("");
  let editStartTime = $state("");
  let editEndTime = $state("");

  // Description expand
  let descExpanded = $state(false);
  let descEl: HTMLDivElement | undefined = $state();
  let descOverflows = $state(false);

  // Delete confirmation
  let confirmDelete = $state(false);
  let deleting = $state(false);

  $effect(() => {
    if (descEl) {
      descOverflows = descEl.scrollHeight > descEl.clientHeight;
    }
  });

  function parseDateTime(dateStr: string) {
    const d = new Date(dateStr);
    const date = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    const time = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
    return { date, time };
  }

  function startEditing() {
    const start = parseDateTime(event.start_date);
    const end = parseDateTime(event.end_date);
    editSummary = event.summary;
    editDescription = event.description ?? "";
    editStartDate = start.date;
    editStartTime = start.time;
    editEndTime = end.time;
    editing = true;
    confirmDelete = false;
  }

  function cancelEditing() {
    editing = false;
  }

  async function saveEdits() {
    saving = true;
    try {
      const updates: { summary?: string; start_date?: string; end_date?: string; description?: string | null } = {};
      if (editSummary !== event.summary) updates.summary = editSummary;
      const newStart = `${editStartDate}T${editStartTime}:00`;
      const newEnd = `${editStartDate}T${editEndTime}:00`;
      if (newStart !== event.start_date.slice(0, 19)) updates.start_date = newStart;
      if (newEnd !== event.end_date.slice(0, 19)) updates.end_date = newEnd;
      if (editDescription !== (event.description ?? "")) updates.description = editDescription || null;

      if (Object.keys(updates).length > 0) {
        await onEdit(event, updates);
      }
      editing = false;
    } finally {
      saving = false;
    }
  }

  function formatTime(dateStr: string): string {
    const d = new Date(dateStr);
    const h = d.getHours();
    const m = d.getMinutes();
    const period = h >= 12 ? "PM" : "AM";
    const hour = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return m === 0 ? `${hour} ${period}` : `${hour}:${m.toString().padStart(2, "0")} ${period}`;
  }

  function formatDateLong(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("en-US", {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }

  function getDurationLabel(startStr: string, endStr: string): string {
    const mins = (new Date(endStr).getTime() - new Date(startStr).getTime()) / 60000;
    if (mins < 60) return `${mins}m`;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
  }

  async function handleDelete() {
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    deleting = true;
    onDelete(event);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (editing) { cancelEditing(); e.stopPropagation(); }
      else if (confirmDelete) { confirmDelete = false; e.stopPropagation(); }
      else onClose();
    }
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && editing) {
      saveEdits();
    }
  }

  const popoverStyle = $derived.by(() => {
    const maxW = 340;
    let x = position.x;
    let y = position.y;
    if (x + maxW > window.innerWidth - 20) x = window.innerWidth - maxW - 20;
    if (x < 20) x = 20;
    if (y + 320 > window.innerHeight - 20) y = Math.max(20, position.y - 320);
    return `left: ${x}px; top: ${y}px;`;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="popover-backdrop" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="popover" role="dialog" style={popoverStyle} onclick={(e) => e.stopPropagation()}>

    <!-- Color accent bar -->
    <div class="accent-bar"></div>

    {#if editing}
      <!-- ═══ EDIT MODE ═══ -->
      <div class="edit-mode">
        <input
          class="edit-title"
          type="text"
          bind:value={editSummary}
          placeholder="Event title"
          autofocus
        />

        <div class="edit-row">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
          </svg>
          <input type="date" class="edit-input" bind:value={editStartDate} />
        </div>

        <div class="edit-row">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
          </svg>
          <input type="time" class="edit-input" bind:value={editStartTime} />
          <span class="edit-sep">–</span>
          <input type="time" class="edit-input" bind:value={editEndTime} />
        </div>

        <div class="edit-row edit-row--desc">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="17" y1="10" x2="3" y2="10" /><line x1="21" y1="6" x2="3" y2="6" /><line x1="21" y1="14" x2="3" y2="14" /><line x1="17" y1="18" x2="3" y2="18" />
          </svg>
          <textarea
            class="edit-textarea"
            bind:value={editDescription}
            placeholder="Add a description..."
            rows="3"
          ></textarea>
        </div>

        <div class="edit-actions">
          <button class="btn-ghost" onclick={cancelEditing}>Cancel</button>
          <button class="btn-save" onclick={saveEdits} disabled={!editSummary.trim() || saving}>
            {saving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>

    {:else}
      <!-- ═══ VIEW MODE ═══ -->
      <div class="view-header">
        <h4 class="event-title">{event.summary}</h4>
        <div class="header-actions">
          <button class="icon-btn" onclick={startEditing} title="Edit event">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
            </svg>
          </button>
          <button class="icon-btn" onclick={onClose} title="Close">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>

      <div class="view-body">
        <!-- Time -->
        <div class="detail-row">
          <div class="detail-icon">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
            </svg>
          </div>
          <div class="detail-content">
            <span class="detail-primary">{formatDateLong(event.start_date)}</span>
            <span class="detail-secondary">{formatTime(event.start_date)} – {formatTime(event.end_date)} <span class="duration-chip">{getDurationLabel(event.start_date, event.end_date)}</span></span>
          </div>
        </div>

        <!-- Calendar -->
        <div class="detail-row">
          <div class="detail-icon">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
            </svg>
          </div>
          <div class="detail-content">
            <span class="detail-calendar">{event.calendar_name}</span>
          </div>
        </div>

        <!-- Description -->
        {#if event.description}
          <div class="detail-row detail-row--desc">
            <div class="detail-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="17" y1="10" x2="3" y2="10" /><line x1="21" y1="6" x2="3" y2="6" /><line x1="21" y1="14" x2="3" y2="14" /><line x1="17" y1="18" x2="3" y2="18" />
              </svg>
            </div>
            <div class="detail-content">
              <div class="description-wrap" class:expanded={descExpanded} bind:this={descEl}>
                {event.description}
              </div>
              {#if descOverflows && !descExpanded}
                <button class="see-more" onclick={() => (descExpanded = true)}>See more</button>
              {/if}
              {#if descExpanded}
                <button class="see-more" onclick={() => (descExpanded = false)}>Show less</button>
              {/if}
            </div>
          </div>
        {/if}

        <!-- URL -->
        {#if event.url}
          <div class="detail-row">
            <div class="detail-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
              </svg>
            </div>
            <div class="detail-content">
              <span class="detail-link">{event.url}</span>
            </div>
          </div>
        {/if}
      </div>

      <div class="view-footer">
        {#if confirmDelete}
          <span class="confirm-text">Delete this event?</span>
          <button class="btn-ghost btn-sm" onclick={() => (confirmDelete = false)}>Cancel</button>
          <button class="btn-danger-solid btn-sm" onclick={handleDelete} disabled={deleting}>
            {deleting ? "..." : "Delete"}
          </button>
        {:else}
          <button class="btn-danger btn-sm" onclick={handleDelete}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
            </svg>
            Delete
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  /* ─── Backdrop & Shell ─── */
  .popover-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
  }

  .popover {
    position: fixed;
    width: 340px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow:
      0 8px 40px rgba(0, 0, 0, 0.4),
      0 0 0 1px rgba(255, 255, 255, 0.03),
      0 0 80px -20px rgba(124, 172, 248, 0.08);
    z-index: 151;
    overflow: hidden;
    animation: popoverIn 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes popoverIn {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .accent-bar {
    height: 3px;
    background: var(--gradient-brand);
    opacity: 0.7;
  }

  /* ─── VIEW MODE ─── */
  .view-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 14px 16px 6px;
    gap: 8px;
  }

  .event-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1.3;
    letter-spacing: -0.02em;
    word-break: break-word;
  }

  .header-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .view-body {
    padding: 6px 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .detail-row {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }

  .detail-icon {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
    color: var(--text-tertiary);
    transition: color 0.12s;
  }

  .detail-row:hover .detail-icon {
    color: var(--accent-blue);
  }

  .detail-content {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    padding-top: 3px;
  }

  .detail-primary {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .detail-secondary {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .duration-chip {
    display: inline-block;
    padding: 1px 6px;
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .detail-calendar {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    letter-spacing: 0.01em;
  }

  .detail-link {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--accent-blue);
    word-break: break-all;
    opacity: 0.8;
  }

  /* ─── Description ─── */
  .description-wrap {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 7.75lh; /* ~5 lines */
    overflow: hidden;
    transition: max-height 0.25s var(--ease-out);
  }

  .description-wrap.expanded {
    max-height: 300px;
    overflow-y: auto;
  }

  .see-more {
    background: none;
    border: none;
    padding: 0;
    margin-top: 4px;
    color: var(--accent-blue);
    font-family: var(--font-body);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    opacity: 0.8;
    transition: opacity 0.12s;
  }

  .see-more:hover {
    opacity: 1;
    text-decoration: underline;
  }

  /* ─── Footer ─── */
  .view-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    padding: 10px 16px 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .confirm-text {
    font-size: 12px;
    color: var(--text-secondary);
    margin-right: auto;
  }

  .btn-sm {
    font-size: 12px;
    padding: 5px 14px;
  }

  .btn-ghost {
    border: 1px solid var(--border-default);
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-family: var(--font-body);
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .btn-ghost:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-danger {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    border: 1px solid rgba(240, 144, 144, 0.15);
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    font-family: var(--font-body);
    cursor: pointer;
    transition: all 0.15s var(--ease-out);
  }

  .btn-danger:hover {
    background: var(--accent-red-dim);
    color: var(--accent-red);
    border-color: rgba(240, 144, 144, 0.25);
  }

  .btn-danger-solid {
    border: none;
    background: var(--accent-red);
    border-radius: var(--radius-sm);
    color: white;
    font-family: var(--font-body);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .btn-danger-solid:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-danger-solid:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ─── EDIT MODE ─── */
  .edit-mode {
    padding: 14px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .edit-title {
    width: 100%;
    padding: 8px 0;
    border: none;
    border-bottom: 2px solid var(--border-default);
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
    outline: none;
    transition: border-color 0.15s;
  }

  .edit-title::placeholder {
    color: var(--text-tertiary);
  }

  .edit-title:focus {
    border-color: var(--accent-blue);
  }

  .edit-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .edit-row svg {
    flex-shrink: 0;
    color: var(--text-tertiary);
  }

  .edit-row--desc {
    align-items: flex-start;
  }

  .edit-row--desc svg {
    margin-top: 8px;
  }

  .edit-input {
    padding: 6px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s;
  }

  .edit-input:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .edit-sep {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .edit-textarea {
    flex: 1;
    padding: 8px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: 13px;
    line-height: 1.5;
    outline: none;
    resize: vertical;
    min-height: 64px;
    transition: border-color 0.15s;
  }

  .edit-textarea::placeholder {
    color: var(--text-tertiary);
  }

  .edit-textarea:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 4px;
  }

  .btn-save {
    padding: 6px 20px;
    border: none;
    background: var(--accent-blue);
    border-radius: var(--radius-sm);
    color: white;
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s var(--ease-out);
  }

  .btn-save:hover:not(:disabled) {
    filter: brightness(1.1);
    box-shadow: 0 2px 12px rgba(124, 172, 248, 0.25);
  }

  .btn-save:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
