# ClearCast Core Examples

This directory contains example applications that demonstrate how to use the ClearCast Core library in different environments.

## Browser Example

A web-based audio processing demo that runs in the browser using WebAssembly.

### Running the Browser Example

1. Build the WebAssembly module:
   ```bash
   npm run build
   ```

2. Start a local web server:
   ```bash
   cd examples/browser
   npx http-server -p 8080
   ```

3. Open `http://localhost:8080` in your web browser.

4. Upload an audio file and click "Process Audio" to apply compression.

## Node.js Example

A server-side example that processes audio files using Node.js and WebAssembly.

### Running the Node.js Example

1. Build the WebAssembly module:
   ```bash
   npm run build
   ```

2. Install dependencies:
   ```bash
   cd examples/node
   npm install
   ```

3. Place an input WAV file named `input.wav` in the `examples/node` directory.

4. Run the example:
   ```bash
   npm start
   ```

5. The processed audio will be saved as `output.wav` in the same directory.

## Example Audio Files

You can use any WAV file for testing. For best results, use 16-bit or 32-bit WAV files. The examples will automatically convert the audio to 32-bit float format if needed.

## Troubleshooting

- Ensure you have built the WebAssembly module before running the examples.
- For the browser example, make sure you're using a modern browser with WebAssembly support.
- For the Node.js example, ensure you have Node.js 14.0.0 or later installed.
- If you encounter any issues, please file a bug report in the [GitHub repository](https://github.com/yourusername/clearcast/issues).
