import {useCallback, useMemo, useState, type ReactNode} from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import {
  AMUD_THEMES,
  themeCssUrl,
  themeSearchText,
  type AmudTheme,
} from '@site/src/data/themes';
import styles from './themes.module.css';

type CopyKind = 'css' | 'url';

function ThemePreview({theme}: {theme: AmudTheme}) {
  const imgSrc = useBaseUrl(theme.previewImage);
  const [imgError, setImgError] = useState(false);
  const showPlaceholder = imgError && theme.id !== 'default';

  return (
    <div className={styles.preview}>
      {!showPlaceholder ? (
        <img
          src={imgSrc}
          alt={`${theme.name} preview`}
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
            Add <code>docs/static/themes/assets/{theme.previewImage.split('/').pop()}</code> for a screenshot
          </span>
        </div>
      )}
    </div>
  );
}

function ThemeCard({
  theme,
  baseUrl,
  onCopy,
  copiedKey,
}: {
  theme: AmudTheme;
  baseUrl: string;
  onCopy: (theme: AmudTheme, kind: CopyKind) => void;
  copiedKey: string | null;
}) {
  const cssUrl = themeCssUrl(baseUrl, theme);
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
            <>
              <button
                type="button"
                className={`${styles.btn} ${styles.btnPrimary} ${
                  copiedKey === `${theme.id}-css` ? styles.btnCopied : ''
                }`}
                onClick={() => onCopy(theme, 'css')}>
                {copiedKey === `${theme.id}-css` ? 'Copied!' : 'Copy CSS'}
              </button>
              {cssUrl && (
                <button
                  type="button"
                  className={`${styles.btn} ${styles.btnSecondary} ${
                    copiedKey === `${theme.id}-url` ? styles.btnCopied : ''
                  }`}
                  onClick={() => onCopy(theme, 'url')}>
                  {copiedKey === `${theme.id}-url` ? 'Copied!' : 'Copy URL'}
                </button>
              )}
            </>
          )}
        </div>
      </div>
    </article>
  );
}

export default function ThemesGallery(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  const baseUrl = siteConfig.baseUrl;
  const [query, setQuery] = useState('');
  const [toast, setToast] = useState<string | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

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
    async (theme: AmudTheme, kind: CopyKind) => {
      try {
        if (kind === 'url') {
          const url = themeCssUrl(baseUrl, theme);
          if (!url) return;
          await navigator.clipboard.writeText(url);
          setCopiedKey(`${theme.id}-url`);
          showToast('Theme URL copied — paste in Settings → Import from URL');
        } else {
          const url = themeCssUrl(baseUrl, theme);
          if (!url) return;
          const res = await fetch(url);
          if (!res.ok) throw new Error('Failed to fetch theme CSS');
          const css = await res.text();
          await navigator.clipboard.writeText(css);
          setCopiedKey(`${theme.id}-css`);
          showToast('CSS copied — paste in Settings → Custom CSS');
        }
        window.setTimeout(() => setCopiedKey(null), 2000);
      } catch {
        showToast('Copy failed — try again or use the raw URL');
      }
    },
    [baseUrl, showToast],
  );

  return (
    <Layout
      title="Theme Gallery"
      description="Browse, search, and apply AMUD dashboard custom CSS themes. Copy CSS or import directly from GitHub Pages URLs.">
      <main className="container margin-vert--lg">
        <header className={styles.hero}>
          <h1 className={styles.heroTitle}>Custom Themes Gallery</h1>
          <p className={styles.heroSubtitle}>
            Search ready-made AMUD themes, preview them, then copy the CSS or use the
            GitHub Pages URL in <strong>Settings → Customization → Custom CSS</strong>.
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
            <li>
              Pick a theme below and click <strong>Copy CSS</strong>, or copy the{' '}
              <strong>URL</strong> for remote import.
            </li>
            <li>
              Open your AMUD dashboard → <strong>Settings → Customization → Custom CSS</strong>.
            </li>
            <li>
              Paste the CSS directly, or paste the URL and click{' '}
              <strong>Import from URL</strong>, then <strong>Save</strong>.
            </li>
            <li>
              For a full theme effect, reset built-in accent color to default or remove{' '}
              <code>--accent-color</code> from the theme CSS — UI branding overrides custom CSS.
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
                baseUrl={baseUrl}
                onCopy={handleCopy}
                copiedKey={copiedKey}
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
