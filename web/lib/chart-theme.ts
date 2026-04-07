import type { CSSProperties } from "react";

/**
 * Colori espliciti per Recharts (SVG): non usare hsl(var(--chart-*)) perché le variabili
 * shadcn sono oklch e in fill SVG spesso risultano invalide → barre nere.
 */
export const chartColors = {
  barDaily: "#22d3ee", // cyan-400
  barHourly: "#a78bfa", // violet-400
  grid: "rgba(255,255,255,0.08)",
  axis: "#a1a1aa", // zinc-400
  tooltipBg: "rgba(24, 24, 27, 0.96)",
  tooltipBorder: "#52525b",
} as const;

/** Palette modelli — contrasto alto su sfondo scuro */
export const modelBarPalette = [
  "#22d3ee",
  "#f472b6",
  "#34d399",
  "#fbbf24",
  "#a78bfa",
  "#fb923c",
] as const;

export function modelBarColor(i: number): string {
  return modelBarPalette[i % modelBarPalette.length];
}

export const tooltipContentStyle: CSSProperties = {
  backgroundColor: chartColors.tooltipBg,
  border: `1px solid ${chartColors.tooltipBorder}`,
  borderRadius: "8px",
  color: "#f4f4f5",
};
