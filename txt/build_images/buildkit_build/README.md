1. 利用buildkit 监听tcp端口进行远端云服务构建
Buildkit tcp:// + TLS C/S构建模式
对openssl进行CA证书签证ip或者dns修改用于远程主机
buildkitd --addr tcp://0.0.0.0:1234 --tlscacert ca.cert --tlscert server.cert --tlskey server.key

sudo buildctl --addr tcp://115.159.115.244:1234  --tlscacert ca.cert   --tlscert client.cert   --tlskey client.key build -frontend gateway.v0 --opt source=docker/dockerfile --local context=. --local dockerfile=. --output type=image,name=ccr.ccs.tencentyun.com/pubcloud/ubuntu20.04:ssh,push=true

1. 通过buildkit k8s进行构建

2. 本地构建
