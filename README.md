# Simple JSON & XML Comparator

<img width="1919" height="1039" alt="image" src="https://github.com/user-attachments/assets/c245544d-3efb-46d6-b6b1-f20e605b2fe2" />


[English](#english)

## Возможности

В каждое поле можно вставить текст или открыть файл `.json` / `.xml` и очистить его.

- `Beautify` форматирует документ.
- `Исправить*` пытается поправить типичные ошибки синтаксиса и сообщает, что именно изменила.
- `Убрать экранирование` заменяет `\"` на кавычки и раскрывает строки, внутри которых лежит объект или массив.

`Сравнение` проверяет оба текста. Если поле пустое, форматы различаются или текст некорректен, сравнение не начинается. Подсвечиваются удалённые, добавленные и изменённые строки; внутри изменённой строки — отличающийся фрагмент. Поля можно поменять местами. `Отчёт HTML` сохраняет этот результат в файл. Пока сравнения не было, отчёт не создаётся.

Сравнение идёт по тексту, который программа заново собирает из разобранного документа. Пробелы и переносы исходника на результат не влияют. Ключи объектов JSON и атрибуты XML упорядочиваются всегда, поэтому их порядок сам по себе не отличие. `Упорядочить структуру` дополнительно сортирует элементы массивов JSON и дочерние элементы XML. Комментарии XML и объявление `<?xml …?>` в сравнение не входят: XML собирается из элементов.

`Только отличия` оставляет строки с отличиями и по три строки вокруг, остальные схлопывает. Можно показать номера строк и отдельно скрыть удалённые или добавленные строки.

Файл больше 16 МБ не открывается. Документ длиннее 8 МБ (8 × 1024 × 1024 байт) не форматируется, не исправляется и не сравнивается.

## Запуск

Откройте `SimpleJsonXmlComparator-1.0.0.exe`.

Рядом с exe лежит `sjxc-lang.txt`. В файле одна строка — код языка интерфейса: `ru`, `en` или `zh`. Если файла нет или код неизвестен, интерфейс русский. Смена языка в программе перезаписывает этот файл. В репозитории копия лежит в `code\sjxc-lang.txt`.

Если операция срывается, программа пишет журнал `sjxc-crash-log-….log` в ту же папку, что и exe.

## Сборка

Нужны Rust и Cargo. Команды выполняются из каталога `code`:

```powershell
cargo build
cargo build --release
```

Отладочный файл: `code\target\debug\SimpleJsonXmlComparator.exe`.
Релизный файл: `code\target\release\SimpleJsonXmlComparator.exe`.

Cargo называет файл по цели `SimpleJsonXmlComparator` из `code\Cargo.toml`. Файл, которым пользуются, называется `SimpleJsonXmlComparator-1.0.0.exe`: номер берётся из поля `version` того же файла. Рядом положите `sjxc-lang.txt`.

## Состав репозитория

- `code\` — исходники программы и `sjxc-lang.txt`.
- `generators\` — `gen-json.bat`, `gen-xml.bat` и `gen-json-escaped.bat`. Каждый сценарий пишет в свою папку пару примеров с текущей датой в имени файла.

## Лицензия

GNU Affero General Public License, версия 3. Текст — в [LICENSE](LICENSE).

## English

[Русский](#simple-json--xml-comparator)

### Features

You can paste text into either field, open a `.json` / `.xml` file, and clear the field.

- `Beautify` formats the document.
- `Repair*` tries to fix typical syntax errors and reports what it changed.
- `Unescape` replaces `\"` with quotes and unwraps strings that contain an object or an array.

`Compare` checks both texts. It does not start when a field is empty, the formats differ, or the text is invalid. Removed, added, and changed lines are highlighted; inside a changed line, the differing fragment is highlighted too. The fields can be swapped. `HTML report` saves that result to a file. Until a comparison has been run, no report is created.

Comparison uses text the program rebuilds from the parsed document. Spaces and line breaks in the source do not affect the result. JSON object keys and XML attributes are always ordered, so their order alone is not a difference. `Order structure` also sorts JSON array items and XML child elements. XML comments and the `<?xml …?>` declaration are left out of the comparison: XML is rebuilt from elements.

`Only differences` keeps the lines that differ and three lines around them, and collapses the rest. Line numbers can be shown, and removed or added lines can be hidden on their own.

A file larger than 16 MB is not opened. A document longer than 8 MB (8 × 1024 × 1024 bytes) is not formatted, repaired, or compared.

### Run

Open `SimpleJsonXmlComparator-1.0.0.exe`.

`sjxc-lang.txt` sits next to the exe. The file is one line: the interface language code, `ru`, `en`, or `zh`. If the file is missing or the code is unknown, the interface is Russian. Changing the language in the program overwrites this file. The copy in the repository is `code\sjxc-lang.txt`.

If an operation fails, the program writes a log named `sjxc-crash-log-….log` in the same folder as the exe.

### Build

Rust and Cargo are required. Run the commands from the `code` directory:

```powershell
cargo build
cargo build --release
```

Debug file: `code\target\debug\SimpleJsonXmlComparator.exe`.
Release file: `code\target\release\SimpleJsonXmlComparator.exe`.

Cargo names the file after the `SimpleJsonXmlComparator` target in `code\Cargo.toml`. The file you run is `SimpleJsonXmlComparator-1.0.0.exe`: the number comes from the `version` field in that same file. Put `sjxc-lang.txt` next to it.

### Repository

- `code\` — program sources and `sjxc-lang.txt`.
- `generators\` — `gen-json.bat`, `gen-xml.bat`, and `gen-json-escaped.bat`. Each script writes a pair of samples into its own folder, with the current date in the file name.

### License

GNU Affero General Public License, version 3. The text is in [LICENSE](LICENSE).

