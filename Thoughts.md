## 1. 配置文件化构建，更简单
example
```yaml
image: kaniko
dockerfile: 
```
## 2. 利用配置文件快速启动
## 需求(用于流程化组件)
1. 启动一个容器环境，开发rust和python，需要能够vscode remote 远程链接(通过dockerCompose解决)
2. 在kubernetes环境中快速启动(yaml可以直接解决)


## 改进
1. 显示出来构造日志(已完成)
2. 使用随机container名字，构建完成自动删除(已完成)