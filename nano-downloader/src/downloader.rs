// mod  dos_lib;

use pyo3::prelude::*;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use std::fs::File;
use std::io::{self, BufRead, Seek, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB
const NUM_THREADS: usize = 4;

#[tokio::main]
async fn main_download(url: String, file_name_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 从标准输入获取URL
    // let mut url = get_url_from_input()?;
    // let url_arc = Arc::new(url); // 假设 url 是需要共享的数据
    // let filename = "downloaded_file";
    let filename = file_name_str;

    // 创建HTTP客户端
    let client = Client::new();

    // 获取文件大小
    let total_size = get_content_length(&client, &url).await?;

    // 创建共享的文件写入器
    let file = Arc::new(Mutex::new(File::create(filename)?));

    // 计算下载分块
    // 定义一个空的向量 chunks，用来存储分块的起始和结束位置
    let chunks : Vec < (u64, u64) >  =
        // 使用 (0..total_size) 创建一个从 0 到 total_size-1 的范围的迭代器
        (0..total_size)
    // 使用 step_by 方法按步长 CHUNK_SIZE（转换为 usize 类型） 进行迭代
    .step_by(CHUNK_SIZE as usize)
    // 对每个迭代的起始位置 start 执行映射操作
    .map( | start | {
        // 计算当前块的结束位置 end，保证不超过 total_size - 1
        let end = (start + CHUNK_SIZE - 1).min(total_size - 1);
        // 返回一个元组 (start, end)，表示当前块的起始和结束位置
        (start, end)
    })
    // 将所有的 (start, end) 元组收集到 chunks 向量中并返回
    .collect();

    // 使用 futures crate 创建并发任务
    let client = Arc::new(client);
    let url_arc = Arc::new(url); // 假设 url 是需要共享的数据
    stream::iter(chunks.into_iter().map(|(start, end)| {
        let client = Arc::clone(&client);
        let file = Arc::clone(&file);
        // 在闭包内使用 Arc 的 clone 方法来增加引用计数，并传递给需要使用的地方
        let url = url_arc.clone();
        tokio::spawn(async move {
            // 在闭包内使用 Arc 的 clone 方法来增加引用计数，并传递给需要使用的地方
            // let url = url_arc.clone();
            download_chunk(client, &url, start, end, file).await
        })
    }))
    .buffer_unordered(NUM_THREADS)
    .for_each(|res| async {
        if let Err(e) = res {
            eprintln!("Download error: {:?}", e);
        }
    })
    .await;

    Ok(())
}

fn get_url_from_input() -> Result<String, Box<dyn std::error::Error>> {
    println!("Please enter the URL to download:");
    let stdin = io::stdin();
    let mut url = String::new();
    stdin.lock().read_line(&mut url)?;
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err("URL cannot be empty".into());
    }
    Ok(url)
}

async fn get_content_length(client: &Client, url: &str) -> Result<u64, reqwest::Error> {
    let response = client.head(url).send().await?;
    let content_length0 = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .unwrap()
        // .ok_or_else(|| {})
        // .as_str()
        // .to_str();
        .as_bytes();
    // let content_length: u64 = content_length0 as u64;
    let content_string = std::str::from_utf8(content_length0).expect("Invalid UTF-8");
    let content_length: u64 = content_string.parse().expect("Not a number");
    // .parse::<u64>()?;
    Ok(content_length)
}

// 定义异步函数 download_chunk，用于下载文件的一部分并写入到指定文件
async fn download_chunk(
    client: Arc<Client>,    // HTTP 客户端，用于发送请求
    url: &str,              // 文件的 URL 地址
    start: u64,             // 下载开始位置（字节）
    end: u64,               // 下载结束位置（字节）
    file: Arc<Mutex<File>>, // 共享的文件写入器，使用 Arc 和 Mutex 确保线程安全
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 返回结果，成功时返回 Ok，失败时返回错误
    // 创建 HTTP Range 头部，指定请求的字节范围
    let range_header = format!("bytes={}-{}", start, end);

    // 发送 HTTP GET 请求，附加 Range 头部来请求指定的字节范围
    let response = client
        .get(url)
        .header("Range", range_header) // 添加 Range 头部
        .send()
        .await?; // 发送请求并等待响应，使用 `?` 操作符处理可能的错误

    // 获取响应的字节内容
    let bytes = response.bytes().await?; // 等待响应的字节数据，并处理可能的错误

    // 锁定文件进行写入操作，确保线程安全
    let mut file = file.lock().await; // 获取互斥锁，锁定文件对象以进行写入操作

    // 移动文件光标到指定的开始位置
    file.seek(io::SeekFrom::Start(start))?; // 移动文件光标到 start 位置，使用 `?` 处理可能的错误

    // 将下载的数据写入到文件
    file.write_all(&bytes)?; // 将字节数据写入文件，使用 `?` 处理可能的错误

    // 返回成功
    Ok(()) // 表示函数执行成功
}

fn main() {
    // let mut url = get_url_from_input()?;
    // main_download(url);
    match get_url_from_input() {
        Ok(url_string) => {
            // 在这里 url_string 是一个 String 类型的字符串
            println!("URL: {}", url_string);

            // 可以将 url_string 传递给需要使用的地方
            // let _ = main_download(url_string);
            // let url_str = &url_string.as_str(); // 提前借用 &str
            // let url_string1 = url_string;
            let url_string1 = url_string.clone(); // 复制 url_string
            let path = Path::new(url_string1.as_str());
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    println!("文件名: {}", file_name_str);

                    match main_download(url_string, file_name_str) {
                        Ok(_) => println!("Download succeeded"),
                        Err(e) => eprintln!("Download failed: {}", e),
                    }
                } else {
                    println!("无法转换文件名为字符串");
                }
            } else {
                println!("没有文件名");
            }
        }
        Err(err) => {
            // 处理错误，这里简单地打印出错误信息
            eprintln!("Error: {}", err);
        }
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn download(a:usize , b:usize) -> PyResult<String> {
    main();
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn nano_downloader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(download, m)?)?;
    Ok(())
}
