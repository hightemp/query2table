<p align="center">
  <img src="../../images/query2table-logo.png" alt="Logotipo de Query2Table" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <b>Español</b> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="Versión en GitHub"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="Descargas en GitHub"></a>
  <a href="#licencia"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Licencia: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Hecho con Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend en Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend en Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Visitas de Query2Table">
</p>

**Query2Table** es una herramienta de investigación local para escritorio (Linux, Windows, macOS) y Android. Describes qué datos necesitas y la aplicación busca de forma autónoma en internet, en varias fuentes e idiomas, descarga las páginas, extrae entidades estructuradas con LLM, elimina los duplicados y presenta el resultado como una tabla que se actualiza en directo, con enlaces a las fuentes de cada fila. Todo se ejecuta en tu equipo, sin un backend en la nube.

Además de las tablas, ofrece tres modos específicos más: **búsqueda de imágenes**, que encuentra imágenes relevantes y las clasifica con un LLM según su relevancia visual; **búsqueda de enlaces**, que lee el contenido completo de las páginas candidatas y devuelve solo los enlaces más relevantes, con descripciones y puntuaciones de relevancia generadas por el LLM; e **investigación**, un modo agéntico en el que el modelo busca, lee páginas, razona y escribe de forma autónoma una respuesta en Markdown con fuentes.

La aplicación también está disponible para teléfonos **Android**: la misma aplicación, con los mismos modos, ajustes e historial, compilada a partir del mismo código y adaptada a la pantalla del teléfono con una barra de pestañas inferior.

Pide algo como *«Crea una tabla comparativa de bases de datos y bibliotecas de búsqueda vectorial de código abierto con su lenguaje, licencia, tipo de despliegue, caso de uso y sitio web»* y mira cómo la tabla se llena en tiempo real.

![Tabla comparativa de bases de datos vectoriales de código abierto con evidencias y fuentes](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Recorrido

### Pregunta con lenguaje natural

Elige un formato de resultado, describe lo que necesitas y fija las condiciones de parada. En las tablas, la aplicación propone columnas que puedes editar antes de que empiece la búsqueda.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Nueva consulta con los cuatro formatos de resultado"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Esquema de tabla propuesto, listo para revisar"></td>
  </tr>
</table>

### Cada fila conserva sus evidencias

Abre una fila para leer los valores completos, copiarlos y comprobar las páginas de las que se extrajeron.

![Detalles de una fila con la confianza de la extracción y las fuentes](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Investigación con preguntas de seguimiento

El agente busca, lee páginas y escribe una respuesta con fuentes y un índice de contenidos. Cada paso se ve en Actividad, y las preguntas de seguimiento continúan la conversación con su historial.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Respuesta de investigación con índice y preguntas de seguimiento sugeridas"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Actividad de la investigación con los pasos de búsqueda y lectura numerados"></td>
  </tr>
</table>

### Imágenes y enlaces

La búsqueda de imágenes ordena los resultados por relevancia, con filtros por sitio y tamaño; la búsqueda de enlaces lee cada página y explica por qué coincide.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Resultados de la búsqueda de imágenes en una cuadrícula justificada"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Enlaces relevantes con descripciones y niveles de coincidencia"></td>
  </tr>
</table>

### Historial y ajustes

Busca, filtra y vuelve a abrir ejecuciones anteriores, ejecútalas de nuevo o exporta varias a la vez. Los ajustes comprueban tus conexiones de LLM y de búsqueda antes de empezar.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Historial de ejecuciones agrupado por día con resultados, duración, coste y modelo"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Ajustes de búsqueda con una prueba de conexión correcta"></td>
  </tr>
</table>

### Temas claro y oscuro

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Respuesta de investigación en el tema oscuro"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Tabla de resultados en el tema oscuro"></td>
  </tr>
</table>

## Descarga

Descarga la última versión desde [Releases](https://github.com/hightemp/query2table/releases/latest):

| Plataforma | Archivo |
|------------|---------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Para pasar tu configuración del ordenador al teléfono, usa **Ajustes → Exportar ajustes…**, marca **Incluir claves de API y proxies** e importa el archivo en el teléfono. Ese archivo contiene tus claves: mantenlo en privado y elimínalo después de importarlo.

En Android, abre el APK descargado y, cuando se te pida, permite instalar aplicaciones desde tu navegador o gestor de archivos. La aplicación de Android guarda sus datos en el teléfono; no tiene bandeja del sistema y las notificaciones aparecen como notificaciones normales de Android. Los servidores de modelos locales (Ollama, llama.cpp) se pueden usar a través de tu red mediante su dirección, por ejemplo `http://192.168.1.10:11434`.

## Funciones

- **Consultas en lenguaje natural**: describe lo que quieres con tus propias palabras
- **Deducción automática del esquema**: la aplicación propone las columnas de la tabla; tú las confirmas o editas antes de la ejecución
- **Búsqueda con varios proveedores**: Brave Search + Serper con respaldo automático
- **Ampliación multilingüe de las consultas**: busca en distintos idiomas y regiones
- **Extracción con LLM**: OpenRouter, Anthropic Claude (caché de prompts y lectura opcional de PDF completos), Ollama (local o Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp o cualquier dirección compatible con OpenAI
- **Resultados en streaming**: las filas aparecen en una tabla en directo a medida que se encuentran
- **Extracción de PDF**: detecta y extrae automáticamente el texto de los documentos PDF encontrados en los resultados de búsqueda
- **Modo de búsqueda de imágenes**: un pipeline específico que busca imágenes en lugar de texto: genera de 6 a 12 variaciones diversas de la consulta (con el LLM o con alternativas estáticas), ejecuta búsquedas de imágenes con Brave / Serper y clasifica cada resultado con un LLM según una rúbrica de relevancia estricta (0,0–1,0); las imágenes con una puntuación inferior a 0,7 se descartan y el resto se guarda ordenado por puntuación, con actualizaciones de la interfaz en tiempo real para cada imagen; el presupuesto, la cancelación y las condiciones de parada (cantidad / coste / tiempo) se respetan en todo momento
- **Modo de búsqueda de enlaces**: un pipeline específico que devuelve las páginas más relevantes en lugar de una tabla: genera variaciones de la consulta de búsqueda a partir de tu petición, ejecuta búsquedas web con Brave / Serper (sin duplicados por URL), descarga y analiza cada página y luego hace que un LLM lea el contenido completo y puntúe su relevancia (0,0–1,0) para tu consulta mientras genera una descripción breve; las páginas por debajo del umbral de confianza se descartan y el resto se guarda ordenado por puntuación, con actualizaciones de la interfaz en tiempo real para cada enlace; el presupuesto, la cancelación y las condiciones de parada se respetan en todo momento
- **Modo de investigación**: un modo agéntico que responde a preguntas abiertas en Markdown en lugar de con una tabla: el LLM dirige un bucle de llamadas a herramientas (buscar en la web, descargar una página como Markdown, pensar, responder) durante un número fijo de pasos como máximo y transmite cada paso a una cronología en directo; los pasos de razonamiento (`think`) y los fallos (pasos `error`, mostrados en rojo) se muestran para que el proceso sea transparente, y la respuesta final en Markdown con fuentes se presenta (saneada) y se puede exportar como `.md`; el presupuesto, la cancelación y las condiciones de parada se respetan en todo momento
- **Fórmulas e imágenes en las respuestas de investigación**: las fórmulas LaTeX se dibujan (MathJax, sin conexión); el agente puede buscar imágenes, examinarlas e incluir las adecuadas en su respuesta, que se guardan con la ejecución
- **Diagramas y gráficos en las respuestas de investigación**: diagramas Mermaid y gráficos de barras, líneas, áreas, circulares, de dispersión, radiales y mapas de calor, dibujados sin conexión con el tema de la aplicación; se guardan como SVG o PNG y se abren a pantalla completa
- **Estadísticas de uso**: dinero, tokens, llamadas a modelos, búsquedas de Brave y Serper y volumen de datos por mes, año y todo el tiempo, con desgloses por modelo y por modo, las ejecuciones más caras, el uso del disco, exportación a CSV y un presupuesto mensual con avisos
- **Fuentes por fila**: cada fila enlaza a las páginas de las que se extrajo
- **Eliminación de duplicados de entidades**: coincidencia aproximada + desambiguación asistida por LLM
- **Condiciones de parada configurables**: número objetivo de filas, coste máximo, duración máxima
- **Historial de ejecuciones**: explora, consulta y vuelve a exportar ejecuciones de investigación anteriores
- **Exportación**: CSV, JSON, XLSX con todos los metadatos de las fuentes
- **Tema oscuro / claro**: se cambia en la barra lateral
- **Bandeja del sistema**: notificaciones al terminar (escritorio)
- **Aplicación para Android**: la misma aplicación en teléfonos, con un diseño para pantallas pequeñas
- **Actualizaciones desde la aplicación**: las versiones nuevas se anuncian con sus cambios y se instalan desde la aplicación tras comprobar su firma (escritorio y Android)
- **Local primero**: todos los datos en SQLite, sin necesidad de backend en la nube

## Tecnologías

| Capa | Tecnología |
|------|------------|
| Contenedor de la aplicación | Tauri v2 (escritorio y Android) |
| Backend | Rust (runtime asíncrono Tokio) |
| Frontend | Svelte 5 (SvelteKit SPA) |
| Estilos de la interfaz | CSS simple con tokens de diseño |
| Base de datos | SQLite (sqlx, modo WAL) |
| LLM | OpenRouter / Ollama (local) / Ollama Cloud / API compatible con OpenAI (p. ej., llama.cpp) |
| Búsqueda | Brave Search API / Serper API |
| Iconos | Lucide |

## Requisitos previos

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (toolchain estable)
- [Requisitos previos de Tauri v2](https://v2.tauri.app/start/prerequisites/) para tu sistema operativo

## Primeros pasos

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

## Configuración

En el primer inicio, la aplicación crea una base de datos SQLite local con los ajustes predeterminados. Abre **Ajustes** desde la barra lateral para configurar:

| Grupo | Ajustes |
|-------|---------|
| **Proveedor de LLM** | Proveedor (OpenRouter / Ollama / Ollama Cloud / compatible con OpenAI), URL, modelo, clave de API, modo JSON, temperatura, tokens máximos |
| **Proveedor de búsqueda** | Proveedor (Brave / Serper), clave de API, respaldo activado o no, resultados por consulta |
| **Ejecución** | Descargas en paralelo (8 de forma predeterminada), extracciones en paralelo (3 de forma predeterminada), tiempo máximo de descarga, limitación de frecuencia, robots.txt |
| **Calidad** | Equilibrio entre precisión y exhaustividad, rigor de las evidencias, umbral de confianza, similitud de duplicados |

Las condiciones de parada (filas objetivo, coste máximo, duración máxima) se fijan para cada consulta en la página Consulta.

**Usar el otro proveedor como respaldo** controla el respaldo automático entre Brave y Serper tanto en las búsquedas web como en las de imágenes. Desactívalo para usar solo el proveedor principal seleccionado, aunque estén guardadas ambas claves de API. La elección se aplica a las ejecuciones nuevas después de Guardar; los reintentos se mantienen con el proveedor seleccionado. Cuando está activado, el otro proveedor solo se usa después de un fallo y necesita su clave guardada.

Ajusta **Ajustes → Búsqueda → Resultados por consulta** para elegir cuántos resultados pide cada consulta de búsqueda en los cuatro modos, incluida Investigación, y luego pulsa Guardar para las ejecuciones nuevas. Las solicitudes a Brave limitan este valor al rango documentado del endpoint: [de 1 a 20 en la búsqueda web](https://api-dashboard.search.brave.com/api-reference/web/search/get) y [hasta 200 en imágenes](https://api-dashboard.search.brave.com/documentation/services/image-search). Esto también afecta a valores demasiado grandes guardados anteriormente. Los valores mayores no solicitan páginas adicionales automáticamente.

### Uso, precios y límites de gasto

**Uso y coste** distingue entre los cargos informados por el proveedor, las estimaciones según las tarifas del modelo, los totales parciales y los costes desconocidos. Cada intento ante un proveedor se cuenta por separado, incluidos los reintentos, las búsquedas de respaldo, las respuestas fallidas con datos de uso y las solicitudes canceladas cuyo cargo final no se pudo obtener. El uso de entrada y salida procede de las respuestas del proveedor; los tokens de razonamiento no se cobran por segunda vez además de los tokens de salida.

- **OpenRouter:** se prefiere el [coste de la solicitud informado](https://openrouter.ai/docs/cookbook/administration/usage-accounting). Si no está disponible, se estima a partir de los tokens informados y de las tarifas públicas del catálogo para el modelo realmente devuelto.
- **Ollama Cloud:** se estima el consumo de créditos de uso a partir de las [tarifas publicadas de los modelos](https://ollama.com/pricing), incluida la entrada en caché y los periodos de precios punta reconocidos. Los créditos incluidos, los pagos de suscripción y los cargos en efectivo pueden diferir de esta estimación. Las tarifas no reconocidas o no disponibles quedan como desconocidas.
- **Tarifas de modelo personalizadas:** configura en Ajustes las tarifas de entrada y salida en USD por millón de tokens. Las tarifas pertenecen a un proveedor, endpoint y modelo concretos. Una tarifa explícita de cero significa uso gratuito; las tarifas que faltan nunca se convierten en cero sin avisar. Las tarifas publicadas se obtienen sin cambiar las tarifas manuales guardadas.
- **Búsqueda:** Brave y Serper tienen por defecto 0 USD por cada 1000 solicitudes, de modo que los cargos de búsqueda quedan fuera de las estimaciones. Indica la tarifa efectiva de tu plan para incluirlos, o deja el precio en blanco si no lo conoces. Los intentos fallidos sin datos de facturación quedan sin precio. Los cambios de ajustes se aplican a las ejecuciones nuevas; el historial guardado no se recalcula. Al actualizar, los antiguos valores predeterminados vacíos pasan a cero una sola vez, conservando las tarifas numéricas existentes y los campos en blanco que se dejen deliberadamente después.

El **Coste máx.** de la ejecución impide nuevas solicitudes a proveedores una vez que el gasto contabilizado alcanza su tope. No es un techo garantizado de la factura: las estimaciones pueden diferir de la facturación, los cargos desconocidos no se pueden incluir y las solicitudes ya en curso pueden seguir generando cargos. Usa los límites del propio proveedor cuando necesites un tope estricto. La contabilidad se conserva cuando la ejecución termina, falla, se cancela o no da resultados. Las entradas antiguas del historial se marcan explícitamente como estimaciones históricas.

La eliminación de duplicados conserva el ID de fila del primer miembro, fusiona los valores y los registros de fuentes del grupo en una transacción y sincroniza la tabla en directo con el historial y la exportación. Los resultados históricos sobrescritos anteriormente no se pueden reconstruir automáticamente; vuelve a ejecutar esas consultas para obtener resultados nuevos.

En Ajustes, **Guardar** y **Descartar** siguen visibles al desplazarte. Los enlaces de sección llevan directamente a los controles correspondientes, y al salir con cambios sin guardar se ofrece Guardar / Descartar / Quedarse. Las ediciones hechas durante un guardado quedan disponibles para el siguiente.

Los resultados usan una tabla continua y virtualizada con búsqueda local y ordenación con el teclado. Abre los **Detalles** de una fila para leer o copiar los valores completos y revisar sus fuentes guardadas. Investigación muestra la respuesta antes de la actividad desplegable. Imágenes admite vistas de cuadrícula y de lista y vistas previas con el teclado; la exportación y la eliminación muestran claramente su progreso y su resultado. Borrar el panel de diagnóstico **Registros** no restablece la actividad de la ejecución.

Las pruebas de regresión en el navegador usan datos sintéticos y comandos de Tauri simulados: instala los navegadores de prueba con `npx playwright install chromium webkit` y luego ejecuta `npm run test:ui`. Las pruebas de frontend existentes se ejecutan con `npm test`.

### Claves de API

Necesitas al menos una clave de API de búsqueda y un proveedor de LLM configurado. Ollama local y los servidores compatibles con OpenAI sin autenticación no requieren clave de API de LLM:

| Servicio | Consigue una clave en |
|----------|-----------------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (local) | https://ollama.com/ — no necesita clave |
| Ollama Cloud | https://ollama.com/settings/keys |
| Compatible con OpenAI | Clave opcional, según tu servidor |

Selecciona el proveedor en **Ajustes → LLM → Proveedor**, rellena sus campos y pulsa **Guardar**. Cada proveedor conserva su propio modelo y sus credenciales cuando cambias de uno a otro.

Las migraciones de ajustes se ejecutan automáticamente al iniciar: las opciones nuevas reciben valores predeterminados y los valores existentes se conservan. Los ajustes renombrados se migran explícitamente en el backend antes de insertar los valores predeterminados; los cambios en los formatos almacenados también requieren código de migración al implementar una versión nueva.

Al final de **Ajustes**, **Archivos de la aplicación** muestra la carpeta de datos real, la base de datos `data.db` y la carpeta de registros. Usa **Copiar ruta** para copiar una ubicación al portapapeles del sistema o **Abrir carpeta** para abrirla en tu gestor de archivos. Abrir la ubicación de la base de datos abre la carpeta que la contiene. Los ajustes, el historial, los resultados y las fuentes están en `data.db`; los registros de diagnóstico se guardan aparte.

- **OpenRouter:** introduce tu clave de API y selecciona un modelo del [catálogo de OpenRouter](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties), que permite buscar. Escribe para filtrar los ID de modelo, elige con el ratón o con las flechas y Enter, y usa **Actualizar** para volver a cargar la lista. El catálogo se puede explorar antes de introducir una clave; la clave es necesaria para la inferencia.
- **Ollama (local):** la URL `http://localhost:11434` y el nombre de un modelo instalado.
- **Ollama Cloud:** la URL `https://ollama.com`, una clave de API de Ollama y un modelo seleccionado del catálogo del servidor, que permite buscar. Haz clic en el campo del modelo y escribe para filtrar; luego elige con el ratón o con las flechas y Enter. **Actualizar** vuelve a cargar el catálogo. Filtrar no cambia el modelo seleccionado hasta que eliges un resultado. El acceso directo a la nube usa la [API nativa de Ollama](https://docs.ollama.com/cloud). Por ahora, Cloud no admite [salida estructurada](https://docs.ollama.com/capabilities/structured-outputs), así que el JSON se pide mediante las instrucciones y lo analiza el pipeline existente.
- **Compatible con OpenAI:** introduce la URL base de la API incluido `/v1` y el ID del modelo del servidor. Para [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server), la URL predeterminada es `http://localhost:8080/v1`; usa el nombre del modelo cargado o el alias configurado. Deja vacía la clave de API salvo que tu servidor la requiera. Desactiva **Modo JSON** si el servidor rechaza `response_format`.

**Razonamiento** controla cuánto razona el modelo antes de responder. **Automático** mantiene los valores predeterminados seguros: las solicitudes estructuradas de Ollama desactivan el razonamiento (GPT-OSS usa Bajo), mientras que los demás proveedores conservan sus valores predeterminados. **Predeterminado del proveedor** deja el razonamiento en manos del servidor. **Desactivado**, **Activado**, **Bajo**, **Medio**, **Alto** y **Máximo** piden un modo o nivel explícito; la compatibilidad depende del modelo y del proveedor. GPT-OSS en Ollama acepta Bajo/Medio/Alto y no puede usar Desactivado ni Máximo. Los cambios se aplican a las ejecuciones nuevas después de Guardar.

**Tokens de salida máx.** es una cuota por solicitud que pueden compartir los tokens de razonamiento y los de la respuesta. Es independiente de los límites de coste y tiempo de la ejecución y de la cuota de la cuenta del proveedor. Un mayor nivel de razonamiento puede dejar menos tokens para la respuesta.

Los problemas en las solicitudes al modelo muestran el proveedor, el modelo, la etapa, el límite de salida solicitado, los recuentos de tokens de entrada, salida y razonamiento informados y si se programó un reintento. Los límites de salida, los límites de contexto, las cuotas del proveedor, los límites de frecuencia, la autenticación y los fallos de red tienen explicaciones y acciones correctivas propias. Los problemas siguen visibles cuando se omiten páginas individuales y están disponibles en el Historial de ejecuciones. Los demás errores muestran una breve explicación, una acción que tomar y detalles técnicos plegables.

**Pausar** suspende el procesamiento en la etapa actual, incluida la revisión del esquema, y **Reanudar** continúa sin volver a enviar la solicitud. Una solicitud ya enviada a un modelo remoto puede terminar allí mientras está en pausa. **Cancelar** detiene el trabajo local y cierra las solicitudes pendientes. Los fallos del pipeline se muestran y se guardan como `failed`, con el error de origen disponible en el registro de la ejecución.

## Arquitectura

El backend usa una **máquina de estados del pipeline** con roles fijos orquestados en secuencia:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Roles del pipeline

| Rol | LLM | Función |
|-----|-----|---------|
| QueryInterpreter | Sí | Convierte la consulta en una intención estructurada |
| SchemaPlanner | Sí | Propone las columnas de la tabla y sus tipos |
| SearchPlanner | Sí | Genera consultas de búsqueda para varios idiomas y regiones |
| QueryExpander | Sí | Traduce las consultas a los idiomas de destino |
| SearchExecutor | No | Llama a las API de búsqueda y reúne las URL candidatas |
| Fetcher | No | Descarga HTTP con limitación de frecuencia y robots.txt |
| DocumentParser | No | HTML → texto limpio (eliminación de contenido repetitivo) |
| PdfParser | No | PDF → texto limpio (mediante pdf-extract) |
| Extractor | Sí | Texto + esquema → filas estructuradas |
| Validator | Parcial | Conformidad con el esquema + comprobaciones semánticas |
| Deduplicator | Parcial | Coincidencia aproximada (strsim) + LLM para los casos dudosos |
| LinkRanker | Sí | Lee el contenido de la página y puntúa su relevancia (0,0–1,0) + genera una descripción (modo de búsqueda de enlaces) |
| ResearchAgent | Sí | Dirige el bucle agéntico de búsqueda/descarga/razonamiento/respuesta y produce una respuesta en Markdown (modo de investigación) |
| StoppingController | No | Evalúa las condiciones de parada (filas, presupuesto, tiempo, saturación) |

### Máquina de estados

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Estructura del proyecto

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

## Base de datos

SQLite en modo WAL con las siguientes tablas:

| Tabla | Función |
|-------|---------|
| `settings` | Configuración del usuario (claves de API, modelos, preferencias) |
| `runs` | Registros de las ejecuciones de investigación con estado y estadísticas |
| `run_schemas` | Esquema de tabla confirmado de cada ejecución |
| `search_queries` | Consultas de búsqueda generadas en cada ejecución |
| `search_results` | URL reunidas de las API de búsqueda |
| `fetched_pages` | Contenido de las páginas descargadas |
| `entity_rows` | Filas estructuradas extraídas (datos JSON) |
| `row_sources` | Evidencias de cada fila (URL, título, fragmento) |
| `image_results` | Imágenes clasificadas de cada ejecución (modo de búsqueda de imágenes) |
| `link_results` | Enlaces relevantes clasificados con descripciones y puntuaciones (modo de búsqueda de enlaces) |
| `research_steps` | Pasos de llamadas a herramientas del agente en cada ejecución: search/fetch/think/error (modo de investigación) |
| `research_results` | Respuesta final en Markdown de cada ejecución (modo de investigación) |
| `run_logs` | Registros de ejecución de cada ejecución |

## Desarrollo

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

Necesita los [requisitos previos de Tauri para Android](https://v2.tauri.app/start/prerequisites/#android): Android SDK y NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) y los targets de Rust (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Conecta un teléfono con la depuración USB activada o inicia un emulador.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

Las compilaciones se firman con la clave de depuración salvo que exista `src-tauri/gen/android/keystore.properties`. `make android-keystore` crea una clave de publicación y ese archivo, y muestra los secretos de GitHub (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`) que usa el flujo de publicación para firmar el APK que añade a cada versión. Conserva la clave: las actualizaciones solo se instalan sobre una aplicación existente si están firmadas con la misma clave.

## Formatos de exportación

| Formato | Contenido |
|---------|-----------|
| **CSV** | Todas las columnas + las fuentes como columna JSON |
| **JSON** | Filas completas con un array anidado de fuentes y los metadatos de la ejecución |
| **XLSX** | Libro con formato, con una hoja de datos, una hoja de fuentes y una hoja de metadatos |

## Licencia

MIT
