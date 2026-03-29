# Mira Roadmap

## Completed

### Phase 0: Rebrand

- Renamed from JaCal to Mira
- Clean repo with only app code at root

### Phase 1: Claude Code Chatbot

- Floating chat widget (bottom-right)
- Spawns Claude Code CLI sessions in the repo
- Streams responses in real-time
- Diff viewer for file changes
- PR creation workflow for community contributions

## Current: Phase 2 — Auto-Scheduler (Priority Slot-Filling + Risk Badges)

- Rust scheduling engine with greedy constraint-satisfaction solver
- Priority ordering: ASAP > deadline urgency > priority level > available slots
- Deadline risk badges: ON_TRACK / AT_RISK / URGENT / OVERDUE
- Schedule preview with "Apply" to commit to Google Calendar
- Two-phase commit: propose then apply

## Future Phases

### Phase 3: Schedule Windows

- Define time boundaries: Work Hours, Personal Hours, Focus Time, No-Meeting Time
- Per-window-type task filtering (focus work only during focus time)
- Visual weekly grid editor in Settings

### Phase 4: Task Chunking

- Split large tasks (>2h) into multiple blocks across days
- Configurable max chunk size (default: 2 hours)
- Spread chunks across days to avoid fatigue
- Track chunk → parent task relationships

### Phase 5: Focus Bar ("What to Work On Now")

- Amber bar at top of dashboard showing current scheduled task
- Shows: task key, summary, time remaining, progress bar
- Actions: Start (open Jira), Snooze 15m, Skip
- When idle: shows next upcoming task

### Phase 6: Dynamic Rescheduling

- "Took longer" / "Done early" actions on Focus Bar
- Full reschedule triggered on state change
- Preserve manually pinned events
- Smart reshuffling for remaining day/week

### Phase 7: Advanced Task Management

- Recurring tasks with repeat patterns
- Task dependencies (A must finish before B)
- Kanban / list view toggle
- Drag-and-drop task reordering

### Phase 8: LLM-Powered Priority Scoring

- Hook priority scoring into an LLM for smarter scheduling
- Context-aware prioritization based on project state
- Natural language task creation ("schedule 2h for the gRPC migration")

