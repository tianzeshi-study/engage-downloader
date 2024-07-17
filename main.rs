use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::TcpStream;
use std::io::prelude::*;

// 定义下载的URL路径和主机名
const URL: &str = "https://videos.aiursoft.cn/media/original/user/anduin/5552db90ed7b494b9850f918e24ba872.mmexport1678851452849.mp4";
const HOST: &str = "videos.aiursoft.cn";
const FILE_NAME: &str = "5552db90ed7b494b9850f918e24ba872.mmexport1678851452849.mp4";

// 获取文件大小函数
fn get_file_size(host: &str, url: &str) -> u64 {
    // 构造HEAD请求以获取文件大小
    let request = format!("HEAD {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", url, host);
    
    // 创建到服务器的TCP连接
    let mut stream = TcpStream::connect((host, 80)).unwrap();
    
    // 发送HEAD请求
    stream.write_all(request.as_bytes()).unwrap();

    // 读取服务器的响应
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    // 解析响应头以提取Content-Length字段
    for line in response.lines() {
        if line.starts_with("Content-Length:") {
            let size = line.split(':').nth(1).unwrap().trim();
            return size.parse().unwrap();
        }
    }
    0
}

// 下载指定分块的函数
fn download_chunk(host: &str, url: &str, start: u64, end: u64, chunk_number: usize, progress: Arc<Mutex<Vec<u64>>>, file_size: u64) {
    // 构造带有Range头的GET请求以下载文件的指定部分
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nRange: bytes={}-{}\r\nConnection: close\r\n\r\n",
        url, host, start, end
    );
    
    // 创建到服务器的TCP连接
    let mut stream = TcpStream::connect((host, 80)).unwrap();
    
    // 发送GET请求
    stream.write_all(request.as_bytes()).unwrap();

    // 读取服务器的响应
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();

    // 找到响应头的结束位置（\r\n\r\n）
    let mut headers_end = 0;
    for i in 0..response.len() {
        if i + 4 < response.len() && &response[i..i + 4] == b"\r\n\r\n" {
            headers_end = i + 4;
            break;
        }
    }

    // 提取响应体（即文件内容的一部分）
    let body = &response[headers_end..];

    // 将响应体写入到对应的分块文件中
    let mut file = File::create(format!("{}.part{}", FILE_NAME, chunk_number)).unwrap();
    file.write_all(body).unwrap();

    // 更新进度
    let mut progress = progress.lock().unwrap();
    progress[chunk_number] += body.len() as u64;
    let downloaded: u64 = progress.iter().sum();
    
    // 打印下载进度
    println!("\r下载进度: {:.2}%", (downloaded as f64 / file_size as f64) * 100.0);
}

// 多线程下载函数
fn multi_thread_download(url: &str, num_threads: usize) {
    // 获取文件大小
    let file_size = get_file_size(HOST, url);
    
    // 计算每个线程下载的块大小
    let chunk_size = (file_size + num_threads as u64 - 1) / num_threads as u64; // 使用整除并向上取整

    // 创建一个线程安全的进度跟踪器
    let progress = Arc::new(Mutex::new(vec![0; num_threads]));
    let mut handles = vec![];

    // 为每个线程创建并启动一个下载任务
    for i in 0..num_threads {
        // 计算当前线程的起始和结束字节位置
        let start = i as u64 * chunk_size;
// 先计算出 start + chunk_size - 1 的值
// let calculated_end = start + chunk_size - 1;
let calculated_end = start.saturating_add(chunk_size).saturating_sub(1);



// 根据计算后的值进行条件判断和赋值
let end = if calculated_end >= file_size || file_size == 0 {
    file_size.saturating_sub(1)  // 如果超出文件末尾或文件大小为0，则取文件末尾的位置
} else {
    calculated_end  // 否则，取计算后的值作为结束位置
};

        // 克隆进度跟踪器和URL字符串以传递给线程
        let progress = Arc::clone(&progress);
        let url = url.to_string();
        
        // 启动线程下载指定的文件块
        let handle = thread::spawn(move || {
            download_chunk(HOST, &url, start, end, i, progress, file_size);
        });

        handles.push(handle);
    }

    // 等待所有线程完成下载
    for handle in handles {
        handle.join().unwrap();
    }

    // 合并所有下载的文件块
    let mut output_file = File::create(FILE_NAME).unwrap();
    for i in 0..num_threads {
        let mut part_file = File::open(format!("{}.part{}", FILE_NAME, i)).unwrap();
        let mut buffer = Vec::new();
        part_file.read_to_end(&mut buffer).unwrap();
        output_file.write_all(&buffer).unwrap();
        std::fs::remove_file(format!("{}.part{}", FILE_NAME, i)).unwrap();
    }
}

// 主函数，开始多线程下载
fn main() {
    multi_thread_download(URL, 4);
}
