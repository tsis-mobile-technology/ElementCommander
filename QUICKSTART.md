# 🚀 Quick Start

## 빌드 및 실행

### 옵션 1: Debug 모드 (개발 중)
```bash
cd ~/Programming/hermes_tail
cargo run
```

### 옵션 2: Release 모드 (최고 성능)
```bash
cargo build --release
./target/release/hermes_tail
```

### 옵션 3: PATH에 추가하여 사용
```bash
cargo install --path .
hermes_tail
```

## 테스트 디렉토리 준비

### 자동 설정 (권장)
```bash
./demo.sh
# /tmp/hermes_demo 에 테스트 파일이 생성됩니다
```

### 수동 설정
```bash
mkdir -p ~/test_files/{docs,projects,archive}
touch ~/test_files/docs/{readme.txt,notes.md}
touch ~/test_files/projects/{file1.rs,file2.rs}
# 등등...
```

## 기본 사용법

### 단일 패널 조작
1. **↑↓** - 커서 이동
2. **Enter** - 디렉토리 진입
3. **Backspace** - 상위 디렉토리로 이동

### 패널 간 작업
1. **Tab** - 좌/우 패널 전환
2. **Insert** - 파일 선택/해제 (현재 패널)
3. **Ctrl+A** - 현재 패널의 모든 파일 선택

### 종료
- **q** 또는 **Esc** 키

## Phase 5 완료 기능

### Phase 1: 네비게이션 ✅
✅ 듀얼 패널 파일 탐색
✅ 파일 정보 표시 (이름, 크기, 날짜, 권한)
✅ 디렉토리 계층 탐색
✅ 멀티 선택

### Phase 2: 파일 작업 ✅
✅ F5: 파일 복사
✅ F6: 파일 이동
✅ F7: 디렉토리 생성
✅ F8: 파일 삭제
✅ F2/Shift+F6: 이름 변경

### Phase 3: 검색 & 필터 ✅
✅ `/`: 빠른 검색 (타입 필터)
✅ `=`: 와일드카드 필터
✅ Ctrl+F: 재귀 파일 검색

### Phase 4: 아카이브 지원 ✅
✅ ZIP/TAR/TAR.GZ 읽기 및 탐색
✅ F5: 아카이브에서 추출
✅ Alt+F5: 파일을 아카이브로 압축

### Phase 5: 폴싱 & 고급 기능 ✅
✅ 설정 파일: `~/.config/hermes_tail/config.toml`
✅ Ctrl+H: 숨김 파일 토글
✅ 인간이 읽기 쉬운 파일 크기 (K, M, G, T)
✅ 비동기 백그라운드 폴더 크기 계산
✅ Shift+F3: 파일 뷰어
✅ Ctrl+L: 새로고침 및 크기 재계산
✅ 파일 시스템 감시

---

## 문제 해결

### "Cannot access terminal" 오류
- WSL/SSH 환경에서는 터미널이 필요합니다
- 로컬 Linux/macOS/Windows 터미널에서 실행하세요

### 글꼴이 깨져 보임
- 터미널 글꼴을 Monospace (예: Menlo, Consolas)로 변경하세요
- UTF-8 지원 확인: `echo $LANG` 출력에 UTF-8이 포함되어야 함

### 색상이 이상함
- 터미널 색상 지원 확인
- `export TERM=xterm-256color` 시도

---

## 다음 단계

개발에 참여하려면:
1. CLAUDE.md 읽기 (아키텍처 이해)
2. Phase 2 작업 시작 (src/ops/ 모듈 구현)
3. 테스트 및 피드백

즐거운 파일 관리! 🎉
