Buildkit tcp:// + TLS C/S构建模式
对openssl进行CA证书签证ip或者dns修改用于远程主机
i>buildkitd --addr tcp://0.0.0.0:1234 --tlscacert ca.cert --tlscert server.cert --tlskey server.key

ii>buildctl --addr tcp://115.159.115.244:1234  --tlscacert ca.cert   --tlscert client.cert   --tlskey client.key build -frontend gateway.v0 --opt source=docker/dockerfile --opt context=https://github.com/loyurs/qkrun.git#master:docker/rust_code_server/ --output type=image,name=docker.io/liloyu/yuxin520:vk

