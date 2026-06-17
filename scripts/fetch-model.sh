#!/usr/bin/env bash
# fetch-model.sh — legixy 第 2 層（semantic）用 ONNX 埋め込みモデルの取得
#
# 既定モデル: paraphrase-multilingual-MiniLM-L12-v2
#   - 日本語・英語を含む 50+ 言語対応、384 次元、mean pooling、prefix 不要
#   - traceability-engine（v0.4.0-alpha4）の Embedder は model.onnx + tokenizer.json を
#     ディレクトリから読み込み、mean pooling + L2 正規化する（モデル非依存）。
#
# 取得後の配置:
#   models/paraphrase-multilingual-MiniLM-L12-v2/model.onnx
#   models/paraphrase-multilingual-MiniLM-L12-v2/tokenizer.json
#
# 取得後、.trace-engine.toml の [semantic] enabled = true にすると第 2 層が有効化される。
#
# Usage: bash scripts/fetch-model.sh

set -euo pipefail
cd "$(dirname "$0")/.."

MODEL="paraphrase-multilingual-MiniLM-L12-v2"
REPO="sentence-transformers/${MODEL}"
DEST="models/${MODEL}"
BASE="https://huggingface.co/${REPO}/resolve/main"

mkdir -p "$DEST"

echo "ONNX モデルを取得します: ${REPO}"

# tokenizer.json（リポジトリ直下）
if [[ ! -f "$DEST/tokenizer.json" ]]; then
  echo "  - tokenizer.json"
  curl -fL "${BASE}/tokenizer.json" -o "$DEST/tokenizer.json"
fi

# model.onnx（onnx/ サブフォルダに配置されている。無い場合は optimum で書き出す必要あり）
if [[ ! -f "$DEST/model.onnx" ]]; then
  echo "  - model.onnx（onnx/model.onnx を試行）"
  if ! curl -fL "${BASE}/onnx/model.onnx" -o "$DEST/model.onnx"; then
    echo ""
    echo "  onnx/model.onnx が見つかりません。optimum で ONNX を書き出してください:" >&2
    echo "    pip install optimum[onnxruntime] sentence-transformers" >&2
    echo "    optimum-cli export onnx -m ${REPO} --task feature-extraction ${DEST}" >&2
    exit 1
  fi
fi

echo ""
echo "完了: $DEST"
ls -la "$DEST"
echo ""
echo "次のステップ:"
echo "  1. .trace-engine.toml の [semantic] enabled = true に変更"
echo "  2. traceability-engine --project-root . embed --all"
echo "  3. traceability-engine --project-root . check   （第 1 層 + 第 2 層 semantic）"
