ARG IMAGE=ubuntu:resolute-20260401
FROM $IMAGE

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc m4 make libc6-dev ca-certificates
