declare namespace echarts {
  function init(el: HTMLElement, theme?: string | null, opts?: { renderer?: string }): ECharts;
  function getInstanceByDom(el: HTMLElement): ECharts | undefined;
  namespace graphic {
    function clipRectByRect(targetRect: Rect, rect: Rect): Rect | false;
  }
  interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
  }
  interface ECharts {
    setOption(option: any, notMerge?: boolean): void;
    dispose(): void;
    resize(): void;
    dispatchAction(payload: { type: string }): void;
    on(eventName: string, handler: (params: any) => void): void;
    getOption(): any;
  }
}
interface RenderItemParams {
  dataIndex: number;
  seriesIndex: number;
  context: Record<string, unknown>;
  coordSys: { x: number; y: number; width: number; height: number };
}
interface RenderItemApi {
  value(dim: number): number;
  coord(data: [number, number]): [number, number];
  size(dataSize: [number, number]): [number, number];
  style(extra?: Record<string, unknown>): Record<string, unknown>;
}
interface RenderItemShape {
  x?: number;
  y?: number;
  x1?: number;
  y1?: number;
  x2?: number;
  y2?: number;
  width?: number;
  height?: number;
}
interface RenderItemElement {
  type: string;
  shape?: RenderItemShape;
  style?: Record<string, unknown>;
  children?: RenderItemElement[];
}
