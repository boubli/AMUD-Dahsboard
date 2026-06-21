import React, {type ReactNode} from 'react';
import Layout from '@theme/Layout';
import type {Props} from '@theme/BlogLayout';

/** Full-width blog layout — no doc-style sidebar column. */
export default function BlogLayout(props: Props): ReactNode {
  const {children, ...layoutProps} = props;
  return <Layout {...layoutProps}>{children}</Layout>;
}
