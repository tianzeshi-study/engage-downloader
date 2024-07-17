import os
import urllib.request
import threading

# 下载文件的URL
# url = 'https://example.com/largefile.zip'
url = "https://videos.aiursoft.cn/media/original/user/anduin/5552db90ed7b494b9850f918e24ba872.mmexport1678851452849.mp4"
# 本地保存路径
file_name = 'largefile.zip'

# 获取文件大小
def get_file_size(url):
    req = urllib.request.Request(url, method='HEAD')
    with urllib.request.urlopen(req) as response:
        return int(response.getheader('Content-Length'))

# 分块下载函数
def download_chunk(url, start, end, chunk_number):
    req = urllib.request.Request(url)
    req.headers['Range'] = f'bytes={start}-{end}'
    with urllib.request.urlopen(req) as response:
        with open(f'{file_name}.part{chunk_number}', 'wb') as f:
            f.write(response.read())

# 多线程下载
def multi_thread_download(url, num_threads=8):
    file_size = get_file_size(url)
    chunk_size = file_size // num_threads
    
    threads = []
    for i in range(num_threads):
        start = i * chunk_size
        end = start + chunk_size - 1 if i < num_threads - 1 else file_size - 1
        thread = threading.Thread(target=download_chunk, args=(url, start, end, i))
        threads.append(thread)
        thread.start()
    
    for thread in threads:
        thread.join()
    
    with open(file_name, 'wb') as output_file:
        for i in range(num_threads):
            with open(f'{file_name}.part{i}', 'rb') as part_file:
                output_file.write(part_file.read())
            os.remove(f'{file_name}.part{i}')

multi_thread_download(url)
