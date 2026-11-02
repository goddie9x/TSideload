// TSideload GUI shell (scaffold). i18n via ./i18n/{en,vi}.json,
// backend via Tauri `invoke`. Install flow is P1 — see src-tauri/src/main.rs.
(function () {
  "use strict";

  var strings = {};
  var lang = localStorage.getItem("tsideload-lang") || "en";

  function t(path, vars) {
    var parts = path.split(".");
    var cur = strings;
    for (var i = 0; i < parts.length; i++) {
      cur = cur && cur[parts[i]];
    }
    var s = typeof cur === "string" ? cur : path;
    if (vars) {
      Object.keys(vars).forEach(function (k) {
        s = s.replace("{" + k + "}", vars[k]);
      });
    }
    return s;
  }

  function applyI18n() {
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      el.textContent = t(el.getAttribute("data-i18n"));
    });
    document.documentElement.lang = lang;
  }

  function loadLang(next) {
    lang = next;
    localStorage.setItem("tsideload-lang", lang);
    fetch("i18n/" + lang + ".json")
      .then(function (r) { return r.json(); })
      .then(function (j) { strings = j; applyI18n(); })
      .catch(function () { /* keep previous strings */ });
  }

  function isTauri() {
    return !!(window.__TAURI__ && window.__TAURI__.core);
  }

  function refreshDevices() {
    var list = document.getElementById("devices");
    var count = document.getElementById("devcount");
    if (!isTauri()) {
      list.innerHTML = "";
      count.textContent = "Preview mode (no Tauri backend).";
      return;
    }
    window.__TAURI__.core.invoke("list_devices").then(function (devices) {
      list.innerHTML = "";
      devices.forEach(function (d) {
        var li = document.createElement("li");
        li.textContent = d.udid + "  (" + d.connection_type + ")";
        list.appendChild(li);
      });
      if (!devices.length) {
        var li = document.createElement("li");
        li.textContent = t("devices.empty");
        list.appendChild(li);
      }
      count.textContent = t("devices.count", { n: devices.length });
    }).catch(function (e) {
      count.textContent = t("install.error", { msg: String(e) });
    });
  }

  document.getElementById("lang").value = lang;
  document.getElementById("lang").addEventListener("change", function (e) {
    loadLang(e.target.value);
  });
  document.getElementById("refresh").addEventListener("click", refreshDevices);

  loadLang(lang);
  // Re-apply device list after strings load.
  setTimeout(refreshDevices, 300);
})();
