// Applies any previously saved accessibility preference (see
// pages::accessibility) before first paint, on every page — independent
// of wasm hydration. Loaded via a plain <script src> (not inlined through
// the view! macro) specifically so it's unambiguously real, unescaped JS.
(function () {
  var root = document.documentElement;
  ["va-font", "va-contrast", "va-motion", "va-text-size"].forEach(function (key) {
    var value = localStorage.getItem(key);
    if (value) {
      root.classList.add(key + "-" + value);
    }
  });
})();

window.__vaSetPref = function (key, value) {
  if (value) {
    localStorage.setItem(key, value);
  } else {
    localStorage.removeItem(key);
  }
  document.documentElement.className = document.documentElement.className
    .split(" ")
    .filter(function (c) {
      return c.indexOf(key + "-") !== 0;
    })
    .join(" ");
  if (value) {
    document.documentElement.classList.add(key + "-" + value);
  }
};
