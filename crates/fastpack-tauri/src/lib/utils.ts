import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/** Merges class names with clsx and resolves Tailwind conflicts via tailwind-merge. */
export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
