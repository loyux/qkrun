## 功能介绍
1. 利用容器、kubernetes、linux快速构建远程开发环境

## ENV
```
$HOME/.kube/config存在，try_default 将使用默认配置去连接kubernetes的api
```
## cli用法
```yaml
USAGE:
    quickrun <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    build      利用giturl构建镜像并自动上传至registry
    devlang    通过vscode的客户端快速安装语言开发插件
    docker     docker容器启动一个ssh ubuntu:20.04镜像，可作为远程开发环境
    help       Print this message or the help of the given subcommand(s)
    quick      快速启动一个带开发环境的pod，即用即销，带数据持久化
    start      启动一个完备高可用的pod作为开发环境  
```