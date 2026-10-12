#!/usr/bin/env bash
# 量一个「加载完数据就常驻」的进程的稳态内存，而不是 /usr/bin/time -l 的峰值。
#
# 为什么要自己采样：mmap 的 .qj 在打开时会顺序扫一遍做校验，那一刻 RSS 是最高的，
# 而我们关心的恰恰是「扫完之后、闲下来时」占多少——那才是输入法长期待着的数字。
# /usr/bin/time -l 的 maximum resident set size 量的是峰值，量不到这个。
#
#   tools/release/measure-memory.sh <要跑的命令...>
#
# 每秒采一次 ps 的 rss（KB），去掉前几秒的加载抖动后取中位数。macOS 用 vm_stat 看物理内存压力，
# Linux 上 ps 的 rss 语义一致。
set -uo pipefail

if [[ $# -eq 0 ]]; then
  echo "用法: $0 <命令...>" >&2
  exit 2
fi

SETTLE=${SETTLE:-5}     # 加载完、等抖动过去的秒数
SAMPLES=${SAMPLES:-10}  # 之后采几次
INTERVAL=${INTERVAL:-1}

"$@" &
pid=$!
# 命令自己退了就算它跑完了
trap 'kill "$pid" 2>/dev/null' EXIT INT TERM

sleep "$SETTLE"
if ! kill -0 "$pid" 2>/dev/null; then
  echo "命令在 $SETTLE 秒内就退出了，采不到稳态" >&2
  wait "$pid"
  exit $?
fi

rss=()
for _ in $(seq "$SAMPLES"); do
  current=$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ')
  [[ -n "$current" ]] && rss+=("$current")
  sleep "$INTERVAL"
done

if [[ ${#rss[@]} -eq 0 ]]; then
  echo "没采到样本" >&2
  exit 1
fi

printf '%s\n' "${rss[@]}" | sort -n | awk -v n="${#rss[@]}" '
  { a[NR] = $1 }
  END {
    mid = int((n + 1) / 2);
    med = (n % 2) ? a[mid] : (a[mid] + a[mid + 1]) / 2;
    printf "稳态 RSS: 中位数 %.1f MB（%d 个样本，min %.1f / max %.1f）\n",
           med / 1024, n, a[1] / 1024, a[n] / 1024;
  }'
