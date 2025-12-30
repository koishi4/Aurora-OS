# phase_6_oscomp_hardening.md

## 目标
- 梳理 OSComp 关键测例的系统加固清单与优先级。

## 进展
- 接入 `scripts/test_oscomp.sh`，支持通过外部 OSComp 测例仓库运行测试并采集日志。
- 产出日志目录 `build/oscomp/`，生成 summary 便于回归记录。

## 问题与定位
- 尚未进入测例加固阶段，暂无问题记录。

## 解决与验证
- `make test-oscomp ARCH=riscv64 PLATFORM=qemu FS=path/to/ext4.img`

## 下一步
- 补齐 OSComp 测例仓库接入路径与 case 选择策略。
- 完成测例加固后进入交付准备。
