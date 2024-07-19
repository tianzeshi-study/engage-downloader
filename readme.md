# downloader homework for engage program

## python   

### how to use 

```
python3 main.py
```

##  rust

没有第三方库的版本(只支持http)

确实搓不出一个tls握手,GPT也让我放弃自己手搓ssl,

q: 我是请你使用标准库完成,不要使用cargo和第三方库
ChatGPT:
Rust标准库没有直接支持HTTPS的功能。实现HTTPS需要手动处理TLS握手和加密，这是一个复杂的过程。通常，这种低级别操作会使用现成的库来简化开发过程。
不过，如果你一定要使用标准库实现，可以尝试使用外部命令行工具（如`openssl`）来辅助完成任务。以下是一个使用Rust标准库和`openssl`命令行工具的示例
......
```


###  usage
```
rustc main.rs
./main
```

## rust第三方库版本

```
cd  nano-downloader
cargo run 
```