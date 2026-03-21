"use strict";
var LxCharts;
(function (LxCharts) {
    var AXIS_FORMATTERS = {
        'identity': LxCharts.formatIdentity,
        'duration': LxCharts.formatDuration,
        'fitness': LxCharts.formatFitness,
        'percent': LxCharts.formatPercent,
        'money': LxCharts.formatMoney,
        'moneyFull': LxCharts.formatMoneyFull,
        'abbreviate': LxCharts.abbreviateNumber,
        'fixed2': LxCharts.formatFixed2,
        'fixed4': LxCharts.formatFixed4,
        'round': LxCharts.formatRound,
        'abbreviateCategory': LxCharts.abbreviateCategory,
        'megabytes': LxCharts.formatMegabytes
    };
    function applyAxisFormatter(axisOpt, fn) {
        if (!axisOpt)
            return;
        var arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
        for (var i = 0; i < arr.length; i++) {
            if (!arr[i].axisLabel)
                arr[i].axisLabel = {};
            arr[i].axisLabel.formatter = fn;
        }
    }
    function applyFormatters(opts, el) {
        var ds = el.dataset;
        if (ds.xFmt && AXIS_FORMATTERS[ds.xFmt]) {
            applyAxisFormatter(opts.xAxis, AXIS_FORMATTERS[ds.xFmt]);
        }
        if (ds.yFmt && AXIS_FORMATTERS[ds.yFmt]) {
            applyAxisFormatter(opts.yAxis, AXIS_FORMATTERS[ds.yFmt]);
        }
        var extra = ds.extra && ds.extra.length > 0 ? JSON.parse(ds.extra) : null;
        if (ds.tooltipFmt) {
            if (!opts.tooltip)
                opts.tooltip = {};
            switch (ds.tooltipFmt) {
                case 'cumulativeGrowth':
                    opts.tooltip.formatter = LxCharts.cumulativeGrowthTooltip;
                    break;
                case 'money':
                    opts.tooltip.valueFormatter = LxCharts.formatMoneyFull;
                    break;
                case 'genTime':
                    opts.tooltip.formatter = LxCharts.genTimeTooltip;
                    break;
                case 'scatter':
                    if (extra) {
                        opts.tooltip.formatter = function (params) {
                            return LxCharts.scatterTooltipFormatter(params, extra.xLabel, extra.yLabel);
                        };
                    }
                    break;
                case 'alps':
                    if (extra) {
                        opts.tooltip.formatter = function (params) {
                            return LxCharts.alpsTooltipFormatter(params, extra.pop, extra.fitness);
                        };
                    }
                    break;
                case 'megabytes':
                    opts.tooltip.formatter = LxCharts.memoryTooltip;
                    break;
            }
        }
        if (ds.labelFmt) {
            if (opts.series) {
                var sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
                for (var i = 0; i < sarr.length; i++) {
                    if (!sarr[i].label)
                        sarr[i].label = {};
                    switch (ds.labelFmt) {
                        case 'alpsPopulation':
                            if (extra) {
                                var popData = extra.pop;
                                sarr[i].label.formatter = function (params) {
                                    return LxCharts.abbreviatePopulation(popData[params.dataIndex]);
                                };
                            }
                            break;
                    }
                }
            }
        }
    }
    function themeAxis(axisOpt, fg, bc, sc) {
        if (!axisOpt)
            return;
        var arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
        for (var i = 0; i < arr.length; i++) {
            var ax = arr[i];
            if (!ax.axisLine)
                ax.axisLine = {};
            if (!ax.axisLine.lineStyle)
                ax.axisLine.lineStyle = {};
            if (!ax.axisLine.lineStyle.color)
                ax.axisLine.lineStyle.color = bc;
            if (!ax.splitLine)
                ax.splitLine = {};
            if (!ax.splitLine.lineStyle)
                ax.splitLine.lineStyle = {};
            if (!ax.splitLine.lineStyle.color)
                ax.splitLine.lineStyle.color = sc;
            if (ax.splitLine.show === undefined)
                ax.splitLine.show = true;
            if (!ax.axisTick)
                ax.axisTick = {};
            if (!ax.axisTick.lineStyle)
                ax.axisTick.lineStyle = {};
            if (!ax.axisTick.lineStyle.color)
                ax.axisTick.lineStyle.color = bc;
            if (!ax.axisLabel)
                ax.axisLabel = {};
            if (!ax.axisLabel.color)
                ax.axisLabel.color = fg;
            if (ax.axisLabel.textBorderWidth === undefined)
                ax.axisLabel.textBorderWidth = 0;
            if (!ax.nameTextStyle)
                ax.nameTextStyle = {};
            if (!ax.nameTextStyle.color)
                ax.nameTextStyle.color = fg;
            if (ax.nameTextStyle.textBorderWidth === undefined)
                ax.nameTextStyle.textBorderWidth = 0;
        }
    }
    function initChart(id, opts) {
        var el = document.getElementById(id);
        if (!el || typeof echarts === 'undefined')
            return;
        var cs = getComputedStyle(document.documentElement);
        var fg = cs.getPropertyValue('--foreground').trim() || '#e5e7eb';
        var bc = cs.getPropertyValue('--color-chart-axis').trim() || '#404040';
        var sc = cs.getPropertyValue('--color-chart-split').trim() || '#333333';
        var tbg = cs.getPropertyValue('--color-chart-tooltip').trim() || '#171717';
        var ds = el.dataset;
        if (ds.title && ds.title.length > 0 && !opts.title) {
            opts.title = { text: ds.title };
        }
        if (!opts.grid)
            opts.grid = {};
        if (opts.grid.containLabel === undefined)
            opts.grid.containLabel = true;
        if (!opts.textStyle)
            opts.textStyle = {};
        opts.textStyle.color = fg;
        themeAxis(opts.xAxis, fg, bc, sc);
        themeAxis(opts.yAxis, fg, bc, sc);
        if (opts.legend) {
            if (!opts.legend.textStyle)
                opts.legend.textStyle = {};
            if (!opts.legend.textStyle.color)
                opts.legend.textStyle.color = fg;
        }
        if (opts.series) {
            var sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
            for (var i = 0; i < sarr.length; i++) {
                var s = sarr[i];
                if (s.label) {
                    if (s.label.textBorderWidth === undefined)
                        s.label.textBorderWidth = 0;
                    if (!s.label.color)
                        s.label.color = fg;
                }
            }
        }
        if (!opts.tooltip)
            opts.tooltip = {};
        if (!opts.tooltip.backgroundColor)
            opts.tooltip.backgroundColor = tbg;
        if (!opts.tooltip.borderColor)
            opts.tooltip.borderColor = bc;
        if (!opts.tooltip.textStyle)
            opts.tooltip.textStyle = {};
        if (!opts.tooltip.textStyle.color)
            opts.tooltip.textStyle.color = fg;
        if (opts.title) {
            var t = Array.isArray(opts.title) ? opts.title[0] : opts.title;
            if (t) {
                if (!t.textStyle)
                    t.textStyle = {};
                if (!t.textStyle.color)
                    t.textStyle.color = fg;
            }
            var g = Array.isArray(opts.grid) ? opts.grid[0] : opts.grid;
            if (g && !g.top)
                g.top = 35;
        }
        applyFormatters(opts, el);
        if (!opts.tooltip.valueFormatter)
            opts.tooltip.valueFormatter = LxCharts.formatNumber;
        var instance = echarts.getInstanceByDom(el);
        if (instance) {
            instance.setOption(opts, true);
        }
        else {
            var chart = echarts.init(el);
            chart.setOption(opts);
            el._resize_observer = new ResizeObserver(function () { chart.resize(); });
            el._resize_observer.observe(el);
        }
    }
    LxCharts.initChart = initChart;
    function disposeChart(id) {
        var el = document.getElementById(id);
        if (el) {
            var instance = echarts.getInstanceByDom(el);
            if (instance)
                instance.dispose();
            if (el._resize_observer)
                el._resize_observer.disconnect();
        }
    }
    LxCharts.disposeChart = disposeChart;
    function restoreChart(id) {
        var el = document.getElementById(id);
        if (el) {
            var instance = echarts.getInstanceByDom(el);
            if (instance)
                instance.dispatchAction({ type: 'restore' });
        }
    }
    LxCharts.restoreChart = restoreChart;
})(LxCharts || (LxCharts = {}));
