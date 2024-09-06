// File: frontend/pages/articles/[slug].tsx

import React from 'react';
import { useRouter } from 'next/router';
import Layout from '@/components/Layout';


const ArticlePage: React.FC = () => {
  const router = useRouter();
  const { slug } = router.query;

  return (
    <Layout>
      <h1>Article: {slug}</h1>
      {/* TODO: Fetch and display article content */}
    </Layout>
  );
};

export default ArticlePage;