FROM nixos/nix:latest AS builder

COPY . /tmp/build
WORKDIR /tmp/build

RUN nix \
  --extra-experimental-features "nix-command flakes" \
  --option filter-syscalls false \
  build

RUN mkdir /tmp/nix-store-closure && \
  cp -R $(nix-store -qR result/) /tmp/nix-store-closure

FROM scratch

WORKDIR /app

COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /tmp/build/result /app

ENV PATH="/app/bin:${PATH}"

ENTRYPOINT ["iliad"]
