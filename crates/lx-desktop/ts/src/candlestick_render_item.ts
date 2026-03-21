function renderItem(params: RenderItemParams, api: RenderItemApi): RenderItemElement {
  var xValue = api.value(0);
  var open = api.value(1);
  var close = api.value(2);
  var low = api.value(3);
  var high = api.value(4);
  var relVol = api.value(5);
  var openPt = api.coord([xValue, open]);
  var closePt = api.coord([xValue, close]);
  var lowPt = api.coord([xValue, low]);
  var highPt = api.coord([xValue, high]);
  var unitWidth = api.size([1, 0])[0];
  var halfWidth = unitWidth * (0.15 + 0.35 * relVol);
  var isUp = close >= open;
  var color = isUp ? '#22c55e' : '#ef4444';
  var bodyTop = Math.min(openPt[1], closePt[1]);
  var bodyHeight = Math.max(Math.abs(openPt[1] - closePt[1]), 1);
  return {
    type: 'group',
    children: [
      {
        type: 'line',
        shape: { x1: lowPt[0], y1: lowPt[1], x2: highPt[0], y2: highPt[1] },
        style: { stroke: color, lineWidth: 1 }
      },
      {
        type: 'rect',
        shape: { x: openPt[0] - halfWidth, y: bodyTop, width: halfWidth * 2, height: bodyHeight },
        style: { fill: color, stroke: color }
      }
    ]
  };
}
