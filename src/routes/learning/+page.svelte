<script lang="ts">
  import { tick } from "svelte";
  import Tex from "$lib/components/Tex.svelte";

  let openSections = $state<Record<string, boolean>>({
    markov: true,
    states: false,
    re24: false,
    matrices: false,
    entropy: false,
    pipeline: false,
  });

  function toggle(key: string) {
    openSections[key] = !openSections[key];
  }

  async function scrollToSection(key: string) {
    openSections[key] = true;
    await tick();
    document.getElementById(key)?.scrollIntoView({ behavior: "smooth" });
  }

  const tex = {
    markovProp: String.raw`P(X_{n+1} = j \mid X_n = i, X_{n-1}, \ldots, X_0) = P(X_{n+1} = j \mid X_n = i)`,
    transProb: String.raw`P(i \to j) = \frac{\text{number of times state } i \text{ transitioned to state } j}{\text{total transitions out of state } i}`,
    absorbing: String.raw`\text{State 25: } 3\text{ outs (absorbing)} \quad P(\text{absorbing} \to \text{absorbing}) = 1`,
    Q: "Q",
    I: "I",
    N: "N",
    R: "R",
    fundamental: String.raw`N = (I - Q)^{-1}`,
    expectedRuns: String.raw`\text{Expected Runs} = N \cdot R`,
    rowStochastic: String.raw`\sum_j P(i \to j) = 1 \quad \text{for every state } i`,
    shannonH: String.raw`H = -\sum_{i=1}^n p_i \ln(p_i)`,
    pi: "p_i",
    i: "i",
    lnN: String.raw`\ln(n)`,
    exampleH: String.raw`H = -(0.60 \ln 0.60 + 0.25 \ln 0.25 + 0.15 \ln 0.15) \approx 0.94`,
    ln3: String.raw`\ln(3) \approx 1.10`,
    weightedH: String.raw`H_{\text{overall}} = \frac{\sum_i w_i \cdot H(\text{row}_i)}{\sum_i w_i}`,
  };
</script>

<h1>Learning: The Math Behind MLB Markov</h1>
<p>
  Everything in this app is built on a few core ideas from probability and information theory.
  This page walks through each one so you can understand exactly what you're looking at
  in the Offense and Pitching views.
</p>

<nav class="toc card">
  <h3>Contents</h3>
  <ol>
    <li><button class="link" onclick={() => scrollToSection('markov')}>What is a Markov Chain?</button></li>
    <li><button class="link" onclick={() => scrollToSection('states')}>The 25 Base-Out States</button></li>
    <li><button class="link" onclick={() => scrollToSection('re24')}>Expected Runs (RE24)</button></li>
    <li><button class="link" onclick={() => scrollToSection('matrices')}>Reading a Transition Matrix</button></li>
    <li><button class="link" onclick={() => scrollToSection('entropy')}>Pitch Sequence Entropy</button></li>
    <li><button class="link" onclick={() => scrollToSection('pipeline')}>The Data Pipeline</button></li>
  </ol>
</nav>

<!-- Section 1: Markov Chains -->
<section class="card section" id="markov">
  <button class="section-header" onclick={() => toggle('markov')}>
    <h2>1. What is a Markov Chain?</h2>
    <span class="chevron" class:open={openSections.markov}>▸</span>
  </button>

  {#if openSections.markov}
    <div class="section-body">
      <p>
        A <strong>Markov chain</strong> is a model where the next state of a system depends
        <em>only on the current state</em>, not on the sequence of events that preceded it.
        This is called the <strong>memoryless property</strong>.
      </p>

      <div class="formula">
        <Tex math={tex.markovProp} display={true} />
      </div>

      <p>
        In plain English: to predict what happens next, all you need to know is where you are
        <em>right now</em>. History doesn't matter.
      </p>

      <h3>Why Baseball Fits</h3>
      <p>
        Baseball is one of the cleanest real-world applications of Markov chains.
        Consider a runner on second base with one out. It doesn't matter <em>how</em> that
        runner got there — walked, singled, stole the base, or was hit by a pitch. The
        probability of what happens on the next plate appearance is the same regardless.
      </p>
      <p>
        The base-out state (who's on base + how many outs) fully describes the situation.
        Every plate appearance is a transition from one state to another. That's a Markov chain.
      </p>

      <h3>The Transition Probability</h3>
      <p>
        Each transition has a probability. If we observe thousands of plate appearances, we can
        estimate these probabilities directly from the data:
      </p>
      <div class="formula">
        <Tex math={tex.transProb} display={true} />
      </div>
      <p>
        That's exactly what the Offense view computes — the empirical transition probabilities
        from an entire season of play-by-play data.
      </p>
    </div>
  {/if}
</section>

<!-- Section 2: Base-Out States -->
<section class="card section" id="states">
  <button class="section-header" onclick={() => toggle('states')}>
    <h2>2. The 25 Base-Out States</h2>
    <span class="chevron" class:open={openSections.states}>▸</span>
  </button>

  {#if openSections.states}
    <div class="section-body">
      <p>
        Every half-inning in baseball can be described by two things: <strong>how many outs</strong>
        there are and <strong>which bases are occupied</strong>.
      </p>

      <h3>The Breakdown</h3>
      <ul>
        <li><strong>Outs:</strong> 0, 1, or 2 (3 levels)</li>
        <li><strong>Base configurations:</strong> 8 possibilities
          <div class="base-grid">
            <span class="badge">---</span> empty
            <span class="badge">1--</span> runner on 1st
            <span class="badge">-2-</span> runner on 2nd
            <span class="badge">--3</span> runner on 3rd
            <span class="badge">12-</span> 1st and 2nd
            <span class="badge">1-3</span> 1st and 3rd
            <span class="badge">-23</span> 2nd and 3rd
            <span class="badge">123</span> bases loaded
          </div>
        </li>
      </ul>

      <p>
        3 out levels × 8 base configurations = <strong>24 active states</strong>.
      </p>

      <h3>The 25th State: Absorbing</h3>
      <p>
        When a third out is recorded, the inning ends. This is the <strong>absorbing state</strong> —
        once the system enters it, it never leaves. No more transitions are possible.
      </p>
      <div class="formula">
        <Tex math={tex.absorbing} display={true} />
      </div>
      <p>
        This absorbing state is what makes the expected runs calculation possible.
        Without it, the math doesn't close — there's no terminal condition for the inning.
      </p>

      <h3>State Encoding</h3>
      <p>
        In this app, states are encoded as <code>outs_bases</code>. For example:
      </p>
      <ul>
        <li><code>0_000</code> — 0 outs, bases empty (start of inning)</li>
        <li><code>1_100</code> — 1 out, runner on 1st</li>
        <li><code>2_011</code> — 2 outs, runners on 2nd and 3rd</li>
        <li><code>3_---</code> — 3 outs, inning over (absorbing)</li>
      </ul>
    </div>
  {/if}
</section>

<!-- Section 3: Expected Runs (RE24) -->
<section class="card section" id="re24">
  <button class="section-header" onclick={() => toggle('re24')}>
    <h2>3. Expected Runs (RE24)</h2>
    <span class="chevron" class:open={openSections.re24}>▸</span>
  </button>

  {#if openSections.re24}
    <div class="section-body">
      <p>
        RE24 (Run Expectancy based on the 24 base-out states) answers a fundamental question:
        <strong>given the current situation, how many runs will score before the inning ends?</strong>
      </p>
      <p>
        This is the "money chart" of baseball analytics. It tells you the value of every situation.
      </p>

      <h3>The Fundamental Matrix</h3>
      <p>
        To compute expected runs, we use a result from absorbing Markov chain theory.
        First, partition the transition matrix into two pieces:
      </p>
      <ul>
        <li><Tex math={tex.Q} /> — the 24×24 submatrix of transitions <em>between active states</em></li>
        <li><Tex math={tex.I} /> — the 24×24 identity matrix</li>
      </ul>
      <p>
        The <strong>fundamental matrix</strong> <Tex math={tex.N} /> tells us the expected number of times
        the chain visits each active state before being absorbed:
      </p>
      <div class="formula">
        <Tex math={tex.fundamental} display={true} />
      </div>
      <p>
        This is a 24×24 matrix inversion. The app computes it using Gaussian elimination with
        partial pivoting — no external linear algebra library needed for a 24×24 system.
      </p>

      <h3>From Visits to Runs</h3>
      <p>
        Let <Tex math={tex.R} /> be a vector where each entry is the average runs scored
        during transitions <em>out of</em> that state. Then:
      </p>
      <div class="formula">
        <Tex math={tex.expectedRuns} display={true} />
      </div>
      <p>
        The result is a 24-element vector — one expected run value for each active state.
      </p>

      <h3>What the Numbers Mean</h3>
      <p>
        Some results are intuitive, others are surprising:
      </p>
      <ul>
        <li><strong>Bases loaded, 0 outs</strong> ≈ 2.3 expected runs — the highest-value state</li>
        <li><strong>Bases empty, 0 outs</strong> ≈ 0.5 expected runs — start of a clean inning</li>
        <li><strong>Runner on 3rd, 2 outs</strong> ≈ 0.4 expected runs — feels valuable, but with 2 outs the chance of scoring before the 3rd out is only about 40%</li>
        <li><strong>Runner on 2nd, 0 outs</strong> ≈ 1.1 expected runs — often worth more than runner on 3rd with 1 out</li>
      </ul>
      <p>
        These numbers shift from team to team. The Offense view lets you compare any team
        against the league average.
      </p>
    </div>
  {/if}
</section>

<!-- Section 4: Transition Matrices -->
<section class="card section" id="matrices">
  <button class="section-header" onclick={() => toggle('matrices')}>
    <h2>4. Reading a Transition Matrix</h2>
    <span class="chevron" class:open={openSections.matrices}>▸</span>
  </button>

  {#if openSections.matrices}
    <div class="section-body">
      <p>
        A <strong>transition matrix</strong> is a grid of probabilities. Each row represents a
        current state, each column represents a possible next state, and each cell is the
        probability of that transition happening.
      </p>

      <h3>How to Read the Heatmap</h3>
      <ul>
        <li>Pick a row (your current state)</li>
        <li>Read across — each cell shows the probability of landing in that column's state</li>
        <li>Every row sums to 1.0 (you have to go <em>somewhere</em>)</li>
        <li>Brighter colors = higher probability</li>
      </ul>

      <h3>A Concrete Example</h3>
      <p>
        Start at <code>0_000</code> (0 outs, bases empty — the beginning of every inning).
        The most common transitions:
      </p>
      <ul>
        <li><code>0_000</code> → <code>1_000</code> — an out was made, nobody reached base (~65%)</li>
        <li><code>0_000</code> → <code>0_100</code> — batter reached 1st, still 0 outs (~22%)</li>
        <li><code>0_000</code> → <code>0_010</code> — batter reached 2nd (double) (~5%)</li>
      </ul>
      <p>
        The remaining ~8% splits across walks, home runs, errors, and rarer events.
      </p>

      <h3>Row-Stochastic</h3>
      <p>
        The matrix is <strong>row-stochastic</strong>: every row sums to exactly 1. This is a
        fundamental property of transition matrices — it means the probabilities are properly
        normalized.
      </p>
      <div class="formula">
        <Tex math={tex.rowStochastic} display={true} />
      </div>
      <p>
        The absorbing state's row is all zeros except for a 1.0 in its own column — once
        the inning is over, it stays over.
      </p>
    </div>
  {/if}
</section>

<!-- Section 5: Entropy -->
<section class="card section" id="entropy">
  <button class="section-header" onclick={() => toggle('entropy')}>
    <h2>5. Pitch Sequence Entropy</h2>
    <span class="chevron" class:open={openSections.entropy}>▸</span>
  </button>

  {#if openSections.entropy}
    <div class="section-body">
      <p>
        <strong>Shannon entropy</strong> measures how "surprising" or unpredictable a random
        variable is. Applied to pitching: how hard is it to guess what pitch comes next?
      </p>

      <h3>The Formula</h3>
      <div class="formula">
        <Tex math={tex.shannonH} display={true} />
      </div>
      <p>
        Where <Tex math={tex.pi} /> is the probability of pitch type <Tex math={tex.i} />.
        Higher entropy = more unpredictable. Lower entropy = more predictable.
      </p>

      <h3>Intuition</h3>
      <ul>
        <li>
          <strong>Minimum entropy (0):</strong> the pitcher throws one pitch type 100% of the time.
          You always know what's coming.
        </li>
        <li>
          <strong>Maximum entropy (<Tex math={tex.lnN} />):</strong> every pitch type is equally likely.
          Maximum uncertainty for the hitter.
        </li>
      </ul>

      <h3>A Real Example</h3>
      <p>
        Imagine a pitcher with three pitch types and these probabilities:
      </p>
      <div class="example-row">
        <span class="badge">FF 60%</span>
        <span class="badge">SL 25%</span>
        <span class="badge">CH 15%</span>
      </div>
      <div class="formula">
        <Tex math={tex.exampleH} display={true} />
      </div>
      <p>
        Maximum possible entropy for 3 pitch types would be <Tex math={tex.ln3} />.
        So this pitcher is moderately unpredictable — leaning fastball but not locked in.
      </p>

      <h3>Why It Matters</h3>
      <p>
        Predictable pitchers get hit harder. If a hitter knows a fastball is coming 80%
        of the time when the count is 3-1, they can sit on it. The Pitching view breaks
        down entropy <strong>by count</strong> — you'll often see entropy drop in hitter's
        counts (3-1, 2-0) and rise in pitcher's counts (0-2, 1-2) as the pitcher can
        afford to expand.
      </p>

      <h3>Weighted Matrix Entropy</h3>
      <p>
        The overall entropy score for a pitcher is the <strong>weighted average</strong> of
        row entropies in their transition matrix, weighted by how often each row (previous
        pitch type) occurs:
      </p>
      <div class="formula">
        <Tex math={tex.weightedH} display={true} />
      </div>
      <p>
        This accounts for the fact that a pitcher might be very predictable after their
        curveball but unpredictable after their fastball. The weights ensure the overall
        score reflects actual usage patterns.
      </p>

      <h3>The Markov Assumption for Pitches</h3>
      <p>
        A first-order Markov chain assumes the next pitch depends only on the <em>previous</em>
        pitch. In reality, a pitcher's decision often depends on the last 2-3 pitches, the
        count, the batter, and the game situation. The Markov assumption is a useful
        simplification, not gospel. The per-count breakdown partially addresses this by
        conditioning on the count state.
      </p>
    </div>
  {/if}
</section>

<!-- Section 6: Data Pipeline -->
<section class="card section" id="pipeline">
  <button class="section-header" onclick={() => toggle('pipeline')}>
    <h2>6. The Data Pipeline</h2>
    <span class="chevron" class:open={openSections.pipeline}>▸</span>
  </button>

  {#if openSections.pipeline}
    <div class="section-body">
      <p>
        Understanding what happens when you click <strong>"Bootstrap Current Season"</strong> will
        help you understand why the initial load takes a while and what you're getting out of it.
      </p>

      <h3>What Gets Fetched</h3>
      <p>
        The app pulls data from the free <strong>MLB Stats API</strong> (<code>statsapi.mlb.com</code>).
        The bootstrap process downloads <em>every completed game</em> in the current MLB season.
        For each game, it fetches the full <strong>play-by-play feed</strong>, which includes:
      </p>
      <ul>
        <li><strong>Every plate appearance:</strong> who batted, who pitched, the event (single, strikeout, walk, etc.), base runners before and after, outs before and after, runs scored</li>
        <li><strong>Every individual pitch:</strong> pitch type (4-seam fastball, slider, changeup, etc.), velocity, the count at the time, and the result (ball, strike, foul, in play)</li>
      </ul>

      <h3>The Scale</h3>
      <p>
        A full MLB season has <strong>~2,430 scheduled games</strong>. By mid-season, roughly
        800-1,200 games will have been played. Each game averages:
      </p>
      <ul>
        <li><strong>~60-80 plate appearances</strong> (plays)</li>
        <li><strong>~250-300 individual pitches</strong></li>
      </ul>
      <p>
        So a full season import is roughly <strong>150,000+ plays</strong> and
        <strong>700,000+ pitches</strong>. That's a lot of data — and it's all stored
        locally in a SQLite database on your machine.
      </p>

      <h3>Why It Takes Time</h3>
      <p>
        Each game requires its own API request. With a 250ms delay between requests
        to avoid overwhelming the MLB API, processing 800 games takes about
        <strong>3-4 minutes</strong>. The initial bootstrap is a one-time cost.
      </p>

      <h3>Incremental Updates</h3>
      <p>
        After the initial bootstrap, subsequent clicks of the button only fetch
        <strong>new games</strong> — ones that have finished since your last import.
        During the regular season that's typically 12-15 games per day, which takes
        just a few seconds.
      </p>
      <p>
        The app tracks which games have been fetched (<code>data_fetched</code> flag)
        and only downloads games marked as <code>Final</code> that haven't been processed
        yet. Scheduled or postponed games are skipped automatically.
      </p>

      <h3>From Raw Data to Models</h3>
      <p>
        Once play-by-play data is in the database, the Markov models are computed
        <strong>on demand</strong> when you visit the Offense or Pitching tabs:
      </p>
      <ol>
        <li>Query all plays for the season (optionally filtered by team)</li>
        <li>Map each play to a state transition: <code>before_state → after_state</code></li>
        <li>Count occurrences of each transition pair</li>
        <li>Normalize rows to get probabilities (divide each count by the row total)</li>
        <li>For offense: solve the fundamental matrix to get expected runs</li>
        <li>For pitching: pair consecutive pitches, build transition matrix, compute entropy</li>
      </ol>
      <p>
        Results are cached in the database so subsequent views are instant.
      </p>
    </div>
  {/if}
</section>

<style>
  .toc {
    margin-bottom: 24px;
  }
  .toc ol {
    margin: 0;
    padding-left: 1.4em;
  }
  .toc li {
    margin-bottom: 4px;
  }
  .toc .link {
    background: none;
    border: none;
    color: var(--accent);
    padding: 0;
    font: inherit;
    cursor: pointer;
    text-align: left;
  }
  .toc .link:hover {
    text-decoration: underline;
    transform: none;
  }

  .section {
    margin-bottom: 16px;
  }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background: none;
    border: none;
    padding: 0;
    color: var(--ink);
    cursor: pointer;
    text-align: left;
  }
  .section-header:hover {
    transform: none;
  }
  .section-header h2 {
    margin: 0;
  }
  .chevron {
    font-size: 1.2rem;
    transition: transform 200ms ease;
    color: var(--ink-mute);
    flex-shrink: 0;
  }
  .chevron.open {
    transform: rotate(90deg);
  }

  .section-body {
    padding-top: 16px;
    border-top: 1px solid var(--line-soft);
    margin-top: 12px;
  }
  .section-body ul,
  .section-body ol {
    color: var(--ink-soft);
    padding-left: 1.4em;
    margin: 0 0 1em;
  }
  .section-body li {
    margin-bottom: 6px;
  }

  .base-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
    align-items: center;
  }
  .base-grid .badge {
    font-family: var(--mono);
  }

  .example-row {
    display: flex;
    gap: 8px;
    margin: 8px 0 4px;
  }

  .formula {
    background: var(--bg-soft);
    border: 1px solid var(--line-soft);
    padding: 14px 18px;
    border-radius: var(--radius);
    margin: 0.6em 0 1.2em;
    overflow-x: auto;
  }

  code {
    font-family: var(--mono);
    font-size: 0.9em;
    background: var(--bg-soft);
    padding: 2px 5px;
    border-radius: 3px;
  }
</style>
