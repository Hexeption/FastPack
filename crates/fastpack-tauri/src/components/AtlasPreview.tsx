import { useRef, useState, useCallback } from "react";
import { useStore } from "../store";
import styles from "./AtlasPreview.module.css";

export default function AtlasPreview() {
  const sheets = useStore((s) => s.sheets);
  const activeSheet = useStore((s) => s.activeSheet);
  const setActiveSheet = useStore((s) => s.setActiveSheet);
  const isPacking = useStore((s) => s.isPacking);

  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const dragStart = useRef<{ mx: number; my: number; px: number; py: number } | null>(null);

  const sheet = sheets[activeSheet] ?? null;

  const onWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    setZoom((z) => Math.max(0.1, Math.min(8, z * factor)));
  }, []);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    dragStart.current = { mx: e.clientX, my: e.clientY, px: pan.x, py: pan.y };
  }, [pan]);

  const onMouseMove = useCallback((e: React.MouseEvent) => {
    if (!dragStart.current) return;
    const dx = e.clientX - dragStart.current.mx;
    const dy = e.clientY - dragStart.current.my;
    setPan({ x: dragStart.current.px + dx, y: dragStart.current.py + dy });
  }, []);

  const onMouseUp = useCallback(() => {
    dragStart.current = null;
  }, []);

  const resetView = () => { setZoom(1); setPan({ x: 0, y: 0 }); };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span>Atlas Preview</span>
        <div className="row">
          {sheets.length > 1 && sheets.map((_, i) => (
            <button
              key={i}
              className={`icon-btn ${i === activeSheet ? styles.activeTab : ""}`}
              onClick={() => setActiveSheet(i)}
            >
              {i + 1}
            </button>
          ))}
          {sheet && (
            <span className={styles.info}>
              {sheet.width}×{sheet.height} · {sheet.frames.length} frames
            </span>
          )}
          <button className="icon-btn" onClick={resetView} title="Reset view">⊙</button>
        </div>
      </div>
      <div
        className={styles.canvas}
        onWheel={onWheel}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
      >
        {isPacking && (
          <div className={styles.overlay}>Packing…</div>
        )}
        {!sheet && !isPacking && (
          <div className={styles.empty}>No atlas yet. Add sprites and click Pack.</div>
        )}
        {sheet && (
          <div
            className={styles.atlasWrap}
            style={{
              transform: `translate(calc(-50% + ${pan.x}px), calc(-50% + ${pan.y}px)) scale(${zoom})`,
            }}
          >
            <img
              src={`data:image/png;base64,${sheet.png_b64}`}
              alt="atlas"
              draggable={false}
              style={{ display: "block", imageRendering: zoom > 1 ? "pixelated" : "auto" }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
