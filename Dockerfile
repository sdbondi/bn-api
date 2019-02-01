FROM quay.io/tarilabs/rust:1.30-alpine as builder

# create a new empty shell project
RUN USER=root cargo new --bin bn-api
WORKDIR /bn-api
# Copy the dependency lists
ADD Cargo.lock ./
ADD Cargo.docker.toml ./Cargo.toml

# this build step will cache our dependencies
RUN cargo build --release
RUN rm src/*.rs

# Add the actual source code
ADD api ./api/
ADD db ./db/
ADD tari-client ./tari-client/
ADD stripe ./stripe/
ADD logging ./logging/
ADD globee ./globee/
ADD embed_dirs_derive ./embed_dirs_derive/
ADD Cargo.lock Cargo.toml ./

RUN cargo build --release

# Create a base minimal image for adding our executables to
FROM quay.io/tarilabs/run:alpine as base
# Now create a new image with only the essentials and throw everything else away
FROM base
COPY --from=builder /bn-api/target/release/server /usr/bin/
COPY --from=builder /bn-api/target/release/bndb_cli /usr/bin/
ADD reset-database.sh /usr/bin/

CMD ["server"]
