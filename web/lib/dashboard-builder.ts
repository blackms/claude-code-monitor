import type { StatsCache } from "./stats-types";
import type { DashboardPayload } from "./dashboard-types";
import type { HistoryEntry } from "./stats-types";
import {
  averagesFromStats,
  calculateCacheEfficiency,
  calculateCostBreakdown,
  calculateTotalCost,
  formatFirstSessionDate,
  modelTokenTotal,
  monthlyProjection,
  shortenModelName,
  webSearchTotal,
} from "./pricing";
import { groupSessions, topProjects } from "./history";

function periodCostFromMessages(stats: StatsCache, days: number): number {
  const rev = [...stats.dailyActivity].reverse();
  const slice = rev.slice(0, Math.max(1, days));
  const msgs = slice.reduce((s, d) => s + d.messageCount, 0);
  const totalCost = calculateTotalCost(stats);
  const perMsg =
    stats.totalMessages > 0 ? totalCost / stats.totalMessages : 0;
  return msgs * perMsg;
}

function todayEstimatedCost(stats: StatsCache): number {
  const today = new Date().toISOString().slice(0, 10);
  const day = stats.dailyActivity.find((d) => d.date === today);
  const msgs = day?.messageCount ?? 0;
  const totalCost = calculateTotalCost(stats);
  const perMsg =
    stats.totalMessages > 0 ? totalCost / stats.totalMessages : 0;
  return msgs * perMsg;
}

export function buildDashboard(
  claudeDir: string,
  stats: StatsCache,
  history: HistoryEntry[],
): DashboardPayload {
  const totalCost = calculateTotalCost(stats);
  const breakdown = calculateCostBreakdown(stats);
  const cache = calculateCacheEfficiency(stats);
  const avg = averagesFromStats(stats);
  const ws = webSearchTotal(stats);

  const modelEntries = Object.entries(stats.modelUsage).map(([name, u]) => ({
    name: shortenModelName(name),
    raw: name,
    tokens: modelTokenTotal(u),
  }));
  modelEntries.sort((a, b) => b.tokens - a.tokens);
  const totalTok = modelEntries.reduce((s, m) => s + m.tokens, 0) || 1;
  const models = modelEntries.slice(0, 8).map((m) => ({
    name: m.name,
    tokens: m.tokens,
    pct: Math.round((m.tokens / totalTok) * 1000) / 10,
  }));

  const dailyActivity = [...stats.dailyActivity]
    .reverse()
    .slice(0, 14)
    .reverse()
    .map((d) => {
      const parts = d.date.split("-");
      const label =
        parts.length >= 3 ? `${parts[1]}/${parts[2]}` : d.date.slice(5);
      return { date: d.date, label, messages: d.messageCount };
    });

  const hourly: { hour: number; count: number }[] = [];
  const hc = stats.hourCounts as Record<string, number>;
  for (let h = 0; h < 24; h++) {
    const k = String(h);
    const alt = h < 10 ? `0${h}` : k;
    const count = hc[k] ?? hc[alt] ?? 0;
    hourly.push({ hour: h, count: Number(count) || 0 });
  }

  const sessions = groupSessions(history, null)
    .slice(0, 40)
    .map((s) => ({
      shortId: s.shortId,
      projectName: s.projectName,
      messages: s.messageCount,
      lastAt: new Date(s.lastTimestamp).toLocaleString(),
      isActive: s.isActive,
    }));

  const projects = topProjects(history, 10, totalCost);

  const first = stats.firstSessionDate
    ? formatFirstSessionDate(stats.firstSessionDate)
    : null;

  return {
    ok: true,
    claudeDir,
    summary: {
      totalSessions: stats.totalSessions,
      totalMessages: stats.totalMessages,
      firstSessionDate: first,
      webSearches: ws,
      totalCost,
      avgMsgsPerSession: Math.round(avg.msgsPerSess * 10) / 10,
      avgTokensPerMessage: Math.round(avg.tokensPerMsg * 10) / 10,
    },
    costByCategory: breakdown,
    apiCostByPeriod: [
      { label: "Today (est.)", amount: todayEstimatedCost(stats), highlight: true },
      { label: "Last 7 days (est.)", amount: periodCostFromMessages(stats, 7) },
      { label: "Last 30 days (est.)", amount: periodCostFromMessages(stats, 30) },
      {
        label: "Projected / mo (est.)",
        amount: monthlyProjection(stats),
        highlight: true,
      },
      { label: "All time (worst-case)", amount: totalCost },
    ],
    cache: {
      hitRatio: Math.round(cache.hitRatio * 10) / 10,
      savingsUsd: cache.savingsUsd,
      perModel: Object.entries(cache.perModel)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 6),
    },
    models,
    dailyActivity,
    hourly,
    monthlyProjection: monthlyProjection(stats),
    sessions,
    topProjects: projects.map((p) => ({
      name: p.name,
      messages: p.messageCount,
      estimatedCost: p.estimatedCost,
    })),
  };
}
