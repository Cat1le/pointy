#![allow(clippy::derive_ord_xor_partial_ord)]

use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File, OpenOptions},
    hash::{Hash, Hasher},
    io::stdout,
    path::PathBuf,
    str::FromStr,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "~/.config/pointy/config.json";

#[derive(Serialize, Deserialize, Default, Hash)]
struct Config {
    tasks: Vec<Task>,
    rewards: Vec<Reward>,
    points: usize,
}

#[derive(PartialEq, Eq, Ord, Serialize, Deserialize, Hash)]
struct Task {
    title: String,
    reward: usize,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.reward.partial_cmp(&other.reward)
    }
}

#[derive(PartialEq, Eq, Ord, Serialize, Deserialize, Hash)]
struct Reward {
    title: String,
    price: usize,
}

impl PartialOrd for Reward {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

#[derive(Hash)]
enum State {
    Main {
        index: usize,
    },
    NewTask {
        op: usize,
        title: String,
        reward: String,
    },
    NewReward {
        op: usize,
        title: String,
        price: String,
    },
    SolveTask {
        index: usize,
    },
    TakeReward {
        index: usize,
    },
}

struct RestoreHandle;

impl Drop for RestoreHandle {
    fn drop(&mut self) {
        execute!(stdout(), Show).unwrap();
        disable_raw_mode().unwrap();
    }
}

fn main() -> anyhow::Result<()> {
    let mut conf = load_config().unwrap_or_default();
    let mut state = State::Main { index: 0 };
    enable_raw_mode().unwrap();
    let _handle = RestoreHandle;
    render(&conf, &state);
    loop {
        let e = event::read()?;
        if matches!(
            e,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
        ) {
            break;
        };
        match state {
            State::Main { ref mut index } => {
                if let Event::Key(key) = e {
                    match key.code {
                        KeyCode::Enter => {
                            state = match index {
                                0 => State::NewTask {
                                    op: 0,
                                    title: String::new(),
                                    reward: String::new(),
                                },
                                1 => State::NewReward {
                                    op: 0,
                                    title: String::new(),
                                    price: String::new(),
                                },
                                2 => State::SolveTask { index: 0 },
                                3 => State::TakeReward { index: 0 },
                                4 => {
                                    conf.points = 0;
                                    update_config(&conf).unwrap();
                                    state
                                }
                                _ => unreachable!(),
                            }
                        }
                        KeyCode::Up => {
                            *index = if *index == 0 { 4 } else { *index - 1 };
                        }
                        KeyCode::Down => {
                            *index = if *index == 4 { 0 } else { *index + 1 };
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
            State::NewTask {
                ref mut op,
                ref mut title,
                ref mut reward,
            } => match op {
                0 => match e {
                    Event::Key(key) => match key.code {
                        KeyCode::Enter => {
                            if !title.is_empty() {
                                *op += 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            title.push(c);
                        }
                        KeyCode::Backspace => {
                            title.pop();
                        }
                        KeyCode::Esc => state = State::Main { index: 0 },
                        _ => {}
                    },
                    Event::Paste(text) => title.push_str(&text),
                    _ => {}
                },
                1 => {
                    if let Event::Key(key) = e {
                        match key.code {
                            KeyCode::Enter => {
                                if !reward.is_empty() {
                                    *op += 1;
                                }
                            }
                            KeyCode::Char(c) => {
                                if c.is_ascii_digit() {
                                    reward.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                reward.pop();
                            }
                            KeyCode::Esc => state = State::Main { index: 0 },
                            _ => {}
                        }
                    }
                }
                2 => {
                    if let Event::Key(key) = e {
                        match key.code {
                            KeyCode::Enter => {
                                conf.tasks.push(Task {
                                    title: title.clone(),
                                    reward: reward.parse().unwrap(),
                                });
                                update_config(&conf).unwrap();
                                state = State::Main { index: 0 }
                            }
                            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                                'y' | 'д' => {
                                    conf.tasks.push(Task {
                                        title: title.clone(),
                                        reward: reward.parse().unwrap(),
                                    });
                                    update_config(&conf).unwrap();
                                    state = State::Main { index: 0 }
                                }
                                'n' | 'н' => state = State::Main { index: 0 },
                                _ => {}
                            },
                            KeyCode::Esc => state = State::Main { index: 0 },
                            _ => {}
                        }
                    }
                }
                _ => todo!(),
            },
            State::NewReward {
                ref mut op,
                ref mut title,
                ref mut price,
            } => match op {
                0 => match e {
                    Event::Key(key) => match key.code {
                        KeyCode::Enter => {
                            if !title.is_empty() {
                                *op += 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            title.push(c);
                        }
                        KeyCode::Backspace => {
                            title.pop();
                        }
                        KeyCode::Esc => state = State::Main { index: 0 },
                        _ => {}
                    },
                    Event::Paste(text) => title.push_str(&text),
                    _ => {}
                },
                1 => {
                    if let Event::Key(key) = e {
                        match key.code {
                            KeyCode::Enter => {
                                if !price.is_empty() {
                                    *op += 1;
                                }
                            }
                            KeyCode::Char(c) => {
                                if c.is_ascii_digit() {
                                    price.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                price.pop();
                            }
                            KeyCode::Esc => state = State::Main { index: 0 },
                            _ => {}
                        }
                    }
                }
                2 => {
                    if let Event::Key(key) = e {
                        match key.code {
                            KeyCode::Enter => {
                                conf.rewards.push(Reward {
                                    title: title.clone(),
                                    price: price.parse().unwrap(),
                                });
                                update_config(&conf).unwrap();
                                state = State::Main { index: 0 }
                            }
                            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                                'y' | 'д' => {
                                    conf.rewards.push(Reward {
                                        title: title.clone(),
                                        price: price.parse().unwrap(),
                                    });
                                    update_config(&conf).unwrap();
                                    state = State::Main { index: 0 }
                                }
                                'n' | 'н' => state = State::Main { index: 0 },
                                _ => {}
                            },
                            KeyCode::Esc => state = State::Main { index: 0 },
                            _ => {}
                        }
                    }
                }
                _ => todo!(),
            },
            State::SolveTask { ref mut index } => {
                if let Event::Key(key) = e {
                    match key.code {
                        KeyCode::Enter => {
                            if !conf.tasks.is_empty() {
                                let task = conf.tasks.remove(*index);
                                conf.points += task.reward;
                                update_config(&conf).unwrap();
                            }
                            state = State::Main { index: 0 }
                        }
                        KeyCode::Up => {
                            if !conf.tasks.is_empty() {
                                *index = if *index == 0 {
                                    conf.tasks.len() - 1
                                } else {
                                    *index - 1
                                };
                            }
                        }
                        KeyCode::Down => {
                            if !conf.tasks.is_empty() {
                                *index = if *index == conf.tasks.len() - 1 {
                                    0
                                } else {
                                    *index + 1
                                };
                            }
                        }
                        KeyCode::Delete => {
                            if !conf.tasks.is_empty() {
                                conf.tasks.remove(*index);
                                update_config(&conf).unwrap();
                            }
                        }
                        KeyCode::Esc => state = State::Main { index: 0 },
                        _ => {}
                    }
                }
            }
            State::TakeReward { ref mut index } => {
                conf.rewards.sort();
                let size = conf
                    .rewards
                    .iter()
                    .filter(|x| x.price <= conf.points)
                    .count();
                if let Event::Key(key) = e {
                    match key.code {
                        KeyCode::Enter => {
                            if size > 0 {
                                let reward = conf.rewards.remove(*index);
                                conf.points -= reward.price;
                                update_config(&conf).unwrap();
                            }
                            state = State::Main { index: 0 }
                        }
                        KeyCode::Up => {
                            if size > 0 {
                                *index = if *index == 0 { size - 1 } else { *index - 1 };
                            }
                        }
                        KeyCode::Down => {
                            if size > 0 {
                                *index = if *index == size - 1 { 0 } else { *index + 1 };
                            }
                        }
                        KeyCode::Delete => {
                            if size > 0 {
                                conf.rewards.remove(*index);
                                update_config(&conf).unwrap();
                            }
                        }
                        KeyCode::Esc => state = State::Main { index: 0 },
                        _ => {}
                    }
                }
            }
        }
        render(&conf, &state);
    }
    Ok(())
}

fn render(conf: &Config, state: &State) {
    static mut HASH: u64 = 0;
    let mut hasher = DefaultHasher::new();
    (conf, state).hash(&mut hasher);
    let hash = hasher.finish();
    unsafe {
        if HASH == hash {
            return;
        }
        HASH = hash;
    }
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    match state {
        State::Main {
            index: current_index,
        } => {
            execute!(
                stdout(),
                Hide,
                Print(format!(
                    "Welcome to pointy! You have {} points.\r\n\r\n",
                    conf.points
                ))
            )
            .unwrap();
            [
                "Add new task",
                "Add new reward",
                "Solve task",
                "Take reward",
                "Clear points",
            ]
            .into_iter()
            .enumerate()
            .for_each(|(index, row)| {
                let string = format!("[+] {row}\r\n");
                execute!(
                    stdout(),
                    Print(if index == *current_index {
                        string.blue()
                    } else {
                        string.reset()
                    })
                )
                .unwrap();
            });
        }
        State::NewTask { op, title, reward } => match *op {
            0 => {
                execute!(
                    stdout(),
                    Show,
                    Print("New task\r\n\r\n"),
                    Print("Title: ".blue()),
                    Print(title)
                )
                .unwrap();
            }
            1 => {
                execute!(
                    stdout(),
                    Show,
                    Print(format!("Task {title}\r\n\r\n")),
                    Print("Reward: ".blue()),
                    Print(reward.to_string())
                )
                .unwrap();
            }
            2 => {
                execute!(
                    stdout(),
                    Show,
                    Print("Almost done\r\n\r\n"),
                    Print("Title: ".blue()),
                    Print(title),
                    Print("\r\nReward: ".blue()),
                    Print(format!("{reward}\r\nCreate? [y/n] "))
                )
                .unwrap();
            }
            _ => unreachable!(),
        },
        State::NewReward { op, title, price } => match *op {
            0 => {
                execute!(
                    stdout(),
                    Show,
                    Print("New reward\r\n\r\n"),
                    Print("Title: ".blue()),
                    Print(title)
                )
                .unwrap();
            }
            1 => {
                execute!(
                    stdout(),
                    Show,
                    Print(format!("Reward {title}\r\n\r\n")),
                    Print("Price: ".blue()),
                    Print(price.to_string())
                )
                .unwrap();
            }
            2 => {
                execute!(
                    stdout(),
                    Show,
                    Print("Almost done\r\n\r\n"),
                    Print("Title: ".blue()),
                    Print(title),
                    Print("\r\nPrice: ".blue()),
                    Print(format!("{price}\r\nCreate? [y/n] "))
                )
                .unwrap();
            }
            _ => unreachable!(),
        },
        State::SolveTask {
            index: current_index,
        } => {
            execute!(
                stdout(),
                Hide,
                Print(format!(
                    "Currently you have {} tasks.\r\n\r\n",
                    conf.tasks.len()
                ))
            )
            .unwrap();
            conf.tasks.iter().enumerate().for_each(|(index, row)| {
                let string = format!("[{}] {}\r\n", row.reward, row.title);
                execute!(
                    stdout(),
                    Print(if index == *current_index {
                        string.blue()
                    } else {
                        string.reset()
                    })
                )
                .unwrap();
            });
        }
        State::TakeReward {
            index: current_index,
        } => {
            execute!(
                stdout(),
                Hide,
                Print(format!(
                    "Currently you have {} rewards.\r\n\r\n",
                    conf.rewards.len()
                ))
            )
            .unwrap();
            conf.rewards.iter().enumerate().for_each(|(index, row)| {
                let string = format!("[{}] {}\r\n", row.price, row.title);
                execute!(
                    stdout(),
                    Print(if row.price > conf.points {
                        string.dark_grey()
                    } else if index == *current_index {
                        string.blue()
                    } else {
                        string.reset()
                    })
                )
                .unwrap();
            });
        }
    }
}

fn load_config() -> anyhow::Result<Config> {
    let file = File::open(CONFIG_PATH)?;
    Ok(serde_json::from_reader(file)?)
}

fn update_config(conf: &Config) -> anyhow::Result<()> {
    fs::create_dir_all(PathBuf::from_str(CONFIG_PATH)?.parent().unwrap())?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(CONFIG_PATH)?;
    file.set_len(0).unwrap();
    Ok(serde_json::to_writer(file, conf)?)
}
