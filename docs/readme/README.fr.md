<p align="center">
  <img src="../../images/query2table-logo.png" alt="Logo de Query2Table" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <b>Français</b> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="Version GitHub"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="Téléchargements GitHub"></a>
  <a href="#licence"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Licence : MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Construit avec Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend en Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend en Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Vues de Query2Table">
</p>

**Query2Table** est un outil de recherche local d’abord, pour ordinateur (Linux, Windows, macOS) et Android. Vous décrivez les données dont vous avez besoin ; l’application cherche de manière autonome sur Internet, dans plusieurs sources et plusieurs langues, télécharge les pages, en extrait des entités structurées à l’aide de LLM, les dédoublonne et présente le résultat sous forme de tableau mis à jour en direct, avec les liens vers les sources de chaque ligne — le tout sur votre machine, sans backend dans le cloud.

Au-delà des tableaux, elle propose trois autres modes dédiés : la **recherche d’images**, qui trouve des images et les classe avec un LLM selon leur pertinence visuelle ; la **recherche de liens**, qui lit le contenu complet des pages candidates et ne renvoie que les liens les plus pertinents, avec une description et un score de pertinence générés par le LLM ; et la **recherche approfondie** (mode Recherche), un mode agentique dans lequel le modèle cherche, lit des pages, raisonne et rédige seul une réponse en Markdown avec ses sources.

L’application existe aussi pour les téléphones **Android** : la même application, avec les mêmes modes, paramètres et historique, construite à partir du même code et adaptée à l’écran d’un téléphone avec une barre d’onglets en bas.

Demandez par exemple *« Construis un tableau comparatif des bases de données et bibliothèques open source de recherche vectorielle avec leur langage, leur licence, leur type de déploiement, leur cas d’usage et leur site web »* et regardez le tableau se remplir en temps réel.

![Tableau comparatif de bases de données vectorielles open source avec fiabilité et sources](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Visite guidée

### Demandez en langage courant

Choisissez un format de résultat, décrivez ce dont vous avez besoin et fixez les conditions d’arrêt. Pour les tableaux, l’application propose des colonnes que vous pouvez modifier avant le début de la recherche.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Nouvelle requête avec les quatre formats de résultat"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Schéma de tableau proposé, prêt à être vérifié"></td>
  </tr>
</table>

### Chaque ligne garde ses preuves

Ouvrez une ligne pour lire les valeurs complètes, les copier et vérifier les pages dont elles ont été extraites.

![Détails d’une ligne avec la confiance de l’extraction et les sources](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Recherche avec questions de suivi

L’agent cherche, lit des pages et rédige une réponse sourcée avec un sommaire. Chaque étape est visible dans Activité, et les questions de suivi poursuivent la conversation avec son historique.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Réponse de recherche avec sommaire et questions de suivi suggérées"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Activité de la recherche avec les étapes numérotées de recherche et de lecture"></td>
  </tr>
</table>

### Images et liens

La recherche d’images classe les résultats par pertinence, avec des filtres par site et par taille ; la recherche de liens lit chaque page et explique pourquoi elle correspond.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Résultats de la recherche d’images dans une grille justifiée"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Liens pertinents avec descriptions et niveaux de correspondance"></td>
  </tr>
</table>

### Historique et paramètres

Recherchez, filtrez et rouvrez les exécutions passées, relancez-les ou exportez-en plusieurs à la fois. Les paramètres vérifient vos connexions au LLM et à la recherche avant que vous ne commenciez.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Historique des exécutions groupé par jour avec résultats, durée, coût et modèle"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Paramètres de recherche avec un test de connexion réussi"></td>
  </tr>
</table>

### Thèmes clair et sombre

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Réponse de recherche dans le thème sombre"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Tableau des résultats dans le thème sombre"></td>
  </tr>
</table>

## Téléchargement

Récupérez la dernière version dans les [Releases](https://github.com/hightemp/query2table/releases/latest) :

| Plateforme | Fichier |
|----------|------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Pour transférer votre configuration d’un ordinateur vers le téléphone, utilisez **Paramètres → Exporter les paramètres…**, cochez **Inclure les clés API et les proxys** et importez le fichier sur le téléphone. Un tel fichier contient vos clés : gardez-le privé et supprimez-le après l’importation.

Sur Android, ouvrez l’APK téléchargé et, lorsque c’est demandé, autorisez l’installation d’applications depuis votre navigateur ou votre gestionnaire de fichiers. L’application Android garde ses données sur le téléphone ; elle n’a pas d’icône dans la zone de notification, et les notifications apparaissent comme des notifications Android ordinaires. Les serveurs de modèles locaux (Ollama, llama.cpp) sont utilisables sur votre réseau via leur adresse, p. ex. `http://192.168.1.10:11434`.

## Fonctionnalités

- **Requêtes en langage naturel** — décrivez ce que vous voulez avec vos propres mots
- **Déduction automatique du schéma** — l’application propose les colonnes du tableau ; vous les confirmez ou les modifiez avant l’exécution
- **Recherche multi-fournisseur** — Brave Search + Serper avec bascule automatique en secours
- **Élargissement multilingue des requêtes** — recherches dans plusieurs langues et zones géographiques
- **Extraction par LLM** — OpenRouter, Anthropic Claude (mise en cache du prompt, lecture facultative des PDF entiers), Ollama (local ou Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp ou toute adresse compatible OpenAI
- **Résultats en continu** — les lignes apparaissent dans un tableau en direct au fur et à mesure qu’elles sont trouvées
- **Extraction depuis les PDF** — détecte automatiquement les documents PDF trouvés dans les résultats de recherche et en extrait le texte
- **Mode Images** — un pipeline dédié qui recherche des images au lieu de texte : il génère 6 à 12 variantes de requête diversifiées (par LLM ou par des variantes statiques de secours), lance les recherches d’images via Brave / Serper, puis classe chaque résultat avec un LLM selon une grille de pertinence stricte (0,0–1,0) ; les images notées sous 0,7 sont écartées, les autres sont enregistrées triées par score, avec une mise à jour de l’interface en temps réel pour chaque image ; le budget, l’annulation et les conditions d’arrêt (nombre / coût / durée) sont respectés tout du long
- **Mode Liens** — un pipeline dédié qui renvoie les pages les plus pertinentes au lieu d’un tableau : il génère des variantes de requête à partir de votre demande, lance des recherches Web via Brave / Serper (dédoublonnées par URL), télécharge et analyse chaque page, puis fait lire son contenu complet à un LLM qui en note la pertinence (0,0–1,0) pour votre requête tout en rédigeant une courte description ; les pages sous le seuil de confiance sont écartées, les autres sont enregistrées triées par score, avec une mise à jour de l’interface en temps réel pour chaque lien ; le budget, l’annulation et les conditions d’arrêt sont respectés tout du long
- **Mode Recherche** — un mode agentique qui répond aux questions ouvertes en Markdown au lieu d’un tableau : le LLM pilote une boucle d’appels d’outils (chercher sur le Web, récupérer une page en Markdown, réfléchir, répondre) jusqu’à un nombre fixe d’étapes, chaque étape s’affichant en direct dans une chronologie ; les étapes de raisonnement (`think`) et les échecs (étapes `error`, en rouge) sont affichés pour que le processus reste transparent, et la réponse finale en Markdown avec ses sources est rendue (assainie) et exportable en `.md` ; le budget, l’annulation et les conditions d’arrêt sont respectés tout du long
- **Formules et images dans les réponses de recherche** — les formules LaTeX sont dessinées (MathJax, hors ligne) ; l’agent peut chercher des images, les examiner et placer celles qui conviennent dans sa réponse, conservées avec l’exécution
- **Diagrammes et graphiques dans les réponses de recherche** — diagrammes Mermaid et graphiques en barres, en courbes, en aires, en secteurs, nuages de points, radars et cartes de chaleur, dessinés hors ligne dans le thème de l’application ; enregistrables en SVG ou PNG, affichables en plein écran
- **Vidéos dans les réponses de recherche** — l’agent trouve des vidéos sur n’importe quel site (recherche vidéo sur le web, ou directement YouTube, Bilibili, Niconico et SoundCloud), lit leurs sous-titres horodatés et cite des moments précis ; les vidéos s’affichent en cartes avec leurs miniatures conservées. Utilise yt-dlp, téléchargé à la première utilisation (Python intégré sur Android)
- **Statistiques d’utilisation** — dépenses, jetons, appels aux modèles, recherches Brave et Serper et volume de données par mois, par an et depuis le début, avec une répartition par modèle et par mode, les exécutions les plus coûteuses, l’espace disque utilisé, l’export CSV et un budget mensuel avec avertissements
- **13 langues d’interface** — anglais, russe, allemand, français, espagnol, italien, portugais (Brésil), polonais, ukrainien, turc, chinois (simplifié), japonais et coréen ; la langue du système est choisie par défaut
- **Sources par ligne** — chaque ligne renvoie aux pages dont elle a été extraite
- **Dédoublonnage des entités** — correspondance approximative + désambiguïsation assistée par LLM
- **Conditions d’arrêt configurables** — nombre de lignes visé, coût maximal, durée maximale
- **Historique des exécutions** — parcourez, consultez et réexportez les recherches passées
- **Export** — CSV, JSON, XLSX avec toutes les métadonnées des sources
- **Thème sombre / clair** — à basculer dans la barre latérale
- **Zone de notification** — notifications de fin d’exécution (ordinateur)
- **Application Android** — la même application sur téléphone, avec une mise en page pour petits écrans
- **Mises à jour intégrées** — les nouvelles versions sont annoncées avec leurs changements et installées depuis l’application après vérification de leur signature (ordinateur et Android)
- **Local d’abord** — toutes les données dans SQLite, aucun backend dans le cloud n’est nécessaire

## Pile technique

| Couche | Technologie |
|-------|------------|
| Coque de l’application | Tauri v2 (ordinateur et Android) |
| Backend | Rust (runtime asynchrone Tokio) |
| Frontend | Svelte 5 (SPA SvelteKit) |
| Styles de l’interface | CSS simple avec des design tokens |
| Base de données | SQLite (sqlx, mode WAL) |
| LLM | OpenRouter / Ollama (local) / Ollama Cloud / API compatible OpenAI (p. ex. llama.cpp) |
| Recherche | Brave Search API / Serper API |
| Icônes | Lucide |

## Prérequis

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (chaîne d’outils stable)
- [Prérequis de Tauri v2](https://v2.tauri.app/start/prerequisites/) pour votre système d’exploitation

## Prise en main

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

## Configuration

Au premier lancement, l’application crée une base de données SQLite locale avec les paramètres par défaut. Ouvrez **Paramètres** depuis la barre latérale pour configurer :

| Groupe | Paramètres |
|-------|----------|
| **Fournisseur LLM** | Fournisseur (OpenRouter / Ollama / Ollama Cloud / compatible OpenAI), URL, modèle, clé API, mode JSON, température, jetons max |
| **Fournisseur de recherche** | Fournisseur (Brave / Serper), clé API, bascule de secours, résultats par requête |
| **Exécution** | Chargements en parallèle (8 par défaut), extractions en parallèle (3 par défaut), délai de chargement, limitation du débit, robots.txt |
| **Qualité** | Équilibre précision/rappel, exigence sur les preuves, seuil de confiance, similarité des doublons |

Les conditions d’arrêt (lignes visées, coût max, durée max) se règlent pour chaque requête sur la page Requête.

**Utiliser l’autre fournisseur en secours** contrôle la bascule automatique Brave/Serper pour les recherches Web comme pour les recherches d’images. Désactivez-le pour n’utiliser que le fournisseur principal sélectionné, même si les deux clés API sont enregistrées. Le choix s’applique aux nouvelles exécutions après l’enregistrement ; les nouvelles tentatives restent chez le fournisseur sélectionné. Lorsqu’il est activé, l’autre fournisseur n’est utilisé qu’après un échec et nécessite sa clé enregistrée.

Réglez **Paramètres → Recherche Web → Résultats par requête** pour choisir combien de résultats chaque requête de recherche demande dans les quatre modes, Recherche comprise, puis enregistrez pour les nouvelles exécutions. Les requêtes Brave ramènent cette valeur dans la plage documentée du point de terminaison : [1 à 20 pour la recherche Web](https://api-dashboard.search.brave.com/api-reference/web/search/get) et [jusqu’à 200 pour les images](https://api-dashboard.search.brave.com/documentation/services/image-search). Cela couvre aussi les valeurs trop grandes enregistrées auparavant. Des valeurs plus grandes ne déclenchent pas automatiquement de pages supplémentaires.

### Utilisation, prix et plafonds de dépenses

**Utilisation et coût** distingue les frais déclarés par le fournisseur, les estimations d’après les tarifs du modèle, les totaux partiels et les coûts inconnus. Chaque tentative auprès d’un fournisseur est comptée séparément, y compris les nouvelles tentatives, les recherches de secours, les complétions échouées avec données d’utilisation et les requêtes annulées dont le coût final n’a pas pu être obtenu. L’utilisation en entrée et en sortie provient des réponses des fournisseurs ; les jetons de réflexion ne sont pas facturés une seconde fois en plus des jetons de sortie.

- **OpenRouter :** le [coût de requête déclaré](https://openrouter.ai/docs/cookbook/administration/usage-accounting) est prioritaire. S’il n’est pas disponible, l’estimation se fait d’après les jetons déclarés et les tarifs publics du catalogue pour le modèle réellement renvoyé.
- **Ollama Cloud :** la consommation de crédits d’utilisation est estimée d’après les [tarifs publiés des modèles](https://ollama.com/pricing), y compris l’entrée en cache et les périodes de tarification de pointe reconnues. Les crédits inclus, les abonnements et les paiements peuvent différer de cette estimation. Les tarifs non reconnus ou indisponibles restent inconnus.
- **Tarifs de modèle personnalisés :** configurez dans les Paramètres l’entrée et la sortie en USD par million de jetons. Les tarifs sont propres à un fournisseur, un point de terminaison et un modèle. Un tarif explicitement nul signifie une utilisation gratuite ; un tarif manquant ne devient jamais nul en silence. Les tarifs publiés sont récupérés sans modifier les tarifs saisis manuellement.
- **Recherche :** Brave et Serper valent par défaut 0 USD pour 1 000 requêtes, ce qui exclut les frais de recherche des estimations. Saisissez le tarif effectif de votre offre pour les inclure, ou laissez le prix vide s’il est inconnu. Les tentatives échouées sans données de facturation restent sans tarif. Les modifications des paramètres s’appliquent aux nouvelles exécutions ; l’historique enregistré n’est pas recalculé. Lors d’une mise à niveau, les anciennes valeurs par défaut vides deviennent nulles une seule fois, en conservant les tarifs numériques existants et les champs volontairement laissés vides ensuite.

Le **Coût max** d’une exécution empêche toute nouvelle requête aux fournisseurs dès que les dépenses comptabilisées atteignent ce plafond. Ce n’est pas un plafond de facturation garanti : les estimations peuvent différer de la facturation, les frais inconnus ne peuvent pas être inclus et les requêtes déjà en cours peuvent encore être facturées. Utilisez les limites côté fournisseur si un plafond strict est nécessaire. La comptabilité est conservée après la fin, l’échec, l’annulation ou une exécution sans résultat. Les anciennes entrées de l’historique sont explicitement marquées comme estimations historiques.

Le dédoublonnage conserve l’identifiant de ligne du premier membre, fusionne les valeurs et les sources du groupe dans une transaction et met en cohérence le tableau en direct avec l’historique et l’export. Les résultats historiques écrasés auparavant ne peuvent pas être reconstitués automatiquement ; relancez ces requêtes pour obtenir des résultats à jour.

Les Paramètres gardent **Enregistrer** et **Abandonner** visibles pendant le défilement. Les liens de section mènent directement aux réglages concernés, et quitter avec des modifications non enregistrées propose Enregistrer / Abandonner / Rester. Les modifications faites pendant un enregistrement restent disponibles pour l’enregistrement suivant.

Les résultats utilisent un tableau continu et virtualisé avec recherche locale et tri au clavier. Ouvrez les **Détails** d’une ligne pour lire ou copier les valeurs complètes et examiner ses sources enregistrées. La Recherche place la réponse avant l’activité dépliable. Les Images proposent des vues en grille et en liste et des aperçus au clavier ; l’export et la suppression affichent clairement leur progression et leur résultat. Vider le panneau de diagnostic **Journaux** ne réinitialise pas l’activité de l’exécution.

Les tests de non-régression dans le navigateur utilisent des données synthétiques et des commandes Tauri simulées : installez les navigateurs de test avec `npx playwright install chromium webkit`, puis lancez `npm run test:ui`. Les tests frontend existants se lancent avec `npm test`.

### Clés API

Il vous faut au moins une clé API de recherche et un fournisseur LLM configuré. Ollama en local et les serveurs compatibles OpenAI sans authentification ne nécessitent pas de clé API LLM :

| Service | Obtenir une clé sur |
|---------|-------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (local) | https://ollama.com/ — aucune clé nécessaire |
| Ollama Cloud | https://ollama.com/settings/keys |
| Compatible OpenAI | Clé facultative, selon votre serveur |

Sélectionnez le fournisseur dans **Paramètres → Fournisseur LLM**, remplissez ses champs et cliquez sur **Enregistrer**. Chaque fournisseur garde son propre modèle et ses identifiants lorsque vous changez de fournisseur.

Les migrations des paramètres s’exécutent automatiquement au démarrage : les nouvelles options reçoivent leurs valeurs par défaut et les valeurs existantes sont conservées. Les paramètres renommés sont migrés explicitement dans le backend avant l’insertion des valeurs par défaut ; un changement de format de stockage exige lui aussi du code de migration lors de l’implémentation d’une nouvelle version.

En bas des **Paramètres**, **Fichiers de l’application** affiche le dossier de données réel, la base de données `data.db` et le dossier des journaux. Utilisez **Copier le chemin** pour copier un emplacement dans le presse-papiers du système ou **Ouvrir le dossier** pour l’ouvrir dans votre gestionnaire de fichiers. Ouvrir l’emplacement de la base de données ouvre le dossier qui la contient. Les paramètres, l’historique, les résultats et les sources se trouvent dans `data.db` ; les journaux de diagnostic sont stockés à part.

- **OpenRouter :** saisissez votre clé API et sélectionnez un modèle dans le [catalogue OpenRouter](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties), avec recherche. Tapez pour filtrer les identifiants de modèles, choisissez à la souris ou avec les flèches et Entrée, et utilisez **Actualiser** pour recharger la liste. Le catalogue peut être parcouru avant de saisir une clé ; une clé est nécessaire pour l’inférence.
- **Ollama (local) :** l’URL `http://localhost:11434` et le nom d’un modèle installé.
- **Ollama Cloud :** l’URL `https://ollama.com`, une clé API Ollama et un modèle choisi dans le catalogue du serveur, avec recherche. Cliquez sur le champ du modèle et tapez pour filtrer, puis choisissez à la souris ou avec les flèches et Entrée ; **Actualiser** recharge le catalogue. Le filtrage ne change pas le modèle sélectionné tant que vous ne choisissez pas un résultat. L’accès direct au cloud utilise l’[API Ollama native](https://docs.ollama.com/cloud). Le cloud ne prend pas encore en charge les [sorties structurées](https://docs.ollama.com/capabilities/structured-outputs) : le JSON est donc demandé par le prompt et analysé par le pipeline existant.
- **Compatible OpenAI :** saisissez l’URL de base de l’API, avec `/v1`, et l’identifiant du modèle sur le serveur. Pour [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server), l’URL par défaut est `http://localhost:8080/v1` ; utilisez le nom du modèle chargé ou l’alias configuré. Laissez la clé API vide sauf si votre serveur en exige une. Désactivez **Mode JSON** si le serveur refuse `response_format`.

**Réflexion** (niveau de raisonnement) contrôle le raisonnement du modèle avant sa réponse. **Auto** conserve les valeurs sûres par défaut : les requêtes structurées à Ollama désactivent la réflexion (GPT-OSS utilise Faible), tandis que les autres fournisseurs gardent leurs valeurs par défaut. **Par défaut du fournisseur** laisse le raisonnement au serveur. **Désactivé**, **Activé**, **Faible**, **Moyen**, **Élevé** et **Max** demandent un mode ou un niveau explicite ; la prise en charge dépend du modèle et du fournisseur. GPT-OSS sur Ollama accepte Faible/Moyen/Élevé et ne peut pas utiliser Désactivé ni Max. Les modifications s’appliquent aux nouvelles exécutions après l’enregistrement.

**Jetons de sortie max** est un quota par requête, éventuellement partagé entre les jetons de réflexion et ceux de la réponse. Il est distinct des limites de coût et de durée de l’exécution et du quota du compte chez le fournisseur. Un niveau de raisonnement plus élevé peut laisser moins de jetons pour la réponse.

Les problèmes de requêtes aux modèles indiquent le fournisseur, le modèle, l’étape, le plafond de sortie demandé, les nombres de jetons d’entrée, de sortie et de réflexion déclarés, et si une nouvelle tentative a été programmée. Les limites de sortie, limites de contexte, quotas du fournisseur, limitations de débit, erreurs d’authentification et pannes réseau ont chacun leur explication et leurs actions correctives. Les problèmes restent visibles lorsque des pages isolées sont ignorées et sont disponibles dans l’Historique des exécutions. Les autres erreurs affichent une courte explication, une action à entreprendre et des détails techniques dépliables.

**Pause** suspend le traitement à l’étape en cours, y compris la révision du schéma, et **Reprendre** continue sans renvoyer la requête. Une requête déjà envoyée à un modèle distant peut encore s’y terminer pendant la pause. **Annuler** arrête le travail local et ferme les requêtes en attente. Les échecs du pipeline sont affichés et enregistrés avec l’état `failed`, l’erreur d’origine étant disponible dans le journal de l’exécution.

## Architecture

Le backend utilise une **machine à états de pipeline** avec des rôles fixes orchestrés en séquence :

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Rôles du pipeline

| Rôle | LLM | Fonction |
|------|-----|---------|
| QueryInterpreter | Oui | Analyser la requête en une intention structurée |
| SchemaPlanner | Oui | Proposer les colonnes du tableau et leurs types |
| SearchPlanner | Oui | Générer des requêtes de recherche pour plusieurs langues et zones géographiques |
| QueryExpander | Oui | Traduire les requêtes dans les langues cibles |
| SearchExecutor | Non | Appeler les API de recherche, collecter les URL candidates |
| Fetcher | Non | Téléchargement HTTP avec limitation du débit et robots.txt |
| DocumentParser | Non | HTML → texte propre (suppression du superflu) |
| PdfParser | Non | PDF → texte propre (via pdf-extract) |
| Extractor | Oui | Texte + schéma → lignes structurées |
| Validator | Partiel | Conformité au schéma + vérifications sémantiques |
| Deduplicator | Partiel | Correspondance approximative (strsim) + LLM pour les cas limites |
| LinkRanker | Oui | Lire le contenu de la page et noter sa pertinence (0,0–1,0) + générer une description (mode Liens) |
| ResearchAgent | Oui | Piloter la boucle agentique chercher/récupérer/réfléchir/répondre et produire une réponse en Markdown (mode Recherche) |
| StoppingController | Non | Évaluer les conditions d’arrêt (lignes, budget, durée, saturation) |

### Machine à états

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Structure du projet

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

## Base de données

SQLite en mode WAL avec les tables suivantes :

| Table | Fonction |
|-------|---------|
| `settings` | Configuration de l’utilisateur (clés API, modèles, préférences) |
| `runs` | Enregistrements des exécutions de recherche avec leur état et leurs statistiques |
| `run_schemas` | Schéma de tableau confirmé pour chaque exécution |
| `search_queries` | Requêtes de recherche générées pour chaque exécution |
| `search_results` | URL collectées depuis les API de recherche |
| `fetched_pages` | Contenu des pages téléchargées |
| `entity_rows` | Lignes structurées extraites (données JSON) |
| `row_sources` | Preuves par ligne (URL, titre, extrait) |
| `image_results` | Images classées pour chaque exécution (mode Images) |
| `link_results` | Liens pertinents classés avec descriptions et scores (mode Liens) |
| `research_steps` | Étapes d’appel d’outils de l’agent pour chaque exécution — search/fetch/think/error (mode Recherche) |
| `research_results` | Réponse finale en Markdown pour chaque exécution (mode Recherche) |
| `run_logs` | Journaux d’exécution pour chaque exécution |

## Développement

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

Nécessite les [prérequis Android de Tauri](https://v2.tauri.app/start/prerequisites/#android) : le SDK et le NDK Android (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) et les cibles Rust (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Branchez un téléphone avec le débogage USB activé, ou démarrez un émulateur.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

Les versions sont signées avec la clé de débogage, sauf si `src-tauri/gen/android/keystore.properties` existe. `make android-keystore` crée une clé de publication et ce fichier, et affiche les secrets GitHub (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`) que le workflow de publication utilise pour signer l’APK qu’il ajoute à chaque version. Conservez la clé : une mise à jour ne s’installe par-dessus une application existante que si elle est signée avec la même clé.

## Formats d’export

| Format | Contenu |
|--------|----------|
| **CSV** | Toutes les colonnes + les sources dans une colonne JSON |
| **JSON** | Lignes complètes avec un tableau de sources imbriqué et les métadonnées de l’exécution |
| **XLSX** | Classeur mis en forme avec une feuille de données, une feuille de sources et une feuille de métadonnées |

## Licence

MIT
