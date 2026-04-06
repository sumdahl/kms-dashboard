import { createIcons, icons } from "/static/js/lucide.esm.js";

document.addEventListener("DOMContentLoaded", () => {
  createIcons({
    icons,
    nameAttr: "data-lucide",
    className: "lucide",
  });
});

window.lucide = { createIcons, icons };
