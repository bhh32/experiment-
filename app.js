// Lumen prototype — minimal interactions for the design demo.

const root = document.documentElement;
const palette = document.getElementById("palette");
const themeToggle = document.getElementById("themeToggle");

// Theme: persist to localStorage, fall back to system preference.
const stored = localStorage.getItem("lumen-theme");
if (stored) root.dataset.theme = stored;
else if (window.matchMedia("(prefers-color-scheme: light)").matches) root.dataset.theme = "light";

themeToggle.addEventListener("click", () => {
  const next = root.dataset.theme === "light" ? "dark" : "light";
  root.dataset.theme = next;
  localStorage.setItem("lumen-theme", next);
});

// Command palette: Ctrl+K opens, Esc closes, click backdrop closes.
function openPalette() {
  palette.setAttribute("aria-hidden", "false");
  const input = palette.querySelector("input");
  if (input) setTimeout(() => input.focus(), 30);
}
function closePalette() {
  palette.setAttribute("aria-hidden", "true");
}
window.addEventListener("keydown", (e) => {
  if (e.ctrlKey && !e.altKey && !e.shiftKey && e.key.toLowerCase() === "k") {
    e.preventDefault();
    palette.getAttribute("aria-hidden") === "false" ? closePalette() : openPalette();
  } else if (e.key === "Escape") {
    closePalette();
  }
});
palette.addEventListener("click", (e) => {
  if (e.target === palette) closePalette();
});

// Tab switching in the co-pilot panel (visual only).
document.querySelectorAll(".cp-tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".cp-tab").forEach((t) => {
      t.classList.remove("active");
      t.setAttribute("aria-selected", "false");
    });
    tab.classList.add("active");
    tab.setAttribute("aria-selected", "true");
  });
});

// Rail buttons — visual active state only.
document.querySelectorAll(".rail-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".rail-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
  });
});

// Tour overlay — numbered pins + floating legend, toggled by the ? button.
// Defaults ON for first visit so newcomers know what they're looking at.
const tourBtn = document.getElementById("tourToggle");
const tourClose = document.getElementById("tourClose");
const TOUR_KEY = "lumen-tour";
function setTour(on) {
  document.body.classList.toggle("tour-on", on);
  if (tourBtn) tourBtn.setAttribute("aria-pressed", on ? "true" : "false");
  localStorage.setItem(TOUR_KEY, on ? "on" : "off");
}
const tourStored = localStorage.getItem(TOUR_KEY);
setTour(tourStored === null ? true : tourStored === "on");
if (tourBtn) tourBtn.addEventListener("click", () => {
  setTour(!document.body.classList.contains("tour-on"));
});
if (tourClose) tourClose.addEventListener("click", () => setTour(false));
