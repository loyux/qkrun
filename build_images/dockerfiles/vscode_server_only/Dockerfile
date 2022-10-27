#将ubuntu -> sshd_ubuntu && listening 8022
FROM docker.io/library/ubuntu:20.04
RUN apt update && apt install ssh -y
RUN sed -i 's/#Port 22/Port 8022/' /etc/ssh/sshd_config \
&& sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
&& sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config && apt clean && apt autoremove
RUN /bin/echo root:li |chpasswd