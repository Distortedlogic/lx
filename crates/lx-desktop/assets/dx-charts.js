var DxCharts = (function(exports) {
	Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
	//#region \0rolldown/runtime.js
	var __defProp = Object.defineProperty;
	var __exportAll = (all, no_symbols) => {
		let target = {};
		for (var name in all) __defProp(target, name, {
			get: all[name],
			enumerable: true
		});
		if (!no_symbols) __defProp(target, Symbol.toStringTag, { value: "Module" });
		return target;
	};
	//#endregion
	//#region src/formatters.ts
	function formatDuration(value) {
		if (value < .1) return value.toFixed(4) + "ms";
		if (value < 1) return value.toFixed(2) + "ms";
		if (value < 1e3) return Math.round(value) + "ms";
		if (value < 6e4) return (value / 1e3).toFixed(1) + "s";
		if (value < 36e5) {
			const m = Math.floor(value / 6e4);
			const s = Math.floor(value % 6e4 / 1e3);
			return m + "m " + s + "s";
		}
		const h = Math.floor(value / 36e5);
		const mn = Math.floor(value % 36e5 / 6e4);
		return h + "h " + mn + "m";
	}
	function formatNumber(value) {
		if (typeof value !== "number") return value;
		if (Math.abs(value) >= 1e3) return value.toLocaleString(void 0, { maximumFractionDigits: 2 });
		if (Math.abs(value) >= 1) return value.toFixed(2);
		if (Math.abs(value) >= .001) return value.toFixed(4);
		return value.toFixed(6);
	}
	function formatFitness(value) {
		if (Math.abs(value) >= 1e3) return value.toFixed(0);
		return value.toFixed(2);
	}
	function formatPercent(value) {
		return value + "%";
	}
	function formatMoney(value) {
		if (typeof value !== "number" || isNaN(value)) return value;
		const abs = Math.abs(value);
		const sign = value < 0 ? "-" : "";
		if (abs >= 1e9) return sign + "$" + (abs / 1e9).toFixed(1) + "B";
		if (abs >= 1e6) return sign + "$" + (abs / 1e6).toFixed(1) + "M";
		if (abs >= 1e3) return sign + "$" + (abs / 1e3).toFixed(1) + "k";
		return sign + "$" + abs.toFixed(2);
	}
	function formatMoneyFull(value) {
		if (typeof value !== "number" || isNaN(value)) return value;
		return (value < 0 ? "-$" : "$") + Math.abs(value).toLocaleString(void 0, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2
		});
	}
	function abbreviateNumber(value) {
		if (isNaN(value)) return value;
		if (Math.abs(value) >= 1e9) return (value / 1e9).toFixed(1) + "B";
		if (Math.abs(value) >= 1e6) return (value / 1e6).toFixed(1) + "M";
		if (Math.abs(value) >= 1e3) return (value / 1e3).toFixed(1) + "k";
		if (Math.abs(value) >= 100) return value.toFixed(0);
		return value.toFixed(1);
	}
	function abbreviatePopulation(value) {
		if (value >= 1e9) return (value / 1e9).toFixed(1) + "B";
		if (value >= 1e6) return (value / 1e6).toFixed(1) + "M";
		if (value >= 1e3) return (value / 1e3).toFixed(1) + "K";
		return "" + value;
	}
	function formatIdentity(value) {
		return "" + value;
	}
	function formatFixed2(value) {
		return value.toFixed(2);
	}
	function formatFixed4(value) {
		return value.toFixed(4);
	}
	function formatRound(value) {
		return "" + Math.round(value);
	}
	function abbreviateCategory(value) {
		const v = parseFloat(value);
		if (isNaN(v)) return value;
		return abbreviateNumber(v);
	}
	function formatMegabytes(mb) {
		if (typeof mb !== "number" || isNaN(mb)) return mb;
		const abs = Math.abs(mb);
		if (abs >= 1024) return (mb / 1024).toFixed(2) + " GB";
		if (abs >= 1) return mb.toFixed(1) + " MB";
		if (abs >= 1 / 1024) return (mb * 1024).toFixed(1) + " KB";
		return (mb * 1024 * 1024).toFixed(0) + " B";
	}
	function alpsTooltipFormatter(params, popData, fitnessData) {
		const idx = params[0].dataIndex;
		const pop = popData[idx];
		const pct = params[0].value.toFixed(1);
		const fit = fitnessData[idx];
		return params[0].name + "<br/>Population: " + pop + " (" + pct + "%)<br/>Best Fitness: " + fit;
	}
	function scatterTooltipFormatter(params, xLabel, yLabel) {
		const d = params.data.value || params.data;
		return xLabel + ": " + d[0].toFixed(2) + "%<br/>" + yLabel + ": " + d[1].toFixed(2) + "%";
	}
	function cumulativeGrowthTooltip(params) {
		const d = params[0];
		return "Period " + d.data[0] + "<br/>Growth: " + d.data[1].toFixed(4) + "x";
	}
	function genTimeTooltip(params) {
		const v = params[0].value;
		return "Gen " + params[0].name + "<br/>" + formatDuration(v);
	}
	function memoryTooltip(params) {
		if (!Array.isArray(params)) params = [params];
		const sorted = params.slice().filter((p) => p.value != null && p.value !== 0);
		sorted.sort((a, b) => Math.abs(b.value) - Math.abs(a.value));
		const lines = ["Gen " + params[0].name];
		for (const p of sorted) lines.push(p.marker + " " + p.seriesName + ": " + formatMegabytes(p.value));
		return lines.join("<br/>");
	}
	//#endregion
	//#region src/chart_init.ts
	var AXIS_FORMATTERS = {
		identity: formatIdentity,
		duration: formatDuration,
		fitness: formatFitness,
		percent: formatPercent,
		money: formatMoney,
		moneyFull: formatMoneyFull,
		abbreviate: abbreviateNumber,
		fixed2: formatFixed2,
		fixed4: formatFixed4,
		round: formatRound,
		abbreviateCategory,
		megabytes: formatMegabytes
	};
	function applyAxisFormatter(axisOpt, fn) {
		if (!axisOpt) return;
		const arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
		for (const ax of arr) {
			if (!ax.axisLabel) ax.axisLabel = {};
			ax.axisLabel.formatter = fn;
		}
	}
	function applyFormatters(opts, el) {
		const ds = el.dataset;
		if (ds.xFmt && AXIS_FORMATTERS[ds.xFmt]) applyAxisFormatter(opts.xAxis, AXIS_FORMATTERS[ds.xFmt]);
		if (ds.yFmt && AXIS_FORMATTERS[ds.yFmt]) applyAxisFormatter(opts.yAxis, AXIS_FORMATTERS[ds.yFmt]);
		const extra = ds.extra && ds.extra.length > 0 ? JSON.parse(ds.extra) : null;
		if (ds.tooltipFmt) {
			if (!opts.tooltip) opts.tooltip = {};
			switch (ds.tooltipFmt) {
				case "cumulativeGrowth":
					opts.tooltip.formatter = cumulativeGrowthTooltip;
					break;
				case "money":
					opts.tooltip.valueFormatter = formatMoneyFull;
					break;
				case "genTime":
					opts.tooltip.formatter = genTimeTooltip;
					break;
				case "scatter":
					if (extra) opts.tooltip.formatter = (params) => scatterTooltipFormatter(params, extra.xLabel, extra.yLabel);
					break;
				case "alps":
					if (extra) opts.tooltip.formatter = (params) => alpsTooltipFormatter(params, extra.pop, extra.fitness);
					break;
				case "megabytes":
					opts.tooltip.formatter = memoryTooltip;
					break;
			}
		}
		if (ds.labelFmt && opts.series) {
			const sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
			for (const s of sarr) {
				if (!s.label) s.label = {};
				if (ds.labelFmt === "alpsPopulation" && extra) {
					const popData = extra.pop;
					s.label.formatter = (params) => abbreviatePopulation(popData[params.dataIndex]);
				}
			}
		}
	}
	function themeAxis(axisOpt, fg, bc, sc) {
		if (!axisOpt) return;
		const arr = Array.isArray(axisOpt) ? axisOpt : [axisOpt];
		for (const ax of arr) {
			if (!ax.axisLine) ax.axisLine = {};
			if (!ax.axisLine.lineStyle) ax.axisLine.lineStyle = {};
			if (!ax.axisLine.lineStyle.color) ax.axisLine.lineStyle.color = bc;
			if (!ax.splitLine) ax.splitLine = {};
			if (!ax.splitLine.lineStyle) ax.splitLine.lineStyle = {};
			if (!ax.splitLine.lineStyle.color) ax.splitLine.lineStyle.color = sc;
			if (ax.splitLine.show === void 0) ax.splitLine.show = true;
			if (!ax.axisTick) ax.axisTick = {};
			if (!ax.axisTick.lineStyle) ax.axisTick.lineStyle = {};
			if (!ax.axisTick.lineStyle.color) ax.axisTick.lineStyle.color = bc;
			if (!ax.axisLabel) ax.axisLabel = {};
			if (!ax.axisLabel.color) ax.axisLabel.color = fg;
			if (ax.axisLabel.textBorderWidth === void 0) ax.axisLabel.textBorderWidth = 0;
			if (!ax.nameTextStyle) ax.nameTextStyle = {};
			if (!ax.nameTextStyle.color) ax.nameTextStyle.color = fg;
			if (ax.nameTextStyle.textBorderWidth === void 0) ax.nameTextStyle.textBorderWidth = 0;
		}
	}
	function initChart(id, opts) {
		const el = document.getElementById(id);
		if (!el || typeof echarts === "undefined") return;
		const cs = getComputedStyle(document.documentElement);
		const fg = cs.getPropertyValue("--foreground").trim() || "#e5e7eb";
		const bc = cs.getPropertyValue("--color-chart-axis").trim() || "#404040";
		const sc = cs.getPropertyValue("--color-chart-split").trim() || "#333333";
		const tbg = cs.getPropertyValue("--color-chart-tooltip").trim() || "#171717";
		const ds = el.dataset;
		if (ds.title && ds.title.length > 0 && !opts.title) opts.title = { text: ds.title };
		if (!opts.grid) opts.grid = {};
		if (opts.grid.containLabel === void 0) opts.grid.containLabel = true;
		if (!opts.textStyle) opts.textStyle = {};
		opts.textStyle.color = fg;
		themeAxis(opts.xAxis, fg, bc, sc);
		themeAxis(opts.yAxis, fg, bc, sc);
		if (opts.legend) {
			if (!opts.legend.textStyle) opts.legend.textStyle = {};
			if (!opts.legend.textStyle.color) opts.legend.textStyle.color = fg;
		}
		if (opts.series) {
			const sarr = Array.isArray(opts.series) ? opts.series : [opts.series];
			for (const s of sarr) if (s.label) {
				if (s.label.textBorderWidth === void 0) s.label.textBorderWidth = 0;
				if (!s.label.color) s.label.color = fg;
			}
		}
		if (!opts.tooltip) opts.tooltip = {};
		if (!opts.tooltip.backgroundColor) opts.tooltip.backgroundColor = tbg;
		if (!opts.tooltip.borderColor) opts.tooltip.borderColor = bc;
		if (!opts.tooltip.textStyle) opts.tooltip.textStyle = {};
		if (!opts.tooltip.textStyle.color) opts.tooltip.textStyle.color = fg;
		if (opts.title) {
			const t = Array.isArray(opts.title) ? opts.title[0] : opts.title;
			if (t) {
				if (!t.textStyle) t.textStyle = {};
				if (!t.textStyle.color) t.textStyle.color = fg;
			}
			const g = Array.isArray(opts.grid) ? opts.grid[0] : opts.grid;
			if (g && !g.top) g.top = 35;
		}
		applyFormatters(opts, el);
		if (!opts.tooltip.valueFormatter) opts.tooltip.valueFormatter = formatNumber;
		const instance = echarts.getInstanceByDom(el);
		if (instance) instance.setOption(opts, true);
		else {
			const chart = echarts.init(el);
			chart.setOption(opts);
			el._resize_observer = new ResizeObserver(() => chart.resize());
			el._resize_observer.observe(el);
		}
	}
	function disposeChart(id) {
		const el = document.getElementById(id);
		if (el) {
			const instance = echarts.getInstanceByDom(el);
			if (instance) instance.dispose();
			if (el._resize_observer) el._resize_observer.disconnect();
		}
	}
	function restoreChart(id) {
		const el = document.getElementById(id);
		if (el) {
			const instance = echarts.getInstanceByDom(el);
			if (instance) instance.dispatchAction({ type: "restore" });
		}
	}
	//#endregion
	//#region src/flow_graph.ts
	function initFlowGraph(id, graphJson) {
		const el = document.getElementById(id);
		if (!el || typeof echarts === "undefined") return;
		const instance = echarts.getInstanceByDom(el);
		if (instance) instance.dispose();
		const cs = getComputedStyle(document.documentElement);
		const fg = cs.getPropertyValue("--foreground").trim() || "#e5e7eb";
		const tbg = cs.getPropertyValue("--color-chart-tooltip").trim() || "#171717";
		const bc = cs.getPropertyValue("--color-chart-axis").trim() || "#404040";
		if (!graphJson.tooltip) graphJson.tooltip = {};
		if (!graphJson.tooltip.backgroundColor) graphJson.tooltip.backgroundColor = tbg;
		if (!graphJson.tooltip.borderColor) graphJson.tooltip.borderColor = bc;
		if (!graphJson.tooltip.textStyle) graphJson.tooltip.textStyle = {};
		if (!graphJson.tooltip.textStyle.color) graphJson.tooltip.textStyle.color = fg;
		if (!graphJson.textStyle) graphJson.textStyle = {};
		graphJson.textStyle.color = fg;
		const chart = echarts.init(el);
		chart.setOption(graphJson);
		el._resize_observer = new ResizeObserver(() => chart.resize());
		el._resize_observer.observe(el);
	}
	function updateFlowGraphStatus(id, nodeId, status) {
		const el = document.getElementById(id);
		if (!el || typeof echarts === "undefined") return;
		const inst = echarts.getInstanceByDom(el);
		if (!inst) return;
		const opt = inst.getOption();
		if (!opt?.series?.[0]?.data) return;
		const data = opt.series[0].data;
		for (const node of data) if (node.name === nodeId) {
			let style = {};
			switch (status) {
				case "running":
					style = {
						borderColor: "#eab308",
						borderWidth: 3
					};
					break;
				case "completed":
					style = {
						borderColor: "#22c55e",
						borderWidth: 2
					};
					break;
				case "error":
					style = {
						borderColor: "#ef4444",
						borderWidth: 3
					};
					break;
				case "active":
					style = {
						borderColor: "#facc15",
						borderWidth: 3
					};
					break;
			}
			if (!node.itemStyle) node.itemStyle = {};
			Object.assign(node.itemStyle, style);
			break;
		}
		inst.setOption({ series: [{ data }] }, false);
	}
	//#endregion
	//#region src/flamegraph.ts
	function setupFlamegraph(id, data, maxY) {
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
				formatter: (params) => params.value[3]
			},
			grid: {
				left: 60,
				right: 20,
				top: 20,
				bottom: 40
			},
			xAxis: {
				type: "value",
				name: "ms",
				axisLabel: { color: fg }
			},
			yAxis: {
				type: "value",
				min: 0,
				max: maxY,
				inverse: true,
				axisLabel: { color: fg }
			},
			series: [{
				type: "custom",
				renderItem: (params, api) => {
					const xStart = api.value(0);
					const depth = api.value(1);
					const xEnd = api.value(2);
					const label = api.value(3);
					const color = api.value(4);
					const start = api.coord([xStart, depth]);
					const end = api.coord([xEnd, depth + .9]);
					const rectShape = echarts.graphic.clipRectByRect({
						x: start[0],
						y: start[1],
						width: end[0] - start[0],
						height: end[1] - start[1]
					}, {
						x: params.coordSys.x,
						y: params.coordSys.y,
						width: params.coordSys.width,
						height: params.coordSys.height
					});
					return rectShape && {
						type: "rect",
						shape: rectShape,
						style: api.style({
							fill: color,
							stroke: "#222",
							text: label,
							textFill: "#fff",
							fontSize: 11,
							truncate: {
								outerWidth: rectShape.width,
								outerHeight: rectShape.height
							}
						})
					};
				},
				encode: {
					x: [0, 2],
					y: 1
				},
				data
			}]
		});
		el._resize_observer = new ResizeObserver(() => chart.resize());
		el._resize_observer.observe(el);
	}
	//#endregion
	//#region src/candlestick.ts
	function candlestickRenderItem(params, api) {
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
		const halfWidth = api.size([1, 0])[0] * (.15 + .35 * relVol);
		const color = close >= open ? "#22c55e" : "#ef4444";
		const bodyTop = Math.min(openPt[1], closePt[1]);
		const bodyHeight = Math.max(Math.abs(openPt[1] - closePt[1]), 1);
		return {
			type: "group",
			children: [{
				type: "line",
				shape: {
					x1: lowPt[0],
					y1: lowPt[1],
					x2: highPt[0],
					y2: highPt[1]
				},
				style: {
					stroke: color,
					lineWidth: 1
				}
			}, {
				type: "rect",
				shape: {
					x: openPt[0] - halfWidth,
					y: bodyTop,
					width: halfWidth * 2,
					height: bodyHeight
				},
				style: {
					fill: color,
					stroke: color
				}
			}]
		};
	}
	window.DxCharts = /* @__PURE__ */ __exportAll({
		abbreviateCategory: () => abbreviateCategory,
		abbreviateNumber: () => abbreviateNumber,
		abbreviatePopulation: () => abbreviatePopulation,
		alpsTooltipFormatter: () => alpsTooltipFormatter,
		candlestickRenderItem: () => candlestickRenderItem,
		cumulativeGrowthTooltip: () => cumulativeGrowthTooltip,
		disposeChart: () => disposeChart,
		formatDuration: () => formatDuration,
		formatFitness: () => formatFitness,
		formatFixed2: () => formatFixed2,
		formatFixed4: () => formatFixed4,
		formatIdentity: () => formatIdentity,
		formatMegabytes: () => formatMegabytes,
		formatMoney: () => formatMoney,
		formatMoneyFull: () => formatMoneyFull,
		formatNumber: () => formatNumber,
		formatPercent: () => formatPercent,
		formatRound: () => formatRound,
		genTimeTooltip: () => genTimeTooltip,
		initChart: () => initChart,
		initFlowGraph: () => initFlowGraph,
		memoryTooltip: () => memoryTooltip,
		restoreChart: () => restoreChart,
		scatterTooltipFormatter: () => scatterTooltipFormatter,
		setupFlamegraph: () => setupFlamegraph,
		updateFlowGraphStatus: () => updateFlowGraphStatus
	});
	//#endregion
	exports.abbreviateCategory = abbreviateCategory;
	exports.abbreviateNumber = abbreviateNumber;
	exports.abbreviatePopulation = abbreviatePopulation;
	exports.alpsTooltipFormatter = alpsTooltipFormatter;
	exports.candlestickRenderItem = candlestickRenderItem;
	exports.cumulativeGrowthTooltip = cumulativeGrowthTooltip;
	exports.disposeChart = disposeChart;
	exports.formatDuration = formatDuration;
	exports.formatFitness = formatFitness;
	exports.formatFixed2 = formatFixed2;
	exports.formatFixed4 = formatFixed4;
	exports.formatIdentity = formatIdentity;
	exports.formatMegabytes = formatMegabytes;
	exports.formatMoney = formatMoney;
	exports.formatMoneyFull = formatMoneyFull;
	exports.formatNumber = formatNumber;
	exports.formatPercent = formatPercent;
	exports.formatRound = formatRound;
	exports.genTimeTooltip = genTimeTooltip;
	exports.initChart = initChart;
	exports.initFlowGraph = initFlowGraph;
	exports.memoryTooltip = memoryTooltip;
	exports.restoreChart = restoreChart;
	exports.scatterTooltipFormatter = scatterTooltipFormatter;
	exports.setupFlamegraph = setupFlamegraph;
	exports.updateFlowGraphStatus = updateFlowGraphStatus;
	return exports;
})({});

//# sourceMappingURL=dx-charts.js.map