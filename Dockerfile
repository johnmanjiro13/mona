FROM amd64/rust:1.60.0

RUN apt-get update && apt-get install -y \
  build-essential
