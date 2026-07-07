import { useState } from "react";

/**
 * Favicon with graceful fallback: when the icon fails to load (offline,
 * missing icon) we render a letter tile instead of a broken image.
 */
export function Favicon({ src, label }: { src?: string | null; label: string }) {
  const [failed, setFailed] = useState(false);
  if (!src || failed) {
    return <div className="favicon favicon-fallback">{label.charAt(0).toUpperCase()}</div>;
  }
  return (
    <img
      className="favicon"
      src={src}
      alt=""
      loading="lazy"
      onError={() => setFailed(true)}
    />
  );
}
