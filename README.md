## tools
```
1. k8s环境快速创建环境
pod
2. docker环境中快速创建环境
环境具备ssh 通过vscode remote ssh快速链接
镜像具备编译环境
3. linux环境快速构造环境
```

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
    build      利用giturl构建镜像并自动上传
    devlang    vscode-client remotessh install tools
    docker     docker容器启动一个ssh ubuntu:20.04镜像
    help       Print this message or the help of the given subcommand(s)
    quick      start a statefulset container quickly
    start      start a statefulset by use kubectl
```