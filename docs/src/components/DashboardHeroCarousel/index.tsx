import React, {useEffect, useState} from 'react';
import useBaseUrl from '@docusaurus/useBaseUrl';

const SLIDE_DEFS = [
  {src: 'img/AMUD-Dashboard.png', alt: 'AMUD Dashboard — default theme', label: 'Default'},
  {src: 'img/AMUD-Dashboard_mobile_taghawsa_theme.png', alt: 'AMUD Dashboard — mobile Taghawsa theme', label: 'Taghawsa Mobile'},
  {src: 'img/AMUD-Dashboard_login_mobile_taghawsa_theme.png', alt: 'AMUD login — mobile Taghawsa theme', label: 'Login Mobile'},
  {src: 'img/hero/brutalist-mono.png', alt: 'AMUD Dashboard — Brutalist Mono theme', label: 'Brutalist Mono'},
  {src: 'img/hero/cotton-candy.png', alt: 'AMUD Dashboard — Cotton Candy theme', label: 'Cotton Candy'},
  {src: 'img/hero/rainforest-mist.png', alt: 'AMUD Dashboard — Rainforest Mist theme', label: 'Rainforest Mist'},
];

const INTERVAL_MS = 5000;

function HeroSlide({
  src,
  alt,
  active,
  eager,
}: Readonly<{src: string; alt: string; active: boolean; eager: boolean}>): React.ReactElement {
  const url = useBaseUrl(src);
  return (
    <img
      src={url}
      alt={alt}
      className={'dashboard-preview-img dashboard-hero-carousel__slide' + (active ? ' is-active' : '')}
      loading={eager ? 'eager' : 'lazy'}
      decoding="async"
    />
  );
}

export default function DashboardHeroCarousel(): React.ReactElement {
  const [active, setActive] = useState(0);
  const [reduceMotion, setReduceMotion] = useState(false);

  useEffect(() => {
    const mq = globalThis.matchMedia?.('(prefers-reduced-motion: reduce)');
    if (!mq) return;
    const update = () => setReduceMotion(mq.matches);
    update();
    mq.addEventListener('change', update);
    return () => mq.removeEventListener('change', update);
  }, []);

  useEffect(() => {
    if (reduceMotion) return;
    const id = globalThis.setInterval(() => {
      setActive((i) => (i + 1) % SLIDE_DEFS.length);
    }, INTERVAL_MS);
    return () => globalThis.clearInterval(id);
  }, [reduceMotion]);

  return (
    <div className="dashboard-hero-carousel">
      <div className="dashboard-hero-carousel__stage">
        {SLIDE_DEFS.map((slide, index) => (
          <HeroSlide
            key={slide.src}
            src={slide.src}
            alt={slide.alt}
            active={index === active}
            eager={index === 0}
          />
        ))}
        <div className="reflection" />
      </div>
      <div className="dashboard-hero-carousel__dots" role="tablist" aria-label="Dashboard preview themes">
        {SLIDE_DEFS.map((slide, index) => (
          <button
            key={slide.src}
            type="button"
            role="tab"
            aria-selected={index === active}
            aria-label={slide.label}
            className={'dashboard-hero-carousel__dot' + (index === active ? ' is-active' : '')}
            onClick={() => setActive(index)}
          />
        ))}
      </div>
    </div>
  );
}
