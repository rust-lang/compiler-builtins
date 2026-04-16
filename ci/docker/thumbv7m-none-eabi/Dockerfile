ARG IMAGE=ubuntu:resolute-20260413
FROM $IMAGE

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc libc6-dev ca-certificates \
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi
ENV BUILD_ONLY=1
