// WebGL 2 detection for the optional 3D cutaway (prompt/TECHNICAL_ARCHITECTURE.md Section 8: the 2D
// cross-section is the fallback). The test context is released at once.

let cached: boolean | null = null;

export function hasWebGL2(): boolean {
  if (cached !== null) return cached;
  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl2');
    cached = gl !== null && gl !== undefined;
    gl?.getExtension('WEBGL_lose_context')?.loseContext();
  } catch {
    cached = false;
  }
  return cached;
}

/** Tests only: forget the cached answer. */
export function resetWebGL2Cache(): void {
  cached = null;
}
