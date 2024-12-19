FROM rust:1.81 AS builder

WORKDIR /usr/src/blockclock-backend
COPY . .
RUN cargo install --path .

FROM --platform=linux/amd64 ubuntu:22.04

RUN echo 'APT::Install-Suggests "0";' >> /etc/apt/apt.conf.d/00-docker
RUN echo 'APT::Install-Recommends "0";' >> /etc/apt/apt.conf.d/00-docker
RUN DEBIAN_FRONTEND=noninteractive \
   apt-get update \
   && rm -rf /var/lib/apt/lists/*
 
USER root

RUN apt-get update
RUN apt-get install -y ca-certificates

# Copy certificates to connect to the ethereum network
COPY certificates/* /usr/local/share/ca-certificates/
RUN update-ca-certificates

COPY --from=builder /usr/local/cargo/bin/blockclock-backend /usr/local/bin/blockclock-backend

EXPOSE 8000/tcp
CMD "blockclock-backend" "--port=8000" "--chain-id=${CHAIN_ID}" "--mysql-user=${MYSQL_USER}" "--mysql-password=${MYSQL_PASSWORD}" "--mysql-host=${MYSQL_HOST}" "--mysql-port=${MYSQL_PORT}" "--mysql-database=${MYSQL_DATABASE}" "--time-window=${TIME_WINDOW}" "--solver-private-key=${SOLVER_PRIVATE_KEY}" "--ws-chain-url=${WS_CHAIN_URL}" "--block-time-address=${BLOCK_TIME_ADDRESS}" "--tick-period=${TICK_PERIOD}"
