/* tslint:disable */
/* eslint-disable */
/**
 * WebAssembly bindings for ClearCast core functionality
 */
export class WasmAudioEngine {
  free(): void;
  /**
   * Create a new audio engine with default settings
   */
  constructor();
  /**
   * Create a new audio engine with custom settings
   * 
   * # Arguments
   * * `noise_threshold` - Threshold for noise reduction (0.0 to 1.0)
   * * `target_level` - Target normalization level (0.0 to 1.0)
   */
  static withSettings(noise_threshold: number, target_level: number): WasmAudioEngine;
  /**
   * Process an audio buffer
   * 
   * # Arguments
   * * `input` - A Float32Array containing the audio samples
   * 
   * # Returns
   * A new Float32Array with the processed audio
   */
  processBuffer(input: Float32Array): Float32Array;
  /**
   * Apply compression to an audio buffer
   * 
   * # Arguments
   * * `input` - A Float32Array containing the audio samples
   * * `threshold` - Compression threshold in dBFS (0 to -60)
   * * `ratio` - Compression ratio (e.g., 4.0 for 4:1)
   * * `attack_ms` - Attack time in milliseconds (1.0 to 100.0)
   * * `release_ms` - Release time in milliseconds (10.0 to 1000.0)
   * 
   * # Returns
   * A new Float32Array with the compressed audio
   */
  compress(input: Float32Array, threshold: number, ratio: number, attack_ms: number, release_ms: number): Float32Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmaudioengine_free: (a: number, b: number) => void;
  readonly wasmaudioengine_new: () => number;
  readonly wasmaudioengine_withSettings: (a: number, b: number) => [number, number, number];
  readonly wasmaudioengine_processBuffer: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmaudioengine_compress: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
