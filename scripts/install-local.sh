#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_ID="org.zugferd.QubaViewer"
BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
DATA_DIR="${HOME}/.local/share/quba-gtk"
WRAPPER_PATH="${BIN_DIR}/quba_gtk"

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}" "${DATA_DIR}"

cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"
if [[ ! -f "${ROOT_DIR}/dist/quba-render-helper.bundle.cjs" ]]; then
  npm --prefix "${ROOT_DIR}" run bundle-helper
fi
install -Dm755 "${ROOT_DIR}/target/release/quba_gtk" "${DATA_DIR}/quba_gtk-real"
install -Dm644 "${ROOT_DIR}/quba-viewer-1.5.0/src/assets/img/logoonly.svg" "${ICON_DIR}/${APP_ID}.svg"
rm -rf "${DATA_DIR}/dist" "${DATA_DIR}/scripts" "${DATA_DIR}/quba-viewer-1.5.0" "${DATA_DIR}/node_modules"
cp -r "${ROOT_DIR}/dist" "${DATA_DIR}/dist"
cp -r "${ROOT_DIR}/quba-viewer-1.5.0" "${DATA_DIR}/quba-viewer-1.5.0"
install -Dm644 "${ROOT_DIR}/package.json" "${DATA_DIR}/package.json"
if [[ -f "${ROOT_DIR}/package-lock.json" ]]; then
  install -Dm644 "${ROOT_DIR}/package-lock.json" "${DATA_DIR}/package-lock.json"
fi

cat > "${WRAPPER_PATH}" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export QUBA_PROJECT_ROOT="${DATA_DIR}"
exec "${DATA_DIR}/quba_gtk-real" "\$@"
EOF
chmod +x "${WRAPPER_PATH}"

sed \
  -e "s|^Exec=.*$|Exec=${WRAPPER_PATH} %U|" \
  -e "s|^Icon=.*$|Icon=${APP_ID}|" \
  "${ROOT_DIR}/data/${APP_ID}.desktop" > "${APP_DIR}/${APP_ID}.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
fi

echo "Installed ${APP_ID} locally."
echo "Desktop file: ${APP_DIR}/${APP_ID}.desktop"
echo "Binary: ${BIN_DIR}/quba_gtk"
echo "You can now pick Quba GTK from your file manager's Open With dialog."
