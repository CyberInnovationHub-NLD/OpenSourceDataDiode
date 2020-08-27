#!/bin/bash
cargo build --release --all
cargo build --target x86_64-unknown-linux-musl --release

./create_images.sh

tar -czvf osdd_ingress.tar.gz osdd.service ../target/release/osdd ../settings/ingress/Config.toml dockers/ph_kafka_ingress.tar dockers/transport_udp_send.tar  dockers/ph_mock_ingress.tar dockers/ph_udp_ingress.tar dockers/filter.tar
tar -czvf osdd_egress.tar.gz osdd.service ../target/release/osdd ../settings/egress/Config.toml dockers/ph_kafka_egress.tar  dockers/transport_udp_receive.tar dockers/ph_mock_egress.tar dockers/ph_udp_egress.tar dockers/filter.tar
