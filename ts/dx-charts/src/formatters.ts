export function formatDuration(value: number): string {
  if (value < 0.1) return value.toFixed(4) + "ms";
  if (value < 1) return value.toFixed(2) + "ms";
  if (value < 1000) return Math.round(value) + "ms";
  if (value < 60000) return (value / 1000).toFixed(1) + "s";
  if (value < 3600000) {
    const m = Math.floor(value / 60000);
    const s = Math.floor((value % 60000) / 1000);
    return m + "m " + s + "s";
  }
  const h = Math.floor(value / 3600000);
  const mn = Math.floor((value % 3600000) / 60000);
  return h + "h " + mn + "m";
}

export function formatNumber(value: number): string {
  if (typeof value !== "number") return value as any;
  if (Math.abs(value) >= 1000)
    return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
  if (Math.abs(value) >= 1) return value.toFixed(2);
  if (Math.abs(value) >= 0.001) return value.toFixed(4);
  return value.toFixed(6);
}

export function formatFitness(value: number): string {
  if (Math.abs(value) >= 1000) return value.toFixed(0);
  return value.toFixed(2);
}

export function formatPercent(value: number): string {
  return value + "%";
}

export function formatMoney(value: number): string {
  if (typeof value !== "number" || isNaN(value)) return value as any;
  const abs = Math.abs(value);
  const sign = value < 0 ? "-" : "";
  if (abs >= 1e9) return sign + "$" + (abs / 1e9).toFixed(1) + "B";
  if (abs >= 1e6) return sign + "$" + (abs / 1e6).toFixed(1) + "M";
  if (abs >= 1e3) return sign + "$" + (abs / 1e3).toFixed(1) + "k";
  return sign + "$" + abs.toFixed(2);
}

export function formatMoneyFull(value: number): string {
  if (typeof value !== "number" || isNaN(value)) return value as any;
  return (
    (value < 0 ? "-$" : "$") +
    Math.abs(value).toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  );
}

export function abbreviateNumber(value: number): string {
  if (isNaN(value)) return value as any;
  if (Math.abs(value) >= 1e9) return (value / 1e9).toFixed(1) + "B";
  if (Math.abs(value) >= 1e6) return (value / 1e6).toFixed(1) + "M";
  if (Math.abs(value) >= 1e3) return (value / 1e3).toFixed(1) + "k";
  if (Math.abs(value) >= 100) return value.toFixed(0);
  return value.toFixed(1);
}

export function abbreviatePopulation(value: number): string {
  if (value >= 1e9) return (value / 1e9).toFixed(1) + "B";
  if (value >= 1e6) return (value / 1e6).toFixed(1) + "M";
  if (value >= 1e3) return (value / 1e3).toFixed(1) + "K";
  return "" + value;
}

export function formatIdentity(value: any): string {
  return "" + value;
}

export function formatFixed2(value: number): string {
  return value.toFixed(2);
}

export function formatFixed4(value: number): string {
  return value.toFixed(4);
}

export function formatRound(value: number): string {
  return "" + Math.round(value);
}

export function abbreviateCategory(value: any): string {
  const v = parseFloat(value);
  if (isNaN(v)) return value;
  return abbreviateNumber(v);
}

export function formatMegabytes(mb: number): string {
  if (typeof mb !== "number" || isNaN(mb)) return mb as any;
  const abs = Math.abs(mb);
  if (abs >= 1024) return (mb / 1024).toFixed(2) + " GB";
  if (abs >= 1) return mb.toFixed(1) + " MB";
  if (abs >= 1 / 1024) return (mb * 1024).toFixed(1) + " KB";
  return (mb * 1024 * 1024).toFixed(0) + " B";
}

export function alpsTooltipFormatter(
  params: any,
  popData: number[],
  fitnessData: string[]
): string {
  const idx = params[0].dataIndex;
  const pop = popData[idx];
  const pct = params[0].value.toFixed(1);
  const fit = fitnessData[idx];
  return (
    params[0].name +
    "<br/>Population: " +
    pop +
    " (" +
    pct +
    "%)<br/>Best Fitness: " +
    fit
  );
}

export function scatterTooltipFormatter(
  params: any,
  xLabel: string,
  yLabel: string
): string {
  const d = params.data.value || params.data;
  return (
    xLabel +
    ": " +
    d[0].toFixed(2) +
    "%<br/>" +
    yLabel +
    ": " +
    d[1].toFixed(2) +
    "%"
  );
}

export function cumulativeGrowthTooltip(params: any): string {
  const d = params[0];
  return "Period " + d.data[0] + "<br/>Growth: " + d.data[1].toFixed(4) + "x";
}

export function genTimeTooltip(params: any): string {
  const v = params[0].value;
  return "Gen " + params[0].name + "<br/>" + formatDuration(v);
}

export function memoryTooltip(params: any): string {
  if (!Array.isArray(params)) params = [params];
  const sorted = params
    .slice()
    .filter((p: any) => p.value != null && p.value !== 0);
  sorted.sort((a: any, b: any) => Math.abs(b.value) - Math.abs(a.value));
  const header = "Gen " + params[0].name;
  const lines = [header];
  for (const p of sorted) {
    lines.push(p.marker + " " + p.seriesName + ": " + formatMegabytes(p.value));
  }
  return lines.join("<br/>");
}
