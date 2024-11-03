FROM rust:1.80.1-bullseye AS build

# Project name
ARG PROJ_NAME="thermal-printer-tg-bot"
# Project target under c 
ARG PROJ_TARGET_C="arm-linux-gnueabihf" 
# Project target under rust 
ARG PROJ_TARGET_RUST="arm-unknown-linux-gnueabihf" 
# Same as PROJ_TARGET_RUST, but in CONSTANT_CASE 
ARG PROJ_TARGET_RUST_ENV="ARM_UNKNOWN_LINUX_GNUEABIHF" 
# CPU Type (see https://rust-lang.github.io/packed_simd/perf-guide/target-feature/rustflags.html#target-cpu)
ARG PROJ_CPU="arm1176jzf-s"

ENV CARGO_TARGET_${PROJ_TARGET_RUST_ENV}_LINKER="${PROJ_TARGET_C}-gcc" 
ENV CC="${PROJ_TARGET_C}-gcc" 
ENV CC_${PROJ_TARGET_RUST}="${PROJ_TARGET_C}-gcc" 
ENV CXX_${PROJ_TARGET_RUST}="${PROJ_TARGET_C}-g++"

RUN rustup target add ${PROJ_TARGET_RUST} && \
    update-ca-certificates

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./tools .

RUN RUSTFLAGS="-C target-cpu=${PROJ_CPU}" cargo build --package ${PROJ_NAME} --target ${PROJ_TARGET_RUST} --release; 
RUN mkdir -p output; \
    cp ./target/${PROJ_TARGET_RUST}/release/${PROJ_NAME} ./output;

FROM scratch
COPY --from=build /output /