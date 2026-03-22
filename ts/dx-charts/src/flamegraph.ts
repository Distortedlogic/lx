export function setupFlamegraph(
  id: string,
  data: any,
  maxY: number
): void {
  const el = document.getElementById(id);
  if (!el || typeof echarts === "undefined") return;
  const instance = echarts.getInstanceByDom(el);
  if (instance) instance.dispose();
  const cs = getComputedStyle(document.documentElement);
  const fg = cs.getPropertyValue("--foreground").trim() || "#fafafa";
  const bg = cs.getPropertyValue("--background").trim() || "#171717";
  const bd = cs.getPropertyValue("--border").trim() || "#404040";
  const chart = echarts.init(el);
  chart.setOption({
    tooltip: {
      trigger: "item",
      backgroundColor: bg,
      borderColor: bd,
      textStyle: { color: fg },
      formatter: (params: any) => params.value[3],
    },
    grid: { left: 60, right: 20, top: 20, bottom: 40 },
    xAxis: { type: "value", name: "ms", axisLabel: { color: fg } },
    yAxis: {
      type: "value",
      min: 0,
      max: maxY,
      inverse: true,
      axisLabel: { color: fg },
    },
    series: [
      {
        type: "custom",
        renderItem: (params: RenderItemParams, api: RenderItemApi) => {
          const xStart = api.value(0);
          const depth = api.value(1);
          const xEnd = api.value(2);
          const label = api.value(3);
          const color = api.value(4);
          const start = api.coord([xStart, depth]);
          const end = api.coord([xEnd, depth + 0.9]);
          const rectShape = echarts.graphic.clipRectByRect(
            {
              x: start[0],
              y: start[1],
              width: end[0] - start[0],
              height: end[1] - start[1],
            },
            {
              x: params.coordSys.x,
              y: params.coordSys.y,
              width: params.coordSys.width,
              height: params.coordSys.height,
            }
          );
          return (
            rectShape && {
              type: "rect",
              shape: rectShape,
              style: api.style({
                fill: color,
                stroke: "#222",
                text: label,
                textFill: "#fff",
                fontSize: 11,
                truncate: {
                  outerWidth: (rectShape as echarts.Rect).width,
                  outerHeight: (rectShape as echarts.Rect).height,
                },
              }),
            }
          );
        },
        encode: { x: [0, 2], y: 1 },
        data,
      },
    ],
  });
  (el as any)._resize_observer = new ResizeObserver(() => chart.resize());
  (el as any)._resize_observer.observe(el);
}
