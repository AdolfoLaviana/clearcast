#!/bin/bash
set -e

# Limpiar compilaciones anteriores
cargo clean

# Compilar el proyecto con wasm-pack
wasm-pack build \
  --target web \
  --release \
  --out-dir ../clearcast-www/pkg \
  --no-typescript \
  --no-pack \
  --no-default-features \
  -- --no-default-features

echo "\n✅ Compilación completada exitosamente!"
echo "Los archivos generados se encuentran en: ../clearcast-www/pkg/"
