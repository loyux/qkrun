FROM docker.io/library/ubuntu:20.04
COPY --from=docker.io/library/rust:latest /usr/local/cargo /usr/local/cargo
COPY --from=docker.io/library/rust:latest /usr/local/rustup /usr/local/rustup
#直接加入vscode扩展和链接信息
COPY --from=localhost/vscode-extention:latest /root/.vscode-server /root/.vscode-server
RUN export PATH=$PATH:/usr/local/cargo/bin
RUN /usr/local/cargo/bin/rustup toolchain link x86_64-unknown-linux-gnu-160 /usr/local/rustup/toolchains/1.60.0-x86_64-unknown-linux-gnu/ && /usr/local/cargo/bin/rustup default x86_64-unknown-linux-gnu-160 &&apt update && apt install ssh -y
RUN sed -i 's/#Port 22/Port 20023/' /etc/ssh/sshd_config \
&& sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
&& sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config
RUN /bin/echo root:li |chpasswd
# ENTRYPOINT ["/bin/bash","-c","service ssh start && bash"]