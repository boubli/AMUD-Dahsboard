import React, {type ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import {useBlogPost} from '@docusaurus/plugin-content-blog/client';
import BlogPostItemContent from '@theme/BlogPostItem/Content';
import {blogCoverForSlug} from '@site/src/data/blog-covers';
import type {Props} from '@theme/BlogPostItem';
import styles from './styles.module.css';

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function ListCard({children}: Readonly<{children: ReactNode}>): ReactNode {
  const {metadata, assets} = useBlogPost();
  const slug = metadata.slug;
  const coverPath =
    (typeof metadata.frontMatter.image === 'string' && metadata.frontMatter.image) ||
    assets.image ||
    blogCoverForSlug(slug);
  const coverUrl = useBaseUrl(coverPath);

  return (
    <article className={styles.card}>
      <Link to={metadata.permalink} className={styles.coverLink}>
        <div className={styles.cover}>
          <img src={coverUrl} alt="" className={styles.coverImg} loading="lazy" />
        </div>
      </Link>
      <div className={styles.body}>
        <div className={styles.meta}>
          <time dateTime={metadata.date}>{formatDate(metadata.date)}</time>
          {metadata.readingTime != null && (
            <span>{Math.ceil(metadata.readingTime)} min read</span>
          )}
        </div>
        <h2 className={styles.title}>
          <Link to={metadata.permalink}>{metadata.title}</Link>
        </h2>
        {metadata.description && (
          <p className={styles.excerpt}>{metadata.description}</p>
        )}
        {metadata.tags.length > 0 && (
          <div className={styles.tags}>
            {metadata.tags.slice(0, 3).map((tag) => (
              <Link
                key={tag.permalink}
                to={tag.permalink}
                className={styles.tag}>
                {tag.label}
              </Link>
            ))}
          </div>
        )}
        <Link to={metadata.permalink} className={styles.readMore}>
          Read article →
        </Link>
      </div>
      <div className={styles.hiddenContent}>{children}</div>
    </article>
  );
}

function PostArticle({children, className}: Props): ReactNode {
  const {metadata, assets} = useBlogPost();
  const slug = metadata.slug;
  const coverPath =
    (typeof metadata.frontMatter.image === 'string' && metadata.frontMatter.image) ||
    assets.image ||
    blogCoverForSlug(slug);
  const coverUrl = useBaseUrl(coverPath);

  return (
    <article className={clsx(styles.post, className)}>
      <div className={styles.postHero}>
        <img src={coverUrl} alt="" className={styles.postHeroImg} />
        <div className={styles.postHeroOverlay} />
      </div>
      <header className={styles.postHeader}>
        <time className={styles.postDate} dateTime={metadata.date}>
          {formatDate(metadata.date)}
          {metadata.readingTime != null &&
            ` · ${Math.ceil(metadata.readingTime)} min read`}
        </time>
        <h1 className={styles.postTitle}>{metadata.title}</h1>
        {metadata.description && (
          <p className={styles.postDescription}>{metadata.description}</p>
        )}
        <p className={styles.postByline}>
          By <strong>Youssef Boubli</strong> · Creator of AMUD Dashboard
        </p>
      </header>
      <BlogPostItemContent>{children}</BlogPostItemContent>
    </article>
  );
}

export default function BlogPostItem({children, className}: Props): ReactNode {
  const {isBlogPostPage} = useBlogPost();
  if (isBlogPostPage) {
    return <PostArticle className={className}>{children}</PostArticle>;
  }
  return <ListCard>{children}</ListCard>;
}
