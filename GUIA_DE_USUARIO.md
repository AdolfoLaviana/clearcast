# Guía de Usuario de ClearCast Core

## Tabla de Contenidos
1. [Introducción](#introducción)
2. [Instalación](#instalación)
3. [Primeros Pasos](#primeros-pasos)
4. [Procesamiento Básico de Audio](#procesamiento-básico-de-audio)
5. [Uso de Efectos](#uso-de-efectos)
6. [Optimización de Rendimiento](#optimización-de-rendimiento)
7. [Solución de Problemas](#solución-de-problemas)
8. [Ejemplos Prácticos](#ejemplos-prácticos)
9. [API de Referencia](#api-de-referencia)
10. [Recursos Adicionales](#recursos-adicionales)

## Introducción

ClearCast Core es una biblioteca de procesamiento de audio de alto rendimiento escrita en Rust, con soporte para WebAssembly. Está diseñada para aplicaciones que requieren procesamiento de audio en tiempo real, como editores de audio, aplicaciones de streaming y herramientas de producción musical.

## Instalación

### Para Proyectos Node.js/Web

```bash
npm install clearcast-core
# o
yarn add clearcast-core
```

### Para Proyectos Rust

Agrega esto a tu `Cargo.toml`:

```toml
[dependencies]
clearcast-core = { version = "0.1", path = "../clearcast-core" }  # Ruta local
# o desde crates.io
# clearcast-core = "0.1"
```

## Primeros Pasos

### En JavaScript/TypeScript

```javascript
import init, { WasmAudioEngine } from 'clearcast-core';

async function setupAudio() {
  // Inicializar el módulo WebAssembly
  await init();
  
  // Crear un nuevo motor de audio
  const engine = new WasmAudioEngine();
  
  // Tu código de procesamiento de audio aquí
  
  return engine;
}
```

### En Rust

```rust
use clearcast_core::AudioEngine;

fn main() {
    let engine = AudioEngine::new();
    // Tu código de procesamiento de audio aquí
}
```

## Procesamiento Básico de Audio

### Cargar y Procesar un Archivo de Audio

```javascript
async function processAudioFile(file) {
  const audioContext = new (window.AudioContext || window.webkitAudioContext)();
  const arrayBuffer = await file.arrayBuffer();
  const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
  
  // Convertir a Float32Array para procesamiento
  const input = audioBuffer.getChannelData(0);
  
  // Procesar con ClearCast
  const output = engine.processBuffer(input);
  
  return output;
}
```

## Uso de Efectos

### Añadir un Efecto de Delay

```javascript
import { Delay } from 'clearcast-core';

// Crear un efecto de delay
const delay = new Delay(
  300,    // 300ms de retraso
  0.5,    // 50% de retroalimentación
  0.3,    // 30% de señal procesada
  0.7,    // 70% de señal seca
  44100   // Frecuencia de muestreo
);

// Añadir al motor
document.getElementById('addDelay').addEventListener('click', () => {
  engine.addEffect(delay);
});
```

### Configurar un Ecualizador

```javascript
import { Equalizer } from 'clearcast-core';

const eq = new Equalizer(44100);
eq.setGain(1000, 6.0);  // Aumentar 6dB a 1kHz
engine.addEffect(eq);
```

## Optimización de Rendimiento

### Tamaño de Buffer Recomendado

```javascript
// Para aplicaciones en tiempo real (baja latencia)
const REALTIME_BUFFER_SIZE = 128;  // ~2.9ms a 44.1kHz

// Para procesamiento por lotes (mayor rendimiento)
const BATCH_BUFFER_SIZE = 4096;    // ~93ms a 44.1kHz
```

### Procesamiento por Hilos de Trabajo

```javascript
// En el hilo principal
const worker = new Worker('audio-worker.js');
worker.postMessage({ type: 'init' });

// En audio-worker.js
importScripts('clearcast-core.js');

self.onmessage = async (e) => {
  if (e.data.type === 'init') {
    await wasm_bindgen(wasm);
    const engine = new wasm_bindgen.WasmAudioEngine();
    // Procesamiento en segundo plano
  }
};
```

## Solución de Problemas

### Problemas Comunes

1. **Audio Distorsionado**
   - Verifica los niveles de ganancia
   - Asegúrate de que no haya recorte (clipping)
   - Revisa la configuración de limitación

2. **Latencia Alta**
   - Reduce el tamaño del buffer
   - Verifica el rendimiento de efectos individuales
   - Considera el uso de Web Workers

3. **Problemas de Inicialización**
   - Asegúrate de esperar a que el módulo WASM se cargue
   - Verifica las frecuencias de muestreo compatibles

## Ejemplos Prácticos

### Grabación de Micrófono

```javascript
async function startRecording() {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  const audioContext = new AudioContext();
  const source = audioContext.createMediaStreamSource(stream);
  
  const processor = audioContext.createScriptProcessor(2048, 1, 1);
  
  processor.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0);
    const output = engine.processBuffer(input);
    e.outputBuffer.getChannelData(0).set(output);
  };
  
  source.connect(processor);
  processor.connect(audioContext.destination);
}
```

### Reproducción en Tiempo Real

```javascript
function playProcessedAudio(processedBuffer) {
  const audioContext = new AudioContext();
  const source = audioContext.createBufferSource();
  
  const audioBuffer = audioContext.createBuffer(
    1,  // Canales
    processedBuffer.length,
    audioContext.sampleRate
  );
  
  audioBuffer.getChannelData(0).set(processedBuffer);
  source.buffer = audioBuffer;
  source.connect(audioContext.destination);
  source.start();
}
```

## API de Referencia

### Clases Principales

#### `WasmAudioEngine` (JavaScript) / `AudioEngine` (Rust)
- `new()` - Crea una nueva instancia del motor de audio
- `processBuffer(input: Float32Array): Float32Array` - Procesa un buffer de audio
- `addEffect(effect: AudioEffect): void` - Añade un efecto a la cadena de procesamiento
- `clearEffects(): void` - Elimina todos los efectos

#### `Delay`
- `new(delayMs, feedback, wet, dry, sampleRate)` - Crea un nuevo efecto de delay
- `setDelayMs(ms: number): void` - Establece el tiempo de retardo
- `setFeedback(amount: number): void` - Establece la cantidad de retroalimentación

## Recursos Adicionales

- [Documentación Técnica](DOCUMENTACION_TECNICA.md)
- [Guía de Desarrollo](GUIA_DESARROLLO.md)
- [Ejemplos en el Repositorio](/examples)
- [Reportar un Problema](https://github.com/tuusuario/clearcast/issues)

---

Esta guía está diseñada para ayudarte a comenzar rápidamente con ClearCast Core. Para obtener información más detallada sobre la API y características avanzadas, consulta la documentación técnica completa.
