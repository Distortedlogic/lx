#[derive(Clone, PartialEq)]
pub enum LineData {
    Pairs(Vec<Vec<f64>>),
    Values(Vec<f64>),
}

impl LineData {
    pub fn len(&self) -> usize {
        match self {
            Self::Pairs(v) => v.len(),
            Self::Values(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Pairs(v) => v.is_empty(),
            Self::Values(v) => v.is_empty(),
        }
    }
}

impl From<Vec<Vec<f64>>> for LineData {
    fn from(v: Vec<Vec<f64>>) -> Self {
        Self::Pairs(v)
    }
}

impl From<Vec<f64>> for LineData {
    fn from(v: Vec<f64>) -> Self {
        Self::Values(v)
    }
}

#[derive(Clone, PartialEq)]
pub struct LineSeries {
    pub name: String,
    pub data: LineData,
    pub color: &'static str,
    pub width: Option<u32>,
    pub area: bool,
    pub stack: Option<String>,
    pub show_symbol: bool,
}

#[derive(Clone, PartialEq)]
pub struct BarSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub color: Option<&'static str>,
    pub stack: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct ScatterSeries {
    pub name: String,
    pub data: Vec<Vec<f64>>,
    pub color: &'static str,
    pub symbol_size: Option<f64>,
}

#[derive(Clone, PartialEq)]
pub struct PieSlice {
    pub name: String,
    pub value: f64,
    pub color: Option<&'static str>,
}

#[derive(Clone, PartialEq)]
pub struct DataZoomConfig {
    pub start: f64,
    pub end: f64,
    pub slider: bool,
    pub inside: bool,
}

impl DataZoomConfig {
    pub fn last_n(data_len: usize, visible: usize) -> Self {
        let start = if data_len > visible {
            (data_len.saturating_sub(visible)) as f64 / data_len.max(1) as f64 * 100.0
        } else {
            0.0
        };
        Self {
            start,
            end: 100.0,
            slider: true,
            inside: true,
        }
    }
}

impl Default for DataZoomConfig {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 100.0,
            slider: true,
            inside: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPosition {
    #[default]
    Top,
    Bottom,
    Right,
}
