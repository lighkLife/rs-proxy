# rs-proxy
A simple tcp proxy service.

![rs-proxy](./doc/rs-proxy.png)

## Why rs-proxy
`Smaller, faster and more convenient. Save your "memory"!`

## Usage

```shell
./rs-proxy -h
rs-proxy 0.2.0
a simple tcp proxy service

USAGE:
    rs-proxy [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>    rs-proxy.toml file
    -h, --help               Print help information
    -V, --version            Print version information

```
## Config
```toml
[[proxy]]
name = "example1" 
listen = 21883
target = "127.0.0.1:1883"

[[proxy]]
enable = false # Optional, default true, set false to disable current proxy
name = "example2"
listen = 22883
target = "127.0.0.1:2883"
```

## Examples
Communication across network segments.

![rs-proxy-mqtt](./doc/rs-proxy-mqtt.png)

## Build
```shell
# build for x86 linux64
cargo build --release  --target x86_64-unknown-linux-gnu

# build for x86 windows64
cargo build --release  --target x86_64-pc-windows-gnu

# build for x86 windows32
cargo build --release  --target x86_64-pc-windows-gnu
```
```


