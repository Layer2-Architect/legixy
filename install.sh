#!/usr/bin/env bash
# install.sh — legixy installer for Linux / macOS.
#
# What it does:
#   1. Downloads a prebuilt legixy release archive from GitHub Releases.
#   2. Installs it under a prefix (default: ~/.local/share/legixy) and links
#      the `legixy` / `legixy-mcp` launchers into <prefix>/bin.
#   3. Downloads the ONNX embedding model (paraphrase-multilingual-MiniLM-L12-v2)
#      from the Hugging Face Hub, so the semantic layer works out of the box.
#
# The model is NOT shipped inside the release archive (~500 MB, separate
# license); this script is how you obtain it. Use --no-model to skip it.
#
# Usage:
#   curl -fsSL <raw-url>/install.sh | bash
#   bash install.sh [--version vX.Y.Z] [--prefix DIR] [--no-model] [--repo OWNER/REPO]
#
# Environment overrides:
#   LEGIXY_REPO     GitHub "owner/repo" to download from   (default below)
#   LEGIXY_VERSION  release tag, or "latest"               (default: latest)
#   LEGIXY_PREFIX   install prefix                          (default: ~/.local)

set -euo pipefail

# ---- defaults (edit REPO, or pass --repo / set LEGIXY_REPO) ----------------
REPO="${LEGIXY_REPO:-Layer2-Architect/legixy}"
VERSION="${LEGIXY_VERSION:-latest}"
PREFIX="${LEGIXY_PREFIX:-$HOME/.local}"
WITH_MODEL=1

MODEL_NAME="paraphrase-multilingual-MiniLM-L12-v2"
HF_REPO="sentence-transformers/${MODEL_NAME}"

err()  { printf '\033[31merror:\033[0m %s\n' "$*" >&2; exit 1; }
info() { printf '\033[36m==>\033[0m %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

usage() { sed -n '2,33p' "$0" | sed 's/^# \{0,1\}//'; exit 0; }

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="${2:?--version needs a value}"; shift 2;;
    --prefix)  PREFIX="${2:?--prefix needs a value}";  shift 2;;
    --repo)    REPO="${2:?--repo needs a value}";      shift 2;;
    --no-model) WITH_MODEL=0; shift;;
    -h|--help) usage;;
    *) err "unknown argument: $1 (try --help)";;
  esac
done

# ---- platform detection -----------------------------------------------------
os="$(uname -s)"; arch="$(uname -m)"
case "$os" in
  Linux)  platform="x86_64-linux"  ;;
  Darwin) err "no prebuilt macOS binary yet. Build from source: cargo build --release -p legixy-cli --features onnx (see docs/manual/manual.en.md)";;
  *) err "unsupported OS: $os";;
esac
case "$arch" in
  x86_64|amd64) : ;;
  *) err "unsupported architecture: $arch (only x86_64 is published)";;
esac

for tool in curl tar; do have "$tool" || err "required tool not found: $tool"; done

# ---- resolve release tag ----------------------------------------------------
api="https://api.github.com/repos/${REPO}/releases"
if [ "$VERSION" = "latest" ]; then
  info "resolving latest release of ${REPO}"
  tag="$(curl -fsSL "${api}/latest" | grep -m1 '"tag_name"' | cut -d'"' -f4)"
  [ -n "$tag" ] || err "could not resolve latest release tag from ${api}/latest"
else
  tag="$VERSION"
fi
asset="legixy-${tag}-${platform}.tar.gz"
url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

# ---- download & unpack ------------------------------------------------------
INSTALL_DIR="${PREFIX}/share/legixy"
BIN_DIR="${PREFIX}/bin"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

info "downloading ${asset} (${tag})"
curl -fL "$url" -o "$tmp/$asset" || err "download failed: $url"

info "installing into ${INSTALL_DIR}"
rm -rf "$INSTALL_DIR"; mkdir -p "$INSTALL_DIR"
tar -xzf "$tmp/$asset" -C "$INSTALL_DIR" --strip-components=1

mkdir -p "$BIN_DIR"
for launcher in legixy legixy-mcp; do
  if [ -f "$INSTALL_DIR/$launcher" ]; then
    chmod +x "$INSTALL_DIR/$launcher" 2>/dev/null || true
    ln -sf "$INSTALL_DIR/$launcher" "$BIN_DIR/$launcher"
  fi
done

# ---- ONNX model -------------------------------------------------------------
if [ "$WITH_MODEL" -eq 1 ]; then
  dest="$INSTALL_DIR/models/${MODEL_NAME}"
  base="https://huggingface.co/${HF_REPO}/resolve/main"
  mkdir -p "$dest"
  info "fetching embedding model: ${HF_REPO} (~500 MB)"
  [ -f "$dest/tokenizer.json" ] || curl -fL "${base}/tokenizer.json" -o "$dest/tokenizer.json" \
    || err "failed to fetch tokenizer.json"
  if [ ! -f "$dest/model.onnx" ]; then
    curl -fL "${base}/onnx/model.onnx" -o "$dest/model.onnx" || {
      rm -f "$dest/model.onnx"
      err "failed to fetch onnx/model.onnx. Export it with: pip install optimum[onnxruntime] && optimum-cli export onnx -m ${HF_REPO} --task feature-extraction \"$dest\""
    }
  fi
  info "model installed at ${dest}"
else
  info "skipping model download (--no-model). The semantic layer needs a model; see docs/manual."
fi

# ---- done -------------------------------------------------------------------
echo
info "legixy ${tag} installed."
"$BIN_DIR/legixy" --version 2>/dev/null || true
case ":$PATH:" in
  *":$BIN_DIR:"*) : ;;
  *) printf '\033[33mnote:\033[0m %s is not on your PATH. Add:\n  export PATH="%s:$PATH"\n' "$BIN_DIR" "$BIN_DIR";;
esac
echo "Next: run 'legixy init' in a project, then 'legixy check --formal'. Full guide: docs/manual/manual.en.md"
