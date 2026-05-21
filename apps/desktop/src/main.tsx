import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/heroui.css';
import './styles/index.less';

createRoot(document.getElementById('app')!).render(
    <StrictMode>
        <App />
    </StrictMode>
);
