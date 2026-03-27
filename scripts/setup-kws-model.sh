#!/usr/bin/env bash
set -euo pipefail

WAKEWORD_DIR="${HOME}/.local/share/lx/wakewords"
WAKEWORD_FILE="${WAKEWORD_DIR}/hey_claude.rpw"

if [ -f "${WAKEWORD_FILE}" ]; then
  echo "Wakeword already exists at ${WAKEWORD_FILE}"
  echo "export RUSTPOTTER_WAKEWORD_PATH=${WAKEWORD_FILE}"
  exit 0
fi

if ! command -v rustpotter-cli &> /dev/null; then
  echo "Installing rustpotter-cli..."
  cargo install rustpotter-cli
fi

mkdir -p "${WAKEWORD_DIR}"
SAMPLES_DIR=$(mktemp -d)
echo ""
echo "Record 3 samples of your wake word (e.g. 'Hey Claude')."
echo "Press Enter to start each recording, then Ctrl+C to stop."
echo ""

for i in 1 2 3; do
  read -p "Press Enter to record sample ${i}/3..."
  rustpotter-cli record "${SAMPLES_DIR}/sample_${i}.wav"
  echo "Saved sample ${i}"
done

echo ""
echo "Building wakeword model..."
rustpotter-cli build \
  --model-name "hey claude" \
  --model-path "${WAKEWORD_FILE}" \
  "${SAMPLES_DIR}/sample_1.wav" \
  "${SAMPLES_DIR}/sample_2.wav" \
  "${SAMPLES_DIR}/sample_3.wav"

rm -rf "${SAMPLES_DIR}"

echo ""
echo "Wakeword saved to ${WAKEWORD_FILE}"
echo ""
echo "Add to your shell profile:"
echo "  export RUSTPOTTER_WAKEWORD_PATH=${WAKEWORD_FILE}"
