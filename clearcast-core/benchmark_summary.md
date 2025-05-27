# Informe de Rendimiento de ClearCast

## Resumen de Benchmarks

Este informe muestra los resultados de rendimiento para diferentes operaciones de procesamiento de audio con varios tamaños de búfer.

### Configuración del Entorno
- **Fecha**: $(date)
- **Sistema**: $(uname -a)
- **CPU**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "No disponible")
- **Memoria**: $(sysctl -n hw.memsize 2>/dev/null | awk '{printf "%.2f GB", $0/1024/1024/1024}' || echo "No disponible")

### Resultados por Operación

### Resultados Detallados

Para ver los resultados completos, incluyendo gráficos y análisis estadístico, abra el siguiente archivo en su navegador:

file:///Users/loraruiz/Documents/GitHub/clearcast/clearcast-core/target/criterion/report/index.html
