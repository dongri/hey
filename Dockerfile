FROM rust
LABEL maintainer="dongrify@gmail.com"

ADD . /source
WORKDIR /source

CMD cargo run
