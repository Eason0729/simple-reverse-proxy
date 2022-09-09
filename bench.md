# Unscientific Benchmark

On Ryzen5-3600 on windows docker(wsl)

```dockercompose
version: "3.9"
services:
  backend:
    build: simple-http-server
  proxy:
    depends_on:
    - "backend"
    build: simple-reverse-proxy
    environment:
      - SR_ADDR=0.0.0.0:80
      - THREAD=8
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1024M
  frontend:
    build: siege
    volumes:
      - .:/siege-log
    depends_on:
    - "proxy"
    command: sh /task.sh
```

### Command

```shell
siege -r10 -b -q -c64 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c128 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c192 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c256 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c320 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c384 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c448 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c512 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c576 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c640 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c704 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c768 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c832 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c896 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c960 --log=/siege-log/1M.log http://proxy/1M
siege -r10 -b -q -c1024 --log=/siege-log/1M.log http://proxy/1M
```

### Result

```csv
        Date & Time,  Trans,  Elap Time,  Data Trans,  Resp Time,  Trans Rate,  Throughput,  Concurrent,    OKAY,   Failed
2022-08-27 08:25:25,    640,       0.68,         640,       0.06,      941.18,      941.18,       55.88,     640,       0
2022-08-27 08:25:26,   1280,       1.21,        1280,       0.11,     1057.85,     1057.85,      116.43,    1280,       0
2022-08-27 08:25:28,   1920,       2.08,        1920,       0.19,      923.08,      923.08,      178.24,    1920,       0
2022-08-27 08:25:31,   2560,       2.65,        2560,       0.25,      966.04,      966.04,      241.01,    2560,       0
2022-08-27 08:25:34,   3200,       2.97,        3200,       0.28,     1077.44,     1077.44,      299.41,    3200,       0
2022-08-27 08:25:37,   3840,       3.73,        3840,       0.34,     1029.49,     1029.49,      354.94,    3840,       0
2022-08-27 08:25:41,   4480,       3.98,        4480,       0.37,     1125.63,     1125.63,      412.59,    4480,       0
2022-08-27 08:25:46,   5120,       4.51,        5120,       0.42,     1135.25,     1135.25,      480.19,    5120,       0
2022-08-27 08:25:51,   5760,       5.11,        5760,       0.48,     1127.20,     1127.20,      539.86,    5760,       0
2022-08-27 08:25:57,   6400,       5.65,        6400,       0.52,     1132.74,     1132.74,      594.51,    6400,       0
2022-08-27 08:26:03,   7040,       6.17,        7040,       0.57,     1141.00,     1141.00,      653.92,    7040,       0
2022-08-27 08:26:10,   7680,       6.75,        7680,       0.62,     1137.78,     1137.78,      709.44,    7680,       0
2022-08-27 08:26:17,   8320,       7.33,        8320,       0.68,     1135.06,     1135.06,      772.14,    8320,       0
2022-08-27 08:26:26,   8960,       8.37,        8960,       0.79,     1070.49,     1070.49,      840.38,    8960,       0
2022-08-27 08:26:34,   9600,       8.74,        9600,       0.82,     1098.40,     1098.40,      898.27,    9600,       0
2022-08-27 08:26:43,  10197,       9.06,       10196,       0.85,     1125.50,     1125.39,      953.74,   10197,      43
```
