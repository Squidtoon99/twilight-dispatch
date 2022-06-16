FROM docker.io/library/alpine:latest AS builder

RUN apk add --no-cache curl clang gcc musl-dev lld cmake make && \
    curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain stable -y

ENV CC clang
ENV CFLAGS "-I/usr/lib/gcc/x86_64-alpine-linux-musl/10.3.1 -L/usr/lib/gcc/x86_64-alpine-linux-musl/10.3.1/"
ENV RUSTFLAGS "-C link-arg=-fuse-ld=lld -C target-cpu=haswell"

RUN rm /usr/bin/ld && \
    rm /usr/bin/cc && \
    ln -s /usr/bin/lld /usr/bin/ld && \
    ln -s /usr/bin/clang /usr/bin/cc && \
    ln -s /usr/lib/gcc/x86_64-alpine-linux-musl/10.3.1/crtbeginS.o /usr/lib/crtbeginS.o && \
    ln -s /usr/lib/gcc/x86_64-alpine-linux-musl/10.3.1/crtendS.o /usr/lib/crtendS.o

WORKDIR /build

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./.cargo ./.cargo

RUN mkdir src/
RUN echo 'fn main() {}' > ./src/main.rs
RUN source $HOME/.cargo/env && \
    cargo build --release

RUN rm -f target/release/deps/twilight_dispatch*

COPY ./src ./src

RUN source $HOME/.cargo/env && \
    cargo build --release && \
    strip /build/target/release/twilight-dispatch

FROM docker.io/library/alpine:edge AS dumb-init

RUN apk update &&  \
    VERSION=$(apk search dumb-init) && \
    mkdir out && \
    cd out && \
    wget "https://dl-cdn.alpinelinux.org/alpine/edge/community/x86_64/$VERSION.apk" -O dumb-init.apk && \
    tar xf dumb-init.apk && \
    mv usr/bin/dumb-init /dumb-init

FROM drone/ca-certs

COPY --from=builder /build/target/release/twilight-dispatch /twilight-dispatch
COPY --from=dumb-init /dumb-init /dumb-init

ENTRYPOINT ["./dumb-init", "--"]
CMD ["./twilight-dispatch"]


# FROM rust:1.60 as builder

# RUN USER=root cargo new --bin twilight-dispatch
# WORKDIR ./twilight-dispatch
# COPY ./Cargo.toml ./Cargo.toml
# RUN cargo build --release
# RUN rm src/*.rs

# ADD . ./

# RUN rm ./target/release/deps/twilight_dispatch*
# RUN cargo build --release


# FROM debian:buster-slim
# ARG APP=/usr/src/app

# RUN apt-get update \
#     && apt-get install -y ca-certificates tzdata \
#     && rm -rf /var/lib/apt/lists/*

# EXPOSE 8000

# ENV TZ=Etc/UTC \
#     APP_USER=appuser

# RUN groupadd $APP_USER \
#     && useradd -g $APP_USER $APP_USER \
#     && mkdir -p ${APP}

# COPY --from=builder /twilight-dispatch/target/release/twilight-dispatch ${APP}/twilight-dispatch

# RUN chown -R $APP_USER:$APP_USER ${APP}

# USER $APP_USER
# WORKDIR ${APP}


# CMD ["./twilight-dispatch"]