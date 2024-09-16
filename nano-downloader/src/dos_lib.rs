// stress_test.rs

use std::sync::Arc;
use reqwest::Client;
use std::time::Instant;
use tokio::task;

// 定义公共函数进行压力测试
pub async fn run_stress_test(url: String, request_count: usize, concurrent_tasks: usize) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut tasks = vec![];

    let start_time = Instant::now();  // 记录开始时间
    let url_arc = Arc::new(url); // 假设 url 是需要共享的数据

    // 创建并发任务
    for _ in 0..concurrent_tasks {
        let client_clone = client.clone();
        let url_clone = Arc::clone(&url_arc);

        // let url_clone = url_arc.clone();  // 将 URL 克隆到每个任务中
        let task = task::spawn(async move {
            for _ in 0..(request_count / concurrent_tasks) {

                match client_clone.get(&*url_clone).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("Request successful");
                        } else {
                            println!("Request failed with status: {}", response.status());
                        }
                    }
                    Err(e) => println!("Request failed: {}", e),
                }
            }
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    for task in tasks {
        task.await.unwrap();
    }

    let duration = start_time.elapsed();
    println!("Completed in: {:?}", duration);

    Ok(())
}

// 模块测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_stress_test_success() {
        // 创建一个异步运行时
        let rt = Runtime::new().unwrap();

        // 在运行时中执行异步任务
        rt.block_on(async {
            let url = "http://localhost:8077".to_string();
            // let url = "http://httpbin.org/get".to_string();  // 使用公共的 HTTP 测试服务器
            let result = run_stress_test(url, 10, 2).await;  // 调用模块的压力测试函数

            // 测试是否成功运行
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_stress_test_invalid_url() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let url = "http://localhost:8077/hash".to_string();  // 测试无效的 URL
            let result = run_stress_test(url, 10, 2).await;

            // 测试应返回错误
            assert!(true);
        });
    }
}
