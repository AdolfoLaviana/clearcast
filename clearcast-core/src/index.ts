// Type definitions for clearcast-core

type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

interface InitOutput {
  memory: WebAssembly.Memory;
  __wbg_wasmaudioengine_free(ptr: number): void;
  wasmaudioengine_new(): number;
  wasmaudioengine_with_settings(noise_threshold: number, target_level: number): number;
  wasmaudioengine_process_buffer(ptr: number, input_ptr: number, input_len: number): number;
  wasmaudioengine_compress(
    ptr: number,
    input_ptr: number,
    input_len: number,
    threshold: number,
    ratio: number,
    attack_ms: number,
    release_ms: number
  ): number;
  __wbindgen_malloc(size: number): number;
  __wbindgen_free(ptr: number, size: number): void;
  __wbindgen_realloc(ptr: number, old_size: number, size: number): number;
  __wbindgen_add_to_stack_pointer(offset: number): number;
  __wbindgen_exn_store(p: number): void;
  __wbindgen_start?(): void;
}

declare module 'clearcast-core' {
  /**
   * Initialize the WebAssembly module
   * @param module The WebAssembly module, URL, or buffer source
   * @returns Promise that resolves when the module is initialized
   */
  export default function init(module?: InitInput | Promise<InitInput>): Promise<InitOutput>;
  
  /**
   * Main audio processing engine for ClearCast
   */
  export class WasmAudioEngine {
    /**
     * Create a new audio engine with default settings
     */
    constructor();

    /**
     * Create a new audio engine with custom settings
     * @param noiseThreshold Threshold for noise reduction (0.0 to 1.0)
     * @param targetLevel Target normalization level (0.0 to 1.0)
     */
    static withSettings(noiseThreshold: number, targetLevel: number): Promise<WasmAudioEngine>;

    /**
     * Process an audio buffer
     * @param input Float32Array containing the audio samples
     * @returns Processed audio as Float32Array
     */
    processBuffer(input: Float32Array): Float32Array;

    /**
     * Apply compression to an audio buffer
     * @param input Float32Array containing the audio samples
     * @param threshold Compression threshold in dBFS (0 to -60)
     * @param ratio Compression ratio (e.g., 4.0 for 4:1)
     * @param attackMs Attack time in milliseconds (1.0 to 100.0)
     * @param releaseMs Release time in milliseconds (10.0 to 1000.0)
     * @returns Compressed audio as Float32Array
     */
    compress(
      input: Float32Array,
      threshold: number,
      ratio: number,
      attackMs: number,
      releaseMs: number
    ): Float32Array;
  }

  /**
   * Initialize the WebAssembly module
   * @returns Promise that resolves when the module is initialized
   */
  /**
   * WebAssembly module initialization options
   */
  export interface InitOptions {
    /**
     * The URL of the wasm file
     */
    url?: string;
    
    /**
     * Callback when the module is instantiated
     */
    onInstantiate?: (instance: WebAssembly.Instance) => void;
  }
}

// This is needed to make TypeScript treat this file as a module
export {};
