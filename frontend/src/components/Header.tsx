// File: frontend/components/Header.tsx

import React from 'react';
import Link from 'next/link';

const Header: React.FC = () => {
  return (
    <header>
      <nav>
        <Link href="/">Home</Link>
        <Link href="/collections">Collections</Link>
        {/* TODO: Add more navigation items */}
      </nav>
    </header>
  );
};

export default Header;