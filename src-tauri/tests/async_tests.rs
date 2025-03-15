use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;

#[tokio::main]
async fn main() {
    // 测试参数
    let iterations = 1_000_000;
    let threads = 4;

    // 测试标准互斥锁
    let std_mutex = Arc::new(StdMutex::new(0));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..threads {
        let mutex = Arc::clone(&std_mutex);
        handles.push(tokio::spawn(async move {
            for _ in 0..iterations / threads {
                let mut value = mutex.lock().unwrap();
                *value += 1;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let std_duration = start.elapsed();
    println!("标准互斥锁耗时: {:?}", std_duration);

    // 测试异步互斥锁
    let tokio_mutex = Arc::new(TokioMutex::new(0));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..threads {
        let mutex = Arc::clone(&tokio_mutex);
        handles.push(tokio::spawn(async move {
            for _ in 0..iterations / threads {
                let mut value = mutex.lock().await;
                *value += 1;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let tokio_duration = start.elapsed();
    println!("异步互斥锁耗时: {:?}", tokio_duration);
    println!(
        "性能差异: {:.2}x",
        tokio_duration.as_secs_f64() / std_duration.as_secs_f64()
    );
}
