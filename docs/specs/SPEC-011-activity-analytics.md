# SPEC-011 — Activity Analytics, Heatmap, Day Replay & Share Card

Status: **specified** (Blueprint §4.6, §10.F8, §18.55, Figures 29–37)
Related: D-011

## 1. Data Source & Backfill

Activity is computed from two sources merged together:
1. Historical backfill from on-disk session files (`~/.claude/projects/**/*.jsonl`,
   `~/.codex/sessions/**/rollout-*.jsonl`, `~/.local/share/opencode/opencode.db`).
2. Append-only event spans recorded by AgentWatch while running (`waiting_on_you` durations,
   presence `here` vs `away` where away = no input / sleep >= 5 min).

## 2. Heatmap & Views (Figures 29, 33–36)

- **Header summary**: `X hours of agent work · N turns in M sessions · D of 365 days · T tokens · ≈ $C at API prices · S day streak · mostly <top_model>`.
- **53×7 Heatmap**: 365 days, shaded by `Agent hours` or `Turns` (toggleable). Clicking any
  square selects that day; `←` / `→` step through days.
- **Day View**:
  - `You — at your Mac` presence bar with `Away` shaded spans.
  - `At once (agents working)` concurrency sparkline across 24 hours with a peak marker
    (e.g. `4 at once · 14:33`).
  - One swim-lane per session active on that day, showing turn bars across the hours.
  - **Replay (`▶ Replay` / `Space`)**: animates a vertical time cursor across the selected
    day's 24-hour timeline, displaying live status at that minute (`13:24 · 2 working · 5 turns done`).
- **Week & Month Views**: hour-by-hour grid per day + `Average day (agent hours by hour)`
  histogram.

## 3. Six Analytic Cards (Figures 30–32)

1. **LEVERAGE**: `X.X× agent hours for every hour you were at your Mac`, split bar for
   `While you were here` vs `While you were away`.
2. **WAITING ON YOU**: total wait time, count of prompts, longest wait, median wait, and
   5 latency buckets (`Under a minute`, `1–5 minutes`, `5–15 minutes`, `15–60 minutes`,
   `Over an hour`).
3. **MODELS**: share of hours and tokens/cost by model (`claude-opus-5`, `gpt-5-codex`, etc.).
4. **SUBAGENTS**: total subagents spawned (`isSidechain` / `parent_id`), breakdown by type
   and top session.
5. **THAT'S LIKE…**: human-scale comparisons (`Xh = N full working days`, `TM tokens = War and Peace K times over`, `≈ $C at API prices`).
6. **RECORDS**: `Longest turn`, `Most at once`, `Busiest day`, `Best streak`, `Night owl`
   (turns finished after 11 pm), `Your rhythm` (busiest weekday & hour). Each record links
   to `Replay the day ↗`.

## 4. Share Card (Figure 37)

- Generates a privacy-safe 1080×1080 summary card (`Totals only: no titles, repositories, or branches`).
- Toggles: Period (`7 days`, `30 days`, `12 months`) and Look (`Night`, `Day`), with
  `Copy Image` and `Save PNG`.
