use polars::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;

fn process_data(df: DataFrame) -> DataFrame {
    let lazy = df.lazy();
    let collected = lazy.clone().collect().unwrap();
    let filtered = collected.lazy().filter(col("value").gt(lit(0))).collect().unwrap();
    let selected = filtered.lazy().select([col("*")]).collect().unwrap();
    selected
}

fn aggregate_manual(series: &Series) -> f64 {
    let mut total = 0.0;
    for i in 0..series.len() {
        let val = series.get(i).unwrap();
        total += val.try_extract::<f64>().unwrap();
    }
    total
}

fn extract_and_transform(df: &DataFrame) -> DataFrame {
    let vals: Vec<f64> = df.column("price").unwrap().f64().unwrap().into_no_null_iter().collect();
    let doubled: Vec<f64> = vals.iter().map(|v| v * 2.0).collect();
    let new_series = Series::new("doubled".into(), doubled);
    df.clone().with_column(new_series).unwrap().clone()
}

fn read_eager(path: &str) -> DataFrame {
    CsvReader::from_path(path).unwrap().finish().unwrap()
}

fn parallel_process(items: &[Vec<f64>]) -> Vec<f64> {
    let results = Mutex::new(Vec::new());
    items.par_iter().for_each(|chunk| {
        let sum: f64 = chunk.iter().sum();
        let mut buf = Vec::new();
        buf.push(sum);
        results.lock().unwrap().extend(buf);
    });
    results.into_inner().unwrap()
}

fn nested_parallel(matrix: &[Vec<f64>]) -> Vec<f64> {
    matrix.par_iter().flat_map(|row| {
        row.par_iter().map(|x| x * 2.0).collect::<Vec<_>>()
    }).collect()
}
