export function candlestickRenderItem(
  params: RenderItemParams,
  api: RenderItemApi
): RenderItemElement {
  const xValue = api.value(0);
  const open = api.value(1);
  const close = api.value(2);
  const low = api.value(3);
  const high = api.value(4);
  const relVol = api.value(5);
  const openPt = api.coord([xValue, open]);
  const closePt = api.coord([xValue, close]);
  const lowPt = api.coord([xValue, low]);
  const highPt = api.coord([xValue, high]);
  const unitWidth = api.size([1, 0])[0];
  const halfWidth = unitWidth * (0.15 + 0.35 * relVol);
  const isUp = close >= open;
  const color = isUp ? "#22c55e" : "#ef4444";
  const bodyTop = Math.min(openPt[1], closePt[1]);
  const bodyHeight = Math.max(Math.abs(openPt[1] - closePt[1]), 1);
  return {
    type: "group",
    children: [
      {
        type: "line",
        shape: {
          x1: lowPt[0],
          y1: lowPt[1],
          x2: highPt[0],
          y2: highPt[1],
        },
        style: { stroke: color, lineWidth: 1 },
      },
      {
        type: "rect",
        shape: {
          x: openPt[0] - halfWidth,
          y: bodyTop,
          width: halfWidth * 2,
          height: bodyHeight,
        },
        style: { fill: color, stroke: color },
      },
    ],
  };
}
