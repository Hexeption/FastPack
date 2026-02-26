import { useEffect, useRef } from "react";
import { useStore } from "../store";
import styles from "./OutputLog.module.css";

export default function OutputLog() {
  const log = useStore((s) => s.log);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log]);

  return (
    <div className={styles.panel}>
      <div className={styles.header}>Output</div>
      <div className={styles.entries}>
        {log.map((entry, i) => (
          <div key={i} className={`${styles.entry} ${styles[entry.level]}`}>
            <span className={styles.time}>{entry.time}</span>
            <span className={styles.msg}>{entry.message}</span>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
