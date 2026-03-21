declare var LxCharts: {
    initFlowGraph(id: string, data: unknown): void;
    updateFlowGraphStatus(id: string, nodeId: string, status: string): void;
    disposeChart(id: string): void;
};
declare var echarts: {
    getInstanceByDom(el: HTMLElement): { on(event: string, handler: (params: unknown) => void): void } | undefined;
};

import { registerWidget } from "./registry";
import type { Widget, Dioxus } from "./registry";

const flowGraphWidget: Widget = {
    mount(id: string, config: unknown, dx: Dioxus) {
        LxCharts.initFlowGraph(id, (config as any).graphData || {});
        var el = document.getElementById(id);
        if (el && typeof echarts !== 'undefined') {
            var inst = echarts.getInstanceByDom(el);
            if (inst) {
                inst.on('click', function(params: any) {
                    if (params.dataType === 'node') {
                        dx.send({
                            type: 'node-click',
                            nodeId: params.data.name,
                            sourceOffset: params.data.value ? params.data.value.sourceOffset : null
                        });
                    }
                });
            }
        }
    },
    update(id: string, data: unknown) {
        var d = data as any;
        if (d.type === 'node-status') {
            LxCharts.updateFlowGraphStatus(id, d.nodeId, d.status);
        } else if (d.type === 'full-update') {
            LxCharts.initFlowGraph(id, d.graphData);
        }
    },
    resize(id: string) {
        var el = document.getElementById(id);
        if (el && typeof echarts !== 'undefined') {
            var inst = echarts.getInstanceByDom(el);
            if (inst) (inst as any).resize();
        }
    },
    dispose(id: string) {
        LxCharts.disposeChart(id);
    }
};

registerWidget('flow-graph', flowGraphWidget);
