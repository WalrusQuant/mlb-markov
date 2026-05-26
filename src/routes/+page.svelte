<script lang="ts">
  let loading = $state(false);
</script>

<h1>MLB Markov</h1>
<p>State transition models for MLB offense and pitching, built on Markov chain logic.</p>

<div class="grid">
  <div class="card">
    <h3>Offense: At-Bat Transitions</h3>
    <p>25 base-out states, transition probabilities, and expected runs (RE24) computed from play-by-play data.</p>
    <a href="/offense">View Offense</a>
  </div>

  <div class="card">
    <h3>Pitching: Sequence Predictability</h3>
    <p>Pitch-type transition matrices and Shannon entropy scores. Find the most predictable pitchers in MLB.</p>
    <a href="/pitching">View Pitching</a>
  </div>
</div>

<hr />

<div class="card">
  <h3>Database Status</h3>
  <p class="muted">Import play-by-play data to get started.</p>
  <div class="stats-row">
    <div class="stat">
      <span class="stat-value mono">--</span>
      <span class="stat-label muted">Games</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">--</span>
      <span class="stat-label muted">Plays</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">--</span>
      <span class="stat-label muted">Pitches</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">--</span>
      <span class="stat-label muted">Players</span>
    </div>
  </div>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 24px;
  }
  .stats-row {
    display: flex;
    gap: 32px;
    margin-top: 12px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stat-value {
    font-size: 1.4rem;
    font-weight: 600;
  }
  .stat-label {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
</style>
