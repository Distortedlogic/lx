"use strict";
var LxCharts;
(function (LxCharts) {
    function formatDuration(value) {
        if (value < 0.1)
            return value.toFixed(4) + 'ms';
        if (value < 1)
            return value.toFixed(2) + 'ms';
        if (value < 1000)
            return Math.round(value) + 'ms';
        if (value < 60000)
            return (value / 1000).toFixed(1) + 's';
        if (value < 3600000) {
            var m = Math.floor(value / 60000);
            var s = Math.floor((value % 60000) / 1000);
            return m + 'm ' + s + 's';
        }
        var h = Math.floor(value / 3600000);
        var mn = Math.floor((value % 3600000) / 60000);
        return h + 'h ' + mn + 'm';
    }
    LxCharts.formatDuration = formatDuration;
    function formatNumber(value) {
        if (typeof value !== 'number')
            return value;
        if (Math.abs(value) >= 1000)
            return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
        if (Math.abs(value) >= 1)
            return value.toFixed(2);
        if (Math.abs(value) >= 0.001)
            return value.toFixed(4);
        return value.toFixed(6);
    }
    LxCharts.formatNumber = formatNumber;
    function formatFitness(value) {
        if (Math.abs(value) >= 1000)
            return value.toFixed(0);
        return value.toFixed(2);
    }
    LxCharts.formatFitness = formatFitness;
    function formatPercent(value) {
        return value + '%';
    }
    LxCharts.formatPercent = formatPercent;
    function formatMoney(value) {
        if (typeof value !== 'number' || isNaN(value))
            return value;
        var abs = Math.abs(value);
        var sign = value < 0 ? '-' : '';
        if (abs >= 1e9)
            return sign + '$' + (abs / 1e9).toFixed(1) + 'B';
        if (abs >= 1e6)
            return sign + '$' + (abs / 1e6).toFixed(1) + 'M';
        if (abs >= 1e3)
            return sign + '$' + (abs / 1e3).toFixed(1) + 'k';
        return sign + '$' + abs.toFixed(2);
    }
    LxCharts.formatMoney = formatMoney;
    function formatMoneyFull(value) {
        if (typeof value !== 'number' || isNaN(value))
            return value;
        return (value < 0 ? '-$' : '$') + Math.abs(value).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    }
    LxCharts.formatMoneyFull = formatMoneyFull;
    function abbreviateNumber(value) {
        if (isNaN(value))
            return value;
        if (Math.abs(value) >= 1e9)
            return (value / 1e9).toFixed(1) + 'B';
        if (Math.abs(value) >= 1e6)
            return (value / 1e6).toFixed(1) + 'M';
        if (Math.abs(value) >= 1e3)
            return (value / 1e3).toFixed(1) + 'k';
        if (Math.abs(value) >= 100)
            return value.toFixed(0);
        return value.toFixed(1);
    }
    LxCharts.abbreviateNumber = abbreviateNumber;
    function abbreviatePopulation(value) {
        if (value >= 1e9)
            return (value / 1e9).toFixed(1) + 'B';
        if (value >= 1e6)
            return (value / 1e6).toFixed(1) + 'M';
        if (value >= 1e3)
            return (value / 1e3).toFixed(1) + 'K';
        return '' + value;
    }
    LxCharts.abbreviatePopulation = abbreviatePopulation;
    function alpsTooltipFormatter(params, popData, fitnessData) {
        var idx = params[0].dataIndex;
        var pop = popData[idx];
        var pct = params[0].value.toFixed(1);
        var fit = fitnessData[idx];
        return params[0].name + '<br/>' + 'Population: ' + pop + ' (' + pct + '%)' + '<br/>' + 'Best Fitness: ' + fit;
    }
    LxCharts.alpsTooltipFormatter = alpsTooltipFormatter;
    function scatterTooltipFormatter(params, xLabel, yLabel) {
        var d = params.data.value || params.data;
        return xLabel + ': ' + d[0].toFixed(2) + '%' + '<br/>' + yLabel + ': ' + d[1].toFixed(2) + '%';
    }
    LxCharts.scatterTooltipFormatter = scatterTooltipFormatter;
    function formatIdentity(value) {
        return '' + value;
    }
    LxCharts.formatIdentity = formatIdentity;
    function formatFixed2(value) {
        return value.toFixed(2);
    }
    LxCharts.formatFixed2 = formatFixed2;
    function formatFixed4(value) {
        return value.toFixed(4);
    }
    LxCharts.formatFixed4 = formatFixed4;
    function formatRound(value) {
        return '' + Math.round(value);
    }
    LxCharts.formatRound = formatRound;
    function abbreviateCategory(value) {
        var v = parseFloat(value);
        if (isNaN(v))
            return value;
        return LxCharts.abbreviateNumber(v);
    }
    LxCharts.abbreviateCategory = abbreviateCategory;
    function cumulativeGrowthTooltip(params) {
        var d = params[0];
        return 'Period ' + d.data[0] + '<br/>Growth: ' + (d.data[1]).toFixed(4) + 'x';
    }
    LxCharts.cumulativeGrowthTooltip = cumulativeGrowthTooltip;
    function genTimeTooltip(params) {
        var v = params[0].value;
        return 'Gen ' + params[0].name + '<br/>' + LxCharts.formatDuration(v);
    }
    LxCharts.genTimeTooltip = genTimeTooltip;
    function formatMegabytes(mb) {
        if (typeof mb !== 'number' || isNaN(mb))
            return mb;
        var abs = Math.abs(mb);
        if (abs >= 1024)
            return (mb / 1024).toFixed(2) + ' GB';
        if (abs >= 1)
            return mb.toFixed(1) + ' MB';
        if (abs >= 1 / 1024)
            return (mb * 1024).toFixed(1) + ' KB';
        return (mb * 1024 * 1024).toFixed(0) + ' B';
    }
    LxCharts.formatMegabytes = formatMegabytes;
    function memoryTooltip(params) {
        if (!Array.isArray(params))
            params = [params];
        var sorted = params.slice().filter(function (p) { return p.value != null && p.value !== 0; });
        sorted.sort(function (a, b) { return Math.abs(b.value) - Math.abs(a.value); });
        var header = 'Gen ' + params[0].name;
        var lines = [header];
        for (var i = 0; i < sorted.length; i++) {
            var p = sorted[i];
            lines.push(p.marker + ' ' + p.seriesName + ': ' + LxCharts.formatMegabytes(p.value));
        }
        return lines.join('<br/>');
    }
    LxCharts.memoryTooltip = memoryTooltip;
})(LxCharts || (LxCharts = {}));
