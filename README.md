# devicecheck

这是一个适用于`iOS`/`iPad`设备的`HTTP`中间人代理，用于抓取`device_token`。现已支持跨平台运行（Windows/Linux/macOS）。

### 前言

最新版的`ChatGPT` APP已上[`SSL pinning`](https://medium.com/trendyol-tech/securing-ios-applications-with-ssl-pinning-38d551945306)验证，使用前提:

- `iOS`/`iPad`设备需要越狱或者已经安装[`巨魔`](https://github.com/opa334/TrollStore)（**越狱后也可以安装**）
- 在[`巨魔`](https://github.com/opa334/TrollStore)商店安装[`TrollFools`](https://github.com/Lessica/TrollFools)，下载[`👉 动态库`](https://github.com/penumbra-x/devicecheck/releases/download/lib/SSLKillSwitch2.dylib)注入到`ChatGPT`

以上只是推荐的方法，当然也有其它方法，目的是绕过[`SSL pinning`](https://medium.com/trendyol-tech/securing-ios-applications-with-ssl-pinning-38d551945306)

### 跨平台兼容性

本项目现已完全支持跨平台运行：

#### Windows系统
- 无需管理员权限
- 服务文件存储在临时目录（`%TEMP%`）
- 使用`taskkill`命令停止服务
- 支持后台服务模式

#### Unix系统（Linux/macOS）
- 真正的守护进程
- 需要root权限运行守护进程
- 使用POSIX信号优雅关闭
- 服务文件存储在`/var/run/`目录

### 命令

#### 通用命令（所有平台）
```bash
$ devicecheck -h
chatgpt preauth devicecheck server

Usage: devicecheck
       devicecheck <COMMAND>

Commands:
  run      Run server
  start    Start server daemon
  restart  Restart server daemon
  stop     Stop server daemon
  log      Show the server daemon log
  status   Show the server daemon process (Windows)
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ devicecheck run -h
Run server

Usage: devicecheck run [OPTIONS]

Options:
  -d, --debug          Debug mode
  -b, --bind <BIND>    Bind address [default: 0.0.0.0:1080]
  -p, --proxy <PROXY>  Upstream proxy
      --cert <CERT>    MITM server CA certificate file path [default: ca/cert.crt]
      --key <KEY>      MITM server CA private key file path [default: ca/key.pem]
  -h, --help           Print help
```

#### 平台特定命令差异

**Windows系统:**
```bash
devicecheck status  # 查看服务状态
```

**Unix系统（Linux/macOS）:**
```bash
sudo devicecheck start    # 需要root权限启动守护进程
sudo devicecheck stop     # 需要root权限停止守护进程
devicecheck ps            # 查看守护进程状态
```

### 安装

#### 前置要求
所有平台都需要安装Rust工具链（需要Rust 1.75+）:

**Windows系统:**
```bash
# 下载并运行rustup安装器
# https://rustup.rs/
```

**Unix系统（Linux/macOS）:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 编译安装
```bash
# 从源码编译安装
cargo install --git https://github.com/penumbra-x/devicecheck

# 或者本地编译
git clone https://github.com/penumbra-x/devicecheck
cd devicecheck
cargo build --release
```

#### Windows特别说明
- 无需额外的系统依赖
- 编译后的可执行文件为`devicecheck.exe`
- 可在普通用户权限下运行

### 使用

该代理不会像正常代理一样提供网络代理，目的是抓包`device_token`。如果害怕使用多了会被封设备，我建议是使用一些一键换机之类的仿冒设备的软件。

#### 1. 启动服务

**直接运行服务（所有平台）:**
```bash
# 基础运行
devicecheck run

# 带代理运行
devicecheck run --proxy http://192.168.1.1:1080

# Windows系统示例
devicecheck.exe run --debug
```

**后台服务模式:**

*Windows系统:*
```bash
# 启动后台服务
devicecheck start

# 带代理启动
devicecheck start --proxy http://192.168.1.1:1080

# 查看服务状态
devicecheck status

# 查看服务日志
devicecheck log

# 停止服务
devicecheck stop
```

*Unix系统（Linux/macOS）:*
```bash
# 启动守护进程（需要root权限）
sudo devicecheck start

# 带代理启动
sudo devicecheck start --proxy http://192.168.1.1:1080

# 查看守护进程状态
devicecheck ps

# 查看日志
devicecheck log

# 停止守护进程
sudo devicecheck stop
```

#### 2. 设置代理

`Wi-Fi`/`Shadowrocket`设置`HTTP`代理

#### 3. 信任证书

浏览器打开`http://192.168.1.100:1080/mitm/cert`，替换你的代理`IP`以及`端口`，打开下载安装以及信任证书。到这里就彻底完成了，由于`Hook`了`ChatGPT`的网络请求，有以下两种抓取更新`device_token`的动作:

- 每次打开和关闭`APP`都会抓取一次，
- 打开`APP`任意点击登录会抓取一次，同理点击取消往复操作也生效。

#### 4. 获取`preauth_cookie`

请求接口`http://192.168.1.100:1080/auth/preauth`，替换你的代理`IP`以及`端口`，示例:

**Request:**
```bash
curl http://127.0.0.1:1080/auth/preauth
```

**Response:**
```json
{
  "preauth_cookie": "900175BB-61C4-4AA2-B400-4DE3B2E1FD7E:1726892032-9nYJ1mU4JSUAEyhACbVOxYoCATD4uXX8H1HZRJzYQ4E%3D"
}
```

到这里项目的使命已经完成，你可以将`preauth_cookie`用在`ios.chat.openai.com`的接口或者登录。

### 注意事项

#### 通用注意事项
- 自动化操作APP使用不需要太频繁，`cookie`大概会在一段时间内过期（具体不记得什么时间了，24小时？）
- 建议不要把服务放到公网，内网使用Cloudflare [Tunnel](https://www.cloudflare.com/zh-cn/products/tunnel/)开放`/auth/preauth`接口

#### Windows特别注意事项
- Windows Defender可能会误报，需要添加信任
- 防火墙可能会阻止端口访问，需要手动允许
- 服务日志和PID文件位于`%TEMP%\devicecheck.*`
- 无需管理员权限即可运行

#### 故障排查
- **Windows**: 检查`%TEMP%\devicecheck.out`和`%TEMP%\devicecheck.err`日志文件
- **Unix**: 检查`/var/run/auth.out`和`/var/run/auth.err`日志文件
- 确保防火墙允许指定端口的入站连接
- 检查证书文件是否正确生成在`ca/`目录
