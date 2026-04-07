export type DashboardOk = {
  ok: true;
  claudeDir: string;
  summary: {
    totalSessions: number;
    totalMessages: number;
    firstSessionDate: string | null;
    webSearches: number;
    totalCost: number;
    avgMsgsPerSession: number;
    avgTokensPerMessage: number;
  };
  costByCategory: {
    input: number;
    output: number;
    cacheRead: number;
    cacheCreate: number;
  };
  apiCostByPeriod: {
    label: string;
    amount: number;
    highlight?: boolean;
  }[];
  cache: {
    hitRatio: number;
    savingsUsd: number;
    perModel: [string, number][];
  };
  models: { name: string; tokens: number; pct: number }[];
  dailyActivity: { date: string; label: string; messages: number }[];
  hourly: { hour: number; count: number }[];
  monthlyProjection: number;
  sessions: {
    shortId: string;
    projectName: string;
    messages: number;
    lastAt: string;
    isActive: boolean;
  }[];
  topProjects: { name: string; messages: number; estimatedCost: number }[];
};

export type DashboardErr = {
  ok: false;
  error: string;
  claudeDir: string;
};

export type DashboardPayload = DashboardOk | DashboardErr;
