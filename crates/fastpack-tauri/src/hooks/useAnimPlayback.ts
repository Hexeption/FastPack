import { useEffect, useRef, useState } from "react";

interface UseAnimPlaybackOptions {
	frameCount: number;
	isOpen: boolean;
	initialFps?: number;
}

export function useAnimPlayback({
	frameCount,
	isOpen,
	initialFps = 24,
}: UseAnimPlaybackOptions) {
	const [playing, setPlaying] = useState(false);
	const [fps, setFps] = useState(initialFps);
	const [looping, setLooping] = useState(true);
	const [pingPong, setPingPong] = useState(false);
	const [currentIdx, setCurrentIdx] = useState(0);
	const [onionSkin, setOnionSkin] = useState(false);
	const directionRef = useRef<1 | -1>(1);

	// Clamp index when frame count changes
	useEffect(() => {
		setCurrentIdx((i) => (frameCount > 0 ? Math.min(i, frameCount - 1) : 0));
	}, [frameCount]);

	// Reset on open/close
	useEffect(() => {
		if (isOpen) {
			setPlaying(true);
			directionRef.current = 1;
		} else {
			setPlaying(false);
		}
	}, [isOpen]);

	// Playback interval
	useEffect(() => {
		if (!playing || frameCount < 2) return;
		const id = setInterval(() => {
			setCurrentIdx((i) => {
				if (pingPong) {
					const next = i + directionRef.current;
					if (next >= frameCount) {
						directionRef.current = -1;
						return i - 1;
					}
					if (next < 0) {
						if (!looping) {
							setPlaying(false);
							return 0;
						}
						directionRef.current = 1;
						return 1;
					}
					return next;
				}
				const next = i + 1;
				if (next >= frameCount) {
					if (looping) return 0;
					setPlaying(false);
					return i;
				}
				return next;
			});
		}, 1000 / fps);
		return () => clearInterval(id);
	}, [playing, fps, looping, pingPong, frameCount]);

	const stepBack = () => {
		setPlaying(false);
		setCurrentIdx((i) => (i - 1 + frameCount) % frameCount);
	};

	const stepForward = () => {
		setPlaying(false);
		setCurrentIdx((i) => (i + 1) % frameCount);
	};

	return {
		playing,
		setPlaying,
		fps,
		setFps,
		looping,
		setLooping,
		pingPong,
		setPingPong,
		currentIdx,
		onionSkin,
		setOnionSkin,
		directionRef,
		stepBack,
		stepForward,
	};
}
