#[macro_use]
extern crate lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let guards = get_guards(&input)?;
    writeln!(
        io::stdout(),
        "product of sleepiest guard id and most frequent minute asleep: {}",
        find_sleepiest_guard_minute_product(&guards)?
    )?;

    writeln!(
        io::stdout(),
        "product of most frequent minute a guard is asleep, and guard id: {}",
        find_guard_minute_most_frequently_asleep(&guards)?
    )?;
    Ok(())
}

fn get_guards(input: &str) -> Result<Vec<Guard>> {
    // parse into Events:
    let mut events: Vec<Event> = vec![];
    for line in input.lines() {
        events.push(line.parse()?);
    }
    // sort the events
    events.sort_by(|ev1, ev2| ev1.timestamp.cmp(&ev2.timestamp));

    // group the events by guard
    let mut grouped_events: HashMap<GuardId, Vec<Event>> = HashMap::new();
    let mut cur_guard_id = None;
    for ev in events {
        if let EventKind::GuardStart { guard_id } = ev.kind {
            cur_guard_id = Some(guard_id);
        } else {
            match cur_guard_id {
                // TODO: replace these panics with proper error handling
                None => panic!("GuardStart event has no guard_id"),
                Some(guard_id) => grouped_events.entry(guard_id).or_default().push(ev),
            }
        }
    }

    // iterate over all the events for each guard, to get populated Guards with minutes
    let guards: Vec<Guard> = grouped_events
        .iter()
        .map(|(&guard_id, events)| Guard {
            id: guard_id,
            sleeps: get_sleep_schedule(events),
        })
        .collect();
    Ok(guards)
}

// part 1
fn find_sleepiest_guard_minute_product(guards: &Vec<Guard>) -> Result<u32> {
    // Find the guard who sleeps the most, and return his sleepiest minute.
    let sleepiest_guard = guards
        .iter()
        .max_by_key(|guard| -> u32 { guard.sleeps.iter().sum() })
        .expect("no guards!");

    let (sleepiest_minute, sleepiest_freq) = sleepiest_guard
        .sleeps
        .iter()
        .enumerate()
        .max_by_key(|(i, freq)| -> u32 { **freq })
        .expect("no minutes?!");

    // TODO: cast a usize into a u32?
    Ok(sleepiest_guard.id * (sleepiest_minute as u32))
}

// part 2
fn find_guard_minute_most_frequently_asleep(guards: &Vec<Guard>) -> Result<u32> {
    let (guard, (sleepiest_minute, _)) = guards
        .iter()
        .map(|guard| -> (&Guard, (usize, u32)) {
            // get the most freq min asleep for this guard:
            let (sleepiest_minute, freq) = guard
                .sleeps
                .iter()
                .enumerate()
                .max_by_key(|(i, freq)| -> u32 { **freq })
                .expect("unable to find the most frequent minute asleep!");
            (guard, (sleepiest_minute, *freq))
        })
        .max_by_key(|(_, (_, freq))| -> u32 {
            // get the guard with the highest minute frequency of being asleep
            *freq
        })
        .expect("unable to find a guard with minutes most frequently asleep!");
    Ok(guard.id * (sleepiest_minute as u32))
}

type SleepSchedule = [u32; 60];

fn get_sleep_schedule(events: &Vec<Event>) -> SleepSchedule {
    let mut schedule = [0; 60];
    let mut iter = events.iter();
    loop {
        match (iter.next(), iter.next()) {
            (
                Some(Event {
                    kind: EventKind::Asleep,
                    timestamp: sleep_time,
                    ..
                }),
                Some(Event {
                    kind: EventKind::Wakeup,
                    timestamp: wake_time,
                    ..
                }),
            ) => {
                let sleep_minute = sleep_time.minute;
                let wake_minute = wake_time.minute;
                for min in sleep_minute..wake_minute {
                    // TODO: case a min as a usize?
                    schedule[min as usize] += 1;
                }
            }
            (None, None) => break,
            _ => panic!("invalid events!"),
        }
    }
    schedule
}

type GuardId = u32;
struct Guard {
    id: GuardId,
    sleeps: SleepSchedule,
}

#[derive(PartialEq, Debug)]
enum EventKind {
    GuardStart { guard_id: GuardId },
    Asleep,
    Wakeup,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
struct DateTime {
    year: u32,
    month: u8,
    day: u16,
    hour: u8,
    minute: u8,
}

struct Event {
    kind: EventKind,
    timestamp: DateTime,
}

impl FromStr for Event {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                    \[
                    # year, month, day, time
                    (?P<year>[0-9]{4})-(?P<month>[0-9]{2})-(?P<day>[0-9]{2})
                    \s+
                    # hour, minute
                    (?P<hour>[0-9]{2}):(?P<minute>[0-9]{2})
                    \]\s+
                    # Event type, and guard number, if available:
                    (?:Guard\ \#(?P<id>[0-9]+)\ begins\ shift|(?P<sleep>.*))
                    "
            )
            .unwrap();
        }

        let caps = RE.captures(s).unwrap();

        let datetime = DateTime {
            year: caps["year"].parse()?,
            month: caps["month"].parse()?,
            // equivalent way of getting the group:
            day: caps.name("day").unwrap().as_str().parse()?,
            hour: caps["hour"].parse()?,
            minute: caps["minute"].parse()?,
        };

        use EventKind::*;

        let event_type = if let Some(guard_id) = caps.name("id") {
            GuardStart {
                guard_id: guard_id.as_str().parse()?,
            }
        } else if let Some(sleep) = caps.name("sleep") {
            if sleep.as_str() == "falls asleep" {
                Asleep
            } else if sleep.as_str() == "wakes up" {
                Wakeup
            } else {
                panic!("invalid sleep statement")
            }
        } else {
            panic!("invalid event type");
        };

        let event = Event {
            kind: event_type,
            timestamp: datetime,
        };

        Ok(event)
    }
}

#[test]
fn test_find_guard() -> Result<()> {
    let s = "\
[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-03 00:24] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-04 00:36] falls asleep
[1518-11-04 00:46] wakes up
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up\
";
    let guards = get_guards(&s)?;
    assert_eq!(find_sleepiest_guard_minute_product(&guards)?, 240);

    assert_eq!(find_guard_minute_most_frequently_asleep(&guards)?, 4455);
    println!("find_guard passes!");
    Ok(())
}
