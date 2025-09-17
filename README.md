# Benchmarking Rust async channels

Quick benchmark to find out how different channel implementations cope with collecting messages from many tasks.

This compares two scenarios:

* a) `cloned-send`: Cloning the sender of a single channel into many tasks
* b) `merged-recv`: Merging the receivers of many channels with [`futures_buffered::FuturesUnordered`](https://docs.rs/futures-buffered/latest/futures_buffered/struct.FuturesUnordered.html)

I added implementations for [`flume`](https://docs.rs/flume/), [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/), and [`async-channel`](https://docs.rs/async-channel).

If you find issues with the benchmark, or have suggestions for improvements, please open an issue.

Results on my laptop (Intel 8250u quad-core), in release mode:

```
lukas@fedora:~/workspace/pulsebeam/rust-channel-bench$   ./target/release/channel-bench
running WebRTC SFU benchmark with 1048576 messages and capacity 65536

# multi_thread,   tasks: 1, messages per task: 1048576, capacity per task: 65536

cloned-send tokio                 146 ms    139 ns/message
cloned-send tokio                 148 ms    141 ns/message
cloned-send tokio                 149 ms    142 ns/message
cloned-send tokio                 162 ms    155 ns/message
cloned-send flume                 130 ms    124 ns/message
cloned-send flume                 134 ms    128 ns/message
cloned-send flume                 139 ms    132 ns/message
cloned-send flume                 161 ms    154 ns/message
cloned-send async-channel          84 ms     80 ns/message
cloned-send async-channel         120 ms    115 ns/message
cloned-send async-channel         133 ms    127 ns/message
cloned-send async-channel         145 ms    138 ns/message
cloned-send futures-channel        90 ms     86 ns/message
cloned-send futures-channel        99 ms     94 ns/message
cloned-send futures-channel       113 ms    108 ns/message
cloned-send futures-channel       123 ms    118 ns/message

# multi_thread,   tasks: 2, messages per task: 524288, capacity per task: 32768

cloned-send tokio                 153 ms    146 ns/message
cloned-send tokio                 167 ms    159 ns/message
cloned-send tokio                 197 ms    188 ns/message
cloned-send tokio                 199 ms    190 ns/message
cloned-send flume                 142 ms    135 ns/message
cloned-send flume                 144 ms    137 ns/message
cloned-send flume                 161 ms    153 ns/message
cloned-send flume                 171 ms    163 ns/message
cloned-send async-channel          82 ms     78 ns/message
cloned-send async-channel          87 ms     83 ns/message
cloned-send async-channel          88 ms     84 ns/message
cloned-send async-channel         101 ms     96 ns/message
cloned-send futures-channel       199 ms    190 ns/message
cloned-send futures-channel       212 ms    203 ns/message
cloned-send futures-channel       229 ms    219 ns/message
cloned-send futures-channel       233 ms    223 ns/message

# multi_thread,   tasks: 4, messages per task: 262144, capacity per task: 16384

cloned-send tokio                 241 ms    230 ns/message
cloned-send tokio                 242 ms    231 ns/message
cloned-send tokio                 250 ms    238 ns/message
cloned-send tokio                 256 ms    244 ns/message
cloned-send flume                 242 ms    230 ns/message
cloned-send flume                 266 ms    253 ns/message
cloned-send flume                 271 ms    259 ns/message
cloned-send flume                 277 ms    265 ns/message
cloned-send async-channel         205 ms    196 ns/message
cloned-send async-channel         208 ms    198 ns/message
cloned-send async-channel         228 ms    218 ns/message
cloned-send async-channel         264 ms    252 ns/message
cloned-send futures-channel       229 ms    218 ns/message
cloned-send futures-channel       258 ms    246 ns/message
cloned-send futures-channel       263 ms    251 ns/message
cloned-send futures-channel       304 ms    290 ns/message

# multi_thread,   tasks: 8, messages per task: 131072, capacity per task: 8192

cloned-send tokio                 331 ms    316 ns/message
cloned-send tokio                 345 ms    329 ns/message
cloned-send tokio                 361 ms    344 ns/message
cloned-send tokio                 365 ms    348 ns/message
cloned-send flume                 158 ms    151 ns/message
cloned-send flume                 171 ms    163 ns/message
cloned-send flume                 235 ms    224 ns/message
cloned-send flume                 270 ms    258 ns/message
cloned-send async-channel         164 ms    157 ns/message
cloned-send async-channel         227 ms    216 ns/message
cloned-send async-channel         245 ms    233 ns/message
cloned-send async-channel         282 ms    269 ns/message
cloned-send futures-channel       155 ms    148 ns/message
cloned-send futures-channel       221 ms    211 ns/message
cloned-send futures-channel       230 ms    219 ns/message
cloned-send futures-channel       286 ms    272 ns/message

# multi_thread,   tasks: 16, messages per task: 65536, capacity per task: 4096

cloned-send tokio                 593 ms    565 ns/message
cloned-send tokio                 631 ms    602 ns/message
cloned-send tokio                 638 ms    608 ns/message
cloned-send tokio                 650 ms    620 ns/message
cloned-send flume                 107 ms    102 ns/message
cloned-send flume                 131 ms    124 ns/message
cloned-send flume                 447 ms    426 ns/message
cloned-send flume                 487 ms    465 ns/message
cloned-send async-channel         218 ms    208 ns/message
cloned-send async-channel         236 ms    225 ns/message
cloned-send async-channel         247 ms    236 ns/message
cloned-send async-channel         311 ms    297 ns/message
cloned-send futures-channel       152 ms    144 ns/message
cloned-send futures-channel       198 ms    188 ns/message
cloned-send futures-channel       213 ms    203 ns/message
cloned-send futures-channel       226 ms    215 ns/message

# multi_thread,   tasks: 64, messages per task: 16384, capacity per task: 1024

cloned-send tokio                1103 ms   1052 ns/message
cloned-send tokio                1122 ms   1070 ns/message
cloned-send tokio                1142 ms   1090 ns/message
cloned-send tokio                1146 ms   1093 ns/message
cloned-send flume                 111 ms    106 ns/message
cloned-send flume                 147 ms    140 ns/message
cloned-send flume                 148 ms    141 ns/message
cloned-send flume                 198 ms    189 ns/message
cloned-send async-channel         146 ms    139 ns/message
cloned-send async-channel         203 ms    193 ns/message
cloned-send async-channel         205 ms    195 ns/message
cloned-send async-channel         224 ms    213 ns/message
cloned-send futures-channel       147 ms    140 ns/message
cloned-send futures-channel       153 ms    146 ns/message
cloned-send futures-channel       221 ms    210 ns/message
cloned-send futures-channel       342 ms    326 ns/message

# multi_thread,   tasks: 128, messages per task: 8192, capacity per task: 512

cloned-send tokio                1284 ms   1225 ns/message
cloned-send tokio                1319 ms   1258 ns/message
cloned-send tokio                1327 ms   1265 ns/message
cloned-send tokio                1335 ms   1273 ns/message
cloned-send flume                 125 ms    119 ns/message
cloned-send flume                 150 ms    143 ns/message
cloned-send flume                 159 ms    151 ns/message
cloned-send flume                 443 ms    423 ns/message
cloned-send async-channel         170 ms    162 ns/message
cloned-send async-channel         174 ms    166 ns/message
cloned-send async-channel         196 ms    187 ns/message
cloned-send async-channel         216 ms    206 ns/message
cloned-send futures-channel       124 ms    118 ns/message
cloned-send futures-channel       142 ms    135 ns/message
cloned-send futures-channel       164 ms    156 ns/message
cloned-send futures-channel       186 ms    178 ns/message

# multi_thread,   tasks: 256, messages per task: 4096, capacity per task: 256

cloned-send tokio                1359 ms   1296 ns/message
cloned-send tokio                1360 ms   1297 ns/message
cloned-send tokio                1369 ms   1306 ns/message
cloned-send tokio                1378 ms   1314 ns/message
cloned-send flume                 116 ms    111 ns/message
cloned-send flume                 133 ms    127 ns/message
cloned-send flume                 148 ms    141 ns/message
cloned-send flume                 554 ms    528 ns/message
cloned-send async-channel         173 ms    165 ns/message
cloned-send async-channel         193 ms    185 ns/message
cloned-send async-channel         228 ms    218 ns/message
cloned-send async-channel         240 ms    229 ns/message
cloned-send futures-channel       143 ms    137 ns/message
cloned-send futures-channel       148 ms    142 ns/message
cloned-send futures-channel       150 ms    143 ns/message
cloned-send futures-channel       230 ms    219 ns/message

# multi_thread,   tasks: 512, messages per task: 2048, capacity per task: 128

cloned-send tokio                 799 ms    762 ns/message
cloned-send tokio                 805 ms    767 ns/message
cloned-send tokio                 806 ms    768 ns/message
cloned-send tokio                 808 ms    770 ns/message
cloned-send flume                 313 ms    298 ns/message
cloned-send flume                 337 ms    321 ns/message
cloned-send flume                 356 ms    340 ns/message
cloned-send flume                 526 ms    502 ns/message
cloned-send async-channel         465 ms    443 ns/message
cloned-send async-channel         482 ms    460 ns/message
cloned-send async-channel         483 ms    460 ns/message
cloned-send async-channel         547 ms    522 ns/message
cloned-send futures-channel       233 ms    222 ns/message
cloned-send futures-channel       235 ms    224 ns/message
cloned-send futures-channel       235 ms    224 ns/message
cloned-send futures-channel       236 ms    225 ns/message
```
