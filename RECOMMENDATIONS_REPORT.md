# Recommendations Report: Warsaw Pool Ranking

## 1. UX & Frontend Improvements (Angular)

### A. Visualizations & Charts
*   **Enhanced Rating History**: You already have `Chart.js`.
    *   *Feature*: Add a "Compare" mode to the rating chart where a user can overlay another player's rating history on top of the current player's graph.
    *   *Feature*: Add a "Win Rate Trend" chart (rolling average of last 10 games).
*   **"Form" Indicators**:
    *   *Feature*: Display a simple visual indicator of "Recent Form" (e.g., last 5 matches: ✅❌✅✅❌) in the Player List and Player Overlay. This gives immediate insight without clicking through.

### B. Navigation & Layout
*   **Tournaments View**:
    *   *Recommendation*: Currently, the app is Player-centric. Create a "Tournaments" page listing past and upcoming tournaments, filtering by Venue or Date.
*   **Venue Hub**:
    *   *Recommendation*: Create a "Venues" page. Clicking a venue shows the "King of the Hill" (top rated player who played there recently) and a list of tournaments hosted there.
*   **Skeleton Loading**:
    *   *UX*: Replace spinning loaders with "Skeleton" placeholders (gray pulsing bars) that mimic the table/card layout. This improves perceived performance significantly.

### C. Mobile Experience
*   **Responsive Tables**: On mobile, standard tables are hard to read.
    *   *Recommendation*: Use a "Card View" for the Player List on small screens (using `@media` queries to switch from `<table>` to `<div>` grid).
*   **Swipe Actions**: Implement swipe-to-compare or swipe-to-favorite in the player list.

### D. Aesthetics & Branding
*   **Logo**: The current app could benefit from a custom logo.
    *   *Idea*: A minimal, geometric design combining a **Pool Ball (8 or 9)** with the **Warsaw Mermaid (Syrenka)** sword/shield, or a stylized "W" made of pool cues.
*   **Theme**: Ensure "Dark Mode" uses a high-contrast palette (e.g., deep slate background with vibrant accent colors like neon green or electric blue) to fit the "Pool Hall" vibe.

## 2. New Data-Driven Features (No Accounts Required)

### A. "The Nemesis" (Rivalry Analysis)
*   *Feature*: In the Player Overlay, add a "Rivalries" tab.
    *   **Nemesis**: The opponent against whom the player has the lowest win rate (min. 5 matches).
    *   **Bunny**: The opponent against whom the player has the highest win rate.
    *   **Most Played**: The opponent they face most often.

### B. "What If" Scenarios
*   *Feature*: A "Rating Calculator" tool. "If I beat Player X (Rating 1500) in a tournament, how much will my rating go up?" This engages users to check the app before/during matches.

### C. Badges / Achievements
*   *Feature*: Auto-calculated badges displayed on Player Profiles.
    *   *Examples*:
        *   🏆 **Streak Master**: Won 5+ games in a row.
        *   🏟️ **Venue Regular**: Played in 3+ different venues.
        *   ⚡ **Active**: Played 10+ games in the last month.
        *   🛡️ **Veteran**: 100+ total games.

## 3. Technical Enhancements

*   **PWA (Progressive Web App)**: Enable PWA support in Angular. This allows users to "Install" the app on their phone home screen, giving an "App-like" feel without an App Store.
*   **SEO / Meta Tags**: Since this is a public ranking, ensure player pages have dynamic `meta` tags (Open Graph) so sharing a profile link on Facebook/WhatsApp shows the player's name and rank preview.

## 4. Implementation Priority (Suggested)

1.  **"Form" Indicators** (High Impact, Low Effort)
2.  **Skeleton Loaders** (High UX Value)
3.  **Rivalry Analysis** (High Engagement)
4.  **Tournaments/Venues Pages** (Architecture expansion)
5.  **Logo/Branding Refresh**
