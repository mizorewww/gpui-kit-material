// The files scripts/build-site.py writes a Brotli copy of, by extension.
const TYPES = {
  ".wasm": "application/wasm",
  ".js": "text/javascript; charset=utf-8",
  ".ttf": "font/ttf",
};

/** The content type of a name that has a Brotli copy, or `undefined`. */
export function contentType(pathname) {
  const extension = pathname.slice(pathname.lastIndexOf("."));
  return Object.hasOwn(TYPES, extension) ? TYPES[extension] : undefined;
}

/** Whether `Accept-Encoding` allows Brotli: listed, and without `q=0`. */
export function acceptsBrotli(header) {
  return (header ?? "").split(",").some((item) => {
    const [coding, ...parameters] = item.trim().toLowerCase().split(";");
    if (coding.trim() !== "br") return false;
    const quality = parameters.find((parameter) => parameter.trim().startsWith("q="));
    return !quality || Number(quality.trim().slice(2)) > 0;
  });
}
