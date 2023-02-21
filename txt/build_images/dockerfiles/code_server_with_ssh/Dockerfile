FROM docker.io/linuxserver/code-server:latest
RUN apt update && apt install ssh -y  \
&& sed -i 's/#Port 22/Port 8022/' /etc/ssh/sshd_config \
&& sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
&& sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config \
&& sed -i '1c root:x:0:0:root:/config:/bin/bash' /etc/passwd 
#将root的目录修改为/config方便映射,配置完成ssh