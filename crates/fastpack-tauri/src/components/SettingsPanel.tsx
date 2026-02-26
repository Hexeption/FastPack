import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import type { Project, AlgorithmConfig } from "../types";
import styles from "./SettingsPanel.module.css";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <details className={styles.section} open>
      <summary className={styles.sectionTitle}>{title}</summary>
      <div className={styles.sectionBody}>{children}</div>
    </details>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className={styles.row}>
      <label className={styles.label}>{label}</label>
      <div className={styles.control}>{children}</div>
    </div>
  );
}

export default function SettingsPanel() {
  const project = useStore((s) => s.project);
  const setProject = useStore((s) => s.setProject);
  const setDirty = useStore((s) => s.setDirty);

  if (!project) return <div className={styles.panel} />;

  const update = (p: Project) => {
    setProject(p);
    setDirty(true);
    invoke("update_project", { project: p });
  };

  const out = project.output;
  const layout = project.layout;
  const sprites = project.sprites;
  const alg = project.algorithm;

  const setOut = (patch: Partial<typeof out>) =>
    update({ ...project, output: { ...out, ...patch } });
  const setLayout = (patch: Partial<typeof layout>) =>
    update({ ...project, layout: { ...layout, ...patch } });
  const setSprites = (patch: Partial<typeof sprites>) =>
    update({ ...project, sprites: { ...sprites, ...patch } });

  const algType = (() => {
    if (alg.type === "grid") return "grid";
    if (alg.type === "basic") return "basic";
    if (alg.type === "max_rects") return "max_rects";
    return "polygon";
  })();

  const setAlgType = (t: string) => {
    let newAlg: AlgorithmConfig;
    if (t === "grid") newAlg = { type: "grid", cell_width: 0, cell_height: 0 };
    else if (t === "basic") newAlg = { type: "basic" };
    else if (t === "polygon") newAlg = { type: "polygon" };
    else newAlg = { type: "max_rects", heuristic: "best_short_side_fit" };
    update({ ...project, algorithm: newAlg });
  };

  const handlePickDir = async () => {
    const p = await invoke<string | null>("open_folder_dialog");
    if (p) setOut({ directory: p });
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>Settings</div>
      <div className={styles.body}>
        <Section title="Output">
          <Row label="Name">
            <input
              type="text"
              value={out.name}
              style={{ width: "100%" }}
              onChange={(e) => setOut({ name: e.target.value })}
            />
          </Row>
          <Row label="Directory">
            <div className="row" style={{ width: "100%" }}>
              <input
                type="text"
                value={typeof out.directory === "string" ? out.directory : String(out.directory)}
                style={{ flex: 1, minWidth: 0 }}
                onChange={(e) => setOut({ directory: e.target.value })}
              />
              <button onClick={handlePickDir}>…</button>
            </div>
          </Row>
          <Row label="Format">
            <select
              value={out.texture_format}
              onChange={(e) => setOut({ texture_format: e.target.value as typeof out.texture_format })}
            >
              <option value="png">PNG</option>
              <option value="jpeg">JPEG</option>
              <option value="webp">WebP</option>
            </select>
          </Row>
          <Row label="Data">
            <select
              value={out.data_format}
              onChange={(e) => setOut({ data_format: e.target.value as typeof out.data_format })}
            >
              <option value="json_hash">JSON Hash</option>
              <option value="json_array">JSON Array</option>
              <option value="phaser3">Phaser 3</option>
              <option value="pixijs">PixiJS</option>
            </select>
          </Row>
          <Row label="Quality">
            <input
              type="number"
              value={out.quality}
              min={1}
              max={100}
              style={{ width: 60 }}
              onChange={(e) => setOut({ quality: Number(e.target.value) })}
            />
          </Row>
          <Row label="Multipack">
            <input
              type="checkbox"
              checked={out.multipack}
              onChange={(e) => setOut({ multipack: e.target.checked })}
            />
          </Row>
        </Section>

        <Section title="Layout">
          <Row label="Max size">
            <div className="row">
              <input
                type="number"
                value={layout.max_width}
                min={1}
                style={{ width: 70 }}
                onChange={(e) => setLayout({ max_width: Number(e.target.value) })}
              />
              <span style={{ color: "var(--text-dim)" }}>×</span>
              <input
                type="number"
                value={layout.max_height}
                min={1}
                style={{ width: 70 }}
                onChange={(e) => setLayout({ max_height: Number(e.target.value) })}
              />
            </div>
          </Row>
          <Row label="Algorithm">
            <select value={algType} onChange={(e) => setAlgType(e.target.value)}>
              <option value="max_rects">MaxRects</option>
              <option value="grid">Grid</option>
              <option value="basic">Basic</option>
              <option value="polygon">Polygon</option>
            </select>
          </Row>
          {alg.type === "max_rects" && (
            <Row label="Heuristic">
              <select
                value={alg.heuristic}
                onChange={(e) =>
                  update({
                    ...project,
                    algorithm: { type: "max_rects", heuristic: e.target.value as typeof alg.heuristic },
                  })
                }
              >
                <option value="best_short_side_fit">Best short side</option>
                <option value="best_long_side_fit">Best long side</option>
                <option value="best_area_fit">Best area</option>
                <option value="bottom_left_rule">Bottom-left</option>
                <option value="contact_point_rule">Contact point</option>
              </select>
            </Row>
          )}
          <Row label="Pack mode">
            <select
              value={layout.pack_mode}
              onChange={(e) => setLayout({ pack_mode: e.target.value as typeof layout.pack_mode })}
            >
              <option value="fast">Fast</option>
              <option value="good">Good</option>
              <option value="best">Best</option>
            </select>
          </Row>
          <Row label="Size constraint">
            <select
              value={layout.size_constraint}
              onChange={(e) => setLayout({ size_constraint: e.target.value as typeof layout.size_constraint })}
            >
              <option value="any_size">Any</option>
              <option value="pot">Power of 2</option>
              <option value="multiple_of4">Multiple of 4</option>
              <option value="word_aligned">Word aligned</option>
            </select>
          </Row>
          <Row label="Border pad">
            <input
              type="number"
              value={layout.border_padding}
              min={0}
              style={{ width: 60 }}
              onChange={(e) => setLayout({ border_padding: Number(e.target.value) })}
            />
          </Row>
          <Row label="Shape pad">
            <input
              type="number"
              value={layout.shape_padding}
              min={0}
              style={{ width: 60 }}
              onChange={(e) => setLayout({ shape_padding: Number(e.target.value) })}
            />
          </Row>
          <Row label="Rotation">
            <input
              type="checkbox"
              checked={layout.allow_rotation}
              onChange={(e) => setLayout({ allow_rotation: e.target.checked })}
            />
          </Row>
          <Row label="Force square">
            <input
              type="checkbox"
              checked={layout.force_square}
              onChange={(e) => setLayout({ force_square: e.target.checked })}
            />
          </Row>
        </Section>

        <Section title="Sprites">
          <Row label="Trim mode">
            <select
              value={sprites.trim_mode}
              onChange={(e) => setSprites({ trim_mode: e.target.value as typeof sprites.trim_mode })}
            >
              <option value="none">None</option>
              <option value="trim">Trim</option>
              <option value="crop">Crop</option>
              <option value="crop_keep_pos">Crop keep pos</option>
              <option value="polygon">Polygon</option>
            </select>
          </Row>
          <Row label="Trim threshold">
            <input
              type="number"
              value={sprites.trim_threshold}
              min={0}
              max={255}
              style={{ width: 60 }}
              onChange={(e) => setSprites({ trim_threshold: Number(e.target.value) })}
            />
          </Row>
          <Row label="Extrude">
            <input
              type="number"
              value={sprites.extrude}
              min={0}
              style={{ width: 60 }}
              onChange={(e) => setSprites({ extrude: Number(e.target.value) })}
            />
          </Row>
          <Row label="Detect aliases">
            <input
              type="checkbox"
              checked={sprites.detect_aliases}
              onChange={(e) => setSprites({ detect_aliases: e.target.checked })}
            />
          </Row>
        </Section>
      </div>
    </div>
  );
}
