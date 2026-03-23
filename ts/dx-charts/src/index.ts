export {
  initChart,
  disposeChart,
  restoreChart,
} from "./chart_init";

export {
  initFlowGraph,
  updateFlowGraphStatus,
} from "./flow_graph";

export { setupFlamegraph } from "./flamegraph";
export { candlestickRenderItem } from "./candlestick";

export {
  formatDuration,
  formatNumber,
  formatFitness,
  formatPercent,
  formatMoney,
  formatMoneyFull,
  abbreviateNumber,
  abbreviatePopulation,
  formatIdentity,
  formatFixed2,
  formatFixed4,
  formatRound,
  abbreviateCategory,
  formatMegabytes,
  alpsTooltipFormatter,
  scatterTooltipFormatter,
  cumulativeGrowthTooltip,
  genTimeTooltip,
  memoryTooltip,
} from "./formatters";

import * as self from "./index";

declare global {
  interface Window {
    DxCharts: typeof self;
  }
}

window.DxCharts = self;
