<script lang="ts">
  import "../app.css";
  import "katex/dist/katex.min.css";
  import { page } from "$app/state";

  let { children } = $props();

  const links = [
    { href: "/", label: "Home" },
    { href: "/offense", label: "Offense" },
    { href: "/pitching", label: "Pitching" },
    { href: "/learning", label: "Learning" },
  ];

  function isActive(href: string): boolean {
    if (href === "/") return page.url.pathname === "/";
    return page.url.pathname.startsWith(href);
  }
</script>

<div class="app-frame">
  <header class="top">
    <div class="shell row">
      <a class="brand" href="/" aria-label="MLB Markov home">
        <span class="diamond" aria-hidden="true"></span>
        <span class="brand-text">
          <span class="brand-title">MLB Markov</span>
          <span class="brand-sub">State Transition Models</span>
        </span>
      </a>
      <nav>
        {#each links as link}
          <a class:active={isActive(link.href)} href={link.href}>{link.label}</a>
        {/each}
      </nav>
    </div>
  </header>

  <main class="main">
    <div class="shell">
      {@render children?.()}
    </div>
  </main>

  <footer class="footer">
    <p class="muted">
      Data from <a href="https://statsapi.mlb.com" target="_blank" rel="noopener">statsapi.mlb.com</a> · Built with Rust + Tauri · Markov chain analysis
    </p>
  </footer>
</div>

<style>
  .app-frame {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    overscroll-behavior: none;
  }
  .top {
    position: sticky;
    top: 0;
    z-index: 10;
    flex-shrink: 0;
    background: color-mix(in srgb, var(--bg) 92%, transparent);
    backdrop-filter: saturate(140%) blur(10px);
    -webkit-backdrop-filter: saturate(140%) blur(10px);
    border-bottom: 1px solid var(--line-soft);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 24px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--ink);
    text-decoration: none;
  }
  .brand:hover {
    text-decoration: none;
  }
  .diamond {
    width: 16px;
    height: 16px;
    background: var(--accent);
    transform: rotate(45deg);
    border-radius: 3px;
    box-shadow: inset 0 -2px 0 rgba(0, 0, 0, 0.18);
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    line-height: 1.05;
  }
  .brand-title {
    font-family: var(--serif);
    font-weight: 600;
    font-size: 1.05rem;
  }
  .brand-sub {
    font-size: 0.72rem;
    color: var(--ink-mute);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  nav {
    display: flex;
    gap: 4px;
  }
  nav a {
    color: var(--ink-soft);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.95rem;
  }
  nav a:hover {
    text-decoration: none;
    background: var(--bg-soft);
    color: var(--ink);
  }
  nav a.active {
    background: var(--ink);
    color: var(--bg-elev);
  }
  .main {
    flex: 1;
    padding: 12px 24px 10px;
    overflow: auto;
    min-height: 0;
    overscroll-behavior: none;
  }
  .main > :global(.shell) {
    max-width: 1600px;
    margin: 0;
  }
  .footer {
    flex-shrink: 0;
    max-width: 1600px;
    padding: 6px 24px 8px;
    border-top: 1px solid var(--line-soft);
    font-size: 0.75rem;
  }
  .footer p {
    margin: 0;
  }
</style>
