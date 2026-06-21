import React, {type ReactNode} from 'react';
import clsx from 'clsx';
import {HtmlClassNameProvider, ThemeClassNames} from '@docusaurus/theme-common';
import {
  BlogPostProvider,
  useBlogPost,
} from '@docusaurus/plugin-content-blog/client';
import BlogLayout from '@theme/BlogLayout';
import BlogPostItem from '@theme/BlogPostItem';
import BlogPostPaginator from '@theme/BlogPostPaginator';
import BlogPostPageMetadata from '@theme/BlogPostPage/Metadata';
import BlogPostPageStructuredData from '@theme/BlogPostPage/StructuredData';
import type {Props} from '@theme/BlogPostPage';

function BlogPostPageContent({
  children,
  sidebar,
}: Readonly<{
  children: ReactNode;
  sidebar: Props['sidebar'];
}>): ReactNode {
  const {metadata} = useBlogPost();
  const {nextItem, prevItem} = metadata;
  return (
    <BlogLayout sidebar={sidebar}>
      <BlogPostItem>{children}</BlogPostItem>
      {(nextItem || prevItem) && (
        <div style={{maxWidth: 760, margin: '0 auto', padding: '0 1rem'}}>
          <BlogPostPaginator nextItem={nextItem} prevItem={prevItem} />
        </div>
      )}
    </BlogLayout>
  );
}

export default function BlogPostPage(props: Props): ReactNode {
  const BlogPostContent = props.content;
  return (
    <BlogPostProvider content={props.content} isBlogPostPage>
      <HtmlClassNameProvider
        className={clsx(
          ThemeClassNames.wrapper.blogPages,
          ThemeClassNames.page.blogPostPage,
        )}>
        <BlogPostPageMetadata />
        <BlogPostPageStructuredData />
        <BlogPostPageContent sidebar={props.sidebar}>
          <BlogPostContent />
        </BlogPostPageContent>
      </HtmlClassNameProvider>
    </BlogPostProvider>
  );
}
