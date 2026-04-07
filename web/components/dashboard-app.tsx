"use client";

import { useCallback, useState } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { DashboardPayload } from "@/lib/dashboard-types";
import {
  chartColors,
  modelBarColor,
  tooltipContentStyle,
} from "@/lib/chart-theme";
import { fmtNum, fmtUsd } from "@/lib/format";
import { Activity, BarChart3, FolderGit2, RefreshCw } from "lucide-react";

type Props = { initial: DashboardPayload };

export function DashboardApp({ initial }: Props) {
  const [data, setData] = useState(initial);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const r = await fetch("/api/dashboard", { cache: "no-store" });
      setData(await r.json());
    } finally {
      setLoading(false);
    }
  }, []);

  if (!data.ok) {
    return (
      <div className="min-h-screen bg-background p-6 md:p-10">
        <Card className="mx-auto max-w-2xl border-destructive/50">
          <CardHeader>
            <CardTitle>Impossibile caricare i dati</CardTitle>
            <CardDescription>{data.error}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            <p>
              Directory attesa:{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-foreground">
                {data.claudeDir}
              </code>
            </p>
            <p>
              Imposta{" "}
              <code className="rounded bg-muted px-1.5 py-0.5">CLAUDE_DIR</code>{" "}
              se Claude Code usa un&apos;altra cartella.
            </p>
            <Button variant="outline" size="sm" onClick={refresh}>
              <RefreshCw className="mr-2 size-4" />
              Riprova
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const d = data;

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b border-border/80 bg-card/40 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-col gap-4 px-4 py-6 sm:flex-row sm:items-center sm:justify-between md:px-8">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
              Claude Code Monitor
            </h1>
            <p className="mt-1 max-w-xl text-sm text-muted-foreground">
              Dashboard web (stessi file di{" "}
              <code className="text-xs">stats-cache.json</code> e{" "}
              <code className="text-xs">history.jsonl</code>). La quota OAuth
              resta nella CLI.
            </p>
            <p className="mt-2 truncate text-xs text-muted-foreground/80">
              {d.claudeDir}
            </p>
          </div>
          <Button onClick={refresh} disabled={loading} variant="secondary">
            <RefreshCw
              className={`mr-2 size-4 ${loading ? "animate-spin" : ""}`}
            />
            Aggiorna
          </Button>
        </div>
      </header>

      <main className="mx-auto max-w-7xl space-y-8 px-4 py-8 md:px-8">
        <Tabs defaultValue="dashboard" className="w-full">
          <TabsList className="grid w-full max-w-md grid-cols-3">
            <TabsTrigger value="dashboard" className="gap-1.5">
              <Activity className="size-3.5 opacity-70" />
              Dashboard
            </TabsTrigger>
            <TabsTrigger value="projects" className="gap-1.5">
              <FolderGit2 className="size-3.5 opacity-70" />
              Projects
            </TabsTrigger>
            <TabsTrigger value="models" className="gap-1.5">
              <BarChart3 className="size-3.5 opacity-70" />
              Models
            </TabsTrigger>
          </TabsList>

          <TabsContent value="dashboard" className="mt-6 space-y-6">
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              <StatCard
                title="Sessioni"
                value={fmtNum(d.summary.totalSessions)}
              />
              <StatCard
                title="Messaggi"
                value={fmtNum(d.summary.totalMessages)}
              />
              <StatCard
                title="Dal"
                value={d.summary.firstSessionDate ?? "—"}
                small
              />
              <StatCard
                title="Costo stim. (worst-case)"
                value={fmtUsd(d.summary.totalCost)}
                accent
              />
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Token per modello</CardTitle>
                  <CardDescription>Distribuzione su tutti i token</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {d.models.map((m) => (
                    <div key={m.name} className="space-y-1.5">
                      <div className="flex justify-between text-sm">
                        <span className="font-medium">{m.name}</span>
                        <span className="text-muted-foreground">
                          {m.pct}% · {fmtNum(m.tokens)}
                        </span>
                      </div>
                      <Progress
                        value={Math.min(100, m.pct)}
                        className="h-2 bg-muted"
                      />
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Costi API (stime)</CardTitle>
                  <CardDescription>
                    Periodi basati su attività giornaliera
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-3">
                    {d.apiCostByPeriod.map((row) => (
                      <li
                        key={row.label}
                        className="flex items-center justify-between text-sm"
                      >
                        <span
                          className={
                            row.highlight ? "font-medium text-primary" : ""
                          }
                        >
                          {row.label}
                        </span>
                        <span className="tabular-nums text-muted-foreground">
                          {fmtUsd(row.amount)}
                        </span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 lg:grid-cols-3">
              <Card className="lg:col-span-1">
                <CardHeader>
                  <CardTitle>Cache efficiency</CardTitle>
                  <CardDescription>
                    Risparmio stimato vs input intero
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-2">
                  <p className="text-3xl font-semibold tabular-nums">
                    {d.cache.hitRatio.toFixed(1)}%
                  </p>
                  <p className="text-sm text-muted-foreground">
                    Risparmio:{" "}
                    <span className="font-medium text-emerald-400">
                      {fmtUsd(d.cache.savingsUsd)}
                    </span>
                  </p>
                  <Separator className="my-3" />
                  <ul className="space-y-1.5 text-sm">
                    {d.cache.perModel.slice(0, 4).map(([name, pct]) => (
                      <li key={name} className="flex justify-between">
                        <span className="text-muted-foreground">{name}</span>
                        <span className="tabular-nums">{pct.toFixed(0)}%</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>

              <Card className="lg:col-span-2">
                <CardHeader>
                  <CardTitle>Attività ultimi 14 giorni</CardTitle>
                  <CardDescription>Messaggi per giorno</CardDescription>
                </CardHeader>
                <CardContent className="h-56 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={d.dailyActivity}>
                      <CartesianGrid
                        strokeDasharray="3 3"
                        stroke={chartColors.grid}
                        vertical={false}
                      />
                      <XAxis
                        dataKey="label"
                        tick={{ fontSize: 11, fill: chartColors.axis }}
                        tickLine={false}
                        axisLine={{ stroke: chartColors.grid }}
                      />
                      <YAxis
                        tick={{ fontSize: 11, fill: chartColors.axis }}
                        width={36}
                        tickLine={false}
                        axisLine={{ stroke: chartColors.grid }}
                      />
                      <Tooltip
                        contentStyle={tooltipContentStyle}
                        labelFormatter={(_, p) =>
                          (p[0]?.payload as { date?: string })?.date ?? ""
                        }
                      />
                      <Bar
                        dataKey="messages"
                        radius={[4, 4, 0, 0]}
                        fill={chartColors.barDaily}
                      />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Distribuzione oraria</CardTitle>
                  <CardDescription>Messaggi per ora del giorno</CardDescription>
                </CardHeader>
                <CardContent className="h-64 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={d.hourly}>
                      <CartesianGrid
                        strokeDasharray="3 3"
                        stroke={chartColors.grid}
                        vertical={false}
                      />
                      <XAxis
                        dataKey="hour"
                        tick={{ fontSize: 10, fill: chartColors.axis }}
                        tickFormatter={(h) => `${h}h`}
                        tickLine={false}
                        axisLine={{ stroke: chartColors.grid }}
                      />
                      <YAxis
                        width={40}
                        tick={{ fontSize: 10, fill: chartColors.axis }}
                        tickLine={false}
                        axisLine={{ stroke: chartColors.grid }}
                      />
                      <Tooltip contentStyle={tooltipContentStyle} />
                      <Bar
                        dataKey="count"
                        radius={[2, 2, 0, 0]}
                        fill={chartColors.barHourly}
                      />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Breakdown costo (all-time)</CardTitle>
                  <CardDescription>
                    Input / output / cache (tariffe scontate)
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2 text-sm">
                    <Row label="Input" v={d.costByCategory.input} />
                    <Row label="Output" v={d.costByCategory.output} />
                    <Row label="Cache read" v={d.costByCategory.cacheRead} />
                    <Row label="Cache create" v={d.costByCategory.cacheCreate} />
                  </ul>
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>Sessioni recenti</CardTitle>
                  <CardDescription>Da history.jsonl</CardDescription>
                </CardHeader>
                <CardContent>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>ID</TableHead>
                        <TableHead>Progetto</TableHead>
                        <TableHead className="text-right">Msg</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {d.sessions.slice(0, 12).map((s) => (
                        <TableRow key={s.shortId + s.lastAt}>
                          <TableCell className="font-mono text-xs">
                            {s.shortId}
                            {s.isActive ? (
                              <Badge className="ml-2" variant="default">
                                Active
                              </Badge>
                            ) : null}
                          </TableCell>
                          <TableCell className="max-w-[140px] truncate text-muted-foreground">
                            {s.projectName}
                          </TableCell>
                          <TableCell className="text-right tabular-nums">
                            {fmtNum(s.messages)}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Top progetti</CardTitle>
                  <CardDescription>Per numero messaggi in history</CardDescription>
                </CardHeader>
                <CardContent>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Progetto</TableHead>
                        <TableHead className="text-right">Msg</TableHead>
                        <TableHead className="text-right">Costo stima</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {d.topProjects.map((p) => (
                        <TableRow key={p.name}>
                          <TableCell className="max-w-[160px] truncate font-medium">
                            {p.name}
                          </TableCell>
                          <TableCell className="text-right tabular-nums">
                            {fmtNum(p.messages)}
                          </TableCell>
                          <TableCell className="text-right tabular-nums text-muted-foreground">
                            {fmtUsd(p.estimatedCost)}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="projects" className="mt-6">
            <Card>
              <CardHeader>
                <CardTitle>Progetti</CardTitle>
                <CardDescription>
                  Ordinati per messaggi nella history
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>#</TableHead>
                      <TableHead>Nome</TableHead>
                      <TableHead className="text-right">Messaggi</TableHead>
                      <TableHead className="text-right">Costo stimato</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {d.topProjects.map((p, i) => (
                      <TableRow key={p.name}>
                        <TableCell className="text-muted-foreground">
                          {i + 1}
                        </TableCell>
                        <TableCell className="font-medium">{p.name}</TableCell>
                        <TableCell className="text-right tabular-nums">
                          {fmtNum(p.messages)}
                        </TableCell>
                        <TableCell className="text-right tabular-nums">
                          {fmtUsd(p.estimatedCost)}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="models" className="mt-6 space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Utilizzo token</CardTitle>
                <CardDescription>Modelli in stats-cache</CardDescription>
              </CardHeader>
              <CardContent className="h-80 w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart
                    layout="vertical"
                    data={d.models}
                    margin={{ left: 8, right: 16 }}
                  >
                    <CartesianGrid
                      strokeDasharray="3 3"
                      stroke={chartColors.grid}
                      horizontal={false}
                    />
                    <XAxis
                      type="number"
                      tick={{ fontSize: 11, fill: chartColors.axis }}
                      tickLine={false}
                      axisLine={{ stroke: chartColors.grid }}
                    />
                    <YAxis
                      type="category"
                      dataKey="name"
                      width={88}
                      tick={{ fontSize: 11, fill: chartColors.axis }}
                      tickLine={false}
                      axisLine={{ stroke: chartColors.grid }}
                    />
                    <Tooltip contentStyle={tooltipContentStyle} />
                    <Bar dataKey="tokens" radius={[0, 4, 4, 0]}>
                      {d.models.map((_, i) => (
                        <Cell key={i} fill={modelBarColor(i)} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}

function StatCard({
  title,
  value,
  small,
  accent,
}: {
  title: string;
  value: string;
  small?: boolean;
  accent?: boolean;
}) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardDescription>{title}</CardDescription>
        <CardTitle
          className={`${small ? "text-lg" : "text-2xl"} ${accent ? "text-primary" : ""} font-semibold tabular-nums`}
        >
          {value}
        </CardTitle>
      </CardHeader>
    </Card>
  );
}

function Row({ label, v }: { label: string; v: number }) {
  return (
    <li className="flex justify-between">
      <span className="text-muted-foreground">{label}</span>
      <span className="tabular-nums">{fmtUsd(v)}</span>
    </li>
  );
}
