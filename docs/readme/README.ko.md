<p align="center">
  <img src="../../images/query2table-logo.png" alt="Query2Table 로고" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <b>한국어</b></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub 릴리스"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub 다운로드 수"></a>
  <a href="#라이선스"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="라이선스: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Tauri v2 기반"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Rust 백엔드"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Svelte 5 프런트엔드"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Query2Table 조회 수">
</p>

**Query2Table**은 데스크톱(Linux, Windows, macOS)과 Android용 로컬 우선 리서치 도구입니다. 필요한 데이터를 설명하면 앱이 여러 출처와 언어에 걸쳐 인터넷을 스스로 검색하고, 페이지를 가져와 LLM으로 구조화된 엔터티를 추출하고, 중복을 제거한 뒤, 행마다 출처 링크가 달린 실시간 갱신 표로 결과를 보여 줍니다. 클라우드 백엔드 없이 모든 작업이 내 컴퓨터에서 실행됩니다.

표 외에도 전용 모드가 세 가지 더 있습니다. **이미지 검색**은 관련 이미지를 찾아 LLM으로 시각적 관련성에 따라 순위를 매깁니다. **링크 검색**은 후보 페이지의 전체 내용을 읽고 LLM이 작성한 설명과 관련성 점수와 함께 가장 관련성 높은 링크만 반환합니다. **리서치**는 모델이 스스로 검색하고, 페이지를 읽고, 추론해 출처가 달린 Markdown 답변을 작성하는 에이전트 모드입니다.

이 앱은 **Android** 휴대폰에서도 사용할 수 있습니다. 같은 코드로 빌드되어 모드, 설정, 기록이 모두 같으며, 하단 탭 바를 갖춘 휴대폰 화면에 맞게 조정되어 있습니다.

*“오픈 소스 벡터 검색 데이터베이스와 라이브러리를 언어, 라이선스, 배포 유형, 사용 사례, 웹사이트와 함께 비교하는 표를 만들어 줘”* 같은 질문을 하고 표가 실시간으로 채워지는 모습을 지켜보세요.

![근거와 출처가 포함된 오픈 소스 벡터 데이터베이스 비교 표](../../screenshots/test-en-2026-10-06/44-table-results.png)

## 둘러보기

### 일상 언어로 질문하기

결과 형식을 고르고, 필요한 것을 설명하고, 중지 조건을 설정하세요. 표의 경우 앱이 열을 제안하며, 검색을 시작하기 전에 이를 수정할 수 있습니다.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="네 가지 결과 형식이 있는 새 쿼리"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="검토할 준비가 된 제안 표 스키마"></td>
  </tr>
</table>

### 모든 행에 근거가 남습니다

행을 열어 전체 값을 읽고, 복사하고, 값을 추출한 페이지를 확인하세요.

![추출 신뢰도와 출처가 표시된 행 세부 정보](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### 후속 질문이 가능한 리서치

에이전트가 검색하고 페이지를 읽은 뒤 목차가 있는 출처 기반 답변을 작성합니다. 모든 단계는 활동에서 확인할 수 있으며, 후속 질문은 기존 기록을 이어받아 대화를 계속합니다.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="목차와 추천 후속 질문이 있는 리서치 답변"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="번호가 매겨진 검색 및 읽기 단계가 있는 리서치 활동"></td>
  </tr>
</table>

### 이미지와 링크

이미지 검색은 결과를 관련성 순으로 정렬하고 사이트와 크기로 필터링할 수 있습니다. 링크 검색은 각 페이지를 읽고 왜 일치하는지 설명합니다.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="정렬된 격자로 표시된 이미지 검색 결과"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="설명과 일치 수준이 표시된 관련 링크"></td>
  </tr>
</table>

### 기록과 설정

지난 실행을 검색, 필터링해 다시 열거나, 다시 실행하거나, 여러 개를 한 번에 내보낼 수 있습니다. 설정에서는 시작하기 전에 LLM과 검색 연결을 확인합니다.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="결과, 소요 시간, 비용, 모델과 함께 날짜별로 묶인 실행 기록"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="연결 테스트에 성공한 검색 설정"></td>
  </tr>
</table>

### 밝은 테마와 어두운 테마

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="어두운 테마의 리서치 답변"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="어두운 테마의 결과 표"></td>
  </tr>
</table>

## 다운로드

[릴리스](https://github.com/hightemp/query2table/releases/latest)에서 최신 빌드를 받으세요.

| 플랫폼 | 파일 |
|----------|------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS(Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

컴퓨터의 설정을 휴대폰으로 옮기려면 **설정 → 설정 내보내기…**를 사용하고 **API 키와 프록시 포함**을 선택한 뒤 휴대폰에서 파일을 가져오세요. 이 파일에는 키가 들어 있으므로 비공개로 보관하고 가져온 뒤에는 삭제하세요.

Android에서는 다운로드한 APK를 열고, 요청이 나오면 브라우저나 파일 관리자에서 앱 설치를 허용하세요. Android 앱은 데이터를 휴대폰에 보관합니다. 트레이는 없으며 알림은 일반 Android 알림으로 표시됩니다. 로컬 모델 서버(Ollama, llama.cpp)는 네트워크 주소로 사용할 수 있습니다. 예: `http://192.168.1.10:11434`.

## 기능

- **자연어 쿼리** — 원하는 것을 일상 언어로 설명합니다
- **자동 스키마 추론** — 앱이 표의 열을 제안하고, 실행 전에 확정하거나 수정합니다
- **다중 공급자 검색** — Brave Search + Serper, 자동 대체 지원
- **다국어 쿼리 확장** — 여러 언어와 지역에 걸쳐 검색합니다
- **LLM 기반 추출** — OpenRouter, Anthropic Claude(프롬프트 캐싱, 선택적 PDF 전체 읽기), Ollama(로컬 또는 Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp 또는 OpenAI 호환 주소
- **스트리밍 결과** — 행을 찾는 즉시 실시간 표에 나타납니다
- **PDF 추출** — 검색 결과에서 찾은 PDF 문서를 자동으로 감지해 텍스트를 추출합니다
- **이미지 검색 모드** — 텍스트 대신 이미지를 검색하는 전용 파이프라인: 다양한 쿼리 변형 6–12개를 생성하고(LLM 또는 정적 대체), Brave / Serper로 이미지 검색을 실행한 뒤, 엄격한 관련성 기준(0.0–1.0)에 따라 LLM으로 모든 결과의 순위를 매깁니다. 0.7 미만인 이미지는 제외하고 나머지는 점수 순으로 저장하며, 이미지마다 UI가 실시간으로 갱신됩니다. 예산, 취소, 중지 조건(개수 / 비용 / 시간)이 전 과정에 적용됩니다
- **링크 검색 모드** — 표 대신 가장 관련성 높은 페이지를 반환하는 전용 파이프라인: 요청에서 검색 쿼리 변형을 생성하고, Brave / Serper로 웹 검색을 실행하고(URL 기준 중복 제거), 각 페이지를 가져와 파싱한 뒤, LLM이 전체 내용을 읽고 쿼리와의 관련성(0.0–1.0)을 평가하며 짧은 설명을 작성합니다. 신뢰도 기준 미만인 페이지는 제외하고 나머지는 점수 순으로 저장하며, 링크마다 UI가 실시간으로 갱신됩니다. 예산, 취소, 중지 조건이 전 과정에 적용됩니다
- **리서치 모드** — 열린 질문에 표 대신 Markdown으로 답하는 에이전트 모드: LLM이 정해진 단계 수까지 도구 호출 루프(웹 검색, 페이지를 Markdown으로 가져오기, 생각하기, 답하기)를 진행하며 각 단계를 실시간 타임라인에 표시합니다. 추론(`think`) 단계와 실패(`error` 단계, 빨간색으로 표시)도 드러나므로 과정이 투명하게 유지되며, 출처가 달린 최종 Markdown 답변은 (안전하게 정리된 상태로) 렌더링되고 `.md`로 내보낼 수 있습니다. 예산, 취소, 중지 조건이 전 과정에 적용됩니다
- **리서치 답변 속 수식과 그림** — LaTeX 수식을 (MathJax로, 오프라인에서) 그립니다. 에이전트는 이미지를 검색하고 살펴본 뒤 알맞은 것을 답변에 넣을 수 있으며, 이미지는 실행과 함께 보관됩니다
- **리서치 답변 속 다이어그램과 차트** — Mermaid 다이어그램과 막대, 꺾은선, 영역, 원형, 분산형, 방사형, 히트맵 차트를 앱 테마에 맞춰 오프라인으로 그립니다. SVG나 PNG로 저장하고 전체 화면으로 열 수 있습니다
- **리서치 답변 속 동영상** — 에이전트가 모든 사이트에서 동영상을 찾고(웹 동영상 검색 또는 YouTube, Bilibili, Niconico, SoundCloud 직접 검색) 시간 표시가 있는 자막을 읽어 특정 장면을 인용합니다. 동영상은 저장된 썸네일이 있는 카드로 표시됩니다. 처음 사용할 때 다운로드되는 yt-dlp를 사용합니다(Android에서는 내장 Python으로 실행)
- **사용 통계** — 비용, 토큰, 모델 호출, Brave 및 Serper 검색, 데이터량을 월별, 연도별, 전체 기간으로 보여 주며 모델별 및 모드별 내역, 가장 비용이 많이 든 실행, 디스크 사용량, CSV 내보내기, 경고가 있는 월 예산을 제공합니다
- **13개 인터페이스 언어** — 영어, 러시아어, 독일어, 프랑스어, 스페인어, 이탈리아어, 포르투갈어(브라질), 폴란드어, 우크라이나어, 터키어, 중국어(간체), 일본어, 한국어를 지원하며 기본값은 시스템 언어입니다
- **행 단위 출처** — 모든 행이 값을 추출한 페이지로 연결됩니다
- **엔터티 중복 제거** — 퍼지 매칭 + LLM 보조 판별
- **설정 가능한 중지 조건** — 목표 행 수, 최대 비용, 최대 시간
- **실행 기록** — 지난 리서치 실행을 둘러보고, 보고, 다시 내보냅니다
- **내보내기** — 전체 출처 메타데이터를 포함한 CSV, JSON, XLSX
- **어두운 / 밝은 테마** — 사이드바에서 전환
- **시스템 트레이** — 완료 알림(데스크톱)
- **Android 앱** — 작은 화면용 레이아웃을 갖춘 휴대폰용 같은 앱
- **앱 내 업데이트** — 새 버전을 변경 사항과 함께 알리고, 서명을 확인한 뒤 앱에서 설치합니다(데스크톱 및 Android)
- **로컬 우선** — 모든 데이터는 SQLite에 저장되며 클라우드 백엔드가 필요 없습니다

## 기술 스택

| 계층 | 기술 |
|-------|------------|
| 앱 셸 | Tauri v2(데스크톱 및 Android) |
| 백엔드 | Rust(Tokio 비동기 런타임) |
| 프런트엔드 | Svelte 5(SvelteKit SPA) |
| UI 스타일 | 디자인 토큰을 사용한 일반 CSS |
| 데이터베이스 | SQLite(sqlx, WAL 모드) |
| LLM | OpenRouter / Ollama(로컬) / Ollama Cloud / OpenAI 호환 API(예: llama.cpp) |
| 검색 | Brave Search API / Serper API |
| 아이콘 | Lucide |

## 사전 요구 사항

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install)(stable 툴체인)
- 사용하는 OS용 [Tauri v2 사전 요구 사항](https://v2.tauri.app/start/prerequisites/)

## 시작하기

```bash
# Clone the repository
git clone https://github.com/hightemp/query2table.git
cd query2table

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build
```

## 구성

처음 실행하면 앱이 기본 설정으로 로컬 SQLite 데이터베이스를 만듭니다. 사이드바에서 **설정**을 열어 다음을 구성하세요.

| 그룹 | 설정 |
|-------|----------|
| **LLM 공급자** | 공급자(OpenRouter / Ollama / Ollama Cloud / OpenAI 호환), URL, 모델, API 키, JSON 모드, 온도, 최대 토큰 |
| **검색 공급자** | 공급자(Brave / Serper), API 키, 대체 사용 여부, 쿼리당 결과 수 |
| **실행** | 동시 페이지 가져오기(기본값 8), 동시 추출(기본값 3), 가져오기 시간 제한, 속도 제한, robots.txt |
| **품질** | 정밀도/재현율 균형, 근거 엄격도, 신뢰도 기준, 중복 유사도 |

중지 조건(목표 행 수, 최대 비용, 최대 시간)은 쿼리 페이지에서 쿼리마다 설정합니다.

**다른 공급자를 백업으로 사용**은 웹 검색과 이미지 검색 모두에서 Brave/Serper 자동 대체를 제어합니다. 끄면 두 API 키가 모두 저장되어 있어도 선택한 기본 공급자만 사용합니다. 이 선택은 저장한 뒤 새 실행에 적용되며, 재시도는 선택한 공급자로 진행됩니다. 켜 두면 다른 공급자는 실패한 뒤에만 사용되며, 그 공급자의 키가 저장되어 있어야 합니다.

**설정 → 검색 → 쿼리당 결과 수**에서 리서치를 포함한 네 가지 모드 모두에서 검색 쿼리마다 요청할 결과 수를 정하고, 새 실행에 적용하려면 저장하세요. Brave 요청은 이 값을 문서에 명시된 엔드포인트 범위로 제한합니다: [웹 검색은 1–20](https://api-dashboard.search.brave.com/api-reference/web/search/get), [이미지는 최대 200](https://api-dashboard.search.brave.com/documentation/services/image-search). 이전에 저장된 지나치게 큰 값에도 적용됩니다. 값을 크게 해도 추가 페이지를 자동으로 요청하지는 않습니다.

### 사용량, 가격, 지출 한도

**사용량 및 비용**은 공급자가 보고한 요금, 모델 요금 기준 추정치, 부분 합계, 알 수 없는 비용을 구분합니다. 재시도, 대체 검색, 사용량 데이터가 있는 실패한 응답 생성, 최종 요금을 받지 못한 취소 요청까지 포함해 공급자 시도는 모두 따로 집계됩니다. 입력/출력 사용량은 공급자 응답에서 가져오며, 사고 토큰은 출력 토큰에 더해 이중으로 청구되지 않습니다.

- **OpenRouter:** [보고된 요청 비용](https://openrouter.ai/docs/cookbook/administration/usage-accounting)을 우선 사용합니다. 사용할 수 없으면 보고된 토큰 수와 실제로 응답한 모델의 공개 카탈로그 요금으로 추정합니다.
- **Ollama Cloud:** 캐시된 입력과 인식된 피크 요금 시간대를 포함해 [공개된 모델 요금](https://ollama.com/pricing)으로 사용 크레딧 소비량을 추정합니다. 포함 크레딧, 구독 결제, 실제 청구 금액은 이 추정치와 다를 수 있습니다. 인식되지 않았거나 사용할 수 없는 요금제는 알 수 없음으로 남습니다.
- **사용자 지정 모델 요금:** 설정에서 100만 토큰당 입력/출력 USD를 지정합니다. 요금은 특정 공급자, 엔드포인트, 모델에 속합니다. 0을 직접 지정하면 무료로 처리하며, 지정하지 않은 요금이 조용히 0이 되는 일은 없습니다. 공개 요금을 가져와도 저장된 수동 요금은 바뀌지 않습니다.
- **검색:** Brave와 Serper의 기본값은 요청 1,000건당 USD 0이며, 검색 요금은 추정치에서 제외됩니다. 포함하려면 요금제의 실제 요금을 지정하고, 모르면 가격을 비워 두세요. 청구 데이터가 없는 실패 시도는 요금 미정으로 남습니다. 설정 변경은 새 실행에 적용되며 저장된 기록은 다시 계산되지 않습니다. 업그레이드할 때 이전의 빈 기본값은 한 번만 0으로 바뀌며, 기존 숫자 요금과 이후 의도적으로 비운 값은 유지됩니다.

실행의 **최대 비용**은 집계된 지출이 한도에 도달하면 새 공급자 요청을 막습니다. 청구 금액의 상한을 보장하지는 않습니다. 추정치는 실제 청구와 다를 수 있고, 알 수 없는 요금은 포함할 수 없으며, 이미 진행 중인 요청에는 요금이 부과될 수 있습니다. 엄격한 상한이 필요하면 공급자 측 한도를 사용하세요. 집계는 완료, 실패, 취소, 결과 없는 실행에서도 유지됩니다. 이전 기록 항목은 이전 추정치로 명시됩니다.

중복 제거는 첫 번째 항목의 행 ID를 유지하고, 그룹의 값과 출처 기록을 트랜잭션으로 병합하며, 실시간 표를 기록 및 내보내기와 일치시킵니다. 이전에 덮어써진 과거 결과는 자동으로 복원할 수 없으므로, 새 결과를 얻으려면 해당 쿼리를 다시 실행하세요.

설정에서는 스크롤해도 **저장**과 **변경 취소**가 계속 보입니다. 섹션 링크로 관련 항목에 바로 이동할 수 있으며, 저장하지 않은 변경 사항이 있는 상태로 나가려 하면 저장 / 변경 취소 / 머무르기를 선택할 수 있습니다. 저장 중에 수정한 내용은 다음 저장 때 반영됩니다.

결과는 로컬 검색과 키보드 정렬을 지원하는 연속 가상화 표로 표시됩니다. 행의 **세부 정보**를 열어 전체 값을 읽거나 복사하고 저장된 출처를 확인하세요. 리서치는 답변을 펼칠 수 있는 활동보다 먼저 보여 줍니다. 이미지는 격자/목록 보기와 키보드 미리 보기를 지원하며, 내보내기와 삭제는 진행 상황과 결과를 명확히 보여 줍니다. 진단용 **로그** 패널을 지워도 실행 활동은 초기화되지 않습니다.

브라우저 회귀 검사는 합성 데이터와 모의 Tauri 명령을 사용합니다. `npx playwright install chromium webkit`으로 테스트 브라우저를 설치한 뒤 `npm run test:ui`를 실행하세요. 기존 프런트엔드 테스트는 `npm test`로 실행합니다.

### API 키

검색 API 키가 하나 이상 있어야 하고 LLM 공급자를 구성해야 합니다. 로컬 Ollama와 인증이 없는 OpenAI 호환 서버에는 LLM API 키가 필요 없습니다.

| 서비스 | 키 발급처 |
|---------|-------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama(로컬) | https://ollama.com/ — 키 필요 없음 |
| Ollama Cloud | https://ollama.com/settings/keys |
| OpenAI 호환 | 서버에 따라 키 선택 사항 |

**설정 → LLM 공급자**에서 공급자를 선택하고 해당 필드를 채운 뒤 **저장**을 클릭하세요. 공급자를 바꿔도 각 공급자는 자신의 모델과 자격 증명을 유지합니다.

설정 마이그레이션은 시작할 때 자동으로 실행됩니다. 새로 추가된 옵션에는 기본값이 적용되고 기존 값은 유지됩니다. 이름이 바뀐 설정은 기본값을 넣기 전에 백엔드에서 명시적으로 마이그레이션되며, 저장 형식이 바뀌는 경우에도 새 버전을 구현할 때 마이그레이션 코드가 필요합니다.

**설정** 하단의 **애플리케이션 파일**에는 실제 데이터 폴더, `data.db` 데이터베이스, 로그 폴더가 표시됩니다. **경로 복사**로 위치를 시스템 클립보드에 복사하거나 **폴더 열기**로 파일 관리자에서 열 수 있습니다. 데이터베이스 위치를 열면 해당 파일이 들어 있는 폴더가 열립니다. 설정, 기록, 결과, 출처는 `data.db`에 있으며 진단 로그는 따로 저장됩니다.

- **OpenRouter:** API 키를 입력하고 검색 가능한 [OpenRouter 카탈로그](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties)에서 모델을 선택하세요. 입력해서 모델 ID를 필터링하고, 마우스나 화살표 키와 Enter로 선택하며, **새로 고침**으로 목록을 다시 불러옵니다. 카탈로그는 키를 입력하기 전에도 둘러볼 수 있지만 추론에는 키가 필요합니다.
- **Ollama(로컬):** URL `http://localhost:11434`와 설치된 모델 이름.
- **Ollama Cloud:** URL `https://ollama.com`, Ollama API 키, 검색 가능한 서버 카탈로그에서 선택한 모델. 모델 필드를 클릭하고 입력해서 필터링한 뒤 마우스나 화살표 키와 Enter로 선택하세요. **새로 고침**은 카탈로그를 다시 불러옵니다. 필터링만으로는 결과를 선택하기 전까지 선택된 모델이 바뀌지 않습니다. 클라우드 직접 접근에는 [Ollama 네이티브 API](https://docs.ollama.com/cloud)를 사용합니다. 현재 Cloud는 [구조화된 출력](https://docs.ollama.com/capabilities/structured-outputs)을 지원하지 않으므로 JSON은 프롬프트로 요청하고 기존 파이프라인에서 파싱합니다.
- **OpenAI 호환:** `/v1`을 포함한 API 기본 URL과 서버의 모델 ID를 입력하세요. [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server)의 기본 URL은 `http://localhost:8080/v1`이며, 로드된 모델 이름이나 설정한 별칭을 사용합니다. 서버에서 요구하지 않으면 API 키는 비워 두세요. 서버가 `response_format`을 거부하면 **JSON 모드**를 끄세요.

**사고 / 추론 강도**는 모델이 답하기 전에 얼마나 추론할지 제어합니다. **자동**은 안전한 기본값을 유지합니다. 구조화된 Ollama 요청에서는 사고를 끄고(GPT-OSS는 낮음 사용), 다른 공급자는 기본값을 그대로 둡니다. **공급자 기본값**은 추론을 서버에 맡깁니다. **끄기**, **켜기**, **낮음**, **보통**, **높음**, **최대**는 특정 모드나 수준을 요청하며, 지원 여부는 모델과 공급자에 따라 다릅니다. Ollama의 GPT-OSS는 낮음/보통/높음만 받으며 끄기나 최대는 사용할 수 없습니다. 변경 사항은 저장한 뒤 새 실행에 적용됩니다.

**최대 출력 토큰**은 요청마다 허용되는 양으로, 사고 토큰과 답변 토큰이 함께 쓸 수 있습니다. 실행의 비용/시간 한도나 공급자 계정 할당량과는 별개입니다. 추론 강도를 높이면 답변에 쓸 토큰이 줄어들 수 있습니다.

모델 요청 문제가 생기면 공급자, 모델, 단계, 요청한 출력 한도, 보고된 입력/출력/사고 토큰 수, 재시도 예약 여부를 보여 줍니다. 출력 한도, 컨텍스트 한도, 공급자 할당량, 속도 제한, 인증, 네트워크 오류에는 각각 다른 설명과 해결 방법이 제시됩니다. 개별 페이지를 건너뛴 경우에도 문제가 계속 표시되며 실행 기록에서 확인할 수 있습니다. 그 밖의 오류에는 짧은 설명, 취할 조치, 펼칠 수 있는 기술 세부 정보가 표시됩니다.

**일시 중지**는 스키마 검토를 포함해 현재 단계에서 처리를 멈추고, **재개**는 요청을 다시 보내지 않고 이어서 진행합니다. 원격 모델에 이미 보낸 요청은 일시 중지 중에도 그쪽에서 끝날 수 있습니다. **취소**는 로컬 작업을 멈추고 대기 중인 요청을 닫습니다. 파이프라인 실패는 `failed`로 표시 및 저장되며, 근본 오류는 실행 로그에서 확인할 수 있습니다.

## 아키텍처

백엔드는 고정된 역할을 순서대로 조율하는 **파이프라인 상태 머신**을 사용합니다.

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### 파이프라인 역할

| 역할 | LLM | 목적 |
|------|-----|---------|
| QueryInterpreter | 예 | 쿼리를 구조화된 의도로 파싱 |
| SchemaPlanner | 예 | 표의 열과 유형 제안 |
| SearchPlanner | 예 | 여러 언어/지역용 검색 쿼리 생성 |
| QueryExpander | 예 | 쿼리를 대상 언어로 번역 |
| SearchExecutor | 아니요 | 검색 API 호출, 후보 URL 수집 |
| Fetcher | 아니요 | 속도 제한과 robots.txt를 지키는 HTTP 가져오기 |
| DocumentParser | 아니요 | HTML → 정리된 텍스트(상용구 제거) |
| PdfParser | 아니요 | PDF → 정리된 텍스트(pdf-extract 사용) |
| Extractor | 예 | 텍스트 + 스키마 → 구조화된 행 |
| Validator | 일부 | 스키마 적합성 + 의미 검사 |
| Deduplicator | 일부 | 퍼지 매칭(strsim) + 경계 사례용 LLM |
| LinkRanker | 예 | 페이지 내용을 읽고 관련성 평가(0.0–1.0) + 설명 생성(링크 검색 모드) |
| ResearchAgent | 예 | 검색/가져오기/생각/답변 에이전트 루프를 진행하고 Markdown 답변 생성(리서치 모드) |
| StoppingController | 아니요 | 중지 조건 평가(행 수, 예산, 시간, 포화) |

### 상태 머신

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## 프로젝트 구조

```
query2table/
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   ├── commands/        # IPC command handlers
│   │   ├── orchestrator/    # Pipeline state machine, budget tracker
│   │   ├── roles/           # Pipeline & research-agent roles
│   │   ├── providers/       # External API clients (LLM, search, HTTP)
│   │   ├── storage/         # SQLite models & repository
│   │   └── export/          # CSV, JSON, XLSX export
│   └── migrations/          # SQLite schema migrations
├── src/                     # Svelte frontend
│   ├── lib/
│   │   ├── components/      # UI components
│   │   ├── stores/          # Svelte stores (run, settings, ui, logs)
│   │   ├── types/           # TypeScript type definitions
│   │   └── api/             # Tauri invoke/listen wrappers
│   └── routes/              # Pages (query, history, settings)
├── package.json
└── src-tauri/Cargo.toml
```

## 데이터베이스

WAL 모드의 SQLite이며 다음 테이블이 있습니다.

| 테이블 | 목적 |
|-------|---------|
| `settings` | 사용자 구성(API 키, 모델, 환경 설정) |
| `runs` | 상태와 통계가 포함된 리서치 실행 기록 |
| `run_schemas` | 실행별 확정된 표 스키마 |
| `search_queries` | 실행별 생성된 검색 쿼리 |
| `search_results` | 검색 API에서 수집한 URL |
| `fetched_pages` | 다운로드한 페이지 내용 |
| `entity_rows` | 추출된 구조화 행(JSON 데이터) |
| `row_sources` | 행 단위 근거(URL, 제목, 발췌) |
| `image_results` | 실행별 순위가 매겨진 이미지(이미지 검색 모드) |
| `link_results` | 설명과 점수가 포함된, 순위가 매겨진 관련 링크(링크 검색 모드) |
| `research_steps` | 실행별 에이전트 도구 호출 단계 — search/fetch/think/error(리서치 모드) |
| `research_results` | 실행별 최종 Markdown 답변(리서치 모드) |
| `run_logs` | 실행별 실행 로그 |

## 개발

```bash
# Frontend dev server only
npm run dev

# Type checking
npm run check

# Run tests
npm test

# Run Rust tests
cd src-tauri && cargo test

# Lint
npm run lint
```

### Android

[Tauri Android 사전 요구 사항](https://v2.tauri.app/start/prerequisites/#android)이 필요합니다: Android SDK와 NDK(`ANDROID_HOME`, `NDK_HOME`), Java 17(`JAVA_HOME`), Rust 타깃(`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). USB 디버깅을 켠 휴대폰을 연결하거나 에뮬레이터를 시작하세요.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

`src-tauri/gen/android/keystore.properties`가 없으면 빌드는 디버그 키로 서명됩니다. `make android-keystore`는 릴리스 키와 이 파일을 만들고, 릴리스 워크플로가 각 릴리스에 추가하는 APK에 서명할 때 쓰는 GitHub 시크릿(`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`)을 출력합니다. 키를 잘 보관하세요. 업데이트는 같은 키로 서명된 경우에만 기존 앱 위에 설치됩니다.

## 내보내기 형식

| 형식 | 내용 |
|--------|----------|
| **CSV** | 모든 열 + JSON 열로 담은 출처 |
| **JSON** | 중첩된 출처 배열과 실행 메타데이터를 포함한 전체 행 |
| **XLSX** | 데이터 시트, 출처 시트, 메타데이터 시트가 있는 서식 지정 통합 문서 |

## 라이선스

MIT
