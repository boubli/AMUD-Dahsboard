import {useCallback, useMemo, useState, type ReactNode} from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import {
  AMUD_THEMES,
  themeSearchText,
  type AmudTheme,
} from '@site/src/data/themes';
import styles from './themes.module.css';

function ThemePreview({theme}: {theme: AmudTheme}) {
  const imgSrc = useBaseUrl(theme.previewImage);
  const [imgError, setImgError] = useState(false);
  const showPlaceholder = imgError && theme.id !== 'default';

  return (
    <div className={styles.preview}>
      {!showPlaceholder ? (
        <img
          src={imgSrc}
          alt={`${theme.name} dashboard preview`}
          className={styles.previewImg}
          onError={() => setImgError(true)}
        />
      ) : (
        <div
          className={styles.previewPlaceholder}
          style={{background: theme.palette.background}}>
          <div className={styles.paletteRow}>
            <span
              className={styles.swatch}
              style={{background: theme.palette.background}}
              title="Background"
            />
            <span
              className={styles.swatch}
              style={{background: theme.palette.card}}
              title="Cards"
            />
            <span
              className={styles.swatch}
              style={{background: theme.palette.accent}}
              title="Accent"
            />
            <span
              className={styles.swatch}
              style={{background: theme.palette.text}}
              title="Text"
            />
          </div>
          <span className={styles.previewPlaceholderLabel}>
            Preview screenshot coming soon
          </span>
        </div>
      )}
    </div>
  );
}

function ThemeCard({
  theme,
  onCopy,
  copiedId,
}: {
  theme: AmudTheme;
  onCopy: (theme: AmudTheme) => void;
  copiedId: string | null;
}) {
  const isDefault = theme.id === 'default';

  return (
    <article className={styles.card}>
      <ThemePreview theme={theme} />
      <div className={styles.cardBody}>
        <h2 className={styles.cardTitle}>
          {theme.name}
          {theme.inspiration && theme.inspirationUrl && (
            <>
              {' '}
              <span style={{fontSize: '0.75rem', fontWeight: 400, opacity: 0.6}}>
                inspired by{' '}
                <a href={theme.inspirationUrl} target="_blank" rel="noopener noreferrer">
                  {theme.inspiration}
                </a>
              </span>
            </>
          )}
        </h2>
        <p className={styles.cardDesc}>{theme.description}</p>
        <div className={styles.tags}>
          {theme.tags.map((tag) => (
            <span key={tag} className={styles.tag}>
              {tag}
            </span>
          ))}
        </div>
        <div className={styles.actions}>
          {isDefault ? (
            <Link
              className={`${styles.btn} ${styles.btnPrimary}`}
              to="/docs/configuration#custom-css-injection">
              How to reset
            </Link>
          ) : (
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary} ${
                copiedId === theme.id ? styles.btnCopied : ''
              }`}
              onClick={() => onCopy(theme)}>
              {copiedId === theme.id ? 'Copied!' : 'Copy CSS'}
            </button>
          )}
        </div>
      </div>
    </article>
  );
}

export default function ThemesGallery(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  const [query, setQuery] = useState('');
  const [toast, setToast] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return AMUD_THEMES;
    return AMUD_THEMES.filter((t) => themeSearchText(t).includes(q));
  }, [query]);

  const showToast = useCallback((message: string) => {
    setToast(message);
    window.setTimeout(() => setToast(null), 2200);
  }, []);

  const handleCopy = useCallback(
    async (theme: AmudTheme) => {
      if (!theme.cssFile) return;
      try {
        const res = await fetch(`${siteConfig.baseUrl}${theme.cssFile}`);
        if (!res.ok) throw new Error('Failed to load theme CSS');
        const css = await res.text();
        await navigator.clipboard.writeText(css);
        setCopiedId(theme.id);
        showToast('CSS copied — paste in Settings → Appearance → Custom CSS');
        window.setTimeout(() => setCopiedId(null), 2000);
      } catch {
        showToast('Copy failed — open the theme docs and copy CSS manually');
      }
    },
    [siteConfig.baseUrl, showToast],
  );

  return (
    <Layout
      title="Theme Gallery"
      description="Browse AMUD dashboard themes with preview screenshots. Copy CSS and paste it into Settings.">
      <main className="container margin-vert--lg">
        <header className={styles.hero}>
          <h1 className={styles.heroTitle}>Custom Themes Gallery</h1>
          <p className={styles.heroSubtitle}>
            Preview each theme with a screenshot, then <strong>Copy CSS</strong> and paste it into{' '}
            <strong>Settings → Appearance → Custom CSS</strong> on your dashboard.
          </p>
          <div className={styles.searchBar}>
            <span className={styles.searchIcon} aria-hidden>
              🔍
            </span>
            <input
              type="search"
              className={styles.searchInput}
              placeholder="Search themes (e.g. nord, neon, purple, warm…)"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              aria-label="Search themes"
            />
          </div>
        </header>

        <section className={styles.howTo}>
          <h2>How to apply a theme</h2>
          <ol>
            <li>Browse the gallery below — each card shows a preview of how the theme looks.</li>
            <li>Click <strong>Copy CSS</strong> on the theme you want.</li>
            <li>
              Open your AMUD dashboard → <strong>Settings → Appearance → Custom CSS</strong>, paste
              the CSS, and click <strong>Save</strong>.
            </li>
            <li>
              For the full theme effect, reset the built-in accent color to default or remove{' '}
              <code>--accent-color</code> from the pasted CSS — UI branding overrides custom CSS.
            </li>
          </ol>
        </section>

        <p className={styles.resultCount}>
          {filtered.length} theme{filtered.length !== 1 ? 's' : ''}
          {query ? ` matching “${query}”` : ''}
        </p>

        {filtered.length === 0 ? (
          <p className={styles.emptyState}>No themes match your search. Try another keyword.</p>
        ) : (
          <div className={styles.grid}>
            {filtered.map((theme) => (
              <ThemeCard
                key={theme.id}
                theme={theme}
                onCopy={handleCopy}
                copiedId={copiedId}
              />
            ))}
          </div>
        )}

        <p style={{textAlign: 'center', color: 'var(--ifm-color-secondary)', marginBottom: '3rem'}}>
          Want to build your own? See the{' '}
          <Link to="/docs/themes">CSS variable reference</Link> and{' '}
          <Link to="/docs/troubleshooting#recovering-from-broken-custom-css">recovery guide</Link>{' '}
          if something breaks.
        </p>
      </main>

      {toast && <div className={styles.toast} role="status">{toast}</div>}
    </Layout>
  );
}
