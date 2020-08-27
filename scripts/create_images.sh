#!/bin/bash
#Create docker images images from last version of project.

INGRESS_DOCKERS_IMAGES=( ph_kafka_ingress transport_udp_send ph_mock_ingress ph_udp_ingress filter)
EGRESS_DOCKERS_IMAGES=( ph_kafka_egress transport_udp_receive ph_mock_egress ph_udp_egress filter)

for i in "${INGRESS_DOCKERS_IMAGES[@]}"
do
	./create_image.sh $i
done

for i in "${EGRESS_DOCKERS_IMAGES[@]}"
do
	./create_image.sh $i
done
