// TypeScript mirrors of Rust types serialized by the Tauri backend.

export type TrimMode = "none" | "trim" | "crop" | "crop_keep_pos" | "polygon";
export type SizeConstraint = "any_size" | "pot" | "multiple_of4" | "word_aligned";
export type PackMode = "fast" | "good" | "best";
export type ScaleMode = "smooth" | "fast" | "scale2x" | "scale3x" | "hq2x" | "eagle";
export type DataFormat = "json_hash" | "json_array" | "phaser3" | "pixijs";
export type TextureFormat = "png" | "jpeg" | "webp" | "etc1" | "etc2" | "pvrtc1" | "pvrtc2" | "dxt1" | "dxt5" | "astc" | "basis";
export type PixelFormat = "rgba8888" | "rgb888" | "rgb565" | "rgba4444" | "rgba5551" | "alpha8";
export type LogLevel = "info" | "warn" | "error";
export type Language = "En" | "Fr" | "Es" | "De" | "It" | "Pt" | "Ja" | "Zh" | "Ko";

export type AlgorithmConfig =
  | { type: "grid"; cell_width: number; cell_height: number }
  | { type: "basic" }
  | { type: "max_rects"; heuristic: "best_short_side_fit" | "best_long_side_fit" | "best_area_fit" | "bottom_left_rule" | "contact_point_rule" }
  | { type: "polygon" };

export interface Point {
  x: number;
  y: number;
}

export interface LayoutConfig {
  max_width: number;
  max_height: number;
  fixed_width: number | null;
  fixed_height: number | null;
  size_constraint: SizeConstraint;
  force_square: boolean;
  allow_rotation: boolean;
  pack_mode: PackMode;
  border_padding: number;
  shape_padding: number;
}

export interface SpriteConfig {
  trim_mode: TrimMode;
  trim_threshold: number;
  trim_margin: number;
  extrude: number;
  common_divisor_x: number;
  common_divisor_y: number;
  detect_aliases: boolean;
  default_pivot: Point;
}

export interface OutputConfig {
  name: string;
  directory: string;
  texture_format: TextureFormat;
  pixel_format: PixelFormat;
  premultiply_alpha: boolean;
  data_format: DataFormat;
  quality: number;
  texture_path_prefix: string;
  multipack: boolean;
}

export interface ScaleVariant {
  scale: number;
  suffix: string;
  scale_mode: ScaleMode;
}

export interface SpriteOverride {
  id: string;
  pivot: Point | null;
}

export interface PackerConfig {
  layout: LayoutConfig;
  sprites: SpriteConfig;
  output: OutputConfig;
  algorithm: AlgorithmConfig;
  variants: ScaleVariant[];
  sprite_overrides: SpriteOverride[];
}

export interface SourceSpec {
  path: string;
  filter: string;
}

export interface Project extends PackerConfig {
  sources: SourceSpec[];
}

export interface FrameData {
  id: string;
  x: number;
  y: number;
  w: number;
  h: number;
  alias_of: string | null;
}

export interface SheetData {
  width: number;
  height: number;
  png_b64: string;
  frames: FrameData[];
}

export interface LogEntry {
  level: LogLevel;
  message: string;
  time: string;
}

export interface Preferences {
  dark_mode: boolean;
  auto_check_updates: boolean;
  language: Language;
  ui_scale: number;
}

export interface PackFinishedPayload {
  sprite_count: number;
  alias_count: number;
  overflow_count: number;
  sheets: SheetData[];
  log: LogEntry[];
}

export interface PackFailedPayload {
  error: string;
}

export interface ReleaseInfo {
  version: string;
  notes: string;
  asset_url: string;
}
