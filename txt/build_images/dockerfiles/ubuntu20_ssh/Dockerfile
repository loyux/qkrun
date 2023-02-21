FROM ubuntu:20.04
RUN apt update && apt install ssh -y  \
&& sed -i 's/#Port 22/Port 8022/' /etc/ssh/sshd_config \
&& sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
&& sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config \
&& echo "root:li"|chpasswd \
&& apt install curl -y && apt-get update && \
export DEBIAN_FRONTEND=noninteractive && \
apt-get install -yq \
        build-essential \
        cmake \
        curl \
        file \
        git \
        graphviz \
        musl-dev \
        musl-tools \
        libpq-dev \
        libsqlite-dev \
        libssl-dev \
        linux-libc-dev \
        pkgconf \
        sudo \
        unzip \
        xutils-dev
