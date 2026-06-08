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
      notATextFile: 'Файл не является текстовым (используйте F6 для расшифровки)',
      errorRead: 'Ошибка чтения файла',
      errorSave: 'Ошибка сохранения: ',
      version: 'v2.0.0',
      shortcuts: '| Ctrl+O — Открыть | Ctrl+S — Сохранить | Ctrl+Q — Закрыть | F1 — Экран | F2 — Язык | F3 — Тема | F4 — Калькулятор | F5 — Зашифровать файл | F6 — Расшифровать файл',
      nqEditor: 'NQ Editor',
      saved: 'Сохранено',
      decrypted: 'Расшифровано',
      closeTitle: 'Несохранённые изменения',
      closeMessage: 'Закрыть программу без сохранения?',
      discMessage: 'Открыть буфер обмена без сохранения текущего документа?',
      cancel: 'Отмена',
      closeApp: 'Закрыть',
      isTextFileDecrypt: 'Это текстовый файл. Используйте Ctrl+O (Ctrl+S для сохранения) для работы с ним, а не F6.'
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
      notATextFile: 'File is not text (use F6 to decrypt)',
      errorRead: 'Error reading file',
      errorSave: 'Error saving file: ',
      version: 'v2.0.0',
      shortcuts: '| Ctrl+O — Open | Ctrl+S — Save | Ctrl+Q — Close | F1 — Fullscreen | F2 — Lang | F3 — Theme | F4 — Calc | F5 — Encrypt file | F6 — Decrypt file',
      nqEditor: 'NQ Editor',
      saved: 'Saved',
      decrypted: 'Decrypted',
      closeTitle: 'Unsaved changes',
      closeMessage: 'Close without saving?',
      discMessage: 'Open buffer without saving current document?',
      cancel: 'Cancel',
      closeApp: 'Close',
      isTextFileDecrypt: 'This is a text file. Use Ctrl+O (Ctrl+S to save) to edit it, not F6.'
    }
  };

  const DEFAULT = {
    argon_time: 8,
    argon_memory: 512 * 1024,
    argon_parallel: 4
  };

  const editor = document.getElementById('editor');
  const titleText = document.getElementById('title-text');
  const modifiedIcon = document.getElementById('modified-icon');
  const shortcutsText = document.getElementById('shortcuts-text');
  const clock = document.getElementById('clock');
  const themeIcon = document.getElementById('theme-icon');
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
  let theme = 'dark';
  let currentFile = null;
  let savedText = '';
  let fontSize = 16;
  let calcOpen = false;
  let calcExpr = '';
  let pendingOpenPath = null;
  let pendingInput = null;
  let openMode = 'text';

  const TAURI = window.__TAURI__;
  const INV = TAURI && TAURI.core ? TAURI.core : TAURI;

  async function invoke(cmd, args) {
    if (!INV || !INV.invoke) throw new Error('Tauri IPC not available');
    return await INV.invoke(cmd, args);
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

    themeIcon.textContent = theme === 'dark' ? '☾' : '☀';

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

  function applyTheme() {
    document.documentElement.dataset.theme = theme;
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
    const tokens = [];
    let i = 0;
    while (i < expr.length) {
      const c = expr[i];
      if (c === ' ') { i++; continue; }
      if (/[0-9.]/.test(c)) {
        let j = i;
        let dots = 0;
        while (j < expr.length && /[0-9.]/.test(expr[j])) {
          if (expr[j] === '.') dots++;
          j++;
        }
        if (dots > 1) return 'Error';
        const num = parseFloat(expr.slice(i, j));
        if (isNaN(num)) return 'Error';
        tokens.push({ type: 'num', value: num });
        i = j;
      } else if ('+-*/%()'.includes(c)) {
        tokens.push({ type: 'op', value: c });
        i++;
      } else {
        return 'Error';
      }
    }

    const prec = { '+': 1, '-': 1, '*': 2, '/': 2, '%': 2 };
    const output = [];
    const ops = [];
    for (const tok of tokens) {
      if (tok.type === 'num') {
        output.push(tok);
      } else if (tok.value === '(') {
        ops.push(tok);
      } else if (tok.value === ')') {
        while (ops.length && ops[ops.length - 1].value !== '(') {
          output.push(ops.pop());
        }
        if (!ops.length) return 'Error';
        ops.pop();
      } else {
        while (ops.length && ops[ops.length - 1].value !== '('
               && prec[ops[ops.length - 1].value] >= prec[tok.value]) {
          output.push(ops.pop());
        }
        ops.push(tok);
      }
    }
    while (ops.length) {
      const op = ops.pop();
      if (op.value === '(' || op.value === ')') return 'Error';
      output.push(op);
    }

    const stack = [];
    for (const tok of output) {
      if (tok.type === 'num') {
        stack.push(tok.value);
      } else {
        if (stack.length < 2) return 'Error';
        const b = stack.pop();
        const a = stack.pop();
        let r;
        switch (tok.value) {
          case '+': r = a + b; break;
          case '-': r = a - b; break;
          case '*': r = a * b; break;
          case '/': r = a / b; break;
          case '%': r = a % b; break;
        }
        if (!isFinite(r)) return 'Error';
        stack.push(r);
      }
    }
    if (stack.length !== 1) return 'Error';
    return String(stack[0]);
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

  function openOpenModal() {
    document.getElementById('open-password').value = '';
    document.getElementById('open-time').value = String(DEFAULT.argon_time);
    document.getElementById('open-mem').value = String(DEFAULT.argon_memory);
    document.getElementById('open-par').value = String(DEFAULT.argon_parallel);
    document.getElementById('open-salt').value = '32';
    document.getElementById('open-nonce').value = '12';
    openModal.classList.remove('hidden');
    document.getElementById('open-password').focus();
  }

  function openSaveModal() {
    document.getElementById('save-password').value = '';
    document.getElementById('save-confirm').value = '';
    document.getElementById('save-time').value = String(DEFAULT.argon_time);
    document.getElementById('save-mem').value = String(DEFAULT.argon_memory);
    document.getElementById('save-par').value = String(DEFAULT.argon_parallel);
    document.getElementById('save-salt').value = '32';
    document.getElementById('save-nonce').value = '12';
    saveModal.classList.remove('hidden');
    document.getElementById('save-password').focus();
  }

  function closeModals() {
    openModal.classList.add('hidden');
    saveModal.classList.add('hidden');
    calcModal.classList.add('hidden');
    calcOpen = false;
  }

  let closeConfirmOpen = false;

  function showCloseConfirm(msgKey, titleKey) {
    if (closeConfirmOpen) return Promise.resolve(false);
    closeConfirmOpen = true;
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'modal';
      overlay.innerHTML =
        '<div class="modal-content pw-content">' +
          '<div class="pw-title"></div>' +
          '<hr class="pw-sep">' +
          '<div class="close-msg"></div>' +
          '<div class="close-buttons">' +
            '<button class="calc-eq close-btn-cancel"></button>' +
            '<button class="calc-eq close-btn-ok"></button>' +
          '</div>' +
        '</div>';

      const title = overlay.querySelector('.pw-title');
      const msg = overlay.querySelector('.close-msg');
      const cancelBtn = overlay.querySelector('.close-btn-cancel');
      const okBtn = overlay.querySelector('.close-btn-ok');
      title.textContent = t(titleKey || 'closeTitle');
      msg.textContent = t(msgKey || 'closeMessage');
      cancelBtn.textContent = t('cancel');
      okBtn.textContent = t('closeApp');

      document.body.appendChild(overlay);

      let resolved = false;
      const cleanup = (result) => {
        if (resolved) return;
        resolved = true;
        closeConfirmOpen = false;
        document.removeEventListener('keydown', onKey, true);
        overlay.remove();
        resolve(result);
      };

      cancelBtn.onclick = () => cleanup(false);
      okBtn.onclick = () => cleanup(true);
      overlay.addEventListener('mousedown', (e) => {
        if (e.target === overlay) cleanup(false);
      });

      const onKey = (e) => {
        if (e.code === 'Escape') { e.preventDefault(); e.stopPropagation(); cleanup(false); }
        else if (e.code === 'Enter') { e.preventDefault(); e.stopPropagation(); cleanup(true); }
      };
      document.addEventListener('keydown', onKey, true);

      setTimeout(() => cancelBtn.focus(), 0);
    });
  }

  function readKdfParams(prefix) {
    return {
      time: parseInt(document.getElementById(prefix + '-time').value) || DEFAULT.argon_time,
      memory: parseInt(document.getElementById(prefix + '-mem').value) || DEFAULT.argon_memory,
      parallel: parseInt(document.getElementById(prefix + '-par').value) || DEFAULT.argon_parallel
    };
  }

  async function submitOpen() {
    if (!pendingOpenPath) return;
    const password = document.getElementById('open-password').value;
    const { time, memory, parallel } = readKdfParams('open');

    showLoading();
    await new Promise(r => requestAnimationFrame(r));
    try {
      if (openMode === 'binary') {
        const result = await invoke('decrypt_file_cmd', {
          input: pendingOpenPath, password,
          argonTime: time, argonMemory: memory, argonParallel: parallel
        });
        currentFile = null;
        savedText = '';
        editor.value = t('decrypted') + ': ' + result.outputPath;
      } else {
        const text = await invoke('open_text', {
          path: pendingOpenPath, password,
          argonTime: time, argonMemory: memory, argonParallel: parallel
        });
        currentFile = pendingOpenPath;
        editor.value = text;
        savedText = text;
      }
      hideLoading();
      openModal.classList.add('hidden');
      updateUI();
    } catch (e) {
      hideLoading();
      const msg = String(e || '');
      if (msg === 'TEXT_FILE') {
        editor.value = t('isTextFileDecrypt');
      } else if (msg.includes('not a text') || msg.includes('not a text document')) {
        editor.value = t('notATextFile');
      } else {
        editor.value = t('invalidPassword');
      }
      savedText = editor.value;
      currentFile = null;
      openModal.classList.add('hidden');
      updateUI();
    }
    pendingOpenPath = null;
  }

  async function submitSave() {
    const pw = document.getElementById('save-password').value;
    const confirm = document.getElementById('save-confirm').value;
    if (!pw || pw !== confirm) {
      document.getElementById('save-confirm').value = '';
      return;
    }
    const { time, memory, parallel } = readKdfParams('save');

    showLoading();
    await new Promise(r => requestAnimationFrame(r));
    try {
      if (openMode === 'binary' && pendingInput) {
        const base = fileName(pendingInput);
        const dot = base.lastIndexOf('.');
        const stem = dot > 0 ? base.slice(0, dot) : base;
        const out = await invoke('pick_save_nqtxt', { suggested: stem + '.nqtxt' });
        if (!out) { hideLoading(); pendingInput = null; return; }
        await invoke('encrypt_file_cmd', {
          input: pendingInput, output: out, password: pw,
          argonTime: time, argonMemory: memory, argonParallel: parallel
        });
        currentFile = out;
        editor.value = t('saved') + ': ' + out;
        savedText = editor.value;
        pendingInput = null;
      } else {
        let out = currentFile;
        if (!out) {
          out = await invoke('pick_save_nqtxt', { suggested: 'untitled.nqtxt' });
          if (!out) { hideLoading(); return; }
        }
        await invoke('save_text', {
          path: out, text: editor.value, password: pw,
          argonTime: time, argonMemory: memory, argonParallel: parallel
        });
        currentFile = out;
        editor.value = t('saved') + ': ' + out;
        savedText = editor.value;
      }
      hideLoading();
      saveModal.classList.add('hidden');
      updateUI();
    } catch (e) {
      hideLoading();
      editor.value = t('errorSave') + ' ' + (e?.toString() || '');
      savedText = editor.value;
      updateUI();
    }
  }

  async function openFile() {
    try {
      const path = await invoke('pick_nqtxt');
      if (!path) return;
      pendingOpenPath = path;
      openMode = 'text';
      openOpenModal();
    } catch (e) {
      editor.value = t('errorRead') + ' ' + (e?.toString() || '');
      savedText = editor.value;
      updateUI();
    }
  }

  async function saveFile() {
    if (!currentFile) {
      try {
        const path = await invoke('pick_save_nqtxt', { suggested: 'untitled.nqtxt' });
        if (!path) return;
        currentFile = path;
      } catch (e) {
        editor.value = 'SAVE ERROR: ' + (e?.toString() || 'unknown');
        savedText = editor.value;
        updateUI();
        return;
      }
    }
    openMode = 'text';
    openSaveModal();
  }

  async function closeWindow() {
    try { await invoke('close_window'); } catch (e) {}
  }

  async function toggleFullscreen() {
    try { await invoke('toggle_fullscreen'); } catch (e) {}
  }

  async function toggleLang() {
    lang = lang === 'ru' ? 'en' : 'ru';
    try {
      const cfg = JSON.parse(await invoke('load_config'));
      cfg.lang = lang;
      await invoke('save_config', { configJson: JSON.stringify(cfg) });
    } catch (e) {}
    try { await invoke('update_tray_lang', { lang }); } catch (e) {}
    updateUI();
  }

  async function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    applyTheme();
    try {
      const cfg = JSON.parse(await invoke('load_config'));
      cfg.theme = theme;
      await invoke('save_config', { configJson: JSON.stringify(cfg) });
    } catch (e) {}
    updateUI();
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

  async function encryptFile() {
    try {
      const path = await invoke('pick_any_file');
      if (!path) return;
      pendingInput = path;
      openMode = 'binary';
      openSaveModal();
    } catch (e) {
      editor.value = t('errorRead') + ' ' + (e?.toString() || '');
      savedText = editor.value;
      updateUI();
    }
  }

  async function decryptFile() {
    try {
      const path = await invoke('pick_nqtxt');
      if (!path) return;
      pendingOpenPath = path;
      openMode = 'binary';
      openOpenModal();
    } catch (e) {
      editor.value = t('errorRead') + ' ' + (e?.toString() || '');
      savedText = editor.value;
      updateUI();
    }
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

    if (e.code === 'F4') {
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
        if (/^[0-9.+\-*/()%]$/.test(e.key)) { calcHandleInput(e.key); e.preventDefault(); }
        else if (e.key === 'Backspace') { calcHandleInput('<-'); e.preventDefault(); }
      }
      return;
    }

    if (e.ctrlKey && e.code === 'KeyO') { e.preventDefault(); openFile(); return; }
    if (e.ctrlKey && e.code === 'KeyS') { e.preventDefault(); saveFile(); return; }
    if (e.ctrlKey && e.code === 'KeyQ') { e.preventDefault(); closeWindow(); return; }
    if (e.code === 'F1') { e.preventDefault(); toggleFullscreen(); return; }
    if (e.code === 'F2') { e.preventDefault(); toggleLang(); return; }
    if (e.code === 'F3') { e.preventDefault(); toggleTheme(); return; }
    if (e.code === 'F5') { e.preventDefault(); encryptFile(); return; }
    if (e.code === 'F6') { e.preventDefault(); decryptFile(); return; }
  });

  themeIcon.addEventListener('click', toggleTheme);

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
    let cfg = {};
    try {
      cfg = JSON.parse(await invoke('load_config'));
    } catch (e) {}
    if (cfg.lang) lang = cfg.lang;
    if (cfg.theme) theme = cfg.theme;
    applyTheme();
    applyFontSize();
    updateUI();
    try { await invoke('update_tray_lang', { lang }); } catch (e) {}

    try {
      const win = window.__TAURI__ && window.__TAURI__.window
        ? window.__TAURI__.window.getCurrentWindow()
        : null;
      if (!win) return;

      if (typeof win.onCloseRequested === 'function') {
        await win.onCloseRequested(async (event) => {
          event.preventDefault();
          if (editor.value !== savedText) {
            const ok = await showCloseConfirm();
            if (!ok) return;
          }
          await win.hide();
        });
      }

      if (typeof win.onDragDropEvent === 'function') {
        await win.onDragDropEvent((event) => {
          const payload = event.payload;
          if (payload.type === 'over') {
            editor.classList.add('drag-over');
          } else if (payload.type === 'leave') {
            editor.classList.remove('drag-over');
          } else if (payload.type === 'drop') {
            editor.classList.remove('drag-over');
            const path = payload.paths[0];
            if (!path) return;
            if (path.toLowerCase().endsWith('.nqtxt')) {
              pendingOpenPath = path;
              openMode = 'text';
              openOpenModal();
            } else {
              pendingInput = path;
              openMode = 'binary';
              openSaveModal();
            }
          }
        });
      }

      if (typeof win.listen === 'function') {
        await win.listen('encrypt-progress', (event) => {
          const { status, progress } = event.payload;
          if (status === 'indeterminate') {
            win.setProgressBar({ status: 'indeterminate' });
          } else if (status === 'normal') {
            win.setProgressBar({ status: 'normal', progress });
          } else {
            win.setProgressBar({ status: 'none' });
          }
        });

        await win.listen('paste-clipboard', async () => {
          if (editor.value !== '') {
            if (!await showCloseConfirm('discMessage')) {
              return;
            }
            editor.value = '';
            savedText = '';
            currentFile = null;
            updateUI();
          }
          try {
            const text = await navigator.clipboard.readText();
            if (text) {
              const start = editor.selectionStart;
              const end = editor.selectionEnd;
              editor.value = editor.value.substring(0, start) + text + editor.value.substring(end);
              editor.selectionStart = editor.selectionEnd = start + text.length;
              updateUI();
            }
          } catch (e) {}
        });

        await win.listen('open-file', (event) => {
          const path = event.payload;
          if (!path) return;
          if (path.toLowerCase().endsWith('.nqtxt')) {
            pendingOpenPath = path;
            openMode = 'text';
            openOpenModal();
          }
        });
      }
    } catch (e) {}

    try {
      const pending = await invoke('read_pending_file');
      if (pending) {
        pendingOpenPath = pending;
        openMode = 'text';
        openOpenModal();
      }
    } catch (e) {}
  }

  if (TAURI) {
    init();
  } else {
    lang = (navigator.language || '').startsWith('ru') ? 'ru' : 'en';
    applyTheme();
    applyFontSize();
    updateUI();
  }
})();
