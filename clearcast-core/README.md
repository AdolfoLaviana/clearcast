# ClearCast Core

[![Crates.io](https://img.shields.io/crates/v/clearcast-core)](https://crates.io/crates/clearcast-core)
[![docs.rs](https://img.shields.io/docsrs/clearcast-core)](https://docs.rs/clearcast-core)
[![License](https://img.shields.io/crates/l/clearcast-core)](LICENSE)
[![Build Status](https://github.com/yourusername/clearcast/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/clearcast/actions)

ClearCast Core es una biblioteca de procesamiento de audio de alto rendimiento escrita en Rust, con enlaces WebAssembly para aplicaciones web y Node.js. Diseñada para ser rápida, segura y fácil de usar, ClearCast Core es ideal para aplicaciones que requieren procesamiento de audio en tiempo real.

## Características Principales

### 🎛️ Procesamiento de Audio
- **Reducción de ruido** - Eliminación inteligente de ruido de fondo
- **Normalización** - Ajuste automático de niveles de audio
- **Limitación suave** - Previene el recorte (clipping) sin distorsión
- **Compresión** - Control dinámico de rango con detección RMS
- **Filtros** - Varios filtros de audio integrados (pasa-bajo, pasa-alto, etc.)

### 🌐 Multiplataforma
- **Rust Nativo** - Máximo rendimiento en aplicaciones nativas
- **WebAssembly** - Ejecución en navegadores y Node.js
- **Seguro para Hilos** - Diseñado para procesamiento en paralelo

### 🛠️ Fácil Integración
- **API Sencilla** - Interfaz intuitiva para JavaScript/TypeScript
- **Tipos Fuertes** - Definiciones de TypeScript incluidas
- **Sin Copias** - Transferencia de datos eficiente entre JS y WASM
- **Sistema de Efectos** - Arquitectura modular para efectos personalizados

### 🎚️ Efectos Integrados
- **Delay/Eco** - Retrasos configurables con retroalimentación
- **Más en desarrollo** - Reverb, chorus, flanger y más próximamente

## Tabla de Contenidos
- [Instalación](#instalación)
- [Uso Rápido](#uso-rápido)
- [API](#api)
  - [Motor de Audio](#motor-de-audio)
  - [Efectos](#efectos)
- [Ejemplos](#ejemplos)
- [Construcción](#construcción)
- [Rendimiento](#rendimiento)
- [Contribución](#contribución)
- [Licencia](#licencia)

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
clearcast-core = { path = "../clearcast-core" }  # Ruta local
# o desde crates.io (cuando se publique)
# clearcast-core = "0.1"
```

## Uso Rápido

### En JavaScript/TypeScript

```javascript
import init, { WasmAudioEngine, Delay } from 'clearcast-core';

async function processAudio(audioBuffer) {
  // Inicializar el módulo WebAssembly
  await init();
  
  // Crear un nuevo motor de audio
  const engine = new WasmAudioEngine();
  
  // Añadir un efecto de delay
  const delay = new Delay(300, 0.5, 0.3, 0.7, 44100); // 300ms, 50% feedback, 30% wet, 70% dry, 44.1kHz
  engine.addEffect(delay);
  
  // Procesar audio
  const input = new Float32Array(audioBuffer);
  const output = engine.processBuffer(input);
  
  return output;
}
```

### En Rust

```rust
use clearcast_core::{
    AudioEngine, 
    effects::{Delay, AudioEffect}
};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Crear un nuevo motor de audio
    let mut engine = AudioEngine::new();
    
    // Añadir un efecto de delay
    let delay = Delay::new(300.0, 0.5, 0.3, 0.7, 44100);
    engine.add_effect(delay.boxed());
    
    // Procesar algunas muestras de audio
    let input = vec![0.1, -0.2, 0.3, -0.4, 0.5];
    let output = engine.process(input)?;
    
    println!("Audio procesado: {:?}", output);
    Ok(())
}
```

## API

### Motor de Audio

#### `AudioEngine`

El componente principal para el procesamiento de audio.

**Métodos:**

- `new() -> Self`
  Crea una nueva instancia del motor de audio con configuración predeterminada.

- `with_settings(noise_threshold: f32, target_peak: f32) -> Result<Self, AudioProcessingError>`
  Crea un nuevo motor con configuración personalizada.
  - `noise_threshold`: Umbral para la reducción de ruido (0.0 a 1.0)
  - `target_peak`: Nivel pico objetivo para normalización (0.0 a 1.0)

- `process(&self, input: Vec<f32>) -> Result<Vec<f32>, AudioProcessingError>`
  Procesa un búfer de audio, aplicando reducción de ruido, normalización y efectos.

- `add_effect(&mut self, effect: Arc<Mutex<dyn AudioEffect>>)`
  Añade un efecto a la cadena de procesamiento.

- `clear_effects(&mut self)`
  Elimina todos los efectos de la cadena de procesamiento.

### Efectos

#### `Delay`

Efecto de delay/eco configurable.

**Métodos:**

- `new(delay_ms: f32, feedback: f32, wet: f32, dry: f32, sample_rate: u32) -> Self`
  Crea un nuevo efecto de delay.
  - `delay_ms`: Tiempo de retardo en milisegundos
  - `feedback`: Cantidad de retroalimentación (0.0 a 1.0)
  - `wet`: Nivel de señal procesada (0.0 a 1.0)
  - `dry`: Nivel de señal original (0.0 a 1.0)
  - `sample_rate`: Tasa de muestreo en Hz

- `set_delay_ms(&mut self, delay_ms: f32)`
  Establece un nuevo tiempo de retardo.

- `set_feedback(&mut self, feedback: f32)`
  Establece la cantidad de retroalimentación.

- `set_mix(&mut self, wet: f32, dry: f32)`
  Establece la relación entre señal procesada y original.

## Ejemplos

### Aplicar múltiples efectos

```rust
use clearcast_core::{
    AudioEngine,
    effects::{Delay, AudioEffect}
};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = AudioEngine::new();
    
    // Añadir un delay
    let delay = Delay::new(300.0, 0.5, 0.3, 0.7, 44100);
    engine.add_effect(delay.boxed());
    
    // Procesar audio
    let input = vec![0.1, -0.2, 0.3, -0.4, 0.5];
    let output = engine.process(input)?;
    
    Ok(())
}
```

### Uso con WebAudio API

```javascript
// En el navegador
const audioContext = new (window.AudioContext || window.webkitAudioContext)();

async function setupAudioProcessing() {
  await init();
  const engine = new WasmAudioEngine();
  
  // Añadir efectos
  const delay = new Delay(300, 0.5, 0.3, 0.7, audioContext.sampleRate);
  engine.addEffect(delay);
  
  // Configurar nodos de audio
  const source = audioContext.createBufferSource();
  const processor = audioContext.createScriptProcessor(4096, 1, 1);
  
  processor.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0);
    const output = engine.processBuffer(input);
    e.outputBuffer.getChannelData(0).set(output);
  };
  
  source.connect(processor);
  processor.connect(audioContext.destination);
  
  return { source, processor };
}
```

## Construcción

### Para Web/Node.js

```bash
# Instalar wasm-pack si no lo tienes
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Instalar dependencias
npm install

# Construir el paquete WebAssembly
npm run build
```

Esto creará un directorio `pkg` con el módulo WebAssembly compilado y los enlaces de JavaScript.

### Para Rust Nativo

```bash
# Construir en modo release para máximo rendimiento
cargo build --release
```

La biblioteca compilada estará disponible en `target/release/libclearcast_core.rlib` para enlazado estático.

## Rendimiento

ClearCast Core está optimizado para ofrecer el máximo rendimiento:

- **Procesamiento por lotes** - Procesa muestras en bloques para mejor rendimiento
- **Seguro para hilos** - Puede procesar múltiples flujos de audio en paralelo
- **Sin asignaciones en tiempo real** - Evita asignaciones de memoria durante el procesamiento

### Benchmarks

```bash
# Ejecutar benchmarks
cargo bench
```

## Contribución

¡Las contribuciones son bienvenidas! Por favor lee nuestra [guía de contribución](CONTRIBUTING.md) para empezar.

## Licencia

ClearCast Core está disponible bajo los términos de la licencia MIT o Apache 2.0, a tu elección.

Ver [LICENSE-MIT](LICENSE-MIT) o [LICENSE-APACHE](LICENSE-APACHE) para más detalles.

## Examples

This repository includes several examples demonstrating how to use ClearCast Core in different environments. See the [examples](./examples/README.md) directory for more information.

### Browser Example

A web-based audio processing demo that runs in the browser using WebAssembly.

```bash
# Build the WebAssembly module
npm run build

# Start a local web server
cd examples/browser
npx http-server -p 8080
```

Then open `http://localhost:8080` in your web browser.

### Node.js Example

A server-side example that processes audio files using Node.js and WebAssembly.

```bash
# Build the WebAssembly module
npm run build

# Run the Node.js example
cd examples/node
npm install
npm start
```

This will process the `input.wav` file in the examples/node directory and save the result as `output.wav`.

## Testing

### Run JavaScript Tests

```bash
npm test
```

### Run Rust Tests

```bash
cargo test
```

## Browser Example

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>ClearCast Core Demo</title>
</head>
<body>
  <input type="file" id="audioInput" accept="audio/*">
  <button id="processButton" disabled>Process Audio</button>
  <audio id="audioPlayer" controls></audio>
  
  <script type="module">
    import init, { WasmAudioEngine } from './pkg/clearcast_core.js';
    
    let audioContext;
    let engine;
    
    async function initApp() {
      await init();
      engine = new WasmAudioEngine();
      document.getElementById('processButton').disabled = false;
    }
    
    document.getElementById('audioInput').addEventListener('change', (e) => {
      const file = e.target.files[0];
      if (!file) return;
      
      const url = URL.createObjectURL(file);
      document.getElementById('audioPlayer').src = url;
    });
    
    document.getElementById('processButton').addEventListener('click', async () => {
      const audioElement = document.getElementById('audioPlayer');
      if (!audioElement.src) return;
      
      // Process the audio
      const audioBuffer = await processAudio(audioElement);
      
      // Play the processed audio
      const blob = new Blob([audioBuffer], { type: 'audio/wav' });
      const url = URL.createObjectURL(blob);
      audioElement.src = url;
      audioElement.play();
    });
    
    async function processAudio(audioElement) {
      if (!audioContext) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)();
      }
      
      const response = await fetch(audioElement.src);
      const arrayBuffer = await response.arrayBuffer();
      const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
      
      // Get the first channel
      const channelData = audioBuffer.getChannelData(0);
      
      // Process with ClearCast
      const output = engine.processBuffer(channelData);
      
      // Apply compression
      const compressed = engine.compress(
        output,
        -20.0,  // threshold
        4.0,    // ratio
        10.0,   // attack (ms)
        100.0   // release (ms)
      );
      
      return compressed;
    }
    
    // Initialize the app
    initApp();
  </script>
</body>
</html>
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
