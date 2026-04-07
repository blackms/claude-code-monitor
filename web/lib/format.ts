export const usd = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export const intl = new Intl.NumberFormat("en-US");

export function fmtUsd(n: number): string {
  return usd.format(n);
}

export function fmtNum(n: number): string {
  return intl.format(Math.round(n));
}
