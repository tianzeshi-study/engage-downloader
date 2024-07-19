use std::path::Path;
use std::fs::File;
use std::io::{self, Seek, Write, BufRead};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use futures::stream::{self, StreamExt};

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
    let chunks: Vec<(u64, u64)> = (0..total_size)
        .step_by(CHUNK_SIZE as usize)
        .map(|start| {
            let end = (start + CHUNK_SIZE - 1).min(total_size - 1);
            (start, end)
        })
        .collect();
    
    // 使用 futures crate 创建并发任务
    let client = Arc::new(client);
    let url_arc = Arc::new(url); // 假设 url 是需要共享的数据
    stream::iter(chunks.into_iter().map(|(start, end)| {
        let client = Arc::clone(&client);
        let file = Arc::clone(&file);
        // let url_arc = Arc::new(url); // 假设 url 是需要共享的数据
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

async fn download_chunk(
    client: Arc<Client>,
    url: &str,
    start: u64,
    end: u64,
    file: Arc<Mutex<File>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let range_header = format!("bytes={}-{}", start, end);
    let response = client.get(url).header("Range", range_header).send().await?;
    let bytes = response.bytes().await?;
    
    // 写入文件
    let mut file = file.lock().await;
    file.seek(io::SeekFrom::Start(start))?;
    file.write_all(&bytes)?;
    Ok(())
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
        },
        Err(err) => {
            // 处理错误，这里简单地打印出错误信息
            eprintln!("Error: {}", err);
        }
    }

}
    