You are a principal AI systems architect and staff-level technical planner.

Your job is to decompose a complex product idea into a complete implementation plan with clear subtasks, dependencies, milestones, architecture decisions, risks, and deliverables.

You are not writing marketing text.
You are not writing a vague brainstorm.
You are producing an execution-grade technical plan that a senior engineer can use to implement the system.

<task> Design and plan a fully local desktop application that can take an arbitrary natural-language research query, search the public internet asynchronously, extract structured entities, and build a table of results with row-level sources.

The product is conceptually similar to a universal “FindAll”-style research tool, but it must be more general-purpose:

the query can ask for companies, people, events, products, articles, jobs, laws, websites, or other entity-like rows
the system should infer a candidate table schema from the query
the user can confirm or modify the schema before execution
the system then performs adaptive multi-step search and extraction until it reaches a configured minimum number of rows or other stopping conditions
results stream into a table as they are found
each final row must have sources attached</task>
<product_constraints> - It is a desktop app, not a SaaS dashboard - It must be local-first - No cloud backend for orchestration - History must be stored locally - Runs on Windows, macOS, Linux - Tauri is the preferred desktop shell - Rust is the preferred core runtime/orchestration language - Search providers should support at least Brave Search API and Serper, with provider switching/fallback strategy - LLM access should support model-per-stage configuration through OpenRouter - Search and processing must be asynchronous - The architecture should prefer orchestrator + fixed roles over free-form autonomous agents - The system should support: - HTML pages - PDFs - JS-rendered sites when necessary - multilingual query expansion - country/language targeting - row-level evidence - deduplication across domains, languages, and naming variations - The system should support configurable: - precision vs recall preference - evidence strictness - stop conditions - budget limits - time limits - result count target - The result is entity rows, not free-form summaries </product_constraints> <desired_capabilities> The system should be able to handle examples like: - find all websites of Israeli companies - find AI security conferences in Europe - find remote Rust jobs at fintech companies - find laws related to crypto taxation in EU countries - find notable researchers working on retrieval-augmented generation - find articles about a specific topic and structure them into rows </desired_capabilities> <technical_preferences> You should assume the likely stack is: - Tauri for desktop shell - Rust for orchestration/core - local SQLite for persistence - async pipeline - search APIs for discovery - page fetching/parsing - LLM extraction/validation/dedup assistance where useful

But you must critically evaluate where LLMs should be used versus where deterministic/rule-based logic is better.
</technical_preferences>

<what_i_need_from_you> I want you to plan this system as if you are preparing a technical blueprint for implementation.

Break the project into major domains and then into actionable subtasks.

You must think in terms of:

system layers
agents/roles
queues
state transitions
storage model
failure handling
observability
UX implications
implementation order
MVP vs later phases
test strategy
cost/performance tradeoffs
debuggability
Do not stay high-level.
Do not avoid hard decisions.
Make reasonable assumptions where needed, but label them clearly.
</what_i_need_from_you>

<important_design_requirements> The architecture must explicitly cover: 1. Query understanding 2. Dynamic schema proposal 3. User schema confirmation/edit step 4. Search planning 5. Query expansion 6. Multi-provider search execution 7. Candidate URL/entity collection 8. Fetching and parsing 9. PDF handling 10. JS-rendered fallback handling 11. Extraction into structured rows 12. Validation 13. Deduplication/entity resolution 14. Row-level source attachment 15. Adaptive search refinement 16. Stop-condition engine 17. Local persistence 18. Resume interrupted runs 19. Export to CSV/XLSX/JSON 20. Run history 21. Templates/presets 22. Settings system 23. Telemetry/logging for debugging 24. Error handling and retries 25. Configurable budgets/rate limits/provider limits </important_design_requirements> <llm_usage_requirements> The plan must explicitly separate which stages are: - LLM-first - deterministic-first - hybrid

For every LLM-involved stage, explain:

why an LLM is needed
what exact input/output contract it should follow
whether structured JSON output is required
what fallback should happen if the LLM output is low quality or invalid
whether that stage should allow per-stage model selection</llm_usage_requirements>
<agent_model_requirement> Do not design “free agents that improvise everything”. Design a controlled orchestrator with fixed roles.

Possible roles may include, if appropriate:

Query Interpreter
Schema Planner
Search Planner
Query Expander
Search Executor
Fetcher
Document Parser
Extractor
Validator
Deduper / Entity Resolver
Stopping Controller
Persistence Manager
UI Event Publisher
You do not need to keep these exact names, but the roles must be explicit and deterministic.
</agent_model_requirement>

<output_format> Return your answer in exactly the following top-level structure:

Executive Summary
Assumptions
Product Goal Restatement
Core Architecture
System Components
Agent / Role Model
End-to-End Pipeline
Subtask Breakdown
Dependency Graph
Data Model
Storage Design
Async Execution Model
LLM Usage by Stage
Search Strategy Design
Extraction and Validation Design
Deduplication Strategy
Stopping Logic
Settings and User Controls
Desktop UX Flow
Error Handling and Recovery
Observability and Debugging
Security / Compliance / Robots Considerations
Performance and Cost Risks
MVP Scope
Phase 2 / Phase 3 Extensions
Recommended Folder / Module Structure
Testing Strategy
Open Questions
Final Recommended Build Order
For sections 4 through 17, be highly concrete.

For section 8 "Subtask Breakdown", provide a table with these columns:

ID
Subtask
Purpose
Inputs
Outputs
Owner Role
Complexity
Priority
Dependencies
For section 9 "Dependency Graph", provide:

a dependency list
then a suggested implementation order
For section 10 "Data Model", define the main entities/tables/records the desktop app should store locally.

For section 13 "LLM Usage by Stage", provide a table with:

Stage
Why LLM is needed
Input contract
Output contract
Structured output format
Failure fallback
Should user choose model for this stage? (yes/no)
For section 24 "MVP Scope", be strict and realistic.

For section 29 "Final Recommended Build Order", produce a staged engineering roadmap from first prototype to robust MVP.
</output_format>

<quality_bar> Your answer must be: - implementation-oriented - technically opinionated - realistic - internally consistent - decomposed into actionable engineering work

Avoid generic phrases like “use AI to improve results” unless you specify exactly how.
Avoid hand-wavy architecture.
Avoid unnecessary theory.
Prefer explicit tradeoffs and justified decisions.
</quality_bar>

<reasoning_instruction> Think carefully and deeply before writing. Internally decompose the system into layers, roles, and execution stages. Identify bottlenecks, failure modes, and ambiguity points. Then produce only the final structured answer. Do not output hidden reasoning. </reasoning_instruction>

Задавай вопросы по всем не ясным моментам, чем больше тем лучше
Спланируй все и запиши план задач в TASKS.md
Проведи исследование по всем не ясным темам с помощью субагента