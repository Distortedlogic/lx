import * as fmt from "./formatters";

const AXIS_FORMATTERS: Record<string, Function> = {
  identity: fmt.formatIdentity,
  duration: fmt.formatDuration,
  fitness: fmt.formatFitness,
  percent: fmt.formatPercent,
  money: fmt.formatMoney,
  moneyFull: fmt.formatMoneyFull,
  abbreviate: fmt.abbreviateNumber,
  fixed2: fmt.formatFixed2,
  fixed4: fmt.formatFixed4,
  round: fmt.formatRound,
  abbreviateCategory: fmt.abbreviateCategory,
  megabytes: fmt.formatMegabytes,
};

function applyAxisFormatter(axisOpt: any, fn: Function): void {
  if (!axisOpt) return;
  const arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
  for (const ax of arr) {
    if (!ax.axisLabel) ax.axisLabel = {};
    ax.axisLabel.formatter = fn;
  }
}

function applyFormatters(opts: any, el: HTMLElement): void {
  const ds = (el as any).dataset;
  if (ds.xFmt && AXIS_FORMATTERS[ds.xFmt]) {
    applyAxisFormatter(opts.xAxis, AXIS_FORMATTERS[ds.xFmt]);
  }
  if (ds.yFmt && AXIS_FORMATTERS[ds.yFmt]) {
    applyAxisFormatter(opts.yAxis, AXIS_FORMATTERS[ds.yFmt]);
  }
  const extra: any =
    ds.extra && ds.extra.length > 0 ? JSON.parse(ds.extra) : null;
  if (ds.tooltipFmt) {
    if (!opts.tooltip) opts.tooltip = {};
    switch (ds.tooltipFmt) {
      case "cumulativeGrowth":
        opts.tooltip.formatter = fmt.cumulativeGrowthTooltip;
        break;
      case "money":
        opts.tooltip.valueFormatter = fmt.formatMoneyFull;
        break;
      case "genTime":
        opts.tooltip.formatter = fmt.genTimeTooltip;
        break;
      case "scatter":
        if (extra) {
          opts.tooltip.formatter = (params: any) =>
            fmt.scatterTooltipFormatter(params, extra.xLabel, extra.yLabel);
        }
        break;
      case "alps":
        if (extra) {
          opts.tooltip.formatter = (params: any) =>
            fmt.alpsTooltipFormatter(params, extra.pop, extra.fitness);
        }
        break;
      case "megabytes":
        opts.tooltip.formatter = fmt.memoryTooltip;
        break;
    }
  }
  if (ds.labelFmt && opts.series) {
    const sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
    for (const s of sarr) {
      if (!s.label) s.label = {};
      if (ds.labelFmt === "alpsPopulation" && extra) {
        const popData = extra.pop;
        s.label.formatter = (params: any) =>
          fmt.abbreviatePopulation(popData[params.dataIndex]);
      }
    }
  }
}

function themeAxis(
  axisOpt: any,
  fg: string,
  bc: string,
  sc: string
): void {
  if (!axisOpt) return;
  const arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
  for (const ax of arr) {
    if (!ax.axisLine) ax.axisLine = {};
    if (!ax.axisLine.lineStyle) ax.axisLine.lineStyle = {};
    if (!ax.axisLine.lineStyle.color) ax.axisLine.lineStyle.color = bc;
    if (!ax.splitLine) ax.splitLine = {};
    if (!ax.splitLine.lineStyle) ax.splitLine.lineStyle = {};
    if (!ax.splitLine.lineStyle.color) ax.splitLine.lineStyle.color = sc;
    if (ax.splitLine.show === undefined) ax.splitLine.show = true;
    if (!ax.axisTick) ax.axisTick = {};
    if (!ax.axisTick.lineStyle) ax.axisTick.lineStyle = {};
    if (!ax.axisTick.lineStyle.color) ax.axisTick.lineStyle.color = bc;
    if (!ax.axisLabel) ax.axisLabel = {};
    if (!ax.axisLabel.color) ax.axisLabel.color = fg;
    if (ax.axisLabel.textBorderWidth === undefined)
      ax.axisLabel.textBorderWidth = 0;
    if (!ax.nameTextStyle) ax.nameTextStyle = {};
    if (!ax.nameTextStyle.color) ax.nameTextStyle.color = fg;
    if (ax.nameTextStyle.textBorderWidth === undefined)
      ax.nameTextStyle.textBorderWidth = 0;
  }
}

export function initChart(id: string, opts: any): void {
  const el = document.getElementById(id);
  if (!el || typeof echarts === "undefined") return;
  const cs = getComputedStyle(document.documentElement);
  const fg = cs.getPropertyValue("--foreground").trim() || "#e5e7eb";
  const bc = cs.getPropertyValue("--color-chart-axis").trim() || "#404040";
  const sc = cs.getPropertyValue("--color-chart-split").trim() || "#333333";
  const tbg = cs.getPropertyValue("--color-chart-tooltip").trim() || "#171717";
  const ds = (el as any).dataset;
  if (ds.title && ds.title.length > 0 && !opts.title) {
    opts.title = { text: ds.title };
  }
  if (!opts.grid) opts.grid = {};
  if (opts.grid.containLabel === undefined) opts.grid.containLabel = true;
  if (!opts.textStyle) opts.textStyle = {};
  opts.textStyle.color = fg;
  themeAxis(opts.xAxis, fg, bc, sc);
  themeAxis(opts.yAxis, fg, bc, sc);
  if (opts.legend) {
    if (!opts.legend.textStyle) opts.legend.textStyle = {};
    if (!opts.legend.textStyle.color) opts.legend.textStyle.color = fg;
  }
  if (opts.series) {
    const sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
    for (const s of sarr) {
      if (s.label) {
        if (s.label.textBorderWidth === undefined) s.label.textBorderWidth = 0;
        if (!s.label.color) s.label.color = fg;
      }
    }
  }
  if (!opts.tooltip) opts.tooltip = {};
  if (!opts.tooltip.backgroundColor) opts.tooltip.backgroundColor = tbg;
  if (!opts.tooltip.borderColor) opts.tooltip.borderColor = bc;
  if (!opts.tooltip.textStyle) opts.tooltip.textStyle = {};
  if (!opts.tooltip.textStyle.color) opts.tooltip.textStyle.color = fg;
  if (opts.title) {
    const t = Array.isArray(opts.title) ? opts.title[0] : opts.title;
    if (t) {
      if (!t.textStyle) t.textStyle = {};
      if (!t.textStyle.color) t.textStyle.color = fg;
    }
    const g = Array.isArray(opts.grid) ? opts.grid[0] : opts.grid;
    if (g && !g.top) g.top = 35;
  }
  applyFormatters(opts, el);
  if (!opts.tooltip.valueFormatter)
    opts.tooltip.valueFormatter = fmt.formatNumber;
  const instance = echarts.getInstanceByDom(el);
  if (instance) {
    instance.setOption(opts, true);
  } else {
    const chart = echarts.init(el);
    chart.setOption(opts);
    (el as any)._resize_observer = new ResizeObserver(() => chart.resize());
    (el as any)._resize_observer.observe(el);
  }
}

export function disposeChart(id: string): void {
  const el = document.getElementById(id);
  if (el) {
    const instance = echarts.getInstanceByDom(el);
    if (instance) instance.dispose();
    if ((el as any)._resize_observer) (el as any)._resize_observer.disconnect();
  }
}

export function restoreChart(id: string): void {
  const el = document.getElementById(id);
  if (el) {
    const instance = echarts.getInstanceByDom(el);
    if (instance) instance.dispatchAction({ type: "restore" });
  }
}
