import type { HistoryEntry } from "./stats-types";

export function extractProjectName(path: string): string {
  const base = path.split(/[/\\]/).filter(Boolean).pop() ?? "Unknown";
  const aliases: Record<string, string> = {
    "massima-energia": "Massima Energia",
    "massima energia": "Massima Energia",
    "crm-wfa": "Massima Energia",
    "crm-wfa-fe": "Massima Energia",
    "Massima Energia": "Massima Energia",
  };
  return aliases[base] ?? base;
}

export type SessionRow = {
  sessionId: string;
  shortId: string;
  project: string;
  projectName: string;
  messageCount: number;
  lastTimestamp: number;
  isActive: boolean;
};

export function groupSessions(
  entries: HistoryEntry[],
  currentSessionId?: string | null,
): SessionRow[] {
  const map = new Map<
    string,
    Omit<SessionRow, "shortId" | "isActive"> & { firstTimestamp: number }
  >();

  for (const e of entries) {
    const projectName = extractProjectName(e.project);
    const cur = map.get(e.sessionId);
    if (!cur) {
      map.set(e.sessionId, {
        sessionId: e.sessionId,
        project: e.project,
        projectName,
        messageCount: 1,
        lastTimestamp: e.timestamp,
        firstTimestamp: e.timestamp,
      });
    } else {
      cur.messageCount += 1;
      cur.lastTimestamp = Math.max(cur.lastTimestamp, e.timestamp);
      cur.firstTimestamp = Math.min(cur.firstTimestamp, e.timestamp);
    }
  }

  const rows: SessionRow[] = [...map.values()].map((s) => ({
    sessionId: s.sessionId,
    shortId: s.sessionId.slice(0, 8),
    project: s.project,
    projectName: s.projectName,
    messageCount: s.messageCount,
    lastTimestamp: s.lastTimestamp,
    isActive: currentSessionId ? s.sessionId === currentSessionId : false,
  }));

  rows.sort((a, b) => b.lastTimestamp - a.lastTimestamp);
  return rows;
}

export function formatSessionTime(ts: number): string {
  const d = new Date(ts);
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(d);
}

export type ProjectTop = {
  name: string;
  path: string;
  messageCount: number;
  estimatedCost: number;
};

export function topProjects(
  entries: HistoryEntry[],
  topN: number,
  totalGlobalCost: number,
): ProjectTop[] {
  const totalHistoryMessages = entries.length;
  const counts = new Map<
    string,
    { path: string; count: number; lastTs: number }
  >();

  for (const e of entries) {
    const name = extractProjectName(e.project);
    if (name === "world_data" || name === "Current") continue;
    const cur = counts.get(name);
    if (!cur) {
      counts.set(name, { path: e.project, count: 1, lastTs: e.timestamp });
    } else {
      cur.count += 1;
      if (e.timestamp > cur.lastTs) cur.lastTs = e.timestamp;
    }
  }

  const list: ProjectTop[] = [...counts.entries()].map(([name, v]) => {
    const ratio =
      totalHistoryMessages > 0 ? v.count / totalHistoryMessages : 0;
    const estCost = totalGlobalCost * ratio;
    return {
      name,
      path: v.path,
      messageCount: v.count,
      estimatedCost: estCost,
    };
  });

  list.sort((a, b) => b.messageCount - a.messageCount);
  return list.slice(0, topN);
}
