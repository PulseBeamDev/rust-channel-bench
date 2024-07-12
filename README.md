# Benchmarking Rust async channels

Quick benchmark to find out how different channel implementations cope with collecting messages from many tasks.

This compares two scenarios:

* a) `cloned-send`: Cloning the sender of a single channel into many tasks
* b) `merged-recv`: Merging the receivers of many channels with [`futures_buffered::FuturesUnordered`](https://docs.rs/futures-buffered/latest/futures_buffered/struct.FuturesUnordered.html)

I added implementations for [`flume`](https://docs.rs/flume/), [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/), and [`async-channel`](https://docs.rs/async-channel).

If you find issues with the benchmark, or have suggestions for improvements, please open an issue.

Results on my laptop (Intel 8250u quad-core), in release mode:

``` 
running channel benchmark with 1048576 messages and capacity 1024

# multi_thread,   tasks: 2, messages per task: 524288, capacity per task: 512

merged-recv tokio                 212 ms    202 ns/message
merged-recv flume                 233 ms    222 ns/message
merged-recv async-channel         131 ms    125 ns/message

cloned-send tokio                 252 ms    240 ns/message
cloned-send flume                 200 ms    191 ns/message
cloned-send async-channel         126 ms    121 ns/message

# current_thread, tasks: 2, messages per task: 524288, capacity per task: 512

merged-recv tokio                 233 ms    222 ns/message
merged-recv flume                 120 ms    115 ns/message
merged-recv async-channel         163 ms    155 ns/message

cloned-send tokio                 162 ms    154 ns/message
cloned-send flume                  73 ms     70 ns/message
cloned-send async-channel         118 ms    112 ns/message

# multi_thread,   tasks: 16, messages per task: 65536, capacity per task: 64

merged-recv tokio                 359 ms    342 ns/message
merged-recv flume                 443 ms    423 ns/message
merged-recv async-channel         417 ms    397 ns/message

cloned-send tokio                 506 ms    482 ns/message
cloned-send flume                 834 ms    795 ns/message
cloned-send async-channel         529 ms    505 ns/message

# current_thread, tasks: 16, messages per task: 65536, capacity per task: 64

merged-recv tokio                 282 ms    269 ns/message
merged-recv flume                 133 ms    127 ns/message
merged-recv async-channel         170 ms    162 ns/message

cloned-send tokio                 183 ms    174 ns/message
cloned-send flume                  80 ms     77 ns/message
cloned-send async-channel         157 ms    150 ns/message

# multi_thread,   tasks: 64, messages per task: 16384, capacity per task: 16

merged-recv tokio                 441 ms    421 ns/message
merged-recv flume                 418 ms    398 ns/message
merged-recv async-channel         393 ms    374 ns/message

cloned-send tokio                 886 ms    845 ns/message
cloned-send flume                3590 ms   3423 ns/message
cloned-send async-channel        1890 ms   1803 ns/message

# current_thread, tasks: 64, messages per task: 16384, capacity per task: 16

merged-recv tokio                 339 ms    323 ns/message
merged-recv flume                 149 ms    142 ns/message
merged-recv async-channel         162 ms    154 ns/message

cloned-send tokio                 218 ms    208 ns/message
cloned-send flume                  92 ms     88 ns/message
cloned-send async-channel         392 ms    374 ns/message

# multi_thread,   tasks: 256, messages per task: 4096, capacity per task: 4

merged-recv tokio                 438 ms    418 ns/message
merged-recv flume                 395 ms    377 ns/message
merged-recv async-channel         357 ms    341 ns/message

cloned-send tokio                 914 ms    871 ns/message
cloned-send flume                7121 ms   6791 ns/message
cloned-send async-channel        2181 ms   2080 ns/message

# current_thread, tasks: 256, messages per task: 4096, capacity per task: 4

merged-recv tokio                 461 ms    440 ns/message
merged-recv flume                 189 ms    180 ns/message
merged-recv async-channel         162 ms    154 ns/message

cloned-send tokio                 389 ms    371 ns/message
cloned-send flume                 162 ms    155 ns/message
cloned-send async-channel         511 ms    487 ns/message

# multi_thread,   tasks: 512, messages per task: 2048, capacity per task: 2

merged-recv tokio                 432 ms    412 ns/message
merged-recv flume                 398 ms    380 ns/message
merged-recv async-channel         259 ms    247 ns/message

cloned-send tokio                 927 ms    884 ns/message
cloned-send flume               11345 ms  10820 ns/message
cloned-send async-channel        2151 ms   2052 ns/message

# current_thread, tasks: 512, messages per task: 2048, capacity per task: 2

merged-recv tokio                 446 ms    425 ns/message
merged-recv flume                 212 ms    203 ns/message
merged-recv async-channel         139 ms    132 ns/message

cloned-send tokio                 358 ms    341 ns/message
cloned-send flume                 319 ms    304 ns/message
cloned-send async-channel         621 ms    592 ns/message
```
