#!/bin/bash

# Script para ejecutar los benchmarks de ClearCast y generar un informe

# Configuración
BENCHMARK_TARGET="target/release/benchmark"
REPORT_DIR="target/criterion"
HTML_REPORT="target/criterion/report/index.html"
SUMMARY_FILE="benchmark_summary.md"

# Asegurarse de que estamos en el directorio correcto
cd "$(dirname "$0")"

# Construir el benchmark en modo release
echo "🔨 Construyendo el benchmark en modo release..."
cargo build --release --all-features --benches

# Crear directorio para informes si no existe
mkdir -p "$REPORT_DIR"

# Ejecutar los benchmarks
echo "🚀 Ejecutando benchmarks..."
cargo criterion --message-format=json | tee benchmark_results.json

# Generar un resumen de los resultados
echo "📊 Generando resumen de resultados..."

# Crear un archivo markdown con los resultados
cat > "$SUMMARY_FILE" << 'EOL'
# Informe de Rendimiento de ClearCast

## Resumen de Benchmarks

Este informe muestra los resultados de rendimiento para diferentes operaciones de procesamiento de audio con varios tamaños de búfer.

### Configuración del Entorno
- **Fecha**: $(date)
- **Sistema**: $(uname -a)
- **CPU**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "No disponible")
- **Memoria**: $(sysctl -n hw.memsize 2>/dev/null | awk '{printf "%.2f GB", $0/1024/1024/1024}' || echo "No disponible")

### Resultados por Operación
EOL

# Procesar los resultados y agregarlos al resumen
jq -r '
  select(.reason == "benchmark-complete") | 
  "#### \(.id)

- **Tiempo**: \(.typical.estimate / 1000) µs/iter
- **Rendimiento**: \(.throughput.Elements / 1000) K samples/s
"' benchmark_results.json >> "$SUMMARY_FILE"

# Agregar información sobre cómo ver los resultados completos
echo -e "\n### Resultados Detallados\n\nPara ver los resultados completos, incluyendo gráficos y análisis estadístico, abra el siguiente archivo en su navegador:\n\nfile://$(pwd)/$HTML_REPORT" >> "$SUMMARY_FILE"

# Limpiar archivo temporal
rm -f benchmark_results.json

echo "✅ Benchmark completado exitosamente!"
echo "📄 Resumen generado en: $SUMMARY_FILE"
echo "📊 Informe completo disponible en: file://$(pwd)/$HTML_REPORT"

# Intentar abrir el informe en el navegador predeterminado (solo macOS)
if [[ "$(uname)" == "Darwin" ]]; then
    open "$HTML_REPORT" 2>/dev/null || echo "No se pudo abrir el informe automáticamente. Por favor, ábralo manualmente."
fi
