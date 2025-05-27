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
   * Process an audio buffer with all enabled effects
   * 
   * # Arguments
   * * `input` - A Float32Array containing the audio samples
   * 
   * # Returns
   * A new Float32Array with the processed audio
   */
  processBuffer(input: Float32Array): Float32Array;
  /**
   * Apply gentle compression to an audio buffer
   * 
   * This function applies RMS compression to control the dynamic range of the audio.
   * It helps maintain a consistent volume level and prevents clipping.
   * 
   * # Arguments
   * * `input` - A Float32Array containing the audio samples
   * * `threshold` - Compression threshold in dBFS (-60 to 0)
   * * `ratio` - Compression ratio (1.0 to 20.0)
   * * `attack_ms` - Attack time in milliseconds (1.0 to 100.0)
   * * `release_ms` - Release time in milliseconds (10.0 to 2000.0)
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
  readonly wasmaudioengine_withSettings: (a: number, b: number, c: number) => void;
  readonly wasmaudioengine_processBuffer: (a: number, b: number, c: number, d: number) => void;
  readonly wasmaudioengine_compress: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
