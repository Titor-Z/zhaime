#!/usr/bin/env bash
# 发版时解析「标签 → 版本」并核对各壳的 Cargo.toml 与标签一致，release.yml 每个平台 job 第一步调用。
#
#   TAG        标签名，如 v0.3.0 / macos-v0.3.0
#   MANIFESTS  该平台要核对的 Cargo.toml，空格分隔
#
# 通过时把 version 写进 $GITHUB_OUTPUT（步骤 id 是 version，用来标预发布）。
# 对不上就失败：安装包文件名、Inno 的版本信息、「关于」页都从 Cargo.toml 取版本，
# 不一致会打出名字与 Release 不符的包。
set -euo pipefail

: "${TAG:?缺 TAG（标签名）}"
: "${MANIFESTS:?缺 MANIFESTS（要核对的 Cargo.toml 清单）}"

case "$TAG" in
  v*) version="${TAG#v}" ;;
  *-v*) version="${TAG##*-v}" ;;
  *) echo "标签要写成 v<版本> 或 <平台>-v<版本>，现在是 $TAG" >&2; exit 1 ;;
esac
if [[ "$version" == *-dev* ]]; then
  echo "标签 $TAG 带着开发版号：发版提交要先去掉 -dev" >&2
  exit 1
fi

for manifest in $MANIFESTS; do
  found="$(sed -n 's/^version = "\(.*\)"/\1/p' "$manifest" | head -1)"
  if [[ "$found" != "$version" ]]; then
    echo "$manifest 是 ${found:-（没读到 version）}，标签 $TAG 要的是 ${version} —— 发版提交先把各壳版本改成 ${version}（去掉 -dev）再打标签" >&2
    exit 1
  fi
done

# macOS runner 的 /bin/bash 是 3.2：变量名后面紧跟全角字符（`$version（`）会被当成变量名的一部分，
# 报 unbound variable —— 这里的变量一律加花括号。
echo "发版版本 ${version}（标签 $TAG）"
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "version=$version" >> "$GITHUB_OUTPUT"
fi
