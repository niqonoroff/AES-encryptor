# 🔐 NQTXT Secure Editor & Encryptor

<p align="center">
  <b>EN</b> · <a href="#ru">RU</a>
</p>

A text editor with **encryption**.

It can be used as a **text editor** with text encryption (Ctrl+S) and as an **encryptor** of any other files (F5, F6): texts, images, archives, music. Everything is saved in the `.nqtxt` format with password protection AES-256-GCM + Argon2id.

Encryption of other binary files (not text) occurs along the chain: Binary -> Base64 -> `.nqtxt`

Rewritten from Python (Tkinter) to Rust (Tauri) using AI (OpenCode) with backward compatibility. This made the application fast and smooth for processing large amounts of text (important for Base64 strings).

![image alt](https://github.com/niqonoroff/aes-secure-text-editor/blob/main/screenshot.png?raw=true)

## ✨ Features

* **Universal encryption** — encrypt any file (F5), decrypt back to original format (F6)
* **AES-256-GCM** with random salt & nonce
* **Argon2id** key derivation (configurable time/memory/parallelism)
* **Dark theme** editor
* **Language toggle** RU/EN (F1)
* **Fullscreen mode** (F2)
* **Built‑in calculator** (F3)
* **Font zoom** `Ctrl + Scroll`

## 🗂 File format

```
NQ01 | salt 32B | nonce 12B | AES-256-GCM ciphertext
```

Defaults: Argon2id `time=8, mem=512MB, parallel=4`.

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
| `F1` | Toggle language |
| `F2` | Toggle fullscreen |
| `F3` | Calculator |
| `F5` | Encrypt any file → `.nqtxt` |
| `F6` | Decrypt `.nqtxt` → original file |
| `Ctrl+Scroll` | Zoom editor font |

---

<h2 id="ru">🔐 NQTXT Secure Editor & Encryptor — RU</h2>

Текстовый редактор с **шифрованием**.

Может использоваться как **текстовый редактор** с шифрованием текста (Ctrl+S) и как **шифровальщик** любых других файлов (F5, F6): текстов, изображений, архивов, музыки. Всё сохраняется в формате `.nqtxt` с парольной защитой AES-256-GCM + Argon2id.

Шифрование других бинарных файлов (не текста) происходит по цепочке: Бинарник -> Base64 -> `.nqtxt`

Переписан с Python (Tkinter) на Rust (Tauri) с помощью ИИ (OpenCode) с обратной совместимостью. Это позволило сделать приложение быстрым и плавным для обработки большого количества текста (важно для Base64 строк).

## ✨ Возможности

* **Универсальное шифрование** — зашифровать любой файл (F5), расшифровать обратно (F6)
* **AES-256-GCM** со случайной солью и nonce
* **Argon2id** (настраиваемые время/память/параллелизм)
* **Тёмная тема**
* **Переключение языка** RU/EN (F1)
* **Полноэкранный режим** (F2)
* **Встроенный калькулятор** (F3)
* **Масштабирование шрифта** `Ctrl + Scroll`

## 🗂 Формат файла

```
NQ01 | salt 32B | nonce 12B | AES-256-GCM ciphertext
```

По умолчанию: Argon2id `time=8, mem=512MB, parallel=4`.

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
| `F1` | Сменить язык |
| `F2` | Полный экран |
| `F3` | Калькулятор |
| `F5` | Зашифровать файл → `.nqtxt` |
| `F6` | Расшифровать `.nqtxt` → оригинал |
| `Ctrl+Scroll` | Масштаб шрифта |
