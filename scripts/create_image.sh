#!/bin/bash
#Create docker image for input file.
#Target is a build with libc
#Save docker image as tar
echo $1
docker image rm $1
cp ../target/x86_64-unknown-linux-musl/release/$1 .
docker build -t $1 . --build-arg file=$1
rm $1
docker save $1 > dockers/$1.tar