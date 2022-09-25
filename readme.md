# A simple reverse proxy

**This is not a production project**

**This is a project for me to learn**

## Get Started

1. Download the project and compile

```shell
git clone ...
```

2. Edit ``config.yml``

```yml
server:
  addr: "0.0.0.0:8081"
  thread: 4
hosts:
  a.example.com:
    header-rewrite: true # experimental
    routing:
      - 127.0.0.1:8000
      - www.example.com:8081
  www.example.com:
    routing:
      - 127.0.0.1:8000
```
Remove comments in yml file before execute the program

3. Ready to run

## Limitation

- Header size should be smaller than 8KiB
- Timeout depends on the upstream server

Limitation can be changed by editing constant in the source code.
