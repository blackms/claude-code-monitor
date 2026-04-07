import type { ModelUsage, StatsCache } from "./stats-types";

export type Pricing = {
  input: number;
  output: number;
  cacheRead: number;
  cacheCreate: number;
};

export function pricingForModel(modelName: string): Pricing {
  const n = modelName.toLowerCase();
  if (n.includes("opus")) {
    return {
      input: 15,
      output: 75,
      cacheRead: 1.5,
      cacheCreate: 18.75,
    };
  }
  if (n.includes("sonnet")) {
    return { input: 3, output: 15, cacheRead: 0.3, cacheCreate: 3.75 };
  }
  if (n.includes("haiku-4-5")) {
    return { input: 1, output: 5, cacheRead: 0.1, cacheCreate: 1.25 };
  }
  if (n.includes("haiku-3-5") || n.includes("3-5-haiku")) {
    return { input: 0.8, output: 4, cacheRead: 0.08, cacheCreate: 1.0 };
  }
  if (n.includes("haiku")) {
    return { input: 0.25, output: 1.25, cacheRead: 0.03, cacheCreate: 0.3 };
  }
  return { input: 3, output: 15, cacheRead: 0.3, cacheCreate: 3.75 };
}

function cacheSavingsPerMillion(p: Pricing): number {
  return p.input - p.cacheRead;
}

/** Worst-case total (matches Rust `calculate_total_cost`). */
export function calculateTotalCost(stats: StatsCache): number {
  let total = 0;
  for (const [modelName, usage] of Object.entries(stats.modelUsage)) {
    const p = pricingForModel(modelName);
    total += (usage.inputTokens / 1e6) * p.input;
    total += (usage.outputTokens / 1e6) * p.output;
    total += (usage.cacheReadInputTokens / 1e6) * p.input;
    total += (usage.cacheCreationInputTokens / 1e6) * p.input;
  }
  return total;
}

/** Detailed breakdown (cache at discounted rates). */
export function calculateCostBreakdown(stats: StatsCache) {
  let input = 0;
  let output = 0;
  let cacheRead = 0;
  let cacheCreate = 0;
  for (const [modelName, usage] of Object.entries(stats.modelUsage)) {
    const p = pricingForModel(modelName);
    input += (usage.inputTokens / 1e6) * p.input;
    output += (usage.outputTokens / 1e6) * p.output;
    cacheRead += (usage.cacheReadInputTokens / 1e6) * p.cacheRead;
    cacheCreate += (usage.cacheCreationInputTokens / 1e6) * p.cacheCreate;
  }
  return { input, output, cacheRead, cacheCreate };
}

export function calculateCacheEfficiency(stats: StatsCache) {
  let totalCacheRead = 0;
  let totalInput = 0;
  let savingsUsd = 0;
  const perModel: Record<string, number> = {};

  for (const [modelName, usage] of Object.entries(stats.modelUsage)) {
    const p = pricingForModel(modelName);
    const cr = usage.cacheReadInputTokens;
    const inp = usage.inputTokens;
    totalCacheRead += cr;
    totalInput += inp;
    const sum = cr + inp;
    if (sum > 0) {
      perModel[shortenModelName(modelName)] = (cr / sum) * 100;
    }
    savingsUsd += (cr / 1e6) * cacheSavingsPerMillion(p);
  }

  const overall = totalCacheRead + totalInput;
  const hitRatio = overall > 0 ? (totalCacheRead / overall) * 100 : 0;
  return { hitRatio, savingsUsd, perModel };
}

export function shortenModelName(name: string): string {
  if (name.includes("opus-4-6")) return "opus-4-6";
  if (name.includes("opus-4-5")) return "opus-4-5";
  if (name.includes("opus-4-1")) return "opus-4-1";
  if (name.includes("opus-4")) return "opus-4";
  if (name.includes("sonnet-4-5")) return "sonnet-4-5";
  if (name.includes("sonnet-4")) return "sonnet-4";
  if (name.includes("haiku-4-5")) return "haiku-4-5";
  if (name.includes("haiku-3-5") || name.includes("3-5-haiku")) return "haiku-3-5";
  if (name.includes("haiku")) return "haiku";
  return name.length > 12 ? name.slice(0, 12) : name;
}

export function modelTokenTotal(u: ModelUsage): number {
  return (
    u.inputTokens +
    u.outputTokens +
    u.cacheReadInputTokens +
    u.cacheCreationInputTokens
  );
}

export function webSearchTotal(stats: StatsCache): number {
  let t = 0;
  for (const u of Object.values(stats.modelUsage)) {
    t += u.webSearchRequests ?? 0;
  }
  return t;
}

export function averagesFromStats(stats: StatsCache) {
  const msgsPerSess =
    stats.totalSessions > 0 ? stats.totalMessages / stats.totalSessions : 0;
  let tokenSum = 0;
  for (const u of Object.values(stats.modelUsage)) {
    tokenSum +=
      u.inputTokens +
      u.outputTokens +
      u.cacheReadInputTokens +
      u.cacheCreationInputTokens;
  }
  const tokensPerMsg =
    stats.totalMessages > 0 ? tokenSum / stats.totalMessages : 0;
  return { msgsPerSess, tokensPerMsg };
}

export function monthlyProjection(stats: StatsCache): number {
  const recent = [...stats.dailyActivity].reverse().slice(0, 7);
  if (recent.length === 0) return 0;
  const totalMsgs = recent.reduce((s, d) => s + d.messageCount, 0);
  const avgPerDay = totalMsgs / recent.length;
  const totalCost = calculateTotalCost(stats);
  const costPerMsg =
    stats.totalMessages > 0 ? totalCost / stats.totalMessages : 0;
  return avgPerDay * costPerMsg * 30;
}

export function formatFirstSessionDate(dateStr: string): string {
  const iso = Date.parse(dateStr);
  if (!Number.isNaN(iso)) {
    return new Intl.DateTimeFormat("en", {
      month: "short",
      day: "numeric",
      year: "numeric",
    }).format(new Date(iso));
  }
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateStr);
  if (m) {
    const d = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]));
    return new Intl.DateTimeFormat("en", {
      month: "short",
      day: "numeric",
      year: "numeric",
    }).format(d);
  }
  return dateStr;
}
