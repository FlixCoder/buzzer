FROM docker.io/rust:slim-bullseye as builder
ARG CARGO_NET_GIT_FETCH_WITH_CLI=true
ARG SQLX_OFFLINE=1

RUN apt update -yqq \
	&& apt install -yqq --no-install-recommends \
	build-essential cmake libssl-dev pkg-config git \
	&& rustup update \
	&& rustup toolchain add stable \
	&& rustup default stable

COPY . /app
WORKDIR /app
RUN cargo build --release


FROM debian:bullseye-slim
RUN apt update && apt install -y ca-certificates libgomp1

RUN mkdir -p /opt/app
WORKDIR /opt/app
COPY --from=builder /app/target/release/buzzer /usr/local/bin/buzzer
CMD ["/usr/local/bin/buzzer"]
