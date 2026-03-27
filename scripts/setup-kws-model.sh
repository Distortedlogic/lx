#!/usr/bin/env bash
set -euo pipefail

MODEL_NAME="sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
MODEL_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models/${MODEL_NAME}.tar.bz2"
MODEL_DIR="${HOME}/.local/share/lx/kws-model"

if [ -d "${MODEL_DIR}" ] && [ -f "${MODEL_DIR}/tokens.txt" ]; then
  echo "Model already exists at ${MODEL_DIR}"
else
  echo "Downloading ${MODEL_NAME}..."
  mkdir -p "${MODEL_DIR}"
  wget -q --show-progress "${MODEL_URL}" -O /tmp/kws-model.tar.bz2
  tar xjf /tmp/kws-model.tar.bz2 -C /tmp/
  cp /tmp/${MODEL_NAME}/*.onnx "${MODEL_DIR}/"
  cp /tmp/${MODEL_NAME}/tokens.txt "${MODEL_DIR}/"
  cp /tmp/${MODEL_NAME}/bpe.model "${MODEL_DIR}/"
  rm -rf /tmp/${MODEL_NAME} /tmp/kws-model.tar.bz2
  echo "Model extracted to ${MODEL_DIR}"
fi

KEYWORDS_FILE="${MODEL_DIR}/keywords.txt"
cat > "${KEYWORDS_FILE}" << 'KEYWORDS'
▁HE Y ▁C LA U DE
▁HE LL O
▁COMP U TER
KEYWORDS

echo "Keywords written to ${KEYWORDS_FILE}"
echo ""
echo "Setup complete. Add to your shell profile:"
echo "  export SHERPA_KWS_MODEL_DIR=${MODEL_DIR}"
