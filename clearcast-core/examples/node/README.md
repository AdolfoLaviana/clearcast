# Node.js Example

This example demonstrates how to use the ClearCast Core library in a Node.js environment to process audio files.

## Prerequisites

- Node.js 14.0.0 or later
- npm or yarn
- The ClearCast Core WebAssembly module (built with `npm run build`)

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Place an input WAV file named `input.wav` in this directory.
   - The example expects a mono or stereo WAV file.
   - The audio will be converted to 32-bit float format if needed.

3. Run the example:
   ```bash
   npm start
   ```

## How It Works

1. The script loads the input WAV file and converts it to 32-bit float format if necessary.
2. It initializes the ClearCast WebAssembly module.
3. The audio is processed using the `WasmAudioEngine`.
4. Compression is applied with the following settings:
   - Threshold: -20 dBFS
   - Ratio: 4:1
   - Attack: 10 ms
   - Release: 100 ms
5. The processed audio is saved as `output.wav` in the same directory.

## Customization

You can modify the compression settings in `index.js`:

```javascript
const compressed = engine.compress(
  output,
  -20.0,  // threshold (dBFS)
  4.0,    // ratio (e.g., 4.0 for 4:1)
  10.0,   // attack (ms)
  100.0   // release (ms)
);
```

## Troubleshooting

- If you get an error about the WebAssembly module not loading, make sure you've built the project with `npm run build`.
- Ensure the input file exists and is a valid WAV file.
- Check the console output for any error messages.

## License

This example is part of the ClearCast Core project and is available under the same license.
