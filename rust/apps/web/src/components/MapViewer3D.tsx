import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { MapData, LayerVisibility } from '../types/map';

interface MapViewer3DProps {
  mapData: MapData | null;
  layers: LayerVisibility;
  colorScheme: 'light' | 'dark';
}

export function MapViewer3D({ mapData, layers, colorScheme }: MapViewer3DProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const frameRef = useRef<number>(0);
  const [webgpuError, setWebgpuError] = useState<string | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;

    const scene = new THREE.Scene();
    sceneRef.current = scene;

    const bgColor = colorScheme === 'dark' ? 0x1a2030 : 0xc8e8f8;
    scene.background = new THREE.Color(bgColor);

    const camera = new THREE.PerspectiveCamera(
      60,
      container.clientWidth / container.clientHeight,
      0.1,
      10000
    );
    camera.position.set(960, 800, 540);
    camera.lookAt(960, 0, 540);
    cameraRef.current = camera;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true });
      renderer.setSize(container.clientWidth, container.clientHeight);
      renderer.setPixelRatio(window.devicePixelRatio);
      container.appendChild(renderer.domElement);
      rendererRef.current = renderer;
    } catch (e) {
      setWebgpuError('Failed to initialize renderer');
      return;
    }

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight.position.set(500, 800, 500);
    scene.add(dirLight);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controls.minDistance = 100;
    controls.maxDistance = 5000;
    controlsRef.current = controls;

    const animate = () => {
      frameRef.current = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    const handleResize = () => {
      if (!container) return;
      camera.aspect = container.clientWidth / container.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(container.clientWidth, container.clientHeight);
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(frameRef.current);
      controls.dispose();
      renderer.dispose();
      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [colorScheme]);

  useEffect(() => {
    if (!mapData || !sceneRef.current) return;
    const scene = sceneRef.current;

    scene.children
      .filter(c => (c.userData as { isMapObject?: boolean }).isMapObject)
      .forEach(c => {
        scene.remove(c);
        if (c instanceof THREE.Line || c instanceof THREE.Points) {
          c.geometry.dispose();
          if (Array.isArray(c.material)) {
            c.material.forEach(m => m.dispose());
          } else {
            (c.material as THREE.Material).dispose();
          }
        }
      });

    const W = mapData.image_width;
    const H = mapData.image_height;

    const addPolylines = (
      polylines: number[][],
      color: number,
      visible: boolean,
      name: string
    ) => {
      if (!visible || polylines.length === 0) return;
      polylines.forEach(path => {
        if (path.length < 4) return;
        const pts: THREE.Vector3[] = [];
        for (let i = 0; i < path.length; i += 2) {
          pts.push(new THREE.Vector3(path[i] * W, 0, path[i + 1] * H));
        }
        const geo = new THREE.BufferGeometry().setFromPoints(pts);
        const mat = new THREE.LineBasicMaterial({ color });
        const line = new THREE.Line(geo, mat);
        line.userData.isMapObject = true;
        line.name = name;
        scene.add(line);
      });
    };

    addPolylines(mapData.contour, 0x8899aa, layers.contour, 'contour');
    addPolylines(mapData.river, 0x4488cc, layers.rivers, 'rivers');
    addPolylines(mapData.territory, 0xcc4444, layers.territory, 'territory');

    if (layers.cities && mapData.city.length >= 2) {
      const geo = new THREE.BufferGeometry();
      const positions: number[] = [];
      for (let i = 0; i < mapData.city.length; i += 2) {
        positions.push(mapData.city[i] * W, 2, mapData.city[i + 1] * H);
      }
      geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
      const mat = new THREE.PointsMaterial({ color: 0x222233, size: 8 });
      const points = new THREE.Points(geo, mat);
      points.userData.isMapObject = true;
      scene.add(points);
    }

    if (layers.towns && mapData.town.length >= 2) {
      const geo = new THREE.BufferGeometry();
      const positions: number[] = [];
      for (let i = 0; i < mapData.town.length; i += 2) {
        positions.push(mapData.town[i] * W, 1, mapData.town[i + 1] * H);
      }
      geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
      const mat = new THREE.PointsMaterial({ color: 0x445566, size: 5 });
      const points = new THREE.Points(geo, mat);
      points.userData.isMapObject = true;
      scene.add(points);
    }

    if (cameraRef.current && controlsRef.current) {
      cameraRef.current.position.set(W / 2, 800, H / 2);
      controlsRef.current.target.set(W / 2, 0, H / 2);
      controlsRef.current.update();
    }
  }, [mapData, layers]);

  useEffect(() => {
    if (!sceneRef.current) return;
    const bgColor = colorScheme === 'dark' ? 0x1a2030 : 0xc8e8f8;
    sceneRef.current.background = new THREE.Color(bgColor);
  }, [colorScheme]);

  return (
    <div ref={containerRef} className="flex-1 relative overflow-hidden">
      {webgpuError && (
        <div className="absolute top-2 right-2 bg-yellow-100 text-yellow-800 text-xs px-2 py-1 rounded">
          ⚠️ {webgpuError}
        </div>
      )}
      {!mapData && (
        <div className="absolute inset-0 flex items-center justify-center bg-map-bg/50">
          <div className="text-center text-gray-400">
            <div className="text-6xl mb-4">🗺️</div>
            <div className="text-lg">No map data loaded</div>
          </div>
        </div>
      )}
    </div>
  );
}
