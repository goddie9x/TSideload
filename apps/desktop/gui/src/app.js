// TSideload GUI. i18n via ./i18n/{en,vi}.json, backend via Tauri `invoke`.
// Install flow: pick IPA (dialog plugin) -> Apple ID login (+2FA dialog,
// mediated by backend events + the `twofa_answer` command) -> sign & install.
// The password is cleared from the DOM on submit and never logged.
(function () {
  "use strict";

  var strings = {};
  var lang = localStorage.getItem("tsideload-lang") || "en";
  var devicesCache = [];
  var lastStage = null;   // {stage, detail} — re-rendered on language change
  var twofaPrompt = null; // last 2FA prompt DTO — re-rendered on language change

  function $(id) { return document.getElementById(id); }

  function t(path, vars) {
    var parts = path.split(".");
    var cur = strings;
    for (var i = 0; i < parts.length; i++) {
      cur = cur && cur[parts[i]];
    }
    var s = typeof cur === "string" ? cur : path;
    if (vars) {
      Object.keys(vars).forEach(function (k) {
        s = s.split("{" + k + "}").join(String(vars[k]));
      });
    }
    return s;
  }

  function applyI18n() {
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      el.textContent = t(el.getAttribute("data-i18n"));
    });
    document.querySelectorAll("[data-i18n-ph]").forEach(function (el) {
      el.setAttribute("placeholder", t(el.getAttribute("data-i18n-ph")));
    });
    document.documentElement.lang = lang;
    renderDeviceOptions();
    renderStage();
    if (twofaPrompt) renderTwofa();
  }

  function loadLang(next) {
    lang = next;
    localStorage.setItem("tsideload-lang", lang);
    fetch("i18n/" + lang + ".json")
      .then(function (r) { return r.json(); })
      .then(function (j) { strings = j; applyI18n(); })
      .catch(function () { /* keep previous strings */ });
  }

  function tauriCore() { return (window.__TAURI__ && window.__TAURI__.core) || null; }
  function tauriEvent() {
    var g = window.__TAURI__;
    return (g && g.event && typeof g.event.listen === "function") ? g.event : null;
  }
  function isTauri() { return !!tauriCore(); }

  function setMsg(text, cls) {
    var el = $("install_msg");
    el.textContent = text || "";
    el.className = cls || "hint";
  }

  // ---- devices ----

  function refreshDevices() {
    var list = $("devices"), count = $("devcount");
    if (!isTauri()) {
      devicesCache = [];
      list.innerHTML = "";
      count.textContent = "Preview mode (no Tauri backend).";
      renderDeviceOptions();
      return;
    }
    tauriCore().invoke("list_devices").then(function (devices) {
      devicesCache = devices || [];
      list.innerHTML = "";
      devicesCache.forEach(function (d) {
        var li = document.createElement("li");
        li.textContent = d.udid + "  (" + d.connection_type + ")";
        list.appendChild(li);
      });
      if (!devicesCache.length) {
        var li = document.createElement("li");
        li.textContent = t("devices.empty");
        list.appendChild(li);
      }
      count.textContent = t("devices.count", { n: devicesCache.length });
      renderDeviceOptions();
    }).catch(function (e) {
      count.textContent = t("install.error", { msg: String(e) });
    });
  }

  function renderDeviceOptions() {
    var sel = $("target_device");
    if (!sel) return;
    var cur = sel.value;
    sel.innerHTML = "";
    var first = document.createElement("option");
    first.value = "";
    first.textContent = t("install.first_device");
    sel.appendChild(first);
    devicesCache.forEach(function (d) {
      var o = document.createElement("option");
      o.value = d.udid;
      o.textContent = d.udid + " (" + d.connection_type + ")";
      sel.appendChild(o);
    });
    var stillThere = cur === "" || devicesCache.some(function (d) { return d.udid === cur; });
    sel.value = stillThere ? cur : "";
  }

  // ---- install ----

  function onBrowse() {
    var core = tauriCore();
    if (!core) { setMsg(t("install.no_dialog"), "err"); return; }
    // tauri-plugin-dialog v2 command; the capability + plugin init live in
    // src-tauri. Works without the npm package via the core invoke bridge.
    core.invoke("plugin:dialog|open", {
      options: { filters: [{ name: "IPA", extensions: ["ipa"] }], multiple: false }
    }).then(function (path) {
      if (Array.isArray(path)) path = path[0] || "";
      if (path) {
        $("ipa_path").value = path;
        setMsg("", "hint");
      }
    }).catch(function (e) {
      setMsg(t("install.error", { msg: String(e) }), "err");
    });
  }

  function onInstall() {
    var core = tauriCore(), ev = tauriEvent();
    if (!core) { setMsg(t("install.no_dialog"), "err"); return; }
    if (!ev) { setMsg(t("install.no_backend"), "err"); return; }
    var ipa = $("ipa_path").value.trim();
    if (!ipa) { setMsg(t("install.no_ipa"), "err"); return; }
    var appleId = $("apple_id").value.trim();
    var pwEl = $("apple_password");
    var password = pwEl.value;
    pwEl.value = ""; // never keep the password in the DOM
    if (!appleId || !password) { setMsg(t("install.no_credentials"), "err"); return; }

    var btn = $("do_install");
    btn.disabled = true;
    setMsg("", "hint");
    lastStage = null;
    renderStage();

    // Single struct arg: Tauri matches it by parameter name (`req`).
    core.invoke("install_ipa", { req: {
      apple_id: appleId,
      password: password,
      ipa_path: ipa,
      udid: $("target_device").value || null,
      increased_memory_limit: $("mem_limit").checked
    }}).catch(function (e) {
      // The backend already emitted an "error" stage on the normal path;
      // this only fires for failures before any stage was emitted.
      if (!lastStage || lastStage.stage !== "error") {
        renderStage({ stage: "error", detail: String((e && e.message) || e) });
      }
    }).finally(function () {
      btn.disabled = false;
    });
  }

  function renderStage(stage) {
    if (stage) lastStage = stage;
    var el = $("progress");
    var s = stage || lastStage;
    if (!s) { el.hidden = true; return; }
    if (s.stage === "done") {
      el.hidden = true;
      setMsg(t("install.success"), "ok");
      return;
    }
    if (s.stage === "error") {
      el.hidden = true;
      setMsg(t("install.error", { msg: s.detail || "" }), "err");
      return;
    }
    el.hidden = false;
    setMsg("", "hint");
    if (s.stage === "login") el.textContent = t("install.stage_login", { id: s.detail || "" });
    else if (s.stage === "signing") el.textContent = t("install.stage_signing");
  }

  // ---- 2FA dialog ----
  // Same contract as the `tsd` CLI: when the prompt has `unknown` set, code
  // entry is refused upstream — only method selection / abort are offered.

  function onTwofaPrompt(e) {
    twofaPrompt = e.payload || {};
    renderTwofa();
    $("twofa_modal").hidden = false;
    if (!twofaPrompt.unknown) {
      var code = $("twofa_code");
      if (code) code.focus();
    }
  }

  function renderTwofa() {
    var p = twofaPrompt;
    if (!p) return;
    var errEl = $("twofa_error");
    errEl.hidden = !p.last_error;
    errEl.textContent = p.last_error || "";
    $("twofa_desc").textContent = p.unknown ? t("auth.twofa_unknown") : t("auth.twofa_prompt");

    $("twofa_code_wrap").hidden = !!p.unknown;
    $("twofa_resend").hidden = !!p.unknown;
    $("twofa_devices").hidden = !(p.unknown || p.sms);

    var smsWrap = $("twofa_sms_wrap");
    smsWrap.hidden = !p.sms;
    if (p.sms) {
      var sel = $("twofa_sms_number");
      sel.innerHTML = "";
      (p.numbers || []).forEach(function (n) {
        var o = document.createElement("option");
        o.value = n.id;
        o.textContent = n.number;
        sel.appendChild(o);
      });
    }
  }

  function answerTwofa(dto) {
    $("twofa_modal").hidden = true;
    twofaPrompt = null;
    var codeEl = $("twofa_code");
    if (codeEl) codeEl.value = "";
    var core = tauriCore();
    if (!core) return;
    // Internally-tagged enum on the backend: {kind:"code",code}, {kind:"resend"},
    // {kind:"send_to_devices"}, {kind:"send_sms",number_id}, {kind:"abort"}.
    core.invoke("twofa_answer", { answer: dto }).catch(function (e) {
      setMsg(t("install.error", { msg: String(e) }), "err");
    });
  }

  function subscribeEvents() {
    var ev = tauriEvent();
    if (!ev) return;
    ev.listen("tsideload://install-stage", function (e) { renderStage(e.payload); });
    ev.listen("tsideload://twofa-prompt", onTwofaPrompt);
  }

  // ---- init ----

  $("lang").value = lang;
  $("lang").addEventListener("change", function (e) { loadLang(e.target.value); });
  $("refresh").addEventListener("click", refreshDevices);
  $("browse").addEventListener("click", onBrowse);
  $("do_install").addEventListener("click", onInstall);
  $("twofa_submit").addEventListener("click", function () {
    answerTwofa({ kind: "code", code: $("twofa_code").value.trim() });
  });
  $("twofa_resend").addEventListener("click", function () { answerTwofa({ kind: "resend" }); });
  $("twofa_devices").addEventListener("click", function () { answerTwofa({ kind: "send_to_devices" }); });
  $("twofa_sms").addEventListener("click", function () {
    answerTwofa({ kind: "send_sms", number_id: parseInt($("twofa_sms_number").value, 10) || 0 });
  });
  $("twofa_abort").addEventListener("click", function () { answerTwofa({ kind: "abort" }); });

  loadLang(lang);
  subscribeEvents();
  // Re-apply device list after strings load.
  setTimeout(refreshDevices, 300);
})();
