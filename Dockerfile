FROM rust:1.82.0-bullseye AS build

# Project name
ARG PROJ_NAME="thermal-printer-tg-bot"
# Project target under c
ARG PROJ_TARGET_C="arm-linux-gnueabihf"
# Project target under rust
ARG PROJ_TARGET_RUST="arm-unknown-linux-gnueabihf"
# Same as PROJ_TARGET_RUST, but in CONSTANT_CASE
ARG PROJ_TARGET_RUST_ENV="ARM_UNKNOWN_LINUX_GNUEABIHF"

ENV CARGO_TARGET_${PROJ_TARGET_RUST_ENV}_LINKER="${PROJ_TARGET_C}-gcc" 
ENV CC="${PROJ_TARGET_C}-gcc" 
ENV CC_${PROJ_TARGET_RUST}="${PROJ_TARGET_C}-gcc" 
ENV CXX_${PROJ_TARGET_RUST}="${PROJ_TARGET_C}-g++"

RUN rustup target add ${PROJ_TARGET_RUST} && \
    apt update && \
    apt install -y gcc-${PROJ_TARGET_C} g++-${PROJ_TARGET_C} && \
    update-ca-certificates

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --package ${PROJ_NAME} --target ${PROJ_TARGET_RUST} --release; \
    mkdir -p output; \
    cp ./target/${PROJ_TARGET_RUST}/release/${PROJ_NAME} ./output;

FROM scratch
COPY --from=build /output /