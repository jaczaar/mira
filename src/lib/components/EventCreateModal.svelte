<script lang="ts">
  import type { CalendarInfo } from "../api";

  interface Props {
    date: string;
    startTime: string;
    endTime?: string;
    initialSummary?: string;
    calendars: CalendarInfo[];
    defaultCalendarUid: string | null;
    onSave: (data: {
      summary: string;
      date: string;
      startTime: string;
      endTime: string;
      calendarUid: string;
      description: string;
    }) => void;
    onClose: () => void;
  }

  let {
    date,
    startTime,
    endTime,
    initialSummary = "",
    calendars,
    defaultCalendarUid,
    onSave,
    onClose,
  }: Props = $props();

  let summary = $state(initialSummary);
  let eventDate = $state(date);
  let eventStartTime = $state(startTime);
  let saving = $state(false);

  const fallbackEnd = $derived.by(() => {
    const [h, m] = eventStartTime.split(":").map(Number);
    const endH = Math.min(h + 1, 23);
    return `${String(endH).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
  });
  let eventEndTime = $state(endTime ?? "");
  $effect(() => {
    if (!eventEndTime) eventEndTime = fallbackEnd;
  });

  let selectedCalendar = $state(defaultCalendarUid ?? calendars[0]?.uid ?? "");
  let description = $state("");

  async function handleSave() {
    if (!summary.trim() || !selectedCalendar) return;
    saving = true;
    try {
      onSave({
        summary: summary.trim(),
        date: eventDate,
        startTime: eventStartTime,
        endTime: eventEndTime,
        calendarUid: selectedCalendar,
        description: description.trim(),
      });
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) handleSave();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="modal-backdrop" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="modal" role="dialog" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h3>New Event</h3>
      <button class="close-btn" onclick={onClose}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="field">
        <input
          type="text"
          bind:value={summary}
          placeholder="Event title"
          class="title-input"
          autofocus
        />
      </div>

      <div class="field-row">
        <div class="field">
          <label>Date</label>
          <input type="date" bind:value={eventDate} />
        </div>
        <div class="field">
          <label>Start</label>
          <input type="time" bind:value={eventStartTime} />
        </div>
        <div class="field">
          <label>End</label>
          <input type="time" bind:value={eventEndTime} />
        </div>
      </div>

      <div class="field">
        <label>Calendar</label>
        <select bind:value={selectedCalendar}>
          {#each calendars as cal}
            <option value={cal.uid}>{cal.name}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label>Description</label>
        <textarea
          bind:value={description}
          placeholder="Optional"
          rows="2"
        ></textarea>
      </div>
    </div>

    <div class="modal-footer">
      <button class="cancel-btn" onclick={onClose}>Cancel</button>
      <button class="save-btn" onclick={handleSave} disabled={!summary.trim() || !selectedCalendar || saving}>
        {saving ? "Creating..." : "Create"}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    animation: fadeIn 0.15s var(--ease-out);
  }

  .modal {
    width: 400px;
    max-width: 90vw;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: fadeInUp 0.2s var(--ease-out);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .modal-header h3 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .close-btn {
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
    transition: all 0.12s;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-row {
    display: flex;
    gap: 10px;
  }

  .field-row .field {
    flex: 1;
  }

  .field label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .title-input {
    padding: 10px 12px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: 15px;
    font-weight: 500;
    outline: none;
    transition: border-color 0.15s;
  }

  .title-input::placeholder {
    color: var(--text-tertiary);
  }

  .title-input:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .field input[type="date"],
  .field input[type="time"],
  .field select,
  .field textarea {
    padding: 7px 10px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
  }

  .field input:focus,
  .field select:focus,
  .field textarea:focus {
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px var(--accent-blue-dim);
  }

  .field textarea {
    resize: vertical;
    min-height: 48px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px 16px;
    border-top: 1px solid var(--border-subtle);
  }

  .cancel-btn {
    padding: 7px 16px;
    border: 1px solid var(--border-default);
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-family: var(--font-body);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .save-btn {
    padding: 7px 20px;
    border: none;
    background: var(--accent-blue);
    border-radius: var(--radius-sm);
    color: white;
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s;
  }

  .save-btn:hover:not(:disabled) {
    opacity: 0.9;
    box-shadow: var(--shadow-glow-blue);
  }

  .save-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
