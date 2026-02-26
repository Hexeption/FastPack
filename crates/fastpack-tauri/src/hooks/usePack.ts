import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { PackFinishedPayload, PackFailedPayload } from "../types";
import { useStore } from "../store";

export function usePack() {
  const setIsPacking = useStore((s) => s.setIsPacking);
  const setSheets = useStore((s) => s.setSheets);
  const appendLog = useStore((s) => s.appendLog);

  useEffect(() => {
    const unlisteners = Promise.all([
      listen("pack:started", () => {
        setIsPacking(true);
        appendLog({ level: "info", message: "Pack started…", time: now() });
      }),

      listen<PackFinishedPayload>("pack:finished", ({ payload }) => {
        setIsPacking(false);
        setSheets(payload.sheets, payload.log);
      }),

      listen<PackFailedPayload>("pack:failed", ({ payload }) => {
        setIsPacking(false);
        appendLog({ level: "error", message: payload.error, time: now() });
      }),
    ]);

    return () => {
      unlisteners.then((fns) => fns.forEach((fn) => fn()));
    };
  }, [setIsPacking, setSheets, appendLog]);
}

function now(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}
