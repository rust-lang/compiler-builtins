FROM ubuntu:18.04

RUN dpkg --add-architecture ppc64el && \
    sed -e "\,security.ubuntu.com/ubuntu,p;s,security.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -e "\,archive.ubuntu.com/ubuntu,p;s,archive.ubuntu.com/ubuntu,ports.ubuntu.com/ubuntu-ports,g" \
        -i /etc/apt/sources.list && \
    sed -e "\,security.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,archive.ubuntu.com/ubuntu,s.^deb .deb [ arch=amd64,i386 ] .g" \
        -e "\,ports.ubuntu.com/ubuntu-ports,s.^deb .deb [ arch=ppc64el ] .g" \
        -i /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc libc6-dev ca-certificates \
    gcc-powerpc64le-linux-gnu libc6-dev-ppc64el-cross \
    clang xz-utils \
    qemu-user-static qemu-system-ppc && \
    cd /tmp && \
    apt-get download libclang-common-6.0-dev:ppc64el && \
    ar xf libclang-common-6.0-dev_*_ppc64el.deb && \
    tar -C/ -xf data.tar.xz ."$(dirname $(clang -rtlib=compiler-rt --print-libgcc-file-name))"

ENV CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=powerpc64le-linux-gnu-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_RUNNER=qemu-ppc64le-static \
    QEMU_CPU=POWER8 \
    QEMU_LD_PREFIX=/usr/powerpc64le-linux-gnu \
    CLANG=clang \
    RUST_TEST_THREADS=1
