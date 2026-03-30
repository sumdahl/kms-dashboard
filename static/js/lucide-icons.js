import { createIcons, icons } from 'https://esm.sh/lucide@0.475.0';

document.addEventListener('DOMContentLoaded', () => {
  createIcons({
    icons,
    nameAttr: 'data-lucide',
    className: 'lucide',
  });
});

window.lucide = { createIcons, icons };
