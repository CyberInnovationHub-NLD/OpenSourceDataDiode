FROM alpine
ARG file
ENV env_file=$file
COPY $env_file .
ENTRYPOINT ./$env_file