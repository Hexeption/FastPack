import * as PIXI from "pixi.js";

/** Creates a 16x16 checkerboard PixiJS texture. Colors adapt to dark or light mode. */
export function makeCheckerTexture(isDark: boolean): PIXI.Texture {
	const colorA = isDark ? "#202020" : "#cccccc";
	const colorB = isDark ? "#303030" : "#d9d9d9";
	const T = 16;
	const c = document.createElement("canvas");
	c.width = c.height = T;
	const ctx = c.getContext("2d")!;
	ctx.fillStyle = colorA;
	ctx.fillRect(0, 0, T, T);
	ctx.fillStyle = colorB;
	ctx.fillRect(0, 0, T / 2, T / 2);
	ctx.fillRect(T / 2, T / 2, T / 2, T / 2);
	return new PIXI.Texture({ source: new PIXI.CanvasSource({ resource: c }) });
}
