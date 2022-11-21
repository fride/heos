# make use of .dockerignore ... especially the target directory
# remember to adjust stuff at the bottom as well
# release stripped              : 14M self-service-project-operator
#
# -1 (0.5 seconds):
# release stripped & compressed : 6.2M self-service-project-operator
#
# -9 (12 seconds):
# release stripped & compressed : 5.2M self-service-project-operator
#
# --brute (15 minutes!):
# release stripped & compressed : 3.7M self-service-project-operator
#
ARG COMPRESSION_FACTOR="-9"
ARG BIN=heos-axum
# sensible choices are scratch, docker.io/busybox (if you need a shell), docker.io/alpine (if you need a shell + package manager)
ARG RUNTIME_IMAGE=docker.io/busybox
# set to 1 if https requests don't work ... try without first
ARG USE_GETADDRINFO=1
ARG RUST_BUILDER_IMAGE=docker.io/rust:latest
#ARG TARGET=x86_64-unknown-linux-musl
#ARG TARGET=x86_64-unknown-linux-gnu
#ARG TARGET=aarch64-unknown-linux-gnu
ARG TARGET=
ARG ARTIFACT=target/${TARGET}/release/${BIN}
################################################### planner stage (collect dependencies)
FROM ${RUST_BUILDER_IMAGE} as planner
ARG TARGET
WORKDIR /app
RUN cargo install ${TARGET:+--target=}${TARGET} cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
################################################### cacher stage (build dependencies)
FROM ${RUST_BUILDER_IMAGE} as cacher
WORKDIR /app
ARG TARGET
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook ${TARGET:+--target=}${TARGET} --release --recipe-path recipe.json
################################################### builder stage (build binary)
FROM ${RUST_BUILDER_IMAGE} as builder
WORKDIR /app
ARG TARGET
ARG ARTIFACT
ARG BIN
ARG USE_GETADDRINFO
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY --from=cacher /app/target target
COPY . .
RUN cargo build ${TARGET:+--target=}${TARGET} --release --bin ${BIN}
RUN strip ${ARTIFACT}
# get all dynamic dependencies
RUN echo ${ARTIFACT} > /tmp/deps
RUN if [ "${USE_GETADDRINFO}" = "1" ]; then find / -name "libnss*" -o -name "libresolv*" -o -name "nsswitch.conf" >> /tmp/deps; fi
# recursively collect all dynamic dependency until we don't find more dependencies
# this is done by calling 'objdump -p <file>|grep NEEDED' on every dependency we have
RUN bash -c "\
    while ! diff /tmp/deps /tmp/new_deps &>/dev/null; do \
      mv -f /tmp/new_deps /tmp/deps 2>/dev/null || true;\
      while read file; do \
        echo \$file >> /tmp/new_deps_tmp;\
        deps=\$(objdump -p \$file 2>/dev/null|grep NEEDED|tr -s ' \t' '\t'|cut -f3);\
        test -n \"\${deps}\" && echo \${deps}|xargs -n 1 find /lib* /usr/lib* -name|sort|uniq >> /tmp/new_deps_tmp;\
      done < /tmp/deps;\
      cat /tmp/new_deps_tmp|sort|uniq|grep -v '^\$' > /tmp/new_deps;\
    done; \
    "
RUN bash -ec "\
    while read file; do\
      (set -x; install -Ds \$file /tmp/buildroot/\${file} 2>/dev/null || install -D \$file /tmp/buildroot/\${file});\
    done < <(cat /tmp/deps|grep -v '${ARTIFACT}');\
    touch /tmp/buildroot"
################################################### compressor stage (compress binary)
FROM docker.io/ubuntu as compressor
ARG ARTIFACT
ARG BIN
ARG COMPRESSION_FACTOR
WORKDIR /app
RUN apt-get update && apt-get install -y upx ca-certificates
COPY --from=builder /app/${ARTIFACT} /app/${ARTIFACT}
RUN upx ${COMPRESSION_FACTOR} ${ARTIFACT}
################################################### final stage (copy binary in run time image)
FROM ${RUNTIME_IMAGE} as runtime
ARG ARTIFACT
ARG BIN
COPY --from=builder /tmp/buildroot/ /
COPY --from=compressor /etc/ssl /etc/ssl
COPY --from=compressor /app/${ARTIFACT} /${BIN}
ENV BIN /${BIN}

CMD ["${BIN}"]
