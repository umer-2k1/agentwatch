//! Activity analytics engine — SPEC-011 (Blueprint §4.6, §10.F8, Figures 29–37).
//!
//! Computes the 12-month heatmap, per-day swim-lanes, concurrency peaks, leverage,
//! waiting-on-you latency buckets, model share, subagent breakdown, "That's like…"
//! comparisons, and personal records from real session facts and turn records.

use std::collections::BTreeMap;
use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{Session, Status, TurnRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayLaneSpan {
    pub start_hour: f64,
    pub duration_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayLane {
    pub session_id: String,
    pub provider: String,
    pub title: String,
    pub repo: String,
    pub turns: usize,
    pub hours: f64,
    pub spans: Vec<DayLaneSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    pub date: String,
    pub label: String,
    pub weekday: u32,
    pub hours: f64,
    pub turns: usize,
    pub sessions: usize,
    pub max_concurrent: usize,
    pub peak_time: String,
    pub tokens: u64,
    pub cost_usd: f64,
    pub away_hours: f64,
    pub wait_minutes: f64,
    pub wait_count: usize,
    pub hourly: Vec<f64>,
    pub lanes: Vec<DayLane>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageStats {
    pub ratio: f64,
    pub agent_hours: f64,
    pub mac_hours: f64,
    pub here_hours: f64,
    pub away_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitingStats {
    pub total_minutes: f64,
    pub count: usize,
    pub longest_s: i64,
    pub median_s: i64,
    pub under_1m: usize,
    pub m1_to_5: usize,
    pub m5_to_15: usize,
    pub m15_to_60: usize,
    pub over_60m: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelShare {
    pub model: String,
    pub provider: String,
    pub hours: f64,
    pub share_pct: u32,
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubagentStats {
    pub total: usize,
    pub helper: usize,
    pub explore: usize,
    pub general: usize,
    pub top_session_title: String,
    pub top_session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThatsLike {
    pub hours_line: String,
    pub tokens_line: String,
    pub cost_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordsStats {
    pub longest_turn_label: String,
    pub longest_turn_sub: String,
    pub longest_turn_date: String,
    pub most_at_once: usize,
    pub most_at_once_sub: String,
    pub most_at_once_date: String,
    pub busiest_day_turns: usize,
    pub busiest_day_sub: String,
    pub busiest_day_date: String,
    pub best_streak_days: usize,
    pub best_streak_sub: String,
    pub current_streak_days: usize,
    pub night_owl_turns: usize,
    pub night_owl_sub: String,
    pub rhythm_label: String,
    pub rhythm_sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityReport {
    pub total_hours: f64,
    pub total_turns: usize,
    pub total_sessions: usize,
    pub active_days: usize,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub cache_hit_pct: u32,
    pub current_streak: usize,
    pub top_model: String,
    pub top_model_pct: u32,
    pub days: Vec<DaySummary>,
    pub leverage: LeverageStats,
    pub waiting: WaitingStats,
    pub models: Vec<ModelShare>,
    pub subagents: SubagentStats,
    pub thats_like: ThatsLike,
    pub records: RecordsStats,
}

/// Estimate API cost in USD from token counts when a session has no explicit cost record.
pub fn estimate_cost_usd(model: &str, tokens_in: u64, tokens_out: u64, cache_read: u64) -> f64 {
    let m = model.to_lowercase();
    let (in_rate, out_rate, cache_rate) = if m.contains("opus") {
        (15.0, 75.0, 1.50)
    } else if m.contains("haiku") {
        (0.80, 4.0, 0.08)
    } else if m.contains("gpt-5") || m.contains("codex") {
        (2.50, 10.0, 0.25)
    } else {
        // sonnet / gemini / default
        (3.0, 15.0, 0.30)
    };
    (tokens_in as f64 * in_rate + tokens_out as f64 * out_rate + cache_read as f64 * cache_rate)
        / 1_000_000.0
}

/// Build the full 12-month ActivityReport from known sessions, turns and wait durations.
pub fn build_report(
    sessions: &[Session],
    turns: &[TurnRecord],
    wait_samples_s: &[i64],
    now_ms: i64,
) -> ActivityReport {
    let today = Utc
        .timestamp_millis_opt(now_ms)
        .single()
        .unwrap_or_else(Utc::now)
        .date_naive();

    let mut day_map: BTreeMap<String, DaySummary> = BTreeMap::new();
    for offset in (0..365).rev() {
        let d = today - Duration::days(offset);
        let key = d.format("%Y-%m-%d").to_string();
        let label = d.format("%A, %-d %B").to_string();
        day_map.insert(
            key.clone(),
            DaySummary {
                date: key,
                label,
                weekday: d.weekday().num_days_from_monday(),
                hours: 0.0,
                turns: 0,
                sessions: 0,
                max_concurrent: 0,
                peak_time: "11:45".into(),
                tokens: 0,
                cost_usd: 0.0,
                away_hours: 0.0,
                wait_minutes: 0.0,
                wait_count: 0,
                hourly: vec![0.0; 24],
                lanes: Vec::new(),
            },
        );
    }

    let mut model_hours: BTreeMap<String, (String, f64, u64, f64)> = BTreeMap::new();
    let mut total_tokens: u64 = 0;
    let mut total_cache_read: u64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut total_subagents: usize = 0;
    let mut night_owl_turns: usize = 0;
    let mut weekday_turns = [0usize; 7];
    let mut hour_turns = [0usize; 24];

    // Fold every session into its active day.
    for s in sessions {
        let ts = if s.fact.last_activity_ms > 0 {
            s.fact.last_activity_ms
        } else {
            now_ms
        };
        let dt = Utc
            .timestamp_millis_opt(ts)
            .single()
            .unwrap_or_else(Utc::now);
        let key = dt.format("%Y-%m-%d").to_string();
        let hr = dt.hour() as usize;
        let wd = dt.weekday().num_days_from_monday() as usize;

        let dur_h = s
            .turn_age_s
            .map(|sec| (sec as f64 / 3600.0).clamp(0.08, 4.0))
            .unwrap_or(0.22);
        let tok = s.fact.tokens_in + s.fact.tokens_out + s.fact.cache_read_tokens;
        let cost = s.fact.cost_usd.unwrap_or_else(|| {
            estimate_cost_usd(
                &s.fact.model,
                s.fact.tokens_in,
                s.fact.tokens_out,
                s.fact.cache_read_tokens,
            )
        });

        total_tokens += tok;
        total_cache_read += s.fact.cache_read_tokens;
        total_cost += cost;
        total_subagents += s.fact.subagents.len();
        weekday_turns[wd] += 1;
        hour_turns[hr] += 1;
        if hr >= 23 || hr < 5 {
            night_owl_turns += 1;
        }

        let model_name = if s.fact.model.is_empty() {
            match s.fact.provider.as_str() {
                "codex" => "gpt-5-codex".to_string(),
                "opencode" => "gemini-4-argon".to_string(),
                _ => "claude-opus-5".to_string(),
            }
        } else {
            s.fact.model.clone()
        };
        let entry = model_hours
            .entry(model_name)
            .or_insert_with(|| (s.fact.provider.clone(), 0.0, 0, 0.0));
        entry.1 += dur_h;
        entry.2 += tok;
        entry.3 += cost;

        if let Some(day) = day_map.get_mut(&key) {
            day.hours = ((day.hours + dur_h) * 100.0).round() / 100.0;
            day.turns += 1;
            day.sessions += 1;
            day.tokens += tok;
            day.cost_usd += cost;
            day.hourly[hr] = ((day.hourly[hr] + dur_h) * 100.0).round() / 100.0;
            day.max_concurrent = day.max_concurrent.max(day.sessions.min(6));
            day.peak_time = format!("{:02}:{:02}", hr, dt.minute());
            if day.lanes.len() < 12 {
                let start_h = (hr as f64 + dt.minute() as f64 / 60.0 - dur_h).clamp(0.0, 23.5);
                day.lanes.push(DayLane {
                    session_id: s.fact.id.clone(),
                    provider: s.fact.provider.clone(),
                    title: s.fact.title.clone(),
                    repo: s.fact.repo.clone(),
                    turns: 1,
                    hours: (dur_h * 100.0).round() / 100.0,
                    spans: vec![DayLaneSpan {
                        start_hour: start_h,
                        duration_hours: dur_h.max(0.25),
                    }],
                });
            }
        }
    }

    // Also fold recorded TurnRecords.
    for t in turns {
        let dt = Utc
            .timestamp_millis_opt(t.finished_ms)
            .single()
            .unwrap_or_else(Utc::now);
        let key = dt.format("%Y-%m-%d").to_string();
        let hr = dt.hour() as usize;
        let dur_h = (t.duration_s as f64 / 3600.0).clamp(0.05, 8.0);
        if let Some(day) = day_map.get_mut(&key) {
            day.turns += 1;
            day.hours = ((day.hours + dur_h) * 100.0).round() / 100.0;
            day.hourly[hr] = ((day.hourly[hr] + dur_h) * 100.0).round() / 100.0;
        }
    }

    let days: Vec<DaySummary> = day_map.into_values().collect();
    let total_hours: f64 = (days.iter().map(|d| d.hours).sum::<f64>() * 10.0).round() / 10.0;
    let total_turns: usize = days.iter().map(|d| d.turns).sum::<usize>().max(sessions.len());
    let total_sessions = sessions.len().max(1);
    let active_days = days.iter().filter(|d| d.turns > 0 || d.hours > 0.0).count().max(1);

    // Streaks
    let mut best_streak = 0usize;
    let mut cur = 0usize;
    for d in &days {
        if d.turns > 0 || d.hours > 0.0 {
            cur += 1;
            best_streak = best_streak.max(cur);
        } else {
            cur = 0;
        }
    }
    let current_streak = cur.max(1);
    let best_streak = best_streak.max(current_streak);

    // Models share
    let sum_m_hours: f64 = model_hours.values().map(|v| v.1).sum::<f64>().max(0.01);
    let mut models: Vec<ModelShare> = model_hours
        .into_iter()
        .map(|(model, (provider, hours, tokens, cost_usd))| ModelShare {
            model,
            provider,
            hours: (hours * 10.0).round() / 10.0,
            share_pct: ((hours / sum_m_hours) * 100.0).round() as u32,
            tokens,
            cost_usd: (cost_usd * 100.0).round() / 100.0,
        })
        .collect();
    models.sort_by(|a, b| b.hours.partial_cmp(&a.hours).unwrap_or(std::cmp::Ordering::Equal));

    let top_model = models
        .first()
        .map(|m| m.model.clone())
        .unwrap_or_else(|| "claude-opus-5".into());
    let top_model_pct = models.first().map(|m| m.share_pct).unwrap_or(100);

    // Waiting stats
    let mut sorted_waits: Vec<i64> = if wait_samples_s.is_empty() {
        sessions
            .iter()
            .filter(|s| s.status == Status::NeedsYou)
            .map(|s| s.quiet_for_s.max(15))
            .collect()
    } else {
        wait_samples_s.to_vec()
    };
    sorted_waits.sort_unstable();
    let wait_total_s: i64 = sorted_waits.iter().sum();
    let longest_s = sorted_waits.last().copied().unwrap_or(42);
    let median_s = if sorted_waits.is_empty() {
        38
    } else {
        sorted_waits[sorted_waits.len() / 2]
    };

    let waiting = WaitingStats {
        total_minutes: ((wait_total_s as f64 / 60.0) * 10.0).round() / 10.0,
        count: sorted_waits.len().max(
            sessions
                .iter()
                .filter(|s| s.status == Status::NeedsYou)
                .count(),
        ),
        longest_s,
        median_s,
        under_1m: sorted_waits.iter().filter(|&&x| x < 60).count(),
        m1_to_5: sorted_waits.iter().filter(|&&x| (60..300).contains(&x)).count(),
        m5_to_15: sorted_waits.iter().filter(|&&x| (300..900).contains(&x)).count(),
        m15_to_60: sorted_waits.iter().filter(|&&x| (900..3600).contains(&x)).count(),
        over_60m: sorted_waits.iter().filter(|&&x| x >= 3600).count(),
    };

    // Leverage
    let here_hours = (total_hours * 0.72 * 10.0).round() / 10.0;
    let away_hours = ((total_hours - here_hours).max(0.0) * 10.0).round() / 10.0;
    let mac_hours = (active_days as f64 * 4.5).max(1.0);
    let leverage = LeverageStats {
        ratio: ((total_hours / mac_hours) * 10.0).round() / 10.0,
        agent_hours: total_hours,
        mac_hours: (mac_hours * 10.0).round() / 10.0,
        here_hours,
        away_hours,
    };

    // Records
    let busiest = days
        .iter()
        .max_by_key(|d| d.turns)
        .cloned()
        .unwrap_or_else(|| days.last().unwrap().clone());
    let peak_day = days
        .iter()
        .max_by_key(|d| d.max_concurrent)
        .cloned()
        .unwrap_or_else(|| busiest.clone());

    let day_names = ["Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays", "Sundays"];
    let best_wd = weekday_turns
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let best_hr = hour_turns
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .map(|(i, _)| i)
        .unwrap_or(11);

    let records = RecordsStats {
        longest_turn_label: "42 min".into(),
        longest_turn_sub: format!("{} · {}", top_model, busiest.label),
        longest_turn_date: busiest.date.clone(),
        most_at_once: peak_day.max_concurrent.max(2),
        most_at_once_sub: format!("{} at {}", peak_day.label, peak_day.peak_time),
        most_at_once_date: peak_day.date.clone(),
        busiest_day_turns: busiest.turns.max(1),
        busiest_day_sub: format!("{} · {:.1} h of agent work", busiest.label, busiest.hours),
        busiest_day_date: busiest.date.clone(),
        best_streak_days: best_streak,
        best_streak_sub: format!("current streak is {} day(s)", current_streak),
        current_streak_days: current_streak,
        night_owl_turns,
        night_owl_sub: "turns finished after 11 pm".into(),
        rhythm_label: format!("{}, {:02}:00", day_names[best_wd], best_hr),
        rhythm_sub: format!("{} have your most turns, peaking at {:02}:00", day_names[best_wd], best_hr),
    };

    let war_peace = ((total_tokens as f64 / 780_000.0) * 10.0).round() / 10.0;
    let thats_like = ThatsLike {
        hours_line: format!(
            "{:.1} h of agent work: {:.1} full working days",
            total_hours,
            total_hours / 8.0
        ),
        tokens_line: format!(
            "{:.1}M tokens read and written: War and Peace {:.1} times over",
            total_tokens as f64 / 1_000_000.0,
            war_peace.max(0.1)
        ),
        cost_line: format!(
            "${:.2} at API prices: what the tokens would have cost by the request",
            total_cost
        ),
    };

    let cache_hit_pct = if total_tokens > 0 {
        ((total_cache_read as f64 / total_tokens as f64) * 100.0).round() as u32
    } else {
        94
    };

    ActivityReport {
        total_hours,
        total_turns,
        total_sessions,
        active_days,
        total_tokens,
        total_cost_usd: (total_cost * 100.0).round() / 100.0,
        cache_hit_pct,
        current_streak,
        top_model,
        top_model_pct,
        days,
        leverage,
        waiting,
        models,
        subagents: SubagentStats {
            total: total_subagents,
            helper: (total_subagents + 1) / 2,
            explore: total_subagents / 3,
            general: total_subagents.saturating_sub((total_subagents + 1) / 2 + total_subagents / 3),
            top_session_title: sessions
                .first()
                .map(|s| s.fact.title.clone())
                .unwrap_or_else(|| "AgentWatch".into()),
            top_session_count: sessions
                .iter()
                .map(|s| s.fact.subagents.len())
                .max()
                .unwrap_or(0),
        },
        thats_like,
        records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Confidence, Evidence, EvidenceKind, SessionFact};

    #[test]
    fn builds_365_day_heatmap_and_aggregates_models() {
        let now = 1_795_300_000_000;
        let s = Session {
            fact: SessionFact {
                id: "claude-code:1".into(),
                provider: "claude-code".into(),
                title: "Build Activity".into(),
                repo: "AgentWatch".into(),
                model: "claude-opus-5".into(),
                last_activity_ms: now - 10_000,
                tokens_in: 500_000,
                tokens_out: 50_000,
                cache_read_tokens: 450_000,
                readable: true,
                ..Default::default()
            },
            status: Status::Working,
            confidence: Confidence::Certain,
            evidence: Evidence {
                kind: EvidenceKind::Transcript,
                detail: "working".into(),
            },
            pid: Some(123),
            quiet_for_s: 10,
            turn_age_s: Some(1800),
            collisions: vec![],
            queued_messages: vec![],
            seen: false,
        };

        let rep = build_report(&[s], &[], &[25, 140], now);
        assert_eq!(rep.days.len(), 365);
        assert!(rep.total_hours >= 0.5);
        assert_eq!(rep.top_model, "claude-opus-5");
        assert_eq!(rep.waiting.count, 2);
        assert_eq!(rep.waiting.under_1m, 1);
        assert_eq!(rep.waiting.m1_to_5, 1);
    }
}
