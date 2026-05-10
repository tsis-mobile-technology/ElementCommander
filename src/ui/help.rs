use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Table, Row, Cell};
use super::theme::Theme;

pub fn render_help(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    
    // 도움말 창 크기 (화면의 90% 사용)
    let width = (area.width as f32 * 0.9) as u16;
    let height = (area.height as f32 * 0.85) as u16;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let help_area = Rect { x, y, width, height };

    frame.render_widget(Clear, help_area);

    let block = Block::default()
        .title(" ⌨️  Hermes Tail 단축키 가이드 (Esc/q로 닫기) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.ai_color).bold())
        .style(Style::default().bg(theme.inactive_bg));

    frame.render_widget(block.clone(), help_area);

    let inner_area = block.inner(help_area);
    
    // 2단 컬럼 레이아웃 생성
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // 왼쪽: 탐색/작업
            Constraint::Percentage(50), // 오른쪽: AI/시스템
        ])
        .split(inner_area);

    let header_style = Style::default().fg(theme.active_cursor_bg).bold();
    let key_style = Style::default().fg(theme.cmdbar_key_fg).bold();
    let desc_style = Style::default().fg(Color::White);

    // --- 왼쪽 컬럼: 탐색 및 일반 작업 ---
    let left_rows = vec![
        Row::new(vec![Cell::from(" [ 탐색 및 선택 ] ").style(header_style)]),
        Row::new(vec![Cell::from(" ↑ / ↓ ").style(key_style), Cell::from("커서 이동").style(desc_style)]),
        Row::new(vec![Cell::from(" PgUp/PgDn ").style(key_style), Cell::from("페이지 단위 이동").style(desc_style)]),
        Row::new(vec![Cell::from(" Enter ").style(key_style), Cell::from("폴더 진입 / 압축 열기").style(desc_style)]),
        Row::new(vec![Cell::from(" Backspace ").style(key_style), Cell::from("상위 폴더 / 압축 나가기").style(desc_style)]),
        Row::new(vec![Cell::from(" Tab ").style(key_style), Cell::from("패널 전환 (L/R)").style(desc_style)]),
        Row::new(vec![Cell::from(" Insert ").style(key_style), Cell::from("항목 선택/해제").style(desc_style)]),
        Row::new(vec![Cell::from(" Ctrl + A ").style(key_style), Cell::from("전체 항목 선택").style(desc_style)]),
        Row::new(vec![Cell::from(" Esc ").style(key_style), Cell::from("선택 해제 / 검색 취소").style(desc_style)]),
        
        Row::new(vec![Cell::from("")]),
        Row::new(vec![Cell::from(" [ 파일 관리 ] ").style(header_style)]),
        Row::new(vec![Cell::from(" F3 ").style(key_style), Cell::from("파일 보기 (Viewer)").style(desc_style)]),
        Row::new(vec![Cell::from(" F5 ").style(key_style), Cell::from("복사 (Copy)").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + F5 ").style(key_style), Cell::from("압축 생성 (Pack)").style(desc_style)]),
        Row::new(vec![Cell::from(" F6 ").style(key_style), Cell::from("이동 (Move)").style(desc_style)]),
        Row::new(vec![Cell::from(" F2 / S-F6 ").style(key_style), Cell::from("이름 변경 (Rename)").style(desc_style)]),
        Row::new(vec![Cell::from(" F7 ").style(key_style), Cell::from("새 폴더 생성 (Mkdir)").style(desc_style)]),
        Row::new(vec![Cell::from(" F8 ").style(key_style), Cell::from("삭제 (Delete)").style(desc_style)]),

        Row::new(vec![Cell::from("")]),
        Row::new(vec![Cell::from(" [ 검색 및 필터 ] ").style(header_style)]),
        Row::new(vec![Cell::from(" / ").style(key_style), Cell::from("빠른 검색 필터 시작").style(desc_style)]),
        Row::new(vec![Cell::from(" = ").style(key_style), Cell::from("와일드카드 필터 설정").style(desc_style)]),
        Row::new(vec![Cell::from(" Ctrl + F ").style(key_style), Cell::from("재귀 검색 / AI 스마트 검색").style(desc_style)]),
    ];

    let left_table = Table::new(left_rows, [Constraint::Length(12), Constraint::Min(20)])
        .style(Style::default().bg(theme.inactive_bg));
    frame.render_widget(left_table, columns[0]);

    // --- 오른쪽 컬럼: AI Commander 및 시스템 ---
    let ai_header_style = Style::default().fg(theme.ai_color).bold();
    let right_rows = vec![
        Row::new(vec![Cell::from(" [ AI Commander ] ").style(ai_header_style)]),
        Row::new(vec![Cell::from(" Ctrl + G ").style(key_style), Cell::from("자연어 명령 실행").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + B ").style(key_style), Cell::from("일괄 이름 변경 (패턴)").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + G ").style(key_style), Cell::from("파일 내용 요약").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + S ").style(key_style), Cell::from("민감 정보/보안 스캔").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + I ").style(key_style), Cell::from("이미지 속성/EXIF 분석").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + C ").style(key_style), Cell::from("소스 코드 구조 분석").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + D ").style(key_style), Cell::from("두 파일 비교 분석").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + A ").style(key_style), Cell::from("폴더 구조/용도 분석").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + N ").style(key_style), Cell::from("파일 메모/태그 저장").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + R ").style(key_style), Cell::from("README 자동 생성").style(desc_style)]),
        Row::new(vec![Cell::from(" Alt + X ").style(key_style), Cell::from("배치 작업 스크립트 생성").style(desc_style)]),

        Row::new(vec![Cell::from("")]),
        Row::new(vec![Cell::from(" [ 시스템 ] ").style(header_style)]),
        Row::new(vec![Cell::from(" F1 ").style(key_style), Cell::from("이 도움말 열기").style(desc_style)]),
        Row::new(vec![Cell::from(" F10 / C-Q ").style(key_style), Cell::from("프로그램 종료").style(desc_style)]),
        Row::new(vec![Cell::from(" Ctrl + H ").style(key_style), Cell::from("숨김 파일 표시 토글").style(desc_style)]),
        Row::new(vec![Cell::from(" Ctrl + L ").style(key_style), Cell::from("화면 강제 새로고침").style(desc_style)]),
        
        Row::new(vec![Cell::from("")]),
        Row::new(vec![Cell::from(" [ AI 뷰어 모드 ] ").style(header_style)]),
        Row::new(vec![Cell::from(" T ").style(key_style), Cell::from("AI 사고 과정 (Thinking) 토글").style(desc_style)]),
        Row::new(vec![Cell::from(" q / Esc ").style(key_style), Cell::from("뷰어 닫기 / 이전으로").style(desc_style)]),
    ];

    let right_table = Table::new(right_rows, [Constraint::Length(12), Constraint::Min(20)])
        .style(Style::default().bg(theme.inactive_bg));
    frame.render_widget(right_table, columns[1]);

    // --- 푸터: 안내 메시지 ---
    let footer_area = Rect {
        x: help_area.x,
        y: help_area.y + help_area.height - 2,
        width: help_area.width,
        height: 1,
    };
    let footer = Paragraph::new("💡 아무 키나 누르면 이 창이 닫힙니다.")
        .style(Style::default().fg(theme.inactive_border).italic())
        .alignment(Alignment::Center);
    frame.render_widget(footer, footer_area);
}
