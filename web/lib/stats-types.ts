/** Mirrors Rust `StatsCache` / related structs (camelCase JSON). */

export type ModelUsage = {
  inputTokens: number;
  outputTokens: number;
  cacheReadInputTokens: number;
  cacheCreationInputTokens: number;
  webSearchRequests?: number;
  costUsd?: number;
  contextWindow?: number;
  maxOutputTokens?: number;
};

export type DailyActivity = {
  date: string;
  messageCount: number;
  sessionCount: number;
  toolCallCount: number;
};

export type StatsCache = {
  version: number;
  lastComputedDate: string;
  dailyActivity: DailyActivity[];
  dailyModelTokens: { date: string; tokensByModel: Record<string, number> }[];
  modelUsage: Record<string, ModelUsage>;
  totalSessions: number;
  totalMessages: number;
  longestSession?: {
    sessionId: string;
    duration: number;
    messageCount: number;
    timestamp: string;
  };
  firstSessionDate?: string;
  hourCounts: Record<string, number>;
  totalSpeculationTimeSavedMs?: number;
};

export type HistoryEntry = {
  display: string;
  pastedContents?: Record<string, string>;
  timestamp: number;
  project: string;
  sessionId: string;
};
