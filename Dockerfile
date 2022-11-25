FROM scratch

LABEL \
    org.opencontainers.image.authors="Richard Liebscher <richard.liebscher@gmail.com>" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.url="https://github.com/R1tschY/game-of-estimates" \
    org.opencontainers.image.source="https://github.com/R1tschY/game-of-estimates"

COPY target/x86_64-unknown-linux-musl/release/game-of-estimates game-of-estimates

ENTRYPOINT ["/game-of-estimates"]