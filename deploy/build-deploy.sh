#!/usr/bin/env bash
# deploy/build-deploy.sh — legixy 実行環境を deploy/ に組み立てる（再現可能）。
#   1. legixy CLI を release build（target/release/legixy）
#   2. ts-mcp(MCP サーバ)を build（dist/）+ ランタイム依存を deploy へ導入
#   3. バイナリ・dist・設定・起動スクリプトを deploy/ に配置
#
# 生成物（deploy/bin, deploy/ts-mcp/{dist,node_modules}, deploy/models）は .gitignore 対象。
# 本スクリプト・README・起動スクリプトのみコミットする。
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
DEPLOY="$HERE"

echo "==> [1/3] release build: legixy CLI"
# LEGIXY_ONNX=1 で embed（実 ONNX 推論）対応ビルド（ort + 470MB モデルが別途必要）。
ONNX_FLAGS=""
if [ "${LEGIXY_ONNX:-}" = "1" ]; then
  ONNX_FLAGS="--features onnx"
  echo "    （onnx feature 有効: embed/実推論。実行には ONNX モデル配置が必要）"
fi
cargo build --release -p legixy-cli $ONNX_FLAGS --manifest-path "$ROOT/Cargo.toml"

echo "==> [2/3] build ts-mcp (MCP サーバ)"
( cd "$ROOT/ts-mcp" && npm ci && npm run build )

echo "==> [3/3] assemble deploy/"
mkdir -p "$DEPLOY/bin" "$DEPLOY/ts-mcp" "$DEPLOY/config" "$DEPLOY/models"

# CLI バイナリ
install -m 0755 "$ROOT/target/release/legixy" "$DEPLOY/bin/legixy"

# ts-mcp: ビルド済み dist + package 定義 + ランタイム依存のみ（--omit=dev）
rm -rf "$DEPLOY/ts-mcp/dist"
cp -r "$ROOT/ts-mcp/dist" "$DEPLOY/ts-mcp/dist"
cp "$ROOT/ts-mcp/package.json" "$ROOT/ts-mcp/package-lock.json" "$DEPLOY/ts-mcp/"
( cd "$DEPLOY/ts-mcp" && npm ci --omit=dev )

# サンプル設定
if [ -f "$ROOT/.trace-engine.toml" ]; then
  cp "$ROOT/.trace-engine.toml" "$DEPLOY/config/.trace-engine.toml"
fi

# ONNX モデル（日本語対応 = 多言語 paraphrase-multilingual-MiniLM-L12-v2、384 次元 mean pooling）。
# LEGIXY_ONNX=1 のときのみ deploy/models へ配置（hardlink で 470MB 重複を回避。別 FS は copy）。
MODEL_NAME="paraphrase-multilingual-MiniLM-L12-v2"
SRC_MODEL="$ROOT/models/$MODEL_NAME"
DST_MODEL="$DEPLOY/models/$MODEL_NAME"
if [ "${LEGIXY_ONNX:-}" = "1" ]; then
  if [ -f "$SRC_MODEL/model.onnx" ] && [ -f "$SRC_MODEL/tokenizer.json" ]; then
    mkdir -p "$DST_MODEL"
    for f in model.onnx tokenizer.json; do
      rm -f "$DST_MODEL/$f"
      cp -l "$SRC_MODEL/$f" "$DST_MODEL/$f" 2>/dev/null || cp "$SRC_MODEL/$f" "$DST_MODEL/$f"
    done
    echo "    （日本語対応 ONNX モデルを配置: $DST_MODEL）"
  else
    echo "    [warn] $SRC_MODEL に model.onnx/tokenizer.json が無いためモデル未配置（embed 不可）"
  fi
fi
cat > "$DEPLOY/models/README.txt" <<'EOF'
embed / 意味層(check 既定モード) / refresh-subnodes が ONNX 推論を使う。
日本語対応 = 多言語モデル paraphrase-multilingual-MiniLM-L12-v2/{model.onnx, tokenizer.json}。
LEGIXY_ONNX=1 で build-deploy すると本ディレクトリへ自動配置（hardlink）。
deploy/legixy ラッパが LGX_MODELS_DIR を本モデルへ自動設定する。
check/impact/investigate/feedback 等はモデル不要。
EOF

echo "==> done. deploy at: $DEPLOY"
echo "    試す: $DEPLOY/legixy --project-root <repo> check --formal"
