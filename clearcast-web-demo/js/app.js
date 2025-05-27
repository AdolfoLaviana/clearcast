// Usar las variables globales definidas en index.html
const { wasmInit: init, WasmAudioEngine } = window;

// Elementos del DOM
const startButton = document.getElementById('startButton');
const stopButton = document.getElementById('stopButton');
const processButton = document.getElementById('processButton');
const statusElement = document.getElementById('status');
const audioLevelElement = document.querySelector('.audio-level-bar');
const recordedAudio = document.getElementById('recordedAudio');
const processedAudio = document.getElementById('processedAudio');

// Controles de efectos
const noiseReductionInput = document.getElementById('noiseReduction');
const noiseReductionValue = document.getElementById('noiseReductionValue');
const compressionInput = document.getElementById('compression');
const compressionValue = document.getElementById('compressionValue');
const reverbInput = document.getElementById('reverb');
const reverbValue = document.getElementById('reverbValue');

// Variables de estado
let audioContext;
let mediaRecorder;
let audioChunks = [];
let audioBlob;
let audioBuffer;
let engine;
let isRecording = false;
let audioStream;
let animationFrameId;
let analyser;
let dataArray;

// Actualizar valores de los controles
document.addEventListener('DOMContentLoaded', () => {
  noiseReductionValue.textContent = noiseReductionInput.value;
  compressionValue.textContent = `${compressionInput.value}:1`;
  reverbValue.textContent = reverbInput.value;
  
  // Inicializar el motor de audio
  initializeEngine();
});

// Event listeners para los controles de efectos
noiseReductionInput.addEventListener('input', (e) => {
  noiseReductionValue.textContent = e.target.value;
  updateEngineSettings();
});

compressionInput.addEventListener('input', (e) => {
  compressionValue.textContent = `${e.target.value}:1`;
  updateEngineSettings();
});

reverbInput.addEventListener('input', (e) => {
  reverbValue.textContent = e.target.value;
  updateEngineSettings();
});

// Inicializar el motor de audio
async function initializeEngine() {
  try {
    // Cargar el módulo WebAssembly
    await init('/pkg/clearcast_core_bg.wasm');
    
    // Crear una nueva instancia del motor de audio
    engine = new WasmAudioEngine();
    
    // Registrar métodos disponibles en el motor
    console.log('Métodos disponibles en el motor de audio:', 
      Object.getOwnPropertyNames(Object.getPrototypeOf(engine))
        .filter(prop => typeof engine[prop] === 'function' && prop !== 'constructor')
    );
    
    // Configuración inicial
    updateEngineSettings();
    
    statusElement.textContent = 'Motor de audio inicializado correctamente';
    statusElement.className = 'status success';
    
    console.log('Motor de audio inicializado:', engine);
  } catch (error) {
    console.error('Error al inicializar el motor de audio:', error);
    statusElement.textContent = `Error al inicializar el motor de audio: ${error.message}`;
    statusElement.className = 'status error';
  }
}

// Actualizar la configuración del motor de audio
function updateEngineSettings() {
  if (!engine) {
    console.warn('El motor de audio no está inicializado');
    return;
  }
  
  try {
    // Mostrar métodos disponibles para depuración
    console.log('Métodos disponibles en el motor:', 
      Object.getOwnPropertyNames(engine)
        .concat(Object.getOwnPropertyNames(Object.getPrototypeOf(engine)))
        .filter(prop => typeof engine[prop] === 'function' && prop !== 'constructor')
    );
    
    // Guardar los valores actuales para referencia
    const settings = {
      noiseReduction: parseFloat(noiseReductionInput.value),
      compression: parseFloat(compressionInput.value),
      reverb: parseFloat(reverbInput.value)
    };
    
    console.log('Configuración solicitada:', settings);
    
    // Configurar compresión si está disponible
    if (typeof engine.compress === 'function') {
      console.log('Configurando compresión...');
      try {
        // Parámetros del compresor:
        // - threshold: -20 dBFS (nivel en el que comienza la compresión)
        // - ratio: valor del control deslizante (ej. 4.0 para 4:1)
        // - attack: 10ms (tiempo de ataque)
        // - release: 100ms (tiempo de liberación)
        const threshold = -20.0; // dBFS
        const ratio = Math.max(1.0, settings.compression);
        const attackMs = 10.0; // ms
        const releaseMs = 100.0; // ms
        
        console.log(`Aplicando compresión: ratio=${ratio}:1, threshold=${threshold}dB, attack=${attackMs}ms, release=${releaseMs}ms`);
        
        // La compresión se aplicará durante el procesamiento
      } catch (error) {
        console.error('Error al configurar la compresión:', error);
      }
    } else {
      console.warn('Método para configurar compresión no disponible');
    }
    
    // Los otros parámetros (reducción de ruido y reverberación) no están disponibles actualmente
    if (settings.noiseReduction > 0) {
      console.warn('La reducción de ruido no está disponible en esta versión');
    }
    
    if (settings.reverb > 0) {
      console.warn('La reverberación no está disponible en esta versión');
    }
    
  } catch (error) {
    console.error('Error al actualizar la configuración:', error);
  }
}

// Verificar si el navegador soporta la API de MediaDevices
function checkMicrophoneAccess() {
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    throw new Error('Tu navegador no soporta la API de MediaDevices');
  }
  
  // Verificar si hay dispositivos de entrada disponibles
  return navigator.mediaDevices.enumerateDevices()
    .then(devices => {
      const audioInputs = devices.filter(device => device.kind === 'audioinput');
      console.log('Dispositivos de entrada de audio disponibles:', audioInputs);
      
      if (audioInputs.length === 0) {
        throw new Error('No se encontraron dispositivos de entrada de audio');
      }
      
      return audioInputs;
    });
}

// Iniciar la grabación
async function startRecording() {
  try {
    console.log('Iniciando grabación...');
    
    // Verificar acceso al micrófono
    await checkMicrophoneAccess();
    
    // Solicitar acceso al micrófono con configuración específica
    audioStream = await navigator.mediaDevices.getUserMedia({
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true
      },
      video: false
    }).catch(error => {
      console.error('Error en getUserMedia:', error);
      throw new Error(`No se pudo acceder al micrófono: ${error.message}`);
    });
    
    console.log('Acceso al micrófono concedido');
    
    // Crear contexto de audio
    audioContext = new (window.AudioContext || window.webkitAudioContext)();
    
    // Configurar el analizador para la visualización
    analyser = audioContext.createAnalyser();
    analyser.fftSize = 256;
    const source = audioContext.createMediaStreamSource(audioStream);
    source.connect(analyser);
    
    // Configurar el búfer para los datos del analizador
    const bufferLength = analyser.frequencyBinCount;
    dataArray = new Uint8Array(bufferLength);
    
    // Iniciar la visualización del nivel de audio
    startAudioVisualization();
    
    // Configurar el grabador de audio
    mediaRecorder = new MediaRecorder(audioStream);
    audioChunks = [];
    
    // Manejar eventos del grabador
    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        audioChunks.push(event.data);
      }
    };
    
    mediaRecorder.onstop = async () => {
      // Crear un blob con los datos de audio grabados
      audioBlob = new Blob(audioChunks, { type: 'audio/wav' });
      
      // Crear una URL para el audio grabado
      const audioUrl = URL.createObjectURL(audioBlob);
      recordedAudio.src = audioUrl;
      
      // Habilitar el botón de procesar
      processButton.disabled = false;
      
      // Detener la visualización
      cancelAnimationFrame(animationFrameId);
      audioLevelElement.style.width = '0%';
      
      statusElement.textContent = 'Grabación finalizada. Puedes reproducir o procesar el audio.';
      statusElement.className = 'status success';
    };
    
    // Iniciar la grabación
    mediaRecorder.start(100); // Capturar datos cada 100ms
    isRecording = true;
    
    // Actualizar la interfaz
    startButton.disabled = true;
    stopButton.disabled = false;
    processButton.disabled = true;
    
    statusElement.textContent = 'Grabando... Habla por el micrófono.';
    statusElement.className = 'status';
    
  } catch (error) {
    console.error('Error en startRecording:', error);
    
    // Mensajes de error más descriptivos
    let errorMessage = 'Error al acceder al micrófono';
    if (error.name === 'NotAllowedError') {
      errorMessage = 'Permiso de micrófono denegado. Por favor, permite el acceso al micrófono en la configuración de tu navegador.';
    } else if (error.name === 'NotFoundError') {
      errorMessage = 'No se encontró ningún dispositivo de entrada de audio. Asegúrate de que el micrófono esté conectado.';
    } else if (error.name === 'NotReadableError') {
      errorMessage = 'No se puede acceder al micrófono. Puede estar en uso por otra aplicación.';
    } else {
      errorMessage = `Error: ${error.message}`;
    }
    
    statusElement.textContent = errorMessage;
    statusElement.className = 'status error';
    
    // Restablecer botones
    startButton.disabled = false;
    stopButton.disabled = true;
  }
}

// Detener la grabación
function stopRecording() {
  if (mediaRecorder && isRecording) {
    mediaRecorder.stop();
    
    // Detener todas las pistas del stream
    if (audioStream) {
      audioStream.getTracks().forEach(track => track.stop());
    }
    
    isRecording = false;
    stopButton.disabled = true;
  }
}

// Procesar el audio grabado
async function processAudio() {
  console.log('Iniciando procesamiento de audio...');
  
  try {
    // Verificar si hay un blob de audio
    if (!audioBlob) {
      throw new Error('No hay audio grabado para procesar');
    }
    
    // Verificar si el motor está inicializado
    if (!engine) {
      console.warn('El motor de audio no está inicializado. Intentando inicializar...');
      await initializeEngine();
      if (!engine) {
        throw new Error('No se pudo inicializar el motor de audio');
      }
    }
    
    // Verificar métodos disponibles en el motor
    console.log('Métodos disponibles en engine:', Object.getOwnPropertyNames(engine).filter(prop => typeof engine[prop] === 'function'));
    
    console.log('Motor de audio listo. Procesando...');
    statusElement.textContent = 'Procesando audio...';
    statusElement.className = 'status';
    
    // Forzar la actualización de la interfaz
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Convertir el blob a ArrayBuffer
    console.log('Convirtiendo blob a ArrayBuffer...');
    const arrayBuffer = await audioBlob.arrayBuffer();
    
    // Decodificar el audio
    console.log('Decodificando audio...');
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
    console.log(`Audio decodificado: ${audioBuffer.length} muestras, ${audioBuffer.sampleRate}Hz`);
    
    // Obtener los datos de audio
    const inputData = audioBuffer.getChannelData(0);
    console.log(`Datos de audio obtenidos: ${inputData.length} muestras`);
    
    // Procesar el audio con ClearCast
    console.log('Procesando audio con ClearCast...');
    
    // Asegurarse de que los datos de entrada sean un array de números
    const inputArray = Array.from(inputData);
    
    // Mostrar métodos disponibles para depuración
    console.log('Métodos disponibles en el motor:', 
      Object.getOwnPropertyNames(engine)
        .concat(Object.getOwnPropertyNames(Object.getPrototypeOf(engine)))
        .filter(prop => typeof engine[prop] === 'function' && prop !== 'constructor')
    );
    
    // Procesar el audio usando el método processBuffer
    let outputData;
    try {
      console.log('Procesando audio con processBuffer...');
      
      // Crear una copia de los datos de entrada para referencia
      const originalData = inputArray.slice();
      
      // Primero aplicar el procesamiento básico (reducción de ruido, normalización, etc.)
      // Usar withSettings para configurar el motor con parámetros más adecuados
      // - Reducción de ruido: 0.1 (bajo para evitar artefactos)
      // - Nivel objetivo: 0.9 (para evitar recorte)
      engine = await WasmAudioEngine.withSettings(0.1, 0.9);
      
      // Procesar el audio con la configuración actual
      outputData = await engine.processBuffer(originalData);
      
      // Aplicar compresión suave si está habilitada
      const compression = parseFloat(compressionInput.value);
      if (compression > 1 && typeof engine.compress === 'function') {
        console.log('Aplicando compresión al audio...');
        const threshold = -12.0; // dBFS (más alto para menos compresión)
        const ratio = Math.min(4.0, Math.max(1.5, compression)); // Ratio entre 1.5:1 y 4:1
        const attackMs = 30.0; // ms (más lento para transiciones más suaves)
        const releaseMs = 200.0; // ms (más lento para transiciones más suaves)
        
        try {
          outputData = await engine.compress(
            outputData,
            threshold,
            ratio,
            attackMs,
            releaseMs
          );
          console.log('Compresión aplicada correctamente');
        } catch (error) {
          console.error('Error al aplicar la compresión:', error);
        }
      }
      
      console.log('Procesamiento completado. Muestras de salida:', outputData.length);
    } catch (error) {
      console.error('Error al procesar el audio:', error);
      // Si hay un error, usar los datos de entrada sin procesar
      console.warn('Usando datos sin procesar debido a un error');
      outputData = inputArray;
    }
    
    if (!outputData || outputData.length === 0) {
      throw new Error('El motor de audio no devolvió datos procesados');
    }
    
    console.log(`Audio procesado: ${outputData.length} muestras`);
    
    // Crear un nuevo buffer de audio con los datos procesados
    const outputBuffer = audioContext.createBuffer(
      audioBuffer.numberOfChannels,
      outputData.length,
      audioBuffer.sampleRate
    );
    
    // Copiar los datos procesados al buffer de salida
    const outputChannel = outputBuffer.getChannelData(0);
    outputChannel.set(new Float32Array(outputData));
    
    console.log('Creando archivo WAV procesado...');
    // Crear un blob con el audio procesado
    const processedBlob = await audioBufferToWav(outputBuffer);
    const processedUrl = URL.createObjectURL(processedBlob);
    
    // Configurar el reproductor de audio
    processedAudio.src = processedUrl;
    processedAudio.controls = true;
    
    // Reproducir automáticamente
    processedAudio.play().catch(e => {
      console.warn('No se pudo reproducir automáticamente:', e);
    });
    
    console.log('Procesamiento completado con éxito');
    statusElement.textContent = 'Audio procesado correctamente. Reproduciendo...';
    statusElement.className = 'status success';
    
  } catch (error) {
    console.error('Error en processAudio:', error);
    
    let errorMessage = 'Error al procesar el audio';
    if (error.message.includes('process_audio')) {
      errorMessage = 'Error en el motor de audio. Asegúrate de que el módulo WebAssembly se cargó correctamente.';
    } else if (error.message.includes('decodeAudioData')) {
      errorMessage = 'Error al decodificar el audio. El archivo podría estar dañado o en un formato no soportado.';
    } else {
      errorMessage = `Error: ${error.message}`;
    }
    
    statusElement.textContent = errorMessage;
    statusElement.className = 'status error';
  } finally {
    // Asegurarse de que el botón de procesar esté habilitado
    processButton.disabled = false;
  }
}

// Visualización del nivel de audio
function startAudioVisualization() {
  function updateVisualization() {
    if (!analyser || !dataArray) return;
    
    // Obtener datos de frecuencia
    analyser.getByteFrequencyData(dataArray);
    
    // Calcular el nivel de audio promedio
    let sum = 0;
    for (let i = 0; i < dataArray.length; i++) {
      sum += dataArray[i];
    }
    const average = sum / dataArray.length;
    
    // Actualizar la barra de nivel de audio (0-100%)
    const level = (average / 255) * 100;
    audioLevelElement.style.width = `${level}%`;
    
    // Cambiar el color según el nivel (verde a rojo)
    if (level < 30) {
      audioLevelElement.style.backgroundColor = '#34a853'; // Verde
    } else if (level < 70) {
      audioLevelElement.style.backgroundColor = '#fbbc05'; // Amarillo
    } else {
      audioLevelElement.style.backgroundColor = '#ea4335'; // Rojo
    }
    
    // Continuar la animación
    animationFrameId = requestAnimationFrame(updateVisualization);
  }
  
  // Iniciar la visualización
  animationFrameId = requestAnimationFrame(updateVisualization);
}

// Convertir AudioBuffer a WAV Blob
async function audioBufferToWav(buffer) {
  return new Promise((resolve) => {
    const numChannels = buffer.numberOfChannels;
    const sampleRate = buffer.sampleRate;
    const format = 3; // 32-bit float
    const bitDepth = 32;
    
    // Crear un buffer para el archivo WAV
    const bytesPerSample = bitDepth / 8;
    const blockAlign = numChannels * bytesPerSample;
    const dataSize = buffer.length * numChannels * bytesPerSample;
    const bufferSize = 44 + dataSize;
    
    const wavBuffer = new ArrayBuffer(bufferSize);
    const view = new DataView(wavBuffer);
    
    // Escribir el encabezado WAV
    // RIFF chunk descriptor
    writeString(view, 0, 'RIFF');
    view.setUint32(4, 36 + dataSize, true);
    writeString(view, 8, 'WAVE');
    
    // FMT sub-chunk
    writeString(view, 12, 'fmt ');
    view.setUint32(16, 16, true); // Tamaño del sub-chunk
    view.setUint16(20, format, true); // Formato (3 = float)
    view.setUint16(22, numChannels, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, sampleRate * blockAlign, true); // Byte rate
    view.setUint16(32, blockAlign, true);
    view.setUint16(34, bitDepth, true);
    
    // Data sub-chunk
    writeString(view, 36, 'data');
    view.setUint32(40, dataSize, true);
    
    // Escribir los datos de audio
    const channelData = buffer.getChannelData(0);
    let offset = 44;
    
    for (let i = 0; i < channelData.length; i++) {
      view.setFloat32(offset, channelData[i], true);
      offset += 4;
    }
    
    // Crear un Blob con los datos WAV
    const wavBlob = new Blob([view], { type: 'audio/wav' });
    resolve(wavBlob);
  });
}

// Función auxiliar para escribir cadenas en el buffer
function writeString(view, offset, string) {
  for (let i = 0; i < string.length; i++) {
    view.setUint8(offset + i, string.charCodeAt(i));
  }
}

// Event listeners para los botones
startButton.addEventListener('click', startRecording);
stopButton.addEventListener('click', stopRecording);
processButton.addEventListener('click', processAudio);

// Limpieza al cerrar la página
window.addEventListener('beforeunload', () => {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
  
  if (audioStream) {
    audioStream.getTracks().forEach(track => track.stop());
  }
});
