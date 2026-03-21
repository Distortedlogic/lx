namespace LxCharts {
    export function initFlowGraph(id: string, graphJson: any): void {
        var el = document.getElementById(id);
        if (!el || typeof echarts === 'undefined') return;
        var instance = echarts.getInstanceByDom(el);
        if (instance) instance.dispose();
        var cs = getComputedStyle(document.documentElement);
        var fg = cs.getPropertyValue('--foreground').trim() || '#e5e7eb';
        var tbg = cs.getPropertyValue('--color-chart-tooltip').trim() || '#171717';
        var bc = cs.getPropertyValue('--color-chart-axis').trim() || '#404040';
        if (!graphJson.tooltip) graphJson.tooltip = {};
        if (!graphJson.tooltip.backgroundColor) graphJson.tooltip.backgroundColor = tbg;
        if (!graphJson.tooltip.borderColor) graphJson.tooltip.borderColor = bc;
        if (!graphJson.tooltip.textStyle) graphJson.tooltip.textStyle = {};
        if (!graphJson.tooltip.textStyle.color) graphJson.tooltip.textStyle.color = fg;
        if (!graphJson.textStyle) graphJson.textStyle = {};
        graphJson.textStyle.color = fg;
        var chart = echarts.init(el);
        chart.setOption(graphJson);
        (el as any)._resize_observer = new ResizeObserver(function() { chart.resize(); });
        (el as any)._resize_observer.observe(el);
    }
    export function updateFlowGraphStatus(id: string, nodeId: string, status: string): void {
        var el = document.getElementById(id);
        if (!el || typeof echarts === 'undefined') return;
        var inst = echarts.getInstanceByDom(el);
        if (!inst) return;
        var opt = (inst as any).getOption();
        if (!opt || !opt.series || !opt.series[0] || !opt.series[0].data) return;
        var data = opt.series[0].data;
        for (var i = 0; i < data.length; i++) {
            if (data[i].name === nodeId) {
                var style: Record<string, any> = {};
                switch (status) {
                    case 'running':
                        style = { borderColor: '#eab308', borderWidth: 3 };
                        break;
                    case 'completed':
                        style = { borderColor: '#22c55e', borderWidth: 2 };
                        break;
                    case 'error':
                        style = { borderColor: '#ef4444', borderWidth: 3 };
                        break;
                    case 'active':
                        style = { borderColor: '#facc15', borderWidth: 3 };
                        break;
                    default:
                        style = {};
                }
                if (!data[i].itemStyle) data[i].itemStyle = {};
                for (var k in style) {
                    data[i].itemStyle[k] = style[k];
                }
                break;
            }
        }
        inst.setOption({ series: [{ data: data }] }, false);
    }
}
