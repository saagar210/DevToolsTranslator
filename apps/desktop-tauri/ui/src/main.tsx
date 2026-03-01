import React from 'react';
import { createRoot } from 'react-dom/client';
import DesktopApp from './App.js';
import './styles.css';

const container = document.getElementById('root');
if (!container) {
  throw new Error('Root container not found');
}

createRoot(container).render(
  <React.StrictMode>
    <DesktopApp />
  </React.StrictMode>,
);
