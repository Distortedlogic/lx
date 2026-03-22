export function initFlowGraph(id: string, graphJson: any): void {
  const el = document.getElementById(id);
  if (!el || typeof echarts === "undefined") return;
  const instance = echarts.getInstanceByDom(el);
  if (instance) instance.dispose();
  const cs = getComputedStyle(document.documentElement);
  const fg = cs.getPropertyValue("--foreground").trim() || "#e5e7eb";
  const tbg =
    cs.getPropertyValue("--color-chart-tooltip").trim() || "#171717";
  const bc = cs.getPropertyValue("--color-chart-axis").trim() || "#404040";
  if (!graphJson.tooltip) graphJson.tooltip = {};
  if (!graphJson.tooltip.backgroundColor)
    graphJson.tooltip.backgroundColor = tbg;
  if (!graphJson.tooltip.borderColor) graphJson.tooltip.borderColor = bc;
  if (!graphJson.tooltip.textStyle) graphJson.tooltip.textStyle = {};
  if (!graphJson.tooltip.textStyle.color)
    graphJson.tooltip.textStyle.color = fg;
  if (!graphJson.textStyle) graphJson.textStyle = {};
  graphJson.textStyle.color = fg;
  const chart = echarts.init(el);
  chart.setOption(graphJson);
  (el as any)._resize_observer = new ResizeObserver(() => chart.resize());
  (el as any)._resize_observer.observe(el);
}

export function updateFlowGraphStatus(
  id: string,
  nodeId: string,
  status: string
): void {
  const el = document.getElementById(id);
  if (!el || typeof echarts === "undefined") return;
  const inst = echarts.getInstanceByDom(el);
  if (!inst) return;
  const opt = (inst as any).getOption();
  if (!opt?.series?.[0]?.data) return;
  const data = opt.series[0].data;
  for (const node of data) {
    if (node.name === nodeId) {
      let style: Record<string, any> = {};
      switch (status) {
        case "running":
          style = { borderColor: "#eab308", borderWidth: 3 };
          break;
        case "completed":
          style = { borderColor: "#22c55e", borderWidth: 2 };
          break;
        case "error":
          style = { borderColor: "#ef4444", borderWidth: 3 };
          break;
        case "active":
          style = { borderColor: "#facc15", borderWidth: 3 };
          break;
      }
      if (!node.itemStyle) node.itemStyle = {};
      Object.assign(node.itemStyle, style);
      break;
    }
  }
  inst.setOption({ series: [{ data }] }, false);
}
