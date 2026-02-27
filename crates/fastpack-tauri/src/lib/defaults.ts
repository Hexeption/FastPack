import type { KeybindsConfig, Preferences } from "../types";

export const DEFAULT_KEYBINDS: KeybindsConfig = {
	new_project: { key: "n", modifier: true, shift: false },
	open_project: { key: "o", modifier: true, shift: false },
	save_project: { key: "s", modifier: true, shift: false },
	save_project_as: { key: "s", modifier: true, shift: true },
	anim_preview: { key: "p", modifier: false, shift: false },
};

export const DEFAULT_PREFS: Preferences = {
	dark_mode: true,
	auto_check_updates: true,
	language: "En",
	ui_scale: 1.0,
	atlas_zoom_speed: 1.0,
	atlas_invert_scroll: false,
	keybinds: DEFAULT_KEYBINDS,
	default_config: {
		layout: {
			max_width: 2048,
			max_height: 2048,
			fixed_width: null,
			fixed_height: null,
			size_constraint: "any_size",
			force_square: false,
			allow_rotation: false,
			pack_mode: "best",
			border_padding: 2,
			shape_padding: 2,
		},
		sprites: {
			trim_mode: "trim",
			trim_threshold: 1,
			trim_margin: 0,
			extrude: 0,
			common_divisor_x: 0,
			common_divisor_y: 0,
			detect_aliases: true,
			default_pivot: { x: 0, y: 0 },
		},
		output: {
			name: "atlas",
			directory: "output",
			texture_format: "png",
			pixel_format: "rgba8888",
			premultiply_alpha: false,
			data_format: "phaser3",
			quality: 95,
			texture_path_prefix: "",
			multipack: false,
		},
		algorithm: { type: "max_rects", heuristic: "best_short_side_fit" },
		variants: [{ scale: 1.0, suffix: "", scale_mode: "smooth" }],
		sprite_overrides: [],
		excludes: [],
	},
};
