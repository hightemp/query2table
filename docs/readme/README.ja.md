<p align="center">
  <img src="../../images/query2table-logo.png" alt="Query2Table のロゴ" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <b>日本語</b> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub のリリース"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub のダウンロード数"></a>
  <a href="#ライセンス"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="ライセンス：MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Tauri v2 で構築"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Rust バックエンド"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Svelte 5 フロントエンド"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Query2Table の閲覧数">
</p>

**Query2Table** は、デスクトップ（Linux、Windows、macOS）と Android 向けのローカルファーストなリサーチツールです。必要なデータを説明すると、アプリが複数のソースと言語にまたがってインターネットを自動で検索し、ページを取得し、LLM で構造化されたエンティティを抽出して重複を除去し、行ごとの出典リンク付きでリアルタイムに更新されるテーブルとして結果を表示します。すべてお使いのマシン上で動作し、クラウドのバックエンドは必要ありません。

テーブルのほかに、専用のモードが 3 つあります。**画像検索**は、関連する画像を探し、見た目の関連度で LLM が順位付けします。**リンク検索**は、候補ページの本文をすべて読み、LLM が生成した説明と関連度スコア付きで、最も関連性の高いリンクだけを返します。**リサーチ**はエージェント型のモードで、モデルが自律的に検索し、ページを読み、推論して、出典付きの Markdown 回答を書きます。

このアプリは **Android** スマートフォンでも利用できます。同じモード、設定、履歴を備えた同じアプリで、同じコードからビルドされ、下部タブバー付きのスマートフォン画面に合わせて調整されています。

たとえば *「オープンソースのベクトル検索データベースとライブラリについて、言語、ライセンス、デプロイ方式、用途、ウェブサイトを比較するテーブルを作成して」* と依頼すれば、テーブルがリアルタイムで埋まっていく様子を確認できます。

![根拠と出典付きのオープンソースベクトルデータベース比較テーブル](../../screenshots/test-en-2026-10-06/44-table-results.png)

## 機能紹介

### ふだんの言葉で質問

結果の形式を選び、必要なものを説明して、停止条件を設定します。テーブルの場合は、検索開始前に編集できる列をアプリが提案します。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="4 つの結果形式を選べる新しいクエリ"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="確認待ちの提案されたテーブルスキーマ"></td>
  </tr>
</table>

### すべての行に根拠が残る

行を開くと、完全な値を読んだりコピーしたりでき、抽出元のページも確認できます。

![抽出の信頼度と出典を表示した行の詳細](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### 追加の質問ができるリサーチ

エージェントが検索し、ページを読み、目次付きの出典のある回答を書きます。各ステップはアクティビティで確認でき、追加の質問をすると履歴を踏まえて会話が続きます。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="目次とおすすめの追加質問を含むリサーチの回答"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="番号付きの検索と読み取りのステップを示すリサーチのアクティビティ"></td>
  </tr>
</table>

### 画像とリンク

画像検索は、サイトとサイズで絞り込みながら関連度順に結果を並べます。リンク検索は各ページを読み、一致する理由を説明します。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="整列されたグリッドで表示した画像検索の結果"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="説明と一致度付きの関連リンク"></td>
  </tr>
</table>

### 履歴と設定

過去の実行を検索、絞り込み、再表示し、再実行したり、まとめてエクスポートしたりできます。設定では、開始前に LLM と検索の接続を確認できます。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="結果、所要時間、コスト、モデルを日ごとにまとめた実行履歴"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="接続テストに成功した検索設定"></td>
  </tr>
</table>

### ライトテーマとダークテーマ

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="ダークテーマのリサーチ回答"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="ダークテーマの結果テーブル"></td>
  </tr>
</table>

## ダウンロード

最新のビルドは [Releases](https://github.com/hightemp/query2table/releases/latest) から入手できます。

| プラットフォーム | ファイル |
|----------|------|
| Linux | `.AppImage`、`.deb`、`.rpm` |
| Windows | `.msi`、`.exe` |
| macOS（Apple Silicon） | `.dmg` |
| Android 7.0 以降 | `Query2Table_<version>_android.apk` |

コンピューターの設定をスマートフォンに移すには、**設定 → 設定をエクスポート…** を使い、**API キーとプロキシを含める** にチェックを入れて、そのファイルをスマートフォンでインポートします。このファイルにはキーが含まれるため、非公開にし、インポート後は削除してください。

Android では、ダウンロードした APK を開き、求められたらブラウザーまたはファイルマネージャーからのアプリのインストールを許可してください。Android アプリはデータをスマートフォン内に保存します。トレイはなく、通知は通常の Android 通知として表示されます。ローカルのモデルサーバー（Ollama、llama.cpp）は、ネットワーク経由でアドレスを指定して使用できます。例：`http://192.168.1.10:11434`。

## 機能

- **自然言語のクエリ** — 欲しいものをふだんの言葉で説明できます
- **スキーマの自動推定** — アプリがテーブルの列を提案し、実行前に確定または編集できます
- **複数の検索プロバイダー** — Brave Search と Serper を自動フォールバック付きで使用
- **多言語へのクエリ拡張** — 複数の言語と地域にまたがって検索します
- **LLM による抽出** — OpenRouter、Anthropic Claude（プロンプトキャッシュ、PDF 全体の読み取りにも対応）、Ollama（ローカルまたは Cloud）、DeepSeek、Groq、Mistral、Gemini、xAI、OpenAI、LM Studio、llama.cpp、または任意の OpenAI 互換アドレス
- **ストリーミング表示** — 見つかった行が順次ライブテーブルに表示されます
- **PDF の抽出** — 検索結果に含まれる PDF ドキュメントを自動で検出し、テキストを抽出します
- **画像検索モード** — テキストではなく画像を検索する専用パイプラインです。多様なクエリのバリエーションを 6～12 個生成し（LLM または静的なフォールバック）、Brave / Serper で画像検索を実行してから、厳密な関連度基準（0.0～1.0）で LLM がすべての結果を採点します。スコアが 0.7 未満の画像は除外され、残りはスコア順に保存され、画像ごとに UI がリアルタイムで更新されます。予算、キャンセル、停止条件（件数 / コスト / 時間）は常に守られます
- **リンク検索モード** — テーブルではなく最も関連性の高いページを返す専用パイプラインです。依頼内容から検索クエリのバリエーションを生成し、Brave / Serper でウェブ検索を実行して（URL で重複除去）、各ページを取得・解析します。その後、LLM が本文をすべて読み、クエリとの関連度（0.0～1.0）を採点しながら短い説明を生成します。信頼度のしきい値を下回るページは除外され、残りはスコア順に保存され、リンクごとに UI がリアルタイムで更新されます。予算、キャンセル、停止条件は常に守られます
- **リサーチモード** — 自由な問いにテーブルではなく Markdown で答えるエージェント型のモードです。LLM がツール呼び出しのループ（ウェブ検索、ページを Markdown として取得、思考、回答）を一定のステップ数まで進め、各ステップをライブのタイムラインに順次表示します。推論（`think`）ステップと失敗（赤で表示される `error` ステップ）も表示されるため、過程が透明に保たれます。出典付きの最終的な Markdown 回答は（サニタイズして）表示され、`.md` としてエクスポートできます。予算、キャンセル、停止条件は常に守られます
- **リサーチ回答内の数式と画像** — LaTeX の数式を描画します（MathJax、オフライン）。エージェントは画像を検索して確認し、適した画像を回答に含めることができ、画像は実行とともに保存されます
- **リサーチ回答内のダイアグラムとグラフ** — Mermaid のダイアグラムと、棒グラフ、折れ線グラフ、面グラフ、円グラフ、散布図、レーダーチャート、ヒートマップを、アプリのテーマに合わせてオフラインで描画します。SVG または PNG で保存でき、全画面でも開けます
- **リサーチ回答内の動画** — エージェントはあらゆるサイトで動画を探し（Web の動画検索、または YouTube・Bilibili・ニコニコ・SoundCloud を直接検索）、タイムスタンプ付きの字幕を読んで特定の場面を引用します。動画は保存済みのサムネイル付きカードで表示されます。初回使用時にダウンロードされる yt-dlp を使います（Android では内蔵の Python で実行）
- **使用量の統計** — 費用、トークン、モデルの呼び出し、Brave と Serper の検索、データ量を月別、年別、全期間で表示します。モデル別・モード別の内訳、コストの高い実行、ディスク使用量、CSV エクスポート、警告付きの月間予算も備えています
- **13 の表示言語** — 英語、ロシア語、ドイツ語、フランス語、スペイン語、イタリア語、ポルトガル語（ブラジル）、ポーランド語、ウクライナ語、トルコ語、中国語（簡体字）、日本語、韓国語。既定ではシステムの言語を使います
- **行ごとの出典** — すべての行が抽出元のページにリンクしています
- **エンティティの重複除去** — あいまい一致と LLM による判別
- **設定可能な停止条件** — 目標行数、最大コスト、最大時間
- **実行履歴** — 過去のリサーチの実行を閲覧、表示、再エクスポート
- **エクスポート** — 出典のメタデータをすべて含む CSV、JSON、XLSX
- **ダーク / ライトテーマ** — サイドバーで切り替え
- **システムトレイ** — 完了の通知（デスクトップ）
- **Android アプリ** — 小さな画面向けのレイアウトを備えた、スマートフォン版の同じアプリ
- **アプリ内アップデート** — 新しいバージョンは変更内容とともに通知され、署名の確認後にアプリからインストールされます（デスクトップと Android）
- **ローカルファースト** — すべてのデータを SQLite に保存し、クラウドのバックエンドは不要です

## 技術スタック

| レイヤー | 技術 |
|-------|------------|
| アプリシェル | Tauri v2（デスクトップと Android） |
| バックエンド | Rust（Tokio 非同期ランタイム） |
| フロントエンド | Svelte 5（SvelteKit SPA） |
| UI のスタイル | デザイントークンを使ったプレーンな CSS |
| データベース | SQLite（sqlx、WAL モード） |
| LLM | OpenRouter / Ollama（ローカル） / Ollama Cloud / OpenAI 互換 API（例：llama.cpp） |
| 検索 | Brave Search API / Serper API |
| アイコン | Lucide |

## 必要なもの

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install)（stable ツールチェーン）
- お使いの OS 向けの [Tauri v2 の前提条件](https://v2.tauri.app/start/prerequisites/)

## はじめに

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

## 設定

初回起動時に、アプリは既定の設定でローカルの SQLite データベースを作成します。サイドバーから **設定** を開いて、次の項目を設定します。

| グループ | 設定 |
|-------|----------|
| **LLM プロバイダー** | プロバイダー（OpenRouter / Ollama / Ollama Cloud / OpenAI 互換）、URL、モデル、API キー、JSON モード、温度、最大トークン |
| **検索プロバイダー** | プロバイダー（Brave / Serper）、API キー、フォールバックの切り替え、クエリあたりの結果数 |
| **実行** | 並列取得数（既定 8）、並列抽出数（既定 3）、取得タイムアウト、レート制限、robots.txt |
| **品質** | 適合率と再現率のバランス、根拠の厳密さ、信頼度のしきい値、重複とみなす類似度 |

停止条件（目標行数、最大コスト、最大時間）は、クエリページでクエリごとに設定します。

**もう一方のプロバイダーを予備に使う** は、ウェブ検索と画像検索の両方で Brave/Serper の自動フォールバックを制御します。オフにすると、両方の API キーが保存されていても、選択したメインのプロバイダーだけを使用します。この選択は保存後の新しい実行に適用され、再試行は選択したプロバイダーのまま行われます。オンの場合、もう一方のプロバイダーは失敗した後にのみ使われ、そのキーが保存されている必要があります。

**設定 → 検索 → クエリあたりの結果数** で、リサーチを含む 4 つのモードすべてで各検索クエリが要求する結果数を指定し、保存すると新しい実行に適用されます。Brave へのリクエストでは、この値がドキュメントに記載されたエンドポイントの範囲に制限されます：[ウェブ検索は 1～20](https://api-dashboard.search.brave.com/api-reference/web/search/get)、[画像は最大 200](https://api-dashboard.search.brave.com/documentation/services/image-search)。以前に保存された範囲外の値にも適用されます。大きな値を指定しても、追加のページが自動で取得されることはありません。

### 使用量、料金、支出上限

**使用量とコスト** では、プロバイダーが報告した請求額、モデル単価による見積もり、部分的な合計、不明なコストを区別します。再試行、フォールバック検索、使用量データのある失敗した応答、最終的な請求額を取得できなかったキャンセル済みリクエストを含め、プロバイダーへの試行はすべて個別に数えられます。入力と出力の使用量はプロバイダーの応答から取得します。思考トークンが出力トークンに加えて二重に課金されることはありません。

- **OpenRouter：** [報告されたリクエストのコスト](https://openrouter.ai/docs/cookbook/administration/usage-accounting)を優先します。取得できない場合は、報告されたトークン数と、実際に応答したモデルの公開カタログ単価から見積もります。
- **Ollama Cloud：** [公開されているモデル単価](https://ollama.com/pricing)から利用クレジットの消費量を見積もります。キャッシュ済み入力や、認識できたピーク料金の時間帯も考慮します。付属クレジット、サブスクリプションの支払い、現金での請求は、この見積もりと異なる場合があります。認識できない料金体系や取得できない料金は不明のままです。
- **カスタムのモデル単価：** 100 万トークンあたりの入力 / 出力の USD 単価を設定で指定します。単価は特定のプロバイダー、エンドポイント、モデルに紐づきます。明示的に 0 を指定すると無料として扱い、未設定の単価が暗黙に 0 になることはありません。公開単価は、保存済みの手動単価を変更せずに取得されます。
- **検索：** Brave と Serper は既定で 1,000 リクエストあたり USD 0 で、検索の料金は見積もりに含まれません。含めるにはご利用プランの実質単価を設定し、不明な場合は空欄のままにしてください。請求データのない失敗した試行は単価未設定のままです。設定の変更は新しい実行に適用され、保存済みの履歴は再計算されません。アップグレード時には、以前の空欄の既定値が一度だけ 0 になります。既存の数値の単価と、その後に意図的に空欄にしたものは保持されます。

実行の **最大コスト** は、計上された支出が上限に達すると、プロバイダーへの新しいリクエストを止めます。請求額の上限を保証するものではありません。見積もりは実際の請求と異なる場合があり、不明な請求額は含められず、すでに処理中のリクエストは課金される可能性があります。厳密な上限が必要な場合は、プロバイダー側の制限を使用してください。集計は、完了、失敗、キャンセル、結果が空の実行でも保持されます。以前の履歴項目には、過去の見積もりであることが明示されます。

重複除去では、最初のメンバーの行 ID を維持し、グループの値と出典の記録をトランザクションで統合して、ライブテーブルを履歴やエクスポートと一致させます。以前に上書きされた過去の結果は自動では復元できないため、新しい結果を得るにはそのクエリを再実行してください。

設定画面では、スクロールしても **保存** と **破棄** が常に表示されます。セクションのリンクから該当する項目へ直接移動でき、未保存の変更があるまま離れようとすると「保存 / 破棄 / このまま残る」を選べます。保存中に行った編集は、次の保存のために残ります。

結果は、ローカル検索とキーボードでの並べ替えに対応した、連続した仮想化テーブルで表示されます。行の **詳細** を開くと、完全な値を読んだりコピーしたりでき、保存された出典を確認できます。リサーチでは、回答が展開可能なアクティビティより先に表示されます。画像はグリッド / リスト表示とキーボードでのプレビューに対応し、エクスポートと削除では進行状況と結果がはっきり表示されます。診断用の **ログ** パネルを消去しても、実行のアクティビティはリセットされません。

ブラウザーでの回帰テストは、合成データとモック化した Tauri コマンドを使います。`npx playwright install chromium webkit` でテスト用ブラウザーをインストールし、`npm run test:ui` を実行してください。既存のフロントエンドのテストは `npm test` で実行します。

### API キー

少なくとも 1 つの検索 API キーと、設定済みの LLM プロバイダーが必要です。ローカルの Ollama と、認証のない OpenAI 互換サーバーには LLM の API キーは不要です。

| サービス | キーの取得先 |
|---------|-------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama（ローカル） | https://ollama.com/ — キー不要 |
| Ollama Cloud | https://ollama.com/settings/keys |
| OpenAI 互換 | サーバーに応じて任意のキー |

**設定 → LLM → プロバイダー** でプロバイダーを選び、各項目を入力して **保存** をクリックします。プロバイダーを切り替えても、それぞれのモデルと認証情報は保持されます。

設定の移行は起動時に自動で実行されます。新しく追加されたオプションには既定値が入り、既存の値は保持されます。名前が変わった設定は、既定値を挿入する前にバックエンドで明示的に移行されます。保存形式を変更する場合も、新しいバージョンの実装時に移行コードが必要です。

**設定** の下部にある **アプリケーションファイル** には、実際のデータフォルダー、`data.db` データベース、ログフォルダーが表示されます。**パスをコピー** で場所をシステムのクリップボードにコピーし、**フォルダーを開く** でファイルマネージャーで開けます。データベースの場所を開くと、それを含むフォルダーが開きます。設定、履歴、結果、出典は `data.db` に保存され、診断用のログは別に保存されます。

- **OpenRouter：** API キーを入力し、検索可能な [OpenRouter のカタログ](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties)からモデルを選択します。入力してモデル ID を絞り込み、マウスまたは矢印キーと Enter で選択します。**更新** で一覧を再読み込みできます。カタログはキーを入力する前でも閲覧できますが、推論にはキーが必要です。
- **Ollama（ローカル）：** URL `http://localhost:11434` と、インストール済みのモデル名を指定します。
- **Ollama Cloud：** URL `https://ollama.com`、Ollama の API キー、検索可能なサーバーカタログから選んだモデルを指定します。モデルの欄をクリックして入力すると絞り込まれ、マウスまたは矢印キーと Enter で選択します。**更新** でカタログを再読み込みします。絞り込みだけでは、結果を選ぶまで選択中のモデルは変わりません。クラウドへの直接アクセスには[ネイティブの Ollama API](https://docs.ollama.com/cloud) を使用します。現在 Cloud は[構造化出力](https://docs.ollama.com/capabilities/structured-outputs)に対応していないため、JSON はプロンプトで要求し、既存のパイプラインで解析します。
- **OpenAI 互換：** `/v1` を含む API ベース URL と、サーバーのモデル ID を入力します。[llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) の既定の URL は `http://localhost:8080/v1` です。読み込まれているモデル名か、設定したエイリアスを使用してください。サーバーが必要としない限り、API キーは空欄のままにします。サーバーが `response_format` を拒否する場合は **JSON モード** をオフにしてください。

**思考（推論レベル）** は、モデルが回答前にどの程度推論するかを制御します。**自動** は安全な既定値を維持します。Ollama の構造化リクエストでは思考をオフにし（GPT-OSS では「低」）、他のプロバイダーでは各プロバイダーの既定値を使います。**プロバイダーの既定** は推論をサーバーに任せます。**オフ**、**オン**、**低**、**中**、**高**、**最大** は明示的なモードまたはレベルを要求しますが、対応状況はモデルとプロバイダーによって異なります。Ollama の GPT-OSS は「低 / 中 / 高」を受け付け、「オフ」と「最大」は使用できません。変更は保存後の新しい実行に適用されます。

**最大出力トークン** はリクエストごとの上限で、思考と回答のトークンで共有される場合があります。実行のコスト / 時間の上限や、プロバイダーアカウントのクォータとは別のものです。推論レベルを上げると、回答に使えるトークンが少なくなることがあります。

モデルへのリクエストで問題が起きた場合は、プロバイダー、モデル、段階、要求した出力上限、報告された入力 / 出力 / 思考のトークン数、再試行が予定されたかどうかが表示されます。出力上限、コンテキスト上限、プロバイダーのクォータ、レート制限、認証、ネットワークの障害には、それぞれ個別の説明と対処方法が示されます。個々のページがスキップされた場合も問題は表示されたままで、実行履歴でも確認できます。その他のエラーでは、短い説明、取るべき対処、折りたたみ可能な技術的な詳細が表示されます。

**一時停止** は、スキーマの確認を含め、現在の段階で処理を中断します。**再開** はリクエストを再送信せずに処理を続けます。リモートのモデルにすでに送信されたリクエストは、一時停止中でもそちらで完了する場合があります。**キャンセル** はローカルの処理を停止し、保留中のリクエストを閉じます。パイプラインの失敗は `failed` として表示・保存され、元のエラーは実行ログで確認できます。

## アーキテクチャ

バックエンドは、固定された役割を順に実行する**パイプラインのステートマシン**を使用します。

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### パイプラインの役割

| 役割 | LLM | 目的 |
|------|-----|---------|
| QueryInterpreter | はい | クエリを構造化された意図に解析 |
| SchemaPlanner | はい | テーブルの列と型を提案 |
| SearchPlanner | はい | 複数の言語と地域向けの検索クエリを生成 |
| QueryExpander | はい | クエリを対象言語に翻訳 |
| SearchExecutor | いいえ | 検索 API を呼び出し、候補の URL を収集 |
| Fetcher | いいえ | レート制限と robots.txt に従った HTTP 取得 |
| DocumentParser | いいえ | HTML → クリーンなテキスト（定型部分の除去） |
| PdfParser | いいえ | PDF → クリーンなテキスト（pdf-extract を使用） |
| Extractor | はい | テキストとスキーマ → 構造化された行 |
| Validator | 一部 | スキーマへの適合と意味的なチェック |
| Deduplicator | 一部 | あいまい一致（strsim）と、判断が難しい場合の LLM |
| LinkRanker | はい | ページの内容を読んで関連度（0.0～1.0）を採点し、説明を生成（リンク検索モード） |
| ResearchAgent | はい | 検索 / 取得 / 思考 / 回答のエージェントループを進め、Markdown の回答を作成（リサーチモード） |
| StoppingController | いいえ | 停止条件（行数、予算、時間、飽和）を評価 |

### ステートマシン

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## プロジェクト構成

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

## データベース

WAL モードの SQLite で、次のテーブルがあります。

| テーブル | 目的 |
|-------|---------|
| `settings` | ユーザーの設定（API キー、モデル、各種設定） |
| `runs` | ステータスと統計を含むリサーチの実行記録 |
| `run_schemas` | 実行ごとの確定済みテーブルスキーマ |
| `search_queries` | 実行ごとに生成された検索クエリ |
| `search_results` | 検索 API から収集した URL |
| `fetched_pages` | ダウンロードしたページの内容 |
| `entity_rows` | 抽出された構造化行（JSON データ） |
| `row_sources` | 行ごとの根拠（URL、タイトル、抜粋） |
| `image_results` | 実行ごとの順位付けされた画像（画像検索モード） |
| `link_results` | 説明とスコア付きの、順位付けされた関連リンク（リンク検索モード） |
| `research_steps` | 実行ごとのエージェントのツール呼び出しステップ — search/fetch/think/error（リサーチモード） |
| `research_results` | 実行ごとの最終的な Markdown 回答（リサーチモード） |
| `run_logs` | 実行ごとの処理ログ |

## 開発

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

[Tauri の Android 向け前提条件](https://v2.tauri.app/start/prerequisites/#android)が必要です：Android SDK と NDK（`ANDROID_HOME`、`NDK_HOME`）、Java 17（`JAVA_HOME`）、Rust のターゲット（`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`）。USB デバッグを有効にしたスマートフォンを接続するか、エミュレーターを起動してください。

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

`src-tauri/gen/android/keystore.properties` がない場合、ビルドはデバッグ用の鍵で署名されます。`make android-keystore` はリリース用の鍵とこのファイルを作成し、リリースワークフローが各リリースに追加する APK の署名に使う GitHub のシークレット（`ANDROID_KEYSTORE_BASE64`、`ANDROID_KEY_PASSWORD`、`ANDROID_KEY_ALIAS`）を表示します。鍵は大切に保管してください。アップデートは、同じ鍵で署名されている場合にのみ既存のアプリに上書きインストールされます。

## エクスポート形式

| 形式 | 内容 |
|--------|----------|
| **CSV** | すべての列と、JSON 列としての出典 |
| **JSON** | 出典の入れ子配列と実行のメタデータを含む完全な行 |
| **XLSX** | データシート、出典シート、メタデータシートを含む書式付きブック |

## ライセンス

MIT
