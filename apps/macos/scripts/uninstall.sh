#!/bin/sh
# 卸载札输入法：删掉输入法本体（系统级 /Library 与用户级 ~/Library 两处都看），加 --purge 连学习数据、配置、日志一起删。
# 这个脚本会打进 .app 的 Resources 里，装了 pkg 的用户直接运行它。
set -u
purge=0
[ "${1:-}" = "--purge" ] && purge=1

pkill -x qingjian-macos 2>/dev/null || true

# 改名之前装的是 Qingjian.app，两个都清掉，不然输入法列表里会留着一条僵尸条目
for dir in "/Library/Input Methods" "$HOME/Library/Input Methods"; do
    for app in Zha.app Qingjian.app; do
        [ -d "$dir/$app" ] || continue
        echo "删除 $dir/$app"
        case "$dir" in
            /Library*) sudo rm -rf "$dir/$app" ;;
            *) rm -rf "$dir/$app" ;;
        esac
    done
done
sudo pkgutil --forget app.zha.inputmethod >/dev/null 2>&1 || true
sudo pkgutil --forget app.qingjian.inputmethod >/dev/null 2>&1 || true

if [ "$purge" = 1 ]; then
    rm -rf "$HOME/Library/Application Support/Zha" "$HOME/Library/Logs/Zha"
    # 改名之前的数据目录
    rm -rf "$HOME/Library/Application Support/Qingjian" "$HOME/Library/Logs/Qingjian"
    echo "学习数据、配置与日志已删除"
else
    echo "保留了 ~/Library/Application Support/Zha（学习数据、配置）与 ~/Library/Logs/Zha；要一起删请加 --purge"
fi
echo "已卸载。输入法列表里的「札输入法」条目会在注销再登录后消失。"
