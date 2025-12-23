# Nullability & Type Safety Report

## 1. Rust Backend Analysis

### Domain Models (`backend/src/domain/models.rs`)
*   **Status**: Healthy.
*   **Details**:
    *   `Tournament::end_date` (`Option<DateTime<Utc>>`): Correctly models ongoing/single-day tournaments.
    *   `Player::cuescore_id` (`Option<i64>`): Correctly models players who might not be linked to CueScore (e.g., manual entries).
    *   `Game`: Strictly typed (no `Option`), which is excellent for data integrity.

### External API Models (`backend/src/fetchers/cuescore_models.rs`)
*   **Status**: Necessary complexity.
*   **Details**: Heavily uses `Option` (`stoptime`, `venues`, etc.). This accurately reflects the external API's inconsistency. **Recommendation**: Keep as is to prevent deserialization errors from external data.

### Repositories
*   **Status**: Idiomatic.
*   **Details**: Methods like `get_player_rating_detail` return `Result<Option<T>>`, forcing callers to handle the "not found" case explicitly.

## 2. Angular Frontend Analysis

### Generated API Models (`frontend/src/app/models/api.ts`)
*   **Status**: **OUT OF SYNC & Overly Permissive.**
*   **Critical Finding**: `api.ts` contains `avatarUrl` fields, but `proto/api.proto` indicates these fields were removed. The generated code is stale.
*   **Issue**: Proto3 defaults make fields optional in generated TypeScript (`cuescoreId?: number | undefined`).
    *   *Example*: `HeadToHeadResponse.player1` is `PlayerDetail | undefined`.
    *   *Reality*: The backend `get_head_to_head_comparison` guarantees these are present (returns 404 otherwise).
    *   *Impact*: Frontend code requires unnecessary `?.` checks or `if (player1)` guards.
*   **Recommendation**:
    1.  **Regenerate** `api.ts` to sync with `api.proto`.
    2.  Consider using `ts-proto` options (like `forceLong=long`, `useOptionals=false` for specific fields) or a wrapper layer to enforce strictness where the backend guarantees it.

### Component State
*   **Status**: Good, improving.
*   **Details**:
    *   Usage of `signal<T | null>(null)` for async data (like `player` in `PlayerOverlayComponent`) is correct usage of nullability to represent "loading/not loaded".
    *   Adoption of `input.required<T>()` (e.g., `MatchHistoryComponent`) successfully eliminates `undefined` checks for parent-provided data.

## 3. Action Plan
1.  **Regenerate Protobuf**: Run `npm run proto:generate` (or equivalent) to remove stale fields like `avatarUrl`.
2.  **Strict Frontend Mapping**: Consider creating mapped Typescript interfaces for key domain objects (like `PlayerDetail`) that are strict (non-nullable) where possible, rather than using the raw Proto interfaces directly in templates.
3.  **Continue `input.required`**: Ensure all new components use this for required props.
