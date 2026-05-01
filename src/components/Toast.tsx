import { useEffect, useState } from "react";

interface ToastProps {
  message: string;
  kind: "success" | "error";
  onClose: () => void;
}

export function Toast({ message, kind, onClose }: ToastProps) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!message) return;

    setVisible(true);
    const timer = setTimeout(() => {
      setVisible(false);
      onClose();
    }, 5000);

    return () => clearTimeout(timer);
  }, [message, onClose]);

  if (!message) return null;

  return (
    <div className={`toast toast--${kind} ${visible ? "is-visible" : ""}`} role="status" aria-live="polite">
      <p>{message}</p>
      <button type="button" onClick={() => { setVisible(false); onClose(); }} aria-label="Dismiss notification">&times;</button>
    </div>
  );
}
