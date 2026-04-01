ARG IMAGE=ubuntu:questing-20251217
FROM $IMAGE

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc-multilib m4 make libc6-dev ca-certificates
