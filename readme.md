# A simple reverse proxy

**This is not a production project**

**This is a project for me to learn**

## Get Started

1. Download the project and compile

```shell
git clone 
```

2. Edit ``config.properties``

```properties
sub1.example.com=127.0.0.1:8001
sub2.example.com=mybackend.example.com:80
```

3. Set environment variable

```env
SR_ADDR=0.0.0.0:80
```

4. Ready to run

## Limitation

- Header size should be smaller than 8KiB
- Whole request size should be smaller than 256MiB(except ``websocket``)
- Default poll rate is ``8ms``, meaning that websocket may have additional ``8ms`` latency
- If ``Keep-Alive`` is unspecified, it would be default to ``2sec``
- No matter what the request is, every connection have the maximum timeout of ``7200sec``

Limitation can be changed by editing constant in the source code.
