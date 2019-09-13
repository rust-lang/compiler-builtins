FROM ubuntu:18.04
RUN dpkg --add-architecture arm64 && \
    sed -e "\,security.ubuntu.com/ubuntu,p;s,security.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -e "\,archive.ubuntu.com/ubuntu,p;s,archive.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -i /etc/apt/sources.list && \
    sed -e "\,security.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,archive.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,ports.ubuntu.com/ubuntu-ports,s.^deb .deb [ arch=arm64 ] .g" \
        -i /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc libc6-dev ca-certificates \
    gcc-aarch64-linux-gnu libc6-dev-arm64-cross \
    clang xz-utils \
    qemu-user-static && \
    cd /tmp && \
    apt-get download libclang-common-6.0-dev:arm64 && \
    ar xf libclang-common-6.0-dev_*_arm64.deb && \
    tar -C/ -xf data.tar.xz ."$(dirname $(clang -rtlib=compiler-rt --print-libgcc-file-name))"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER=qemu-aarch64-static \
    QEMU_LD_PREFIX=/usr/aarch64-linux-gnu \
    CLANG=clang \
    RUST_TEST_THREADS=1
