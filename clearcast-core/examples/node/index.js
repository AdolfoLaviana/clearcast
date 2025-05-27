// Node.js example for ClearCast Core
const fs = require('fs').promises;
const path = require('path');
const { performance } = require('perf_hooks');
const { WaveFile } = require('wavefile');

// Import the compiled WebAssembly module
const { default: init, WasmAudioEngine } = require('../../pkg/clearcast_core');

async function main() {
  console.log('ClearCast Core - Node.js Example');
  console.log('==============================\n');

  try {
    // Initialize the WebAssembly module
    console.log('Initializing WebAssembly module...');
    await init();
    
    // Create a new audio engine
    console.log('Creating audio engine...');
    const engine = new WasmAudioEngine();
    
    // Alternatively, create with custom settings
    // const engine = await WasmAudioEngine.withSettings(0.1, 0.9);
    
    // Read the input WAV file
    const inputPath = path.join(__dirname, 'input.wav');
    console.log(`Reading input file: ${inputPath}`);
    
    const wavData = await fs.readFile(inputPath);
    const wav = new WaveFile(wavData);
    
    // Convert to 32-bit float if needed
    if (wav.fmt.bitsPerSample !== 32 || wav.fmt.float !== 1) {
      wav.toBitDepth('32f');
    }
    
    // Get the first channel (mono)
    const channelData = wav.getSamples(true)[0];
    const sampleRate = wav.fmt.sampleRate;
    
    console.log(`Processing ${channelData.length} samples at ${sampleRate}Hz...`);
    
    // Process the audio
    const startTime = performance.now();
    
    // Process with ClearCast
    const output = engine.processBuffer(channelData);
    
    // Apply compression
    const compressed = engine.compress(
      output,
      -20.0,  // threshold: -20dBFS
      4.0,    // ratio: 4:1
      10.0,   // attack: 10ms
      100.0   // release: 100ms
    );
    
    const endTime = performance.now();
    console.log(`Processing completed in ${(endTime - startTime).toFixed(2)}ms`);
    
    // Create a new WAV file with the processed audio
    const outputWav = new WaveFile();
    outputWav.fromScratch(
      1,                      // number of channels
      sampleRate,             // sample rate
      '32f',                  // bit depth
      [new Float32Array(compressed)]  // samples
    );
    
    // Save the processed audio
    const outputPath = path.join(__dirname, 'output.wav');
    await fs.writeFile(outputPath, outputWav.toBuffer());
    
    console.log(`\nProcessing complete!`);
    console.log(`- Input file: ${inputPath}`);
    console.log(`- Output file: ${outputPath}`);
    console.log(`- Sample rate: ${sampleRate}Hz`);
    console.log(`- Duration: ${(compressed.length / sampleRate).toFixed(2)}s`);
    
  } catch (error) {
    console.error('An error occurred:', error);
    process.exit(1);
  }
}

// Run the example
main();
