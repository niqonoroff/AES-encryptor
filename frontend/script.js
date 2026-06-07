(() => {
  const L = {
    ru: {
      openSettings: 'Открытие файла',
      saveSettings: 'Сохранение файла',
      enterPassword: 'Введите пароль:',
      newPassword: 'Новый пароль:',
      confirmPassword: 'Подтвердите пароль:',
      argonTime: 'Argon время:',
      argonMemory: 'Argon память:',
      argonParallel: 'Argon параллелизм:',
      saltSize: 'Размер соли:',
      nonceSize: 'Размер nonce:',
      calculator: 'Калькулятор',
      invalidPassword: 'Неверный пароль | Неверный формат | Файл повреждён',
      errorRead: 'Ошибка чтения файла',
      errorSave: 'Ошибка сохранения: ',
      version: 'v1.2.0',
      shortcuts: '| Ctrl+O — Открыть | Ctrl+S — Сохранить | Ctrl+Q — Закрыть | F1 — Язык | F2 — Экран | F3 — Калькулятор | F5 — Зашифровать файл | F6 — Расшифровать файл',
      nqEditor: 'NQ Editor',
      saved: 'Сохранено'
    },
    en: {
      openSettings: 'Open Settings',
      saveSettings: 'Save Settings',
      enterPassword: 'Enter password:',
      newPassword: 'New password:',
      confirmPassword: 'Confirm password:',
      argonTime: 'Argon time:',
      argonMemory: 'Argon memory:',
      argonParallel: 'Argon parallelism:',
      saltSize: 'Salt size (bytes):',
      nonceSize: 'Nonce size (bytes):',
      calculator: 'Calculator',
      invalidPassword: 'Invalid password | Invalid file format | Corrupted file',
      errorRead: 'Error reading file',
      errorSave: 'Error saving file: ',
      version: 'v1.2.0',
      shortcuts: '| Ctrl+O — Open | Ctrl+S — Save | Ctrl+Q — Close | F1 — Lang | F2 — Fullscreen | F3 — Calc | F5 — Encrypt file | F6 — Decrypt file',
      nqEditor: 'NQ Editor',
      saved: 'Saved'
    }
  };

  const DEFAULT = {
    argon_time: 8,
    argon_memory: 512 * 1024,
    argon_parallel: 4,
    salt_size: 32,
    nonce_size: 12
  };

  const editor = document.getElementById('editor');
  const titleText = document.getElementById('title-text');
  const modifiedIcon = document.getElementById('modified-icon');
  const shortcutsText = document.getElementById('shortcuts-text');
  const clock = document.getElementById('clock');
  const calcModal = document.getElementById('calc-modal');
  const calcInput = document.getElementById('calc-input');
  const calcResult = document.getElementById('calc-result');
  const calcButtons = document.getElementById('calc-buttons');
  const calcEq = document.getElementById('calc-eq');
  const openModal = document.getElementById('open-modal');
  const saveModal = document.getElementById('save-modal');
  const loadingOverlay = document.getElementById('loading-overlay');
  const loadingText = document.getElementById('loading-text');

  let lang = 'ru';
  let currentFile = null;
  let savedText = '';
  let fontSize = 16;
  let calcOpen = false;
  let calcExpr = '';
  let pendingOpenData = null;
  let pendingOpenPath = null;
  let pendingMeta = null;
  let pendingIsFile = false;

  const TAURI = window.__TAURI__;
  const INV = TAURI && TAURI.core ? TAURI.core : TAURI;

  async function invoke(cmd, args) {
    if (!INV) throw new Error('Tauri IPC not available');
    if (INV.invoke) return await INV.invoke(cmd, args);
    throw new Error('Tauri invoke not found');
  }

  function t(key) { return L[lang][key]; }

  function fileName(path) {
    if (!path) return '';
    return path.replace(/\\/g, '/').split('/').pop();
  }

  function updateUI() {
    const display = currentFile || t('nqEditor');
    titleText.textContent = display;
    const short = currentFile ? fileName(currentFile) : 'NQ Editor';
    document.title = currentFile ? short + ' - NQ Editor' : 'NQ Editor';

    modifiedIcon.textContent = editor.value !== savedText ? '~' : '';
    shortcutsText.textContent = t('shortcuts');

    const time = new Date().toTimeString().slice(0, 8);
    clock.textContent = time + ' | ' + t('version');

    document.getElementById('calc-title').textContent = t('calculator');
    document.getElementById('open-title').textContent = t('openSettings');
    document.getElementById('open-pw-label').textContent = t('enterPassword');
    document.getElementById('open-time-label').textContent = t('argonTime');
    document.getElementById('open-mem-label').textContent = t('argonMemory');
    document.getElementById('open-par-label').textContent = t('argonParallel');
    document.getElementById('open-salt-label').textContent = t('saltSize');
    document.getElementById('open-nonce-label').textContent = t('nonceSize');
    document.getElementById('save-title').textContent = t('saveSettings');
    document.getElementById('save-new-label').textContent = t('newPassword');
    document.getElementById('save-confirm-label').textContent = t('confirmPassword');
    document.getElementById('save-time-label').textContent = t('argonTime');
    document.getElementById('save-mem-label').textContent = t('argonMemory');
    document.getElementById('save-par-label').textContent = t('argonParallel');
    document.getElementById('save-salt-label').textContent = t('saltSize');
    document.getElementById('save-nonce-label').textContent = t('nonceSize');
  }

  function applyFontSize() {
    editor.style.fontSize = fontSize + 'px';
  }

  function showLoading() {
    loadingText.textContent = lang === 'ru' ? 'Загрузка...' : 'Loading...';
    loadingOverlay.classList.remove('hidden');
    void loadingOverlay.offsetHeight;
  }

  function hideLoading() {
    loadingOverlay.classList.add('hidden');
  }

  function clockTick() {
    const time = new Date().toTimeString().slice(0, 8);
    clock.textContent = time + ' | ' + t('version');
  }

  const calcRows = [
    ['(', ')', '%', '<-'],
    ['7', '8', '9', '/'],
    ['4', '5', '6', '*'],
    ['1', '2', '3', '-'],
    ['0', '.', 'C', '+']
  ];

  function buildCalculator() {
    for (const row of calcRows) {
      for (const b of row) {
        const btn = document.createElement('button');
        btn.textContent = b;
        btn.dataset.val = b;
        calcButtons.appendChild(btn);
      }
    }
  }
  buildCalculator();

  function calcEval(expr) {
    const s = expr.replace(/%/g, '/100.0').replace(/\s/g, '');
    try {
      const result = Function('"use strict"; return (' + s + ')')();
      if (!isFinite(result)) return 'Error';
      return String(result);
    } catch (e) {
      return 'Error';
    }
  }

  function calcHandleInput(val) {
    if (val === 'C') { calcExpr = ''; calcResult.textContent = ''; }
    else if (val === '<-') { calcExpr = calcExpr.slice(0, -1); }
    else calcExpr += val;
    calcInput.value = calcExpr;
    if (calcExpr) {
      const r = calcEval(calcExpr);
      calcResult.textContent = r !== 'Error' ? '= ' + r : '';
    } else {
      calcResult.textContent = '';
    }
  }

  function calcEvalAndSet() {
    if (!calcExpr) return;
    const r = calcEval(calcExpr);
    calcInput.value = r;
    calcExpr = r;
    calcResult.textContent = '';
  }

  calcButtons.addEventListener('click', (e) => {
    const btn = e.target.closest('button');
    if (!btn) return;
    calcHandleInput(btn.dataset.val);
  });

  calcEq.addEventListener('click', calcEvalAndSet);

  function openOpenModal(data_b64, path) {
    pendingOpenData = data_b64;
    pendingOpenPath = path;
    document.getElementById('open-password').value = '';
    document.getElementById('open-time').value = String(DEFAULT.argon_time);
    document.getElementById('open-mem').value = String(DEFAULT.argon_memory);
    document.getElementById('open-par').value = String(DEFAULT.argon_parallel);
    document.getElementById('open-salt').value = String(DEFAULT.salt_size);
    document.getElementById('open-nonce').value = String(DEFAULT.nonce_size);
    openModal.classList.remove('hidden');
    document.getElementById('open-password').focus();
  }

  function openSaveModal() {
    document.getElementById('save-password').value = '';
    document.getElementById('save-confirm').value = '';
    document.getElementById('save-time').value = String(DEFAULT.argon_time);
    document.getElementById('save-mem').value = String(DEFAULT.argon_memory);
    document.getElementById('save-par').value = String(DEFAULT.argon_parallel);
    document.getElementById('save-salt').value = String(DEFAULT.salt_size);
    document.getElementById('save-nonce').value = String(DEFAULT.nonce_size);
    saveModal.classList.remove('hidden');
    document.getElementById('save-password').focus();
  }

  function closeModals() {
    openModal.classList.add('hidden');
    saveModal.classList.add('hidden');
    calcModal.classList.add('hidden');
    calcOpen = false;
  }

  async function submitOpen() {
    if (!pendingOpenData) return;
    const password = document.getElementById('open-password').value;
    const time = parseInt(document.getElementById('open-time').value) || DEFAULT.argon_time;
    const mem = parseInt(document.getElementById('open-mem').value) || DEFAULT.argon_memory;
    const par = parseInt(document.getElementById('open-par').value) || DEFAULT.argon_parallel;
    const salt = parseInt(document.getElementById('open-salt').value) || DEFAULT.salt_size;
    const nonce = parseInt(document.getElementById('open-nonce').value) || DEFAULT.nonce_size;

    showLoading();
    await new Promise(r => requestAnimationFrame(r));
    let text;
    try {
      text = await invoke('decrypt_data', {
        dataB64: pendingOpenData,
        password,
        argonTime: time,
        argonMemory: mem,
        argonParallel: par,
        saltSize: salt,
        nonceSize: nonce
      });
      hideLoading();
      openModal.classList.add('hidden');
    } catch (e) {
      hideLoading();
      editor.value = t('invalidPassword');
      savedText = editor.value;
      currentFile = null;
      openModal.classList.add('hidden');
      updateUI();
      pendingOpenData = null;
      pendingOpenPath = null;
      pendingIsFile = false;
      return;
    }

    if (pendingIsFile) {
      let saveData = text, saveExt = 'bin', saveName = 'file.bin';
      try {
        const meta = JSON.parse(text);
        saveData = meta.data || text;
        saveExt = meta.ext || base64Ext(saveData);
        saveName = meta.name || ('file.' + saveExt);
      } catch (_) {
        saveExt = base64Ext(text);
        saveName = 'file.' + saveExt;
      }
      const savePath = await invoke('pick_save_filter', { ext: saveExt, name: saveName });
      if (savePath) {
        const cleanData = saveData.replace(/\s/g, '');
        try {
          await invoke('write_file', { path: savePath, dataB64: cleanData });
          editor.value = t('saved') + ': ' + savePath;
          savedText = editor.value;
          updateUI();
        } catch (e) {
          try {
            await invoke('write_raw', { path: savePath, data: text });
            editor.value = t('saved') + ': ' + savePath;
            savedText = editor.value;
            updateUI();
          } catch (e2) {
            editor.value = 'Write error: ' + (e2?.toString() || '');
            savedText = editor.value;
            updateUI();
          }
        }
      } else {
        editor.value = text;
        savedText = text;
        currentFile = pendingOpenPath;
        updateUI();
      }
    } else {
      editor.value = text;
      savedText = text;
      currentFile = pendingOpenPath;
      updateUI();
    }
    pendingOpenData = null;
    pendingOpenPath = null;
    pendingIsFile = false;
  }

  async function submitSave() {
    const pw = document.getElementById('save-password').value;
    const confirm = document.getElementById('save-confirm').value;
    if (!pw || pw !== confirm) {
      document.getElementById('save-confirm').value = '';
      return;
    }
    const time = parseInt(document.getElementById('save-time').value) || DEFAULT.argon_time;
    const mem = parseInt(document.getElementById('save-mem').value) || DEFAULT.argon_memory;
    const par = parseInt(document.getElementById('save-par').value) || DEFAULT.argon_parallel;
    const salt = parseInt(document.getElementById('save-salt').value) || DEFAULT.salt_size;
    const nonce = parseInt(document.getElementById('save-nonce').value) || DEFAULT.nonce_size;

    const isFile = pendingMeta !== null;
    const plainText = isFile ? pendingMeta : editor.value;

    showLoading();
    await new Promise(r => requestAnimationFrame(r));
    try {
      const encryptedB64 = await invoke('encrypt_text', {
        text: plainText,
        password: pw,
        argonTime: time,
        argonMemory: mem,
        argonParallel: par,
        saltSize: salt,
        nonceSize: nonce
      });

      if (!currentFile || isFile) {
        const path = await invoke('pick_save_file');
        if (!path) { hideLoading(); return; }
        currentFile = path;
      }

      await invoke('write_file', { path: currentFile, dataB64: encryptedB64 });
      editor.value = t('saved') + ': ' + currentFile;
      savedText = editor.value;
      hideLoading();
      saveModal.classList.add('hidden');
      updateUI();
      pendingMeta = null;
    } catch (e) {
      hideLoading();
      if (!isFile) {
        editor.value = t('errorSave') + ' ' + (e?.toString() || '');
        savedText = editor.value;
      }
      pendingMeta = null;
    }
  }

  async function openFile() {
    try {
      const result = await invoke('pick_and_read_file');
      const [path, data_b64] = result;
      openOpenModal(data_b64, path);
    } catch (e) {
      if (e !== 'Cancelled') {
        editor.value = t('errorRead') + ' ' + (e?.toString() || '');
        savedText = editor.value;
        updateUI();
      }
    }
  }

  async function saveFile() {
    if (!currentFile) {
      try {
        const path = await invoke('pick_save_file');
        if (!path) return;
        currentFile = path;
      } catch (e) {
        editor.value = 'SAVE ERROR: ' + (e?.toString() || 'unknown');
        savedText = editor.value;
        updateUI();
        return;
      }
    }
    openSaveModal();
  }

  async function saveFileAs() {
    currentFile = null;
    await saveFile();
  }

  async function closeWindow() {
    try { await invoke('close_window'); } catch (e) {}
  }

  async function toggleFullscreen() {
    try { await invoke('toggle_fullscreen'); } catch (e) {}
  }

  function base64Ext(b64) {
    const h = b64.replace(/\s/g, '').slice(0, 8);
    if (h.startsWith('iVBOR')) return 'png';
    if (h.startsWith('/9j/') || h.startsWith('/9k=')) return 'jpg';
    if (h.startsWith('R0lGOD')) return 'gif';
    if (h.startsWith('UEsDBA')) return 'zip';
    if (h.startsWith('JVBER')) return 'pdf';
    return 'bin';
  }

  function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const bytes = new Uint8Array(reader.result);
        let bin = '';
        for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
        resolve(btoa(bin));
      };
      reader.onerror = () => reject('Read error');
      reader.readAsArrayBuffer(file);
    });
  }

  async function encryptFile() {
    try {
      const result = await invoke('pick_and_read_any_file');
      const [path, fileB64] = result;
      const name = fileName(path);
      const ext = name.includes('.') ? name.split('.').pop().toLowerCase() : '';
      const meta = JSON.stringify({ name, ext, data: fileB64 });
      pendingMeta = meta;
      openSaveModal();
    } catch (e) {
      if (e !== 'Cancelled') {
        editor.value = t('errorRead') + ' ' + (e?.toString() || '');
        savedText = editor.value;
        updateUI();
      }
    }
  }

  async function decryptFile() {
    try {
      const result = await invoke('pick_and_read_file');
      const [path, dataB64] = result;
      pendingOpenData = dataB64;
      pendingOpenPath = path;
      pendingIsFile = true;

      document.getElementById('open-password').value = '';
      document.getElementById('open-time').value = String(DEFAULT.argon_time);
      document.getElementById('open-mem').value = String(DEFAULT.argon_memory);
      document.getElementById('open-par').value = String(DEFAULT.argon_parallel);
      document.getElementById('open-salt').value = String(DEFAULT.salt_size);
      document.getElementById('open-nonce').value = String(DEFAULT.nonce_size);
      openModal.classList.remove('hidden');
      document.getElementById('open-password').focus();
    } catch (e) {
      if (e !== 'Cancelled') {
        editor.value = t('errorRead') + ' ' + (e?.toString() || '');
        savedText = editor.value;
        updateUI();
      }
    }
  }

  function toggleCalculator() {
    calcOpen = !calcOpen;
    if (calcOpen) {
      calcExpr = '';
      calcInput.value = '';
      calcResult.textContent = '';
      calcModal.classList.remove('hidden');
      calcInput.focus();
    } else {
      calcModal.classList.add('hidden');
    }
  }

  async function toggleLang() {
    lang = lang === 'ru' ? 'en' : 'ru';
    try {
      const cfg = JSON.parse(await invoke('load_config'));
      cfg.lang = lang;
      await invoke('save_config', { configJson: JSON.stringify(cfg) });
    } catch (e) {
      try {
        const cfg = JSON.parse(localStorage.getItem('nq-config') || '{}');
        cfg.lang = lang;
        localStorage.setItem('nq-config', JSON.stringify(cfg));
      } catch (e2) {}
    }
    updateUI();
  }

  document.addEventListener('keydown', (e) => {
    const pwOpen = !openModal.classList.contains('hidden');
    const pwSave = !saveModal.classList.contains('hidden');
    const inCalc = !calcModal.classList.contains('hidden');
    const hasModal = pwOpen || pwSave || inCalc;

    if (e.code === 'Escape') {
      if (hasModal) { closeModals(); e.preventDefault(); }
      return;
    }

    if (e.code === 'F3') {
      if (!pwOpen && !pwSave) { toggleCalculator(); e.preventDefault(); }
      return;
    }

    if (e.code === 'Enter') {
      if (pwOpen) { submitOpen(); e.preventDefault(); return; }
      if (pwSave) { submitSave(); e.preventDefault(); return; }
      if (inCalc) { calcEvalAndSet(); e.preventDefault(); return; }
    }

    if (hasModal) {
      if (inCalc) {
        if (/^[0-9.+\-*/().%]$/.test(e.key)) { calcHandleInput(e.key); e.preventDefault(); }
        else if (e.key === 'Backspace') { calcHandleInput('<-'); e.preventDefault(); }
      }
      return;
    }

    if (e.ctrlKey && e.code === 'KeyO') { e.preventDefault(); openFile(); return; }
    if (e.ctrlKey && e.code === 'KeyS') {
      e.preventDefault();
      if (e.shiftKey) saveFileAs();
      else saveFile();
      return;
    }
    if (e.ctrlKey && e.code === 'KeyQ') { e.preventDefault(); closeWindow(); return; }
    if (e.code === 'F1') { e.preventDefault(); toggleLang(); return; }
    if (e.code === 'F2') { e.preventDefault(); toggleFullscreen(); return; }
    if (e.code === 'F5') { e.preventDefault(); encryptFile(); return; }
    if (e.code === 'F6') { e.preventDefault(); decryptFile(); return; }
  });

  editor.addEventListener('wheel', (e) => {
    if (e.ctrlKey) {
      e.preventDefault();
      if (e.deltaY < 0) fontSize = Math.min(60, fontSize + 1);
      else fontSize = Math.max(6, fontSize - 1);
      applyFontSize();
    }
  }, { passive: false });

  editor.addEventListener('input', updateUI);
  setInterval(clockTick, 1000);

  async function init() {
    let loaded = false;
    try {
      const cfgStr = await invoke('load_config');
      const cfg = JSON.parse(cfgStr);
      if (cfg.lang) { lang = cfg.lang; loaded = true; }
    } catch (e) {}
    if (!loaded) {
      try {
        const ls = JSON.parse(localStorage.getItem('nq-config') || '{}');
        if (ls.lang) {
          lang = ls.lang;
          invoke('save_config', { configJson: JSON.stringify(ls) }).catch(() => {});
        }
      } catch (e) {}
    }
    applyFontSize();
    updateUI();

    try {
      const pending = await invoke('read_pending_file');
      if (pending) {
        openOpenModal(pending[1], pending[0]);
      }
    } catch (e) {}
  }

  if (TAURI) {
    init();
  } else {
    lang = (navigator.language || '').startsWith('ru') ? 'ru' : 'en';
    applyFontSize();
    updateUI();
  }
})();
