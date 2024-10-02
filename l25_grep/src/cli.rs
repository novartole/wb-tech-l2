use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// патерн для поиска
    pub pattern: String,

    /// искать паттерн в файле
    pub file: PathBuf,

    /// печатать +N строк после совпадения
    #[clap(short = 'A', default_value_t = 0)]
    pub after_context: u32,

    /// печатать +N строк до совпадения
    #[clap(short = 'B', default_value_t = 0)]
    pub before_context: u32,

    /// (A+B) печатать ±N строк вокруг совпадения
    #[clap(short = 'C', default_value_t = 0)]
    pub context: u32,

    /// печатать количество строк
    #[clap(short)]
    pub count: bool,

    /// игнорировать регистр
    #[clap(short)]
    pub ignore_case: bool,

    /// вместо совпадения, исключать
    #[clap(short = 'v')]
    pub invert_match: bool,

    /// точное совпадение со строкой, не паттерн
    #[clap(short = 'F')]
    pub fixed_string: bool,

    /// печатать номер строки
    #[clap(short = 'n')]
    pub line_number: bool,
}
