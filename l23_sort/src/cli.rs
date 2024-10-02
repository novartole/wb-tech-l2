pub use Sort as SortType;
use std::path::PathBuf;
use clap::{ArgAction, Parser, ValueEnum};

#[derive(Parser)]
#[clap(disable_help_flag = true)]
pub struct Cli {
    /// файл с несортированными строками
    pub file: PathBuf,

    /// указание колонки для сортировки
    #[clap(
        short, 
        default_value_t = 1, 
        value_parser = clap::value_parser!(u32).range(1..),
    )]
    pub key: u32,

    // тип сортировки
    #[clap(long, default_value = "string")]
    pub sort: Sort,

    // /// сортировать по числовому значению
    // #[clap(short = 'n')]
    // pub numeric_sort: bool,

    /// сортировать в обратном порядке
    #[clap(short)]
    pub reverse: bool,

    /// не выводить повторяющиеся строки
    #[clap(short)]
    pub unique: bool,

    // /// сортировать по названию месяца
    // #[clap(short = 'M')]
    // pub month_sort: bool,

    /// игнорировать хвостовые пробелы
    #[clap(short = 'b')]
    pub ignore_leading_blanks: bool,

    /// проверять отсортированы ли данные
    #[clap(short)]
    pub check: bool,

    // /// сортировать по числовому значению с учетом суффиксов
    // #[clap(short = 'h')]
    // pub human_numeric_sort: bool,

    /// показать это сообщение
    #[clap(long, action = ArgAction::Help)]
    help: Option<bool>,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum Sort {
    /// сортировать по числовому значению с учетом суффиксов
    Human,
    /// сортировать по названию месяца
    Month,
    /// сортировать по числовому значению
    Numeric,
    /// сортировать как строку
    String,
}
