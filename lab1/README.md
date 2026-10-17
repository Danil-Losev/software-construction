# Задание 1. Парсер файлов задач

Консольное приложение, которое разбирает файлы задач в форматах KRSU, CATS, EJUDGE и PCMS2 и отображает их в терминале.
Написано на Rust с использованием библиотеки ratatui и паттернов проектирования Strategy, Factory Method, Registry и Facade.


- ФИО: Лосев Данил Александрович
- Группа: ЕПИ-3-23
- Система сборки: Rust (cargo, если вручную собирать) Rust Rover (если собирать через IDE)
- ОС: Windows 11 Pro x64
- Аппаратная конфигурация: Intel Core i5-12400HX


Выполненные задания:
- [+] Разобор формата KRSU (testinfo.xml)
- [+] Разбор формата CATS (*.xml)
- [+] Разбор формата EJUDGE (problem.cfg)
- [+] Разбор формата PCMS2 (problem.xml)


---

## Сборка

```powershell
# В корне репозитория
cargo build --release
```

Бинарный файл появится по пути:
- Windows: `target\release\lab1.exe`
- Linux / macOS: `target/release/lab1`

---

## Запуск

```
lab1.exe <подкоманда> <путь_к_папке_задачи>
```

### Примеры

```powershell
# KRSU — папка должна содержать testinfo.xml
lab1.exe krsu .\KRSU\

# CATS — папка должна содержать *.xml файл задачи
lab1.exe cats .\CATS\

# EJUDGE — папка должна содержать problem.cfg
lab1.exe ejudge .\EJUDGE\sample

# PCMS2 — папка должна содержать problem.xml
lab1.exe pcms2 .\PCMS2\sample
lab1.exe pcms2 .\PCMS2\dungeon
```

---

## Архитектура проекта

```
src/
├── main.rs                  # Точка входа, регистрация систем
├── core/
│   ├── system.rs            # Трейт System — основа любого формата
│   ├── parser.rs            # Трейт Parser + тип ParserError
│   ├── ui.rs                # Трейт Ui
│   ├── terminal_ui.rs       # Общий UI-движок (ratatui)
│   ├── common.rs            # Общие структуры данных (лимиты и т.д.)
│   ├── pipeline.rs          # Связующий пайплайн parser → ui
│   └── registry.rs          # Глобальный реестр зарегистрированных систем
└── systems/
    ├── mod.rs               # Подключение модулей форматов
    ├── krsu/
    │   ├── mod.rs           # Структура KrsuSystem + impl System
    │   ├── problem.rs       # Структуры данных задачи KRSU
    │   ├── parser.rs        # Парсер testinfo.xml
    │   └── ui.rs            # UI-рендерер для KRSU
    ├── cats/    (аналогично)
    ├── ejudge/  (аналогично)
    └── pcms2/   (аналогично)
```

---

## Как добавить новый формат

### Шаг 1. Создайте папку модуля

```
src/systems/myformat/
├── mod.rs
├── problem.rs
├── parser.rs
└── ui.rs
```

### Шаг 2. Опишите структуру задачи — `problem.rs`

Создайте одну или несколько структур, описывающих данные задачи в вашем формате:

```rust
#[derive(Debug, Clone)]
pub struct MyFormatProblem {
    pub title: String,
    pub time_limit_ms: Option<u64>,
    pub memory_limit_bytes: Option<u64>,
    // … другие поля
}
```

### Шаг 3. Напишите парсер — `parser.rs`

Реализуйте трейт `Parser` из `crate::core::parser`:

```rust
use std::path::Path;
use crate::core::parser::{Parser, ParserError};
use super::problem::MyFormatProblem;

pub struct MyFormatParser;

impl Parser for MyFormatParser {
    type Output = MyFormatProblem;

    fn parse(&self, path: &Path) -> Result<Self::Output, ParserError> {
        // Найти нужный файл в `path`, разобрать его, вернуть MyFormatProblem
        todo!()
    }
}
```

### Шаг 4. Реализуйте UI — `ui.rs`

Реализуйте трейт `Ui` из `crate::core::ui` и постройте модель для общего UI-движка:

```rust
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::core::terminal_ui::{
    render_interactive_problem_view,
    TerminalUiProblemView,
    TerminalUiSection,
};
use crate::core::ui::Ui;
use super::problem::MyFormatProblem;

pub struct MyFormatUI;

impl Ui for MyFormatUI {
    type Input = MyFormatProblem;

    fn render(&self, problem: &Self::Input) {
        // 1. Статистика (отображается в шапке-блоке KEY METRICS)
        let mut summary_statistics = Vec::new();
        if let Some(tl) = problem.time_limit_ms {
            summary_statistics.push(Line::from(vec![
                Span::styled("  • Лимит времени:  ", Style::default().fg(Color::White)),
                Span::styled(format!("{} мс", tl), Style::default().fg(Color::Green)),
            ]));
        }

        // 2. Секции — блоки с подробными данными
        let mut sections = Vec::new();

        // Текстовая секция
        sections.push(TerminalUiSection {
            table: None,   // ← ОБЯЗАТЕЛЬНО указывать, None = только текст
            section_title: "Основная информация".to_string(),
            lines: vec![
                Line::from(format!("  Название: {}", problem.title)),
            ],
        });

        // Секция с таблицей (опционально)
        // sections.push(TerminalUiSection::new_table(
        //     "Тест-группы",
        //     vec![
        //         TerminalTableColumn { header: "ID".into(), width_ratio: 1, min_width: 4 },
        //         TerminalTableColumn { header: "Баллы".into(), width_ratio: 2, min_width: 6 },
        //     ],
        //     rows,
        // ));

        // 3. Собрать view и отрендерить
        let view = TerminalUiProblemView {
            format_display_name: "MYFORMAT v1.0".to_string(),
            problem_title: problem.title.clone(),
            summary_statistics,
            sections,
        };

        render_interactive_problem_view(&view);
    }
}
```

### Шаг 5. Зарегистрируйте систему — `mod.rs`

```rust
use crate::core::system::System;
pub mod parser;
pub mod problem;
pub mod ui;

pub struct MyFormatSystem;

impl System for MyFormatSystem {
    const NAME: &'static str = "myformat"; // это и есть подкоманда CLI

    type Problem = problem::MyFormatProblem;
    type Parser  = parser::MyFormatParser;
    type Ui      = ui::MyFormatUI;

    fn parser() -> Self::Parser { parser::MyFormatParser }
    fn ui()     -> Self::Ui     { ui::MyFormatUI }
}
```

### Шаг 6. Подключите модуль к `src/systems/mod.rs`

```rust
pub mod myformat;
```

### Шаг 7. Зарегистрируйте систему в `src/main.rs`

```rust
use lab1::systems::myformat;

// В функции main, рядом с остальными регистрациями:
registry::register_system::<myformat::MyFormatSystem>();
```

После этого подкоманда `myformat` будет работать автоматически через глобальный реестр.

---

## Причины такой архитектуры

Новый формат подключается как отдельный модуль и не требует переписывать общий
пайплайн приложения. Его парсер отвечает только за чтение и преобразование
собственного формата, а UI — только за отображение полученной модели. Общая
последовательность действий остаётся неизменной:

```text
аргументы CLI → реестр → parser.parse(path) → ui.render(problem)
```

Это даёт несколько важных преимуществ:

- **Изоляция изменений.** Ошибки или особенности XML/CFG одного формата не
  смешиваются с кодом остальных форматов.
- **Расширяемость.** Чтобы добавить формат, достаточно реализовать его модуль и
  зарегистрировать систему; существующие парсеры и UI изменять не нужно.
- **Повторное использование.** Все форматы используют единый интерфейс
  `Parser`, единый интерфейс `Ui`, общий пайплайн и общий терминальный UI-движок.
- **Единообразное поведение.** Обработка ошибок, запуск парсера и передача
  результата в интерфейс выполняются одинаково для всех систем.
- **Тестируемость.** Парсер, модель задачи и UI можно проверять независимо,
  поскольку между ними есть явные границы и типизированные контракты.


## Использованные паттерны проектирования

### Strategy (Стратегия)

Трейты `Parser` и `Ui` задают взаимозаменяемые алгоритмы:

- `KrsuParser`, `CatsParser`, `EjudgeParser` и `Pcms2Parser` по-разному разбирают
  входные файлы;
- `KrsuUI`, `CatsUI`, `EjudgeUI` и `Pcms2UI` по-разному представляют результат.

Пайплайн не знает деталей конкретного формата и работает с выбранными
реализациями через трейты. Поэтому алгоритм разбора или отображения можно
заменить, не меняя код запуска приложения.

### Factory Method (Фабричный метод)

Трейт `System` содержит методы `parser()` и `ui()`. Конкретная система
(`KrsuSystem`, `CatsSystem` и другие) сама предоставляет соответствующие
объекты. Пайплайн вызывает эти методы, а не создаёт конкретные типы напрямую.
Это отделяет общий код запуска от способов создания компонентов конкретного
формата.

### Registry (Реестр)

`core/registry.rs` хранит соответствие между именем подкоманды и функцией
запуска пайплайна. Вызов `register_system::<MyFormatSystem>()` добавляет новый
формат в реестр, после чего он находится по значению `System::NAME`.
Такой подход позволяет не добавлять в `main` отдельную ветку `match` для каждого
формата и уменьшает связанность между CLI и реализациями систем.

### Facade (Фасад)

Функция `pipeline::run` предоставляет простой единый вход для всей операции:
создать парсер, разобрать путь, создать UI и отобразить результат. Сложность
взаимодействия между `Parser`, `Ui` и обработкой ошибки скрыта за одним вызовом
из реестра.

### Adapter / нормализация данных

Каждый формат имеет собственную структуру задачи в `problem.rs`, потому что
входные схемы форматов различаются. При этом структуры форматов могут
формировать общий `CommonProblemSummary` из `core/common.rs`: ограничения,
потоки ввода-вывода и тесты приводятся к единой модели. Это играет роль
адаптера между внешней моделью конкретной контестной системы и общими
компонентами приложения.

## Реализация принципов SOLID

- **S.** Каждый модуль отвечает за свою задачу:
  `parser.rs` разбирает входные файлы, `problem.rs` хранит модель данных,
  `ui.rs` отвечает за отображение, `pipeline.rs` связывает парсер и UI,
  а `registry.rs` регистрирует доступные форматы.
- **O.** Новый формат добавляется отдельным модулем с
  реализациями `System`, `Parser` и `Ui`. Общий пайплайн и существующие форматы
  при этом изменять не нужно.
- **L.** Любой конкретный парсер или UI можно использовать
  через соответствующий трейт. `System` гарантирует, что парсер и UI работают с
  одной моделью задачи.
- **I.** Используются небольшие специализированные
  трейты: `Parser` содержит только `parse`, а `Ui` — только `render`.
- **D.** Общий код зависит от абстракций `System`,
  `Parser` и `Ui`, а не от конкретных форматов. Реализация подключается через
  регистрацию в реестре.
