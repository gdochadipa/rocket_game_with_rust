#!/usr/bin/env bash
# Build macroquad + matchbox_socket (wasm-bindgen) untuk browser.
#
# Pemakaian (dari folder package, setelah `chmod +x build-wasm.sh`):
#   ./build-wasm.sh space-shooter-client            # debug build
#   ./build-wasm.sh space-shooter-client --release  # release build
#
# Versi 2: TIDAK lagi menambal internal wasm-bindgen dengan regex (rapuh, rusak
# di wasm-bindgen 0.2.129). Sekarang hanya tiga langkah kecil:
#   1. impor "env" (milik macroquad, bukan wasm-bindgen) diganti objek kosong
#      supaya browser tidak gagal me-resolve modul bernama "env"
#   2. tambahkan dua fungsi ekspor yang memanggil fungsi internal wasm-bindgen
#   3. HTML tidak memanggil init(); loader macroquad yang meng-instantiate .wasm
#
# Hasil: folder dist/ (di folder tempat skrip dijalankan)
# Jalankan:  python3 -m http.server 8080 --directory dist

set -e

die()  { echo "Error: $*" >&2; exit 1; }
warn() { echo "Peringatan: $*" >&2; }

# ---------- argumen ----------
RELEASE=""
PROJECT_NAME=""
for arg in "$@"; do
  case "$arg" in
    --release) RELEASE=yes ;;
    -*) die "opsi tidak dikenal: $arg" ;;
    *) PROJECT_NAME="$arg" ;;
  esac
done
[ -z "$PROJECT_NAME" ] && die "pemakaian: ./build-wasm.sh NAMA_PACKAGE [--release]"

if [ -n "$RELEASE" ]; then
  PROFILE=release
  CARGO_FLAGS="--release"
else
  PROFILE=debug
  CARGO_FLAGS=""
fi

# ---------- prasyarat ----------
command -v wasm-bindgen >/dev/null 2>&1 \
  || die "wasm-bindgen CLI belum terpasang. Jalankan: cargo install wasm-bindgen-cli"
rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$' \
  || die "target WASM belum terpasang. Jalankan: rustup target add wasm32-unknown-unknown"

# ---------- build ----------
# getrandom (dipakai dependency Matchbox) butuh backend wasm_js di browser.
export RUSTFLAGS="${RUSTFLAGS:-} "'--cfg getrandom_backend="wasm_js"'

echo "==> cargo build ($PROFILE)"
cargo build --target wasm32-unknown-unknown $CARGO_FLAGS

# ---------- cari hasil build (mendukung workspace) ----------
TARGET_DIR="${TARGET_DIR:-}"
if [ -z "$TARGET_DIR" ]; then
  if [ -f "target/wasm32-unknown-unknown/$PROFILE/$PROJECT_NAME.wasm" ]; then
    TARGET_DIR=target
  elif [ -f "../target/wasm32-unknown-unknown/$PROFILE/$PROJECT_NAME.wasm" ]; then
    TARGET_DIR=../target
  else
    die "tidak menemukan $PROJECT_NAME.wasm. Cek nama package, atau set TARGET_DIR=/path/ke/target"
  fi
fi
WASM_FILE="$TARGET_DIR/wasm32-unknown-unknown/$PROFILE/$PROJECT_NAME.wasm"
[ -f "$WASM_FILE" ] || die "file tidak ada: $WASM_FILE"

# ---------- versi wasm-bindgen CLI harus cocok dengan Cargo.lock ----------
CLI_VER=$(wasm-bindgen --version | awk '{print $2}')
LOCK_FILE=""
for f in Cargo.lock ../Cargo.lock; do
  if [ -f "$f" ]; then LOCK_FILE="$f"; break; fi
done
if [ -n "$LOCK_FILE" ]; then
  LOCK_VER=$(awk '/^name = "wasm-bindgen"$/ {getline; gsub(/version = |"/, ""); print; exit}' "$LOCK_FILE")
  if [ -n "$LOCK_VER" ] && [ "$LOCK_VER" != "$CLI_VER" ]; then
    die "versi wasm-bindgen tidak cocok (Cargo.lock: $LOCK_VER, CLI: $CLI_VER). Jalankan: cargo install wasm-bindgen-cli --version $LOCK_VER --force"
  fi
fi

# ---------- wasm-bindgen ----------
OUT=dist
mkdir -p "$OUT"
echo "==> wasm-bindgen $CLI_VER"
wasm-bindgen "$WASM_FILE" --out-dir "$OUT" --target web --no-typescript

JS="$OUT/$PROJECT_NAME.js"
echo "==> folder $OUT (relatif terhadap: $(pwd))"
ls -la "$OUT"
[ -f "$JS" ] || die "file $JS TIDAK ADA. wasm-bindgen mungkin memberi nama output berbeda (lihat daftar di atas), atau skrip dijalankan dari folder lain."

# ---------- adaptasi output wasm-bindgen untuk loader macroquad ----------
# Kita hanya bergantung pada DUA nama internal. Kalau wasm-bindgen mengubahnya
# di versi mendatang, skrip berhenti di sini dengan pesan yang jelas.
for fn in __wbg_get_imports __wbg_finalize_init; do
  grep -Eq "(function|const|let|var) +$fn[ (=]" "$JS" \
    || die "fungsi internal $fn tidak ditemukan di $JS (wasm-bindgen $CLI_VER). Tempel 'grep -n $fn $JS' ke Claude."
done

echo "==> adaptasi $JS"
# 1. impor ES dari "env" -> objek kosong (macroquad menyediakan "env"-nya sendiri)
sed -E -i.bak 's/^import \* as (import[0-9]+) from "env";?[[:space:]]*$/const \1 = {};/' "$JS"
rm -f "$JS.bak"

# 2. dua fungsi ekspor milik kita
cat >> "$JS" <<'EOF'

// ---- ditambahkan oleh build-wasm.sh (adaptasi untuk loader macroquad) ----
export function mq_get_imports() {
  return __wbg_get_imports();
}
export function mq_finish(exports) {
  return __wbg_finalize_init({ exports: exports }, undefined);
}
EOF

# Cek: browser tidak bisa memuat modul yang masih punya import ES dengan nama "telanjang".
if grep -q '^import ' "$JS"; then
  echo "Import yang tersisa di $JS:" >&2
  grep -n '^import ' "$JS" >&2
  die "masih ada import ES selain 'env'. Tempel daftar di atas ke Claude."
fi

# ---------- index.html ----------
cat > "$OUT/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>${PROJECT_NAME}</title>
  <style>
    html, body, canvas {
      margin: 0; padding: 0; width: 100%; height: 100%;
      overflow: hidden; position: absolute; background: black; z-index: 0;
    }
  </style>
</head>
<body>
  <canvas id="glcanvas" tabindex="1"></canvas>
  <!-- Loader macroquad (di-host oleh pembuat miniquad). Untuk produksi, simpan salinan sendiri. -->
  <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
  <script type="module">
    import { mq_get_imports, mq_finish } from "./${PROJECT_NAME}.js";

    miniquad_add_plugin({
      register_plugin: (importObject) => {
        // Salin impor milik wasm-bindgen ke objek impor milik macroquad.
        // Namespace "env" milik macroquad sendiri, jangan ditimpa.
        const imports = mq_get_imports();
        for (const key of Object.keys(imports)) {
          if (key !== "env") importObject[key] = imports[key];
        }
      },
      on_init: () => mq_finish(wasm_exports),
      version: "0.0.1",
      name: "wbg",
    });

    load("./${PROJECT_NAME}_bg.wasm");
  </script>
</body>
</html>
EOF

echo
echo "Selesai. Jalankan:"
echo "  python3 -m http.server 8080 --directory $OUT"
echo "lalu buka http://127.0.0.1:8080 dan DevTools console (Cmd+Option+J)."
