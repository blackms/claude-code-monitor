import "server-only";
import fs from "fs/promises";
import path from "path";
import os from "os";
import type { StatsCache, HistoryEntry } from "./stats-types";
import type { DashboardPayload } from "./dashboard-types";
import { buildDashboard } from "./dashboard-builder";

function resolveClaudeDir(): string {
  const env = process.env.CLAUDE_DIR?.trim();
  if (env) return path.resolve(env.replace(/^~/, os.homedir()));
  return path.join(os.homedir(), ".claude");
}

async function readJsonFile<T>(file: string): Promise<T> {
  const raw = await fs.readFile(file, "utf-8");
  return JSON.parse(raw) as T;
}

async function readHistoryJsonl(file: string): Promise<HistoryEntry[]> {
  let raw: string;
  try {
    raw = await fs.readFile(file, "utf-8");
  } catch {
    return [];
  }
  const entries: HistoryEntry[] = [];
  for (const line of raw.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    try {
      const o = JSON.parse(t) as HistoryEntry;
      if (o.sessionId && typeof o.timestamp === "number") entries.push(o);
    } catch {
      /* skip */
    }
  }
  return entries;
}

export async function loadDashboardPayload(): Promise<DashboardPayload> {
  const claudeDir = resolveClaudeDir();
  const statsPath = path.join(claudeDir, "stats-cache.json");
  const historyPath = path.join(claudeDir, "history.jsonl");

  let stats: StatsCache;
  try {
    stats = await readJsonFile<StatsCache>(statsPath);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return {
      ok: false,
      claudeDir,
      error: `Cannot read stats-cache.json (${statsPath}): ${msg}`,
    };
  }

  let history: HistoryEntry[] = [];
  try {
    history = await readHistoryJsonl(historyPath);
  } catch {
    history = [];
  }

  return buildDashboard(claudeDir, stats, history);
}
