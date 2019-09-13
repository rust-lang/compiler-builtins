FROM ubuntu:18.04
RUN dpkg --add-architecture armhf && \
    sed -e "\,security.ubuntu.com/ubuntu,p;s,security.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -e "\,archive.ubuntu.com/ubuntu,p;s,archive.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -i /etc/apt/sources.list && \
    sed -e "\,security.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,archive.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,ports.ubuntu.com/ubuntu-ports,s.^deb .deb [ arch=armhf ] .g" \
        -i /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc libc6-dev ca-certificates \
    gcc-arm-linux-gnueabihf libc6-dev-armhf-cross \
    clang xz-utils \
    qemu-user-static && \
    cd /tmp && \
    apt-get download libclang-common-6.0-dev:armhf && \
    ar xf libclang-common-6.0-dev_*_armhf.deb && \
    tar -C/ -xf data.tar.xz ."$(dirname $(clang -rtlib=compiler-rt --print-libgcc-file-name))"
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUNNER=qemu-arm-static \
    QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf \
    CLANG=clang \
    RUST_TEST_THREADS=1
