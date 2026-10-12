#!/usr/bin/env bash
# 按 tools/release/data.lock 下载并校验单包产品数据。
#
#   tools/release/data-fetch.sh            # 下载 + 校验 + 解开
#   tools/release/data-fetch.sh --verify   # 只校验 target/release-data/ 里已下载的文件
#
# 数据源默认仍是上游的 data-vN Release（词库与语言模型的语料统计产物只那里有）。
# 换成自己的数据源时给 QINGJIAN_DATA_REPO 指定「owner/repo」即可，哈希仍按 data.lock 校验：
#   QINGJIAN_DATA_REPO=Titor-Z/zhaime tools/release/data-fetch.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOCK="$ROOT/tools/release/data.lock"
OUT="$ROOT/target/release-data"
REPO="${QINGJIAN_DATA_REPO:-qingjian-team/qingjian}"
ASSETS=(qingjian-data.tar.gz)
cd "$ROOT"

[[ -f "$LOCK" ]] || { echo "缺少 $LOCK" >&2; exit 1; }
lock_value() { sed -nE "s/^$1 *= *//p" "$LOCK" | head -1 | tr -d '[:space:]'; }
sha256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

TAG="$(lock_value tag)"
[[ -n "$TAG" ]] || { echo "$LOCK 里没有 tag" >&2; exit 1; }
mkdir -p "$OUT"

if [[ "${1:-}" != "--verify" ]]; then
  for f in "${ASSETS[@]}"; do
    rm -f "$OUT/$f"
    if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
      gh release download "$TAG" --repo "$REPO" --pattern "$f" --dir "$OUT"
    else
      curl -fL --retry 3 -o "$OUT/$f" "https://github.com/$REPO/releases/download/$TAG/$f"
    fi
  done
fi

for f in "${ASSETS[@]}"; do
  expected="$(lock_value "$f")"
  actual="$(sha256 "$OUT/$f")"
  [[ -n "$expected" && "$actual" == "$expected" ]] || { echo "$f 与 data.lock 不符（$TAG）：期望 $expected，实际 $actual" >&2; exit 1; }
  echo "$f  $actual"
done
[[ "${1:-}" == "--verify" ]] && exit 0

tar -xzf "$OUT/qingjian-data.tar.gz" -C "$ROOT"
[[ -f data/generated/dict.qj ]] || {
  echo "产品数据包缺少词库（$TAG）" >&2; exit 1;
}
# 解出来的 mtime 比 checkout 出来的 TSV 旧，bundle.sh 会以为要重打
find data/generated -type f -exec touch {} +
echo "产品数据 $TAG 已就位"

if [[ -n "${GITHUB_ENV:-}" ]]; then
  {
    echo "DATA_TAG=$TAG"
    echo "DATA_SHA256=$(lock_value qingjian-data.tar.gz)"
  } >> "$GITHUB_ENV"
fi
