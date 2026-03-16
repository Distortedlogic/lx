use std::sync::Mutex;

static DB: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn insert(item: String) {
    let mut lock = DB.lock().unwrap();
    lock.push(item);
}

pub fn get_all() -> Vec<String> {
    let lock = DB.lock().unwrap();
    lock.clone()
}

pub fn find(name: &str) -> String {
    let lock = DB.lock().unwrap();
    lock.iter().find(|s| s.contains(name)).unwrap().clone()
}

pub fn remove(index: usize) -> String {
    let mut lock = DB.lock().unwrap();
    if index < lock.len() {
        lock.remove(index)
    } else {
        panic!("index out of bounds");
    }
}
