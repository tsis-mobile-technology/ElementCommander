#!/bin/bash

# hermes_tail 자동 재빌드 및 실행 스크립트
# 파일 변경 시 자동으로 rebuild & restart

set -e

echo "🔄 cargo-watch 시작 (파일 변경 감시 중...)"
echo "   수정된 파일을 저장하면 자동으로 재빌드 및 재실행됩니다"
echo "   종료하려면 Ctrl+C 를 누르세요"
echo ""

. "$HOME/.cargo/env"

# 옵션 1: Debug 모드로 실행 (개발 중 권장)
RUST_LOG=debug cargo watch -x run

# 옵션 2: Release 모드로 실행 (성능 테스트)
# cargo watch -x "build --release" -x "run --release"

# 옵션 3: 특정 파일만 감시
# cargo watch --watch src/ui/panel.rs -x run

exit 0;
