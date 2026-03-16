use std::sync::{Arc, RwLock};

struct Config {
    threshold: f64,
    max_items: usize,
    batch_size: usize,
    verbose: bool,
}

struct DataHandle {
    inner: Arc<RwLock<Vec<f64>>>,
}

impl DataHandle {
    fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
    fn sum(&self) -> f64 {
        self.inner.read().unwrap().iter().sum()
    }
    fn avg(&self) -> f64 {
        let data = self.inner.read().unwrap();
        data.iter().sum::<f64>() / data.len() as f64
    }
    fn max(&self) -> f64 {
        self.inner.read().unwrap().iter().copied().fold(f64::MIN, f64::max)
    }
    fn push(&self, val: f64) {
        self.inner.write().unwrap().push(val);
    }
}

struct TransferPayload {
    values: Vec<f64>,
    label: String,
}

fn compute(payload: TransferPayload) -> f64 {
    payload.values.iter().sum()
}

fn process(threshold: f64, max_items: usize, batch_size: usize, verbose: bool) {
    if verbose {
        println!("Processing with threshold={} max={} batch={}", threshold, max_items, batch_size);
    }
    run_inner(threshold, max_items, batch_size, verbose);
}

fn run_inner(threshold: f64, max_items: usize, batch_size: usize, verbose: bool) {
    do_work(threshold, max_items, batch_size, verbose);
}

fn do_work(_threshold: f64, _max_items: usize, _batch_size: usize, _verbose: bool) {
    println!("working");
}

fn analyze(handle: &DataHandle) {
    let count = handle.len();
    let total = handle.sum();
    let average = handle.avg();
    let maximum = handle.max();
    println!("count={} total={} avg={} max={}", count, total, average, maximum);
}

fn main() {
    let config = Config {
        threshold: 0.5,
        max_items: 100,
        batch_size: 10,
        verbose: true,
    };
    process(config.threshold, config.max_items, config.batch_size, config.verbose);

    let payload = TransferPayload {
        values: vec![1.0, 2.0, 3.0],
        label: "test".into(),
    };
    let result = compute(payload);
    println!("{}", result);
}
