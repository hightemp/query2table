<p align="center">
  <img src="../../images/query2table-logo.png" alt="Logotipo do Query2Table" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <b>Português (Brasil)</b> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="Versão no GitHub"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="Downloads no GitHub"></a>
  <a href="#licença"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Licença: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Feito com Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend em Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend em Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Visualizações do Query2Table">
</p>

**Query2Table** é uma ferramenta de pesquisa local-first para desktop (Linux, Windows, macOS) e Android. Você descreve os dados de que precisa; o app pesquisa a internet de forma autônoma em várias fontes e idiomas, baixa as páginas, extrai entidades estruturadas com LLMs, remove duplicatas e apresenta o resultado como uma tabela atualizada em tempo real, com links para as fontes de cada linha — tudo rodando na sua máquina, sem backend na nuvem.

Além das tabelas, ele oferece mais três modos dedicados: **busca de imagens**, que encontra imagens relevantes e as classifica com um LLM pela relevância visual; **busca de links**, que lê o conteúdo completo das páginas candidatas e retorna só os links mais relevantes, com descrições e notas de relevância geradas pelo LLM; e **pesquisa**, um modo com agente em que o modelo busca, lê páginas, raciocina e escreve uma resposta em Markdown com fontes, de forma autônoma.

O app também está disponível para celulares **Android**: o mesmo app, com os mesmos modos, configurações e histórico, compilado a partir do mesmo código e adaptado à tela do celular, com uma barra de abas na parte inferior.

Peça algo como *"Monte uma tabela comparativa de bancos de dados e bibliotecas open source de busca vetorial, com linguagem, licença, tipo de implantação, caso de uso e site"* e veja a tabela se preencher em tempo real.

![Tabela comparativa de bancos de dados vetoriais open source com evidências e fontes](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Visão geral

### Pergunte em linguagem natural

Escolha um formato de resultado, descreva o que você precisa e defina as condições de parada. Nas tabelas, o app propõe colunas que você pode editar antes de a busca começar.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Nova consulta com os quatro formatos de resultado"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Esquema de tabela proposto, pronto para revisão"></td>
  </tr>
</table>

### Cada linha guarda suas evidências

Abra uma linha para ler os valores completos, copiá-los e conferir as páginas de onde foram extraídos.

![Detalhes da linha com confiança da extração e fontes](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Pesquisa com perguntas seguintes

O agente busca, lê páginas e escreve uma resposta com fontes e sumário. Cada etapa fica visível em Atividade, e as perguntas seguintes continuam a conversa com o histórico dela.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Resposta da pesquisa com sumário e perguntas seguintes sugeridas"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Atividade da pesquisa com etapas numeradas de busca e leitura"></td>
  </tr>
</table>

### Imagens e links

A busca de imagens classifica os resultados por relevância, com filtros por site e tamanho; a busca de links lê cada página e explica por que ela corresponde.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Resultados da busca de imagens em uma grade justificada"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Links relevantes com descrições e níveis de correspondência"></td>
  </tr>
</table>

### Histórico e configurações

Pesquise, filtre e reabra execuções anteriores, execute-as de novo ou exporte várias de uma vez. As configurações testam suas conexões com o LLM e a busca antes de você começar.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Histórico de execuções agrupado por dia, com resultados, duração, custo e modelo"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Configurações de busca com um teste de conexão bem-sucedido"></td>
  </tr>
</table>

### Temas claro e escuro

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Resposta da pesquisa no tema escuro"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Tabela de resultados no tema escuro"></td>
  </tr>
</table>

## Download

Baixe a versão mais recente em [Releases](https://github.com/hightemp/query2table/releases/latest):

| Plataforma | Arquivo |
|------------|---------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Para levar sua configuração do computador para o celular, use **Configurações → Exportar configurações…**, marque **Incluir chaves de API e proxies** e importe o arquivo no celular. Esse arquivo contém suas chaves: mantenha-o em sigilo e exclua-o depois de importar.

No Android, abra o APK baixado e, quando solicitado, permita a instalação de apps pelo navegador ou gerenciador de arquivos. O app para Android guarda os dados no celular; ele não tem ícone na bandeja, e as notificações aparecem como notificações normais do Android. Servidores de modelos locais (Ollama, llama.cpp) podem ser usados pela rede pelo endereço deles, por exemplo `http://192.168.1.10:11434`.

## Recursos

- **Consultas em linguagem natural** — descreva o que você quer com suas próprias palavras
- **Inferência automática do esquema** — o app propõe as colunas da tabela; você confirma ou edita antes da execução
- **Busca com vários provedores** — Brave Search + Serper com reserva automática
- **Ampliação multilíngue das consultas** — buscas em vários idiomas e regiões
- **Extração com LLMs** — OpenRouter, Anthropic Claude (cache de prompt, leitura opcional de PDFs inteiros), Ollama (local ou Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp ou qualquer endereço compatível com OpenAI
- **Resultados em tempo real** — as linhas aparecem em uma tabela ao vivo assim que são encontradas
- **Extração de PDF** — detecta e extrai automaticamente o texto de documentos PDF encontrados nos resultados de busca
- **Modo de busca de imagens** — um pipeline dedicado que busca imagens em vez de texto: gera de 6 a 12 variações diversas da consulta (por LLM ou com alternativas estáticas), faz buscas de imagens pelo Brave / Serper e classifica cada resultado com um LLM segundo critérios rígidos de relevância (0,0–1,0); imagens com nota abaixo de 0,7 são descartadas, e as demais são salvas por ordem de nota, com a interface atualizada imagem por imagem em tempo real; orçamento, cancelamento e condições de parada (quantidade / custo / tempo) são respeitados do início ao fim
- **Modo de busca de links** — um pipeline dedicado que retorna as páginas mais relevantes em vez de uma tabela: gera variações de consultas de busca a partir do seu pedido, faz buscas na web pelo Brave / Serper (sem URLs duplicadas), baixa e analisa cada página e faz um LLM ler o conteúdo completo, dar uma nota de relevância (0,0–1,0) em relação à sua consulta e escrever uma descrição curta; páginas abaixo do limite de confiança são descartadas, e as demais são salvas por ordem de nota, com a interface atualizada link por link em tempo real; orçamento, cancelamento e condições de parada são respeitados do início ao fim
- **Modo de pesquisa** — um modo com agente que responde a perguntas abertas em Markdown em vez de uma tabela: o LLM conduz um ciclo de chamadas de ferramentas (buscar na web, baixar uma página como Markdown, pensar, responder) por até um número fixo de etapas, transmitindo cada etapa para uma linha do tempo ao vivo; as etapas de raciocínio (`think`) e as falhas (etapas `error`, em vermelho) ficam visíveis para que o processo seja transparente, e a resposta final em Markdown com fontes é renderizada (sanitizada) e pode ser exportada como `.md`; orçamento, cancelamento e condições de parada são respeitados do início ao fim
- **Fórmulas e imagens nas respostas da pesquisa** — fórmulas LaTeX são desenhadas (MathJax, offline); o agente pode buscar imagens, analisá-las e colocar as adequadas na resposta, guardadas com a execução
- **Diagramas e gráficos nas respostas da pesquisa** — diagramas Mermaid e gráficos de barras, linhas, área, pizza, dispersão, radar e mapa de calor, desenhados offline no tema do app; salvos como SVG ou PNG e abertos em tela cheia
- **Estatísticas de uso** — dinheiro, tokens, chamadas aos modelos, buscas no Brave e no Serper e volume de dados por mês, ano e todo o período, com detalhamento por modelo e modo, as execuções mais caras, uso de disco, exportação em CSV e um orçamento mensal com avisos
- **13 idiomas de interface** — inglês, russo, alemão, francês, espanhol, italiano, português (Brasil), polonês, ucraniano, turco, chinês (simplificado), japonês e coreano; por padrão, usa o idioma do sistema
- **Fontes por linha** — cada linha tem links para as páginas de onde foi extraída
- **Remoção de duplicatas de entidades** — correspondência aproximada + desambiguação assistida por LLM
- **Condições de parada configuráveis** — número de linhas desejado, custo máximo, duração máxima
- **Histórico de execuções** — navegue, visualize e exporte de novo execuções de pesquisa anteriores
- **Exportação** — CSV, JSON, XLSX com todos os metadados das fontes
- **Tema escuro / claro** — alterne na barra lateral
- **Bandeja do sistema** — notificações de conclusão (desktop)
- **App para Android** — o mesmo app no celular, com um layout para telas pequenas
- **Atualizações no app** — novas versões são anunciadas com suas mudanças e instaladas pelo próprio app depois de verificada a assinatura (desktop e Android)
- **Local-first** — todos os dados no SQLite, sem precisar de backend na nuvem

## Tecnologias

| Camada | Tecnologia |
|--------|------------|
| Shell do app | Tauri v2 (desktop e Android) |
| Backend | Rust (runtime assíncrono Tokio) |
| Frontend | Svelte 5 (SvelteKit SPA) |
| Estilos da interface | CSS puro com design tokens |
| Banco de dados | SQLite (sqlx, modo WAL) |
| LLM | OpenRouter / Ollama (local) / Ollama Cloud / API compatível com OpenAI (por exemplo, llama.cpp) |
| Busca | Brave Search API / Serper API |
| Ícones | Lucide |

## Pré-requisitos

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (toolchain stable)
- [Pré-requisitos do Tauri v2](https://v2.tauri.app/start/prerequisites/) para o seu sistema operacional

## Primeiros passos

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

## Configuração

Na primeira execução, o app cria um banco de dados SQLite local com as configurações padrão. Abra **Configurações** na barra lateral para definir:

| Grupo | Configurações |
|-------|---------------|
| **Provedor de LLM** | Provedor (OpenRouter / Ollama / Ollama Cloud / compatível com OpenAI), URL, modelo, chave de API, modo JSON, temperatura, máximo de tokens |
| **Provedor de busca** | Provedor (Brave / Serper), chave de API, reserva ativada ou não, resultados por consulta |
| **Execução** | Carregamentos em paralelo (padrão 8), extrações em paralelo (padrão 3), tempo limite de carregamento, limitação de taxa, robots.txt |
| **Qualidade** | Equilíbrio entre precisão e abrangência, rigor das evidências, limite de confiança, semelhança de duplicatas |

As condições de parada (linhas desejadas, custo máximo, duração máxima) são definidas por consulta na página Consulta.

**Usar o outro provedor como reserva** controla a troca automática entre Brave e Serper nas buscas na web e de imagens. Desative para usar só o provedor principal selecionado, mesmo que as duas chaves de API estejam salvas. A escolha vale para novas execuções depois de Salvar; as novas tentativas continuam no provedor selecionado. Quando ativada, o outro provedor só é usado depois de uma falha e precisa ter a chave salva.

Defina **Configurações → Busca → Resultados por consulta** para escolher quantos resultados cada consulta de busca pede nos quatro modos, incluindo Pesquisa, e clique em Salvar para as novas execuções. As solicitações ao Brave limitam esse valor ao intervalo documentado do endpoint: [1–20 na busca na web](https://api-dashboard.search.brave.com/api-reference/web/search/get) e [até 200 nas imagens](https://api-dashboard.search.brave.com/documentation/services/image-search). Isso também vale para valores grandes demais salvos anteriormente. Valores maiores não acionam páginas extras automáticas.

### Uso, preços e limites de gastos

**Uso e custo** distingue cobranças informadas pelo provedor, estimativas pelas tarifas do modelo, totais parciais e custos desconhecidos. Cada tentativa junto a um provedor é contada separadamente, incluindo novas tentativas, buscas de reserva, conclusões com falha que têm dados de uso e solicitações canceladas cuja cobrança final não pôde ser obtida. O uso de entrada/saída vem das respostas dos provedores; os tokens de raciocínio não são cobrados uma segunda vez além dos tokens de saída.

- **OpenRouter:** dá preferência ao [custo informado da solicitação](https://openrouter.ai/docs/cookbook/administration/usage-accounting). Se ele não estiver disponível, estima a partir dos tokens informados e das tarifas públicas do catálogo para o modelo efetivamente retornado.
- **Ollama Cloud:** estima o consumo de créditos de uso a partir das [tarifas publicadas dos modelos](https://ollama.com/pricing), incluindo entrada em cache e os períodos de preço de pico reconhecidos. Créditos incluídos, pagamentos de assinatura e cobranças em dinheiro podem diferir dessa estimativa. Tarifas não reconhecidas ou indisponíveis ficam como desconhecidas.
- **Tarifas personalizadas dos modelos:** configure em USD por milhão de tokens de entrada/saída nas Configurações. As tarifas pertencem a um provedor, endpoint e modelo específicos. Tarifas zero explícitas significam uso gratuito; tarifas ausentes nunca viram zero silenciosamente. As tarifas publicadas são obtidas sem alterar as tarifas manuais salvas.
- **Busca:** Brave e Serper têm por padrão USD 0 por 1.000 solicitações, o que exclui as cobranças de busca das estimativas. Informe a tarifa efetiva do seu plano para incluí-las ou deixe o preço em branco se não souber. Tentativas com falha sem dados de cobrança ficam sem preço. Mudanças nas configurações valem para novas execuções; o histórico salvo não é recalculado. Na atualização, os antigos padrões vazios viram zero uma única vez, preservando as tarifas numéricas existentes e os campos deixados em branco de propósito depois disso.

O **Custo máximo** da execução impede novas solicitações aos provedores quando os gastos contabilizados atingem o limite. Ele não é um teto garantido da fatura: as estimativas podem diferir da cobrança, cobranças desconhecidas não podem ser incluídas e solicitações já em andamento ainda podem gerar cobranças. Use os limites do lado do provedor quando precisar de um teto rígido. A contabilização se mantém em execuções concluídas, com falha, canceladas e sem resultados. Entradas antigas do histórico são rotuladas explicitamente como estimativas históricas.

A remoção de duplicatas preserva o ID da linha do primeiro membro, mescla os valores e os registros de fontes do grupo de forma transacional e sincroniza a tabela ao vivo com o histórico e a exportação. Resultados históricos sobrescritos anteriormente não podem ser reconstruídos automaticamente; execute essas consultas de novo para obter resultados novos.

As configurações mantêm **Salvar** e **Descartar** visíveis durante a rolagem. Os links das seções levam direto aos controles correspondentes, e sair com alterações não salvas oferece Salvar / Descartar / Ficar. As edições feitas durante um salvamento ficam disponíveis para o próximo.

Os resultados usam uma tabela contínua e virtualizada, com pesquisa local e ordenação pelo teclado. Abra os **Detalhes** de uma linha para ler ou copiar os valores completos e conferir as fontes salvas. A Pesquisa mostra a resposta antes da atividade expansível. As Imagens têm exibição em grade/lista e visualização pelo teclado; exportação e exclusão mostram progresso e resultados explícitos. Limpar o painel de diagnóstico **Logs** não apaga a atividade da execução.

As verificações de regressão no navegador usam dados sintéticos e comandos Tauri simulados: instale os navegadores de teste com `npx playwright install chromium webkit` e depois rode `npm run test:ui`. Os testes de frontend existentes rodam com `npm test`.

### Chaves de API

Você precisa de pelo menos uma chave de API de busca e de um provedor de LLM configurado. O Ollama local e os servidores compatíveis com OpenAI sem autenticação não exigem chave de API de LLM:

| Serviço | Obtenha uma chave em |
|---------|----------------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (local) | https://ollama.com/ — não precisa de chave |
| Ollama Cloud | https://ollama.com/settings/keys |
| Compatível com OpenAI | Chave opcional, dependendo do seu servidor |

Selecione o provedor em **Configurações → Provedor de LLM**, preencha os campos dele e clique em **Salvar**. Cada provedor mantém o próprio modelo e as próprias credenciais quando você troca de um para outro.

As migrações das configurações rodam automaticamente ao iniciar: as opções recém-adicionadas recebem os valores padrão e os valores existentes são preservados. Configurações renomeadas são migradas explicitamente no backend antes de os padrões serem inseridos; mudanças nos formatos armazenados também exigem código de migração ao implementar uma nova versão.

No fim de **Configurações**, **Arquivos do aplicativo** mostra a pasta de dados real, o banco de dados `data.db` e a pasta de logs. Use **Copiar caminho** para copiar um local para a área de transferência do sistema ou **Abrir pasta** para abri-lo no gerenciador de arquivos. Abrir o local do banco de dados abre a pasta que o contém. Configurações, histórico, resultados e fontes ficam em `data.db`; os logs de diagnóstico são guardados à parte.

- **OpenRouter:** informe sua chave de API e selecione um modelo no [catálogo do OpenRouter](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties), que pode ser pesquisado. Digite para filtrar os IDs dos modelos, escolha com o mouse ou com as setas e Enter, e use **Atualizar** para recarregar a lista. O catálogo pode ser consultado antes de informar uma chave; a chave é necessária para a inferência.
- **Ollama (local):** URL `http://localhost:11434` e o nome de um modelo instalado.
- **Ollama Cloud:** URL `https://ollama.com`, uma chave de API do Ollama e um modelo selecionado no catálogo do servidor, que pode ser pesquisado. Clique no campo do modelo e digite para filtrar, depois escolha com o mouse ou com as setas e Enter; **Atualizar** recarrega o catálogo. Filtrar não altera o modelo selecionado até você escolher um resultado. O acesso direto à nuvem usa a [API nativa do Ollama](https://docs.ollama.com/cloud). No momento, a nuvem não tem [suporte a saída estruturada](https://docs.ollama.com/capabilities/structured-outputs), por isso o JSON é pedido pelos prompts e analisado pelo pipeline existente.
- **Compatível com OpenAI:** informe a URL base da API incluindo `/v1` e o ID do modelo no servidor. No [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server), a URL padrão é `http://localhost:8080/v1`; use o nome do modelo carregado ou o alias configurado. Deixe a chave de API em branco, a menos que o servidor exija uma. Desative o **Modo JSON** se o servidor rejeitar `response_format`.

**Raciocínio** controla quanto o modelo raciocina antes de responder. **Automático** mantém os padrões seguros: as solicitações estruturadas do Ollama desativam o raciocínio (o GPT-OSS usa Baixo), enquanto os outros provedores mantêm seus padrões. **Padrão do provedor** deixa o raciocínio a cargo do servidor. **Desativado**, **Ativado**, **Baixo**, **Médio**, **Alto** e **Máximo** pedem um modo ou nível explícito; o suporte depende do modelo e do provedor. O GPT-OSS no Ollama aceita Baixo/Médio/Alto e não pode usar Desativado nem Máximo. As alterações valem para novas execuções depois de Salvar.

**Máximo de tokens de saída** é uma cota por solicitação, que pode ser compartilhada entre os tokens de raciocínio e os da resposta. Ela é separada dos limites de custo/tempo da execução e da cota da conta no provedor. Um nível de raciocínio mais alto pode deixar menos tokens para a resposta.

Os problemas nas solicitações aos modelos mostram o provedor, o modelo, a etapa, o limite de saída solicitado, as contagens de tokens de entrada/saída/raciocínio informadas e se uma nova tentativa foi agendada. Limites de saída, limites de contexto, cotas do provedor, limites de taxa, autenticação e falhas de rede têm explicações e ações corretivas próprias. Os problemas continuam visíveis quando páginas individuais são ignoradas e ficam disponíveis no Histórico de execuções. Outros erros mostram uma explicação curta, uma ação a tomar e detalhes técnicos recolhíveis.

**Pausar** suspende o processamento na etapa atual, inclusive na revisão do esquema, e **Retomar** continua sem reenviar a solicitação. Uma solicitação já enviada a um modelo remoto ainda pode terminar lá durante a pausa. **Cancelar** interrompe o trabalho local e fecha as solicitações pendentes. As falhas do pipeline são exibidas e salvas como `failed`, com o erro subjacente disponível no log da execução.

## Arquitetura

O backend usa uma **máquina de estados do pipeline** com papéis fixos orquestrados em sequência:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Papéis do pipeline

| Papel | LLM | Função |
|-------|-----|--------|
| QueryInterpreter | Sim | Transforma a consulta em uma intenção estruturada |
| SchemaPlanner | Sim | Propõe as colunas e os tipos da tabela |
| SearchPlanner | Sim | Gera consultas de busca para vários idiomas/regiões |
| QueryExpander | Sim | Traduz as consultas para os idiomas de destino |
| SearchExecutor | Não | Chama as APIs de busca e coleta as URLs candidatas |
| Fetcher | Não | Download HTTP com limitação de taxa e robots.txt |
| DocumentParser | Não | HTML → texto limpo (remoção de conteúdo repetitivo) |
| PdfParser | Não | PDF → texto limpo (via pdf-extract) |
| Extractor | Sim | Texto + esquema → linhas estruturadas |
| Validator | Parcial | Conformidade com o esquema + verificações semânticas |
| Deduplicator | Parcial | Correspondência aproximada (strsim) + LLM nos casos limítrofes |
| LinkRanker | Sim | Lê o conteúdo da página, dá uma nota de relevância (0,0–1,0) e gera uma descrição (modo de busca de links) |
| ResearchAgent | Sim | Conduz o ciclo do agente de busca/download/raciocínio/resposta e produz uma resposta em Markdown (modo de pesquisa) |
| StoppingController | Não | Avalia as condições de parada (linhas, orçamento, tempo, saturação) |

### Máquina de estados

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Estrutura do projeto

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

## Banco de dados

SQLite em modo WAL com as seguintes tabelas:

| Tabela | Função |
|--------|--------|
| `settings` | Configuração do usuário (chaves de API, modelos, preferências) |
| `runs` | Registros das execuções de pesquisa, com status e estatísticas |
| `run_schemas` | Esquema da tabela confirmado por execução |
| `search_queries` | Consultas de busca geradas por execução |
| `search_results` | URLs coletadas das APIs de busca |
| `fetched_pages` | Conteúdo das páginas baixadas |
| `entity_rows` | Linhas estruturadas extraídas (dados em JSON) |
| `row_sources` | Evidências por linha (URL, título, trecho) |
| `image_results` | Imagens classificadas por execução (modo de busca de imagens) |
| `link_results` | Links relevantes classificados, com descrições e notas (modo de busca de links) |
| `research_steps` | Etapas de chamadas de ferramentas do agente por execução — search/fetch/think/error (modo de pesquisa) |
| `research_results` | Resposta final em Markdown por execução (modo de pesquisa) |
| `run_logs` | Logs de execução por execução |

## Desenvolvimento

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

É preciso ter os [pré-requisitos do Tauri para Android](https://v2.tauri.app/start/prerequisites/#android): Android SDK e NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) e os targets do Rust (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Conecte um celular com a depuração USB ativada ou inicie um emulador.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

As compilações são assinadas com a chave de depuração, a menos que exista `src-tauri/gen/android/keystore.properties`. `make android-keystore` cria uma chave de release e esse arquivo, e mostra os secrets do GitHub (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`) que o workflow de release usa para assinar o APK que adiciona a cada versão. Guarde a chave: as atualizações só se instalam sobre um app existente quando assinadas com a mesma chave.

## Formatos de exportação

| Formato | Conteúdo |
|---------|----------|
| **CSV** | Todas as colunas + fontes como coluna JSON |
| **JSON** | Linhas completas com um array aninhado de fontes e os metadados da execução |
| **XLSX** | Pasta de trabalho formatada com planilha de dados, planilha de fontes e planilha de metadados |

## Licença

MIT
