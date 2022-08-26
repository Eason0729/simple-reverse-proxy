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
THREAD=4
```

4. Ready to run

## Limitation

- Header size should be smaller than 8KiB
- Timeout depends on the upstream server

Limitation can be changed by editing constant in the source code.
