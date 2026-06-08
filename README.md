# 🔐 NQTXT Secure Editor & Encryptor

<p align="center">
  <b>EN</b> · <a href="#ru">RU</a>
</p>

A text editor with **encryption**.

It can be used as a **text editor** with text encryption (Ctrl+S) and as an **encryptor** of any other files (F5, F6): texts, images, archives, music. Everything is saved in the `.nqtxt` format with password protection AES-256-GCM + Argon2id.

Encryption of any file (text or binary) is performed on **raw bytes** — no Base64 conversion, no intermediate text layer. Large files are processed in 4 MB chunks with parallel decryption/encryption across CPU cores (via `rayon`).

Rewritten from Python (Tkinter) to Rust (Tauri) using AI (OpenCode) for performance and a clean architecture. The original Python/Tkinter prototype and the Rust rewrite remain in the project history as a story of the evolution from prototype to native desktop app.

![image alt](https://github.com/niqonoroff/AES-encryptor/blob/main/screenshot.png?raw=true)

## ✨ Features

* **Universal encryption** — encrypt any file (F5), decrypt back to original format (F6)
* **AES-256-GCM** with random salt & nonce
* **Argon2id** key derivation (configurable time/memory/parallelism)
* **Raw-byte encryption** — no Base64 overhead, no intermediate text layer
* **Chunked mode** — 4 MB chunks, parallel processing across CPU cores via `rayon`
* **Dark and light themes** — soft sakura pink panels + warm beige editor in light mode (F3)
* **Language toggle** RU/EN (F2)
* **Fullscreen mode** (F1)
* **Built-in calculator** (F4) with safe expression parser (no `eval`)
* **Tray icon** — app minimizes to tray on close; click the tray icon to paste clipboard contents
* **Autostart to tray** — when enabled, app starts hidden in system tray at boot (no window)
* **Font zoom** `Ctrl + Scroll`

## 🗂 File format

```
NQ02 | salt 32B | nonce 12B | meta_len 4B (LE) | meta (utf8 JSON) | chunk1 || chunk2 || ...
```

Each chunk is `[u32 LE chunk_len][AES-GCM(plaintext_chunk, nonce_i, aad=i)]` where `nonce_i = base_nonce XOR chunk_index`.

Defaults: Argon2id `time=8, mem=512MB, parallel=4`.

The plaintext payload is the metadata JSON followed by the file data. Metadata is authenticated by the AEAD tag, so any tampering breaks decryption.

## 🔧 Build

```bash
cd src-tauri
cargo tauri build --no-bundle
```

Output: `src-tauri/target/release/nq-editor.exe`

## ⌨ Hotkeys

| Key | Action |
|---|---|
| `Ctrl+O` | Open `.nqtxt` |
| `Ctrl+S` | Save |
| `Ctrl+Q` | Close window |
| `F1` | Toggle fullscreen |
| `F2` | Toggle language |
| `F3` | Toggle theme |
| `F4` | Calculator |
| `F5` | Encrypt any file → `.nqtxt` |
| `F6` | Decrypt `.nqtxt` → original file |
| `Ctrl+Scroll` | Zoom editor font |

---

<h2 id="ru">🔐 NQTXT Secure Editor & Encryptor — RU</h2>

Текстовый редактор с **шифрованием**.

Может использоваться как **текстовый редактор** с шифрованием текста (Ctrl+S) и как **шифровальщик** любых других файлов (F5, F6): текстов, изображений, архивов, музыки. Всё сохраняется в формате `.nqtxt` с парольной защитой AES-256-GCM + Argon2id.

Шифрование любого файла (текст или бинарник) выполняется над **сырыми байтами** — без конвертации в Base64, без промежуточного текстового слоя. Большие файлы обрабатываются чанками по 4 МБ с параллельным шифрованием/дешифрованием по ядрам процессора (через `rayon`).

Переписан с Python (Tkinter) на Rust (Tauri) с помощью ИИ (OpenCode) ради скорости и чистой архитектуры. Оригинальный прототип на Python/Tkinter и его переписывание на Rust остаются в истории проекта как путь эволюции от прототипа к нативному десктопному приложению.

## ✨ Возможности

* **Универсальное шифрование** — зашифровать любой файл (F5), расшифровать обратно (F6)
* **AES-256-GCM** со случайной солью и nonce
* **Argon2id** (настраиваемые время/память/параллелизм)
* **Шифрование сырых байт** — без Base64, без промежуточного текстового слоя
* **Чанковый режим** — 4 МБ чанки, параллельная обработка по ядрам CPU через `rayon`
* **Тёмная и светлая темы** — нежные sakura pink панели + тёплый бежевый редактор в светлой (F3)
* **Переключение языка** RU/EN (F2)
* **Полноэкранный режим** (F1)
* **Встроенный калькулятор** (F4) с безопасным парсером выражений (без `eval`)
* **Иконка в трее** — приложение сворачивается в трей; клик по иконке вставляет буфер обмена
* **Автозагрузка в трей** — при включении автозапуска при старте системы окно скрыто, только иконка в трее
* **Масштабирование шрифта** `Ctrl + Scroll`

## 🗂 Формат файла

```
NQ02 | salt 32B | nonce 12B | meta_len 4B (LE) | meta (utf8 JSON) | chunk1 || chunk2 || ...
```

Каждый чанк — `[u32 LE chunk_len][AES-GCM(plaintext_chunk, nonce_i, aad=i)]`, где `nonce_i = base_nonce XOR chunk_index`.

По умолчанию: Argon2id `time=8, mem=512MB, parallel=4`.

Полезная нагрузка шифрования — JSON метаданных, за которым идут байты файла. Метаданные защищены AEAD-тегом, поэтому любая подмена ломает дешифровку.

## 🔧 Сборка

```bash
cd src-tauri
cargo tauri build --no-bundle
```

Результат: `src-tauri/target/release/nq-editor.exe`

## ⌨ Горячие клавиши

| Клавиша | Действие |
|---|---|
| `Ctrl+O` | Открыть `.nqtxt` |
| `Ctrl+S` | Сохранить |
| `Ctrl+Q` | Закрыть окно |
| `F1` | Полный экран |
| `F2` | Язык |
| `F3` | Тема |
| `F4` | Калькулятор |
| `F5` | Зашифровать файл → `.nqtxt` |
| `F6` | Расшифровать `.nqtxt` → оригинал |
| `Ctrl+Scroll` | Масштаб шрифта |
