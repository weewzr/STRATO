import { Canvas, useFrame, useThree } from '@react-three/fiber';
import { useEffect, useMemo, useRef, type KeyboardEvent, type PointerEvent as ReactPointerEvent } from 'react';
import * as THREE from 'three';
import { LAYER_TOKEN_NAMES } from '../design/layers';
import type { RunOk } from '../solver/types';
import { useStrato } from '../state/store';
import { cellColours, cellOfBins } from './colormap';

// Optional 3D cutaway of the particle (prompt/DESIGN_SPECIFICATION.md Section 4.1;
// prompt/TECHNICAL_ARCHITECTURE.md Section 4): nested spherical shells in the layer colours with a
// quarter wedge removed; the two cut faces are painted with the solver's concentration at the
// playback row through the same colour mapping as the 2D view (each radius takes the colour of the
// mesh cell that contains it). Loaded lazily, only when the reader picks the 3D view. The scene is
// rendered on demand: playback updates a texture and asks for one frame, and React does not
// re-render per frame. Orbit by dragging, arrow keys rotate, + and - zoom, 0 resets the view.
// Multisampling is used only with a GPU below a device pixel ratio of 2 (see useAntialias); it
// changes edge smoothing only, never a colour of the cut faces.

/** Radius bins of the cut-face texture. */
const BINS = 1024;
const WEDGE_START = Math.PI / 2;
const WEDGE_LENGTH = (3 * Math.PI) / 2;
// Seen from above and in front of the open wedge, turned so that both cut faces and the outer shell show.
export const DEFAULT_VIEW = { azimuth: -0.35, polar: 0.8, distance: 3.6 };
const ROTATE_STEP = Math.PI / 18;
const ZOOM_STEP = 1.15;
const MIN_DIST = 1.8;
const MAX_DIST = 8;

/**
 * Whether this browser renders WebGL in software (SwiftShader, llvmpipe and similar): Chromium then
 * refuses a context that asks for `failIfMajorPerformanceCaveat`, and the renderer string names the
 * rasteriser. Multisampling multiplies the fragment work of a software rasteriser, so the view
 * draws without it there; with a GPU it keeps multisampling below a device pixel ratio of 2 (at 2 or
 * more the denser pixels already smooth the edges).
 */
export function softwareWebGL(): boolean {
  try {
    const probe = document.createElement('canvas');
    const fast = probe.getContext('webgl2', { failIfMajorPerformanceCaveat: true });
    if (!fast) return true;
    const ext = fast.getExtension('WEBGL_debug_renderer_info');
    const name = String(fast.getParameter(ext ? ext.UNMASKED_RENDERER_WEBGL : fast.RENDERER));
    fast.getExtension('WEBGL_lose_context')?.loseContext();
    return /swiftshader|llvmpipe|softpipe|software/i.test(name);
  } catch {
    return true;
  }
}

export function useAntialias(): boolean {
  return useMemo(() => !softwareWebGL() && (window.devicePixelRatio || 1) < 2, []);
}

export interface OrbitState {
  azimuth: number;
  polar: number;
  distance: number;
}

function layerColours(): THREE.Color[] {
  const style = getComputedStyle(document.documentElement);
  return LAYER_TOKEN_NAMES.map((t) => new THREE.Color(style.getPropertyValue(t).trim() || '#888888'));
}

const vertex = /* glsl */ `
  varying vec2 vPos;
  void main() {
    vPos = position.xy;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

// The cut face is a half disc of radius 1 (the outer radius); the colour of radius r is the texel
// of the radius bin that contains it. No lighting and no tone mapping: the colours are the table's.
const fragment = /* glsl */ `
  uniform sampler2D lut;
  varying vec2 vPos;
  void main() {
    float r = length(vPos);
    if (r > 1.0 || vPos.x < 0.0) discard;
    gl_FragColor = texture2D(lut, vec2(r, 0.5));
  }
`;

/** Writes the colour of each radius bin (the colour of the cell that contains it) into the texture. */
function fillTexture(texture: THREE.DataTexture, colours: readonly (readonly [number, number, number])[], bins: readonly number[]): void {
  const data = texture.image.data as Uint8Array;
  for (let k = 0; k < BINS; k++) {
    const rgb = colours[bins[k] ?? 0] ?? [128, 128, 128];
    data[4 * k] = rgb[0];
    data[4 * k + 1] = rgb[1];
    data[4 * k + 2] = rgb[2];
    data[4 * k + 3] = 255;
  }
  texture.needsUpdate = true;
}

function Scene({ run, orbit, onFrame }: { run: RunOk; orbit: { current: OrbitState }; onFrame: () => void }) {
  const { camera, invalidate } = useThree();
  const faces = run.mesh.cell_faces_m;
  const outer = faces[faces.length - 1] ?? 1;
  const regions = run.inputs.regions;

  const texture = useMemo(() => {
    const tex = new THREE.DataTexture(new Uint8Array(BINS * 4), BINS, 1, THREE.RGBAFormat, THREE.UnsignedByteType);
    tex.magFilter = THREE.NearestFilter;
    tex.minFilter = THREE.NearestFilter;
    tex.generateMipmaps = false;
    tex.needsUpdate = true;
    return tex;
  }, []);
  const bins = useMemo(() => cellOfBins(faces, BINS), [faces]);
  const material = useMemo(
    () => new THREE.ShaderMaterial({ uniforms: { lut: { value: texture } }, vertexShader: vertex, fragmentShader: fragment, side: THREE.DoubleSide }),
    [texture],
  );

  const shells = useMemo(
    () =>
      regions.map((r, i) => {
        const geo = new THREE.SphereGeometry(r.outer_radius_m / outer, 64, 32, WEDGE_START, WEDGE_LENGTH);
        const mat = new THREE.MeshBasicMaterial({
          transparent: true,
          opacity: i === regions.length - 1 ? 0.3 : 0.14,
          side: THREE.DoubleSide,
          depthWrite: false,
        });
        return new THREE.Mesh(geo, mat);
      }),
    [regions, outer],
  );

  // Layer boundaries on the two cut faces: half circles in the layer colours.
  const arcs = useMemo(
    () =>
      regions.flatMap((r) => {
        const rad = r.outer_radius_m / outer;
        const pts: THREE.Vector3[] = [];
        for (let k = 0; k <= 64; k++) {
          const a = -Math.PI / 2 + (k / 64) * Math.PI;
          pts.push(new THREE.Vector3(rad * Math.cos(a), rad * Math.sin(a), 0.001));
        }
        const geo = new THREE.BufferGeometry().setFromPoints(pts);
        return [new THREE.Line(geo, new THREE.LineBasicMaterial()), new THREE.Line(geo, new THREE.LineBasicMaterial())];
      }),
    [regions, outer],
  );

  // Colours of the layers follow the theme.
  useEffect(() => {
    const apply = () => {
      const colours = layerColours();
      shells.forEach((m, i) => (m.material as THREE.MeshBasicMaterial).color.copy(colours[i] ?? new THREE.Color('#888888')));
      arcs.forEach((l, j) => (l.material as THREE.LineBasicMaterial).color.copy(colours[Math.floor(j / 2)] ?? new THREE.Color('#888888')));
      invalidate();
    };
    apply();
    const obs = new MutationObserver(apply);
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    const mq = typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null;
    mq?.addEventListener('change', apply);
    return () => {
      obs.disconnect();
      mq?.removeEventListener('change', apply);
    };
  }, [shells, arcs, invalidate]);

  // Paint the cut faces for the playback row; one frame is rendered per change.
  useEffect(() => {
    const paint = () => {
      const s = useStrato.getState();
      fillTexture(texture, cellColours(run, s.playback.tIndex, s.view.colormap, s.view.logColour ? 'log' : 'linear'), bins);
      invalidate();
    };
    paint();
    return useStrato.subscribe((s, p) => {
      if (s.playback.tIndex !== p.playback.tIndex || s.view.colormap !== p.view.colormap || s.view.logColour !== p.view.logColour) paint();
    });
  }, [run, texture, bins, invalidate]);

  useEffect(
    () => () => {
      texture.dispose();
      material.dispose();
      shells.forEach((m) => {
        m.geometry.dispose();
        (m.material as THREE.Material).dispose();
      });
      arcs.forEach((l, j) => {
        if (j % 2 === 0) l.geometry.dispose();
        (l.material as THREE.Material).dispose();
      });
    },
    [texture, material, shells, arcs],
  );

  useFrame(() => {
    const o = orbit.current;
    camera.position.setFromSpherical(new THREE.Spherical(o.distance, o.polar, o.azimuth));
    camera.lookAt(0, 0, 0);
    onFrame();
  });

  return (
    <>
      {/* Cut face in the plane z = 0 on the side x < 0, and in the plane x = 0 on the side z > 0. */}
      <mesh rotation={[0, Math.PI, 0]} material={material}>
        <planeGeometry args={[2, 2]} />
      </mesh>
      <mesh rotation={[0, -Math.PI / 2, 0]} material={material}>
        <planeGeometry args={[2, 2]} />
      </mesh>
      {arcs.map((l, j) => (
        <primitive key={`arc-${j}`} object={l} rotation={j % 2 === 0 ? [0, Math.PI, 0] : [0, -Math.PI / 2, 0]} />
      ))}
      {shells.map((m, i) => (
        <primitive key={`shell-${i}`} object={m} />
      ))}
    </>
  );
}

/** Keeps the camera of the demand-driven canvas in step with the orbit state. */
function Invalidator({ register }: { register: (fn: () => void) => void }) {
  const { invalidate } = useThree();
  useEffect(() => register(() => invalidate()), [register, invalidate]);
  return null;
}

export default function Particle3D({ run, label, onError }: { run: RunOk; label: string; onError: (message: string) => void }) {
  const orbit = useRef<OrbitState>({ ...DEFAULT_VIEW });
  const wrap = useRef<HTMLDivElement>(null);
  const redraw = useRef<() => void>(() => undefined);
  const drag = useRef<{ x: number; y: number } | null>(null);
  const frames = useRef(0);
  const antialias = useAntialias();

  const stamp = () => {
    const el = wrap.current;
    if (!el) return;
    const o = orbit.current;
    el.dataset.azimuth = o.azimuth.toFixed(4);
    el.dataset.polar = o.polar.toFixed(4);
    el.dataset.distance = o.distance.toFixed(4);
  };
  const change = (patch: Partial<OrbitState>) => {
    const o = { ...orbit.current, ...patch };
    o.polar = Math.min(Math.PI - 0.05, Math.max(0.05, o.polar));
    o.distance = Math.min(MAX_DIST, Math.max(MIN_DIST, o.distance));
    orbit.current = o;
    stamp();
    redraw.current();
  };

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const o = orbit.current;
    const keys: Record<string, () => void> = {
      ArrowLeft: () => change({ azimuth: o.azimuth - ROTATE_STEP }),
      ArrowRight: () => change({ azimuth: o.azimuth + ROTATE_STEP }),
      ArrowUp: () => change({ polar: o.polar - ROTATE_STEP }),
      ArrowDown: () => change({ polar: o.polar + ROTATE_STEP }),
      '+': () => change({ distance: o.distance / ZOOM_STEP }),
      '=': () => change({ distance: o.distance / ZOOM_STEP }),
      '-': () => change({ distance: o.distance * ZOOM_STEP }),
      '0': () => change({ ...DEFAULT_VIEW }),
      Home: () => change({ ...DEFAULT_VIEW }),
    };
    const fn = keys[e.key];
    if (fn) {
      e.preventDefault();
      fn();
    }
  };
  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    drag.current = { x: e.clientX, y: e.clientY };
    e.currentTarget.setPointerCapture(e.pointerId);
  };
  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    if (!d) return;
    const o = orbit.current;
    change({ azimuth: o.azimuth - (e.clientX - d.x) * 0.01, polar: o.polar - (e.clientY - d.y) * 0.01 });
    drag.current = { x: e.clientX, y: e.clientY };
  };
  const onPointerUp = () => {
    drag.current = null;
  };

  useEffect(stamp, []);

  return (
    <div className="p3d" data-testid="particle-3d">
      {/* The stage takes the place of the 2D drawing at no greater height: the zoom and reset buttons sit
          over the empty right margin of the scene, and the key help is one short line under it. */}
      <div className="p3d__stage">
        <div
          ref={wrap}
          className="p3d__canvas"
          tabIndex={0}
          role="img"
          aria-label={label}
          aria-describedby="p3d-keys"
          data-testid="p3d-canvas"
          data-antialias={antialias ? 'on' : 'off'}
          onKeyDown={onKeyDown}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        >
          <Canvas
            frameloop="demand"
            flat
            dpr={[1, 2]}
            camera={{ fov: 35, near: 0.1, far: 50 }}
            gl={{ antialias, alpha: true, powerPreference: 'default' }}
            onCreated={({ gl }) => {
              if (!gl.getContext()) onError('WebGL context not created');
            }}
          >
            <Invalidator register={(fn) => (redraw.current = fn)} />
            <Scene
              run={run}
              orbit={orbit}
              onFrame={() => {
                frames.current += 1;
                if (wrap.current) wrap.current.dataset.frames = String(frames.current);
              }}
            />
          </Canvas>
        </div>
        <div className="p3d__controls" role="group" aria-label="3D view controls" data-testid="p3d-controls">
          <button type="button" className="btn btn--small btn--icon" aria-label="Zoom in" title="Zoom in" onClick={() => change({ distance: orbit.current.distance / ZOOM_STEP })} data-testid="p3d-zoom-in">
            <span aria-hidden="true">+</span>
          </button>
          <button type="button" className="btn btn--small btn--icon" aria-label="Zoom out" title="Zoom out" onClick={() => change({ distance: orbit.current.distance * ZOOM_STEP })} data-testid="p3d-zoom-out">
            <span aria-hidden="true">−</span>
          </button>
          <button type="button" className="btn btn--small btn--icon" aria-label="Reset view" title="Reset view" onClick={() => change({ ...DEFAULT_VIEW })} data-testid="p3d-reset">
            <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true" focusable="false">
              <path d="M3.5 8a4.5 4.5 0 1 0 1.32-3.18" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
              <path d="M4.2 1.8v3.4h3.4" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </div>
      </div>
      <p id="p3d-keys" className="p3d__keys" data-testid="p3d-keys">
        Drag or arrow keys: rotate. + and −: zoom. 0: reset.
      </p>
    </div>
  );
}
