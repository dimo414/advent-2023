use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use std::str::FromStr;
use anyhow::*;
use lazy_regex::regex_captures;

fn main() -> Result<()> {
    let mut input: Configuration = include_str!("input.txt").parse()?;

    for _ in 0..1000 {
        input.press_button()?;
    }
    let low = input.counts[&Pulse::Low];
    let high = input.counts[&Pulse::High];
    println!("Pulses (low:{} high:{}): {}", low, high, low * high);

    while input.conjunction_cycles().iter().filter(|&&c| c > 1).count() < 4 {
        input.press_button()?;
    }
    println!("Cycle Length: {}", fold_lcm(input.conjunction_cycles()));

    Ok(())
}

// Borrowed from Day 8, opting not to generalize atm
fn fold_lcm(inputs: impl IntoIterator<Item=u64>) -> u64 {
    inputs.into_iter().fold(1, |lcm,v| num::integer::lcm(lcm, v))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Pulse {
    Low, High,
}

#[derive(Debug)]
enum Module {
    FlipFlop(bool),
    Conjunction(BTreeMap<Rc<str>, Pulse>, Option<u64>),
    Broadcaster,
    Output(Vec<Vec<Pulse>>),
    Rx,
}

impl Module {
    fn receive(&mut self, source: &Rc<str>, pulse: Pulse, press: u64) -> Option<Pulse> {
        match self {
            Module::FlipFlop(state) => {
                // Flip-flop modules (prefix %) are either on or off; they are initially off. If a
                // flip-flop module receives a high pulse, it is ignored and nothing happens.
                // However, if a flip-flop module receives a low pulse, it flips between on and off.
                // If it was off, it turns on and sends a high pulse. If it was on, it turns off and
                // sends a low pulse.
                if pulse == Pulse::High { None } else {
                    *state = !*state;
                    Some(if *state { Pulse::High } else { Pulse::Low })
                }
            },
            Module::Conjunction(inputs, first_low) => {
                // Conjunction modules (prefix &) remember the type of the most recent pulse
                // received from each of their connected input modules; they initially default to
                // remembering a low pulse for each input. When a pulse is received, the conjunction
                // module first updates its memory for that input. Then, if it remembers high pulses
                // for all inputs, it sends a low pulse; otherwise, it sends a high pulse.
                debug_assert!(inputs.contains_key(source));
                inputs.insert(source.clone(), pulse);
                if inputs.values().all(|&v| v == Pulse::High) {
                    if first_low.is_none() {
                        *first_low = Some(press);
                    }
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            },
            Module::Broadcaster => {
                Some(pulse)
            },
            Module::Output(seen) => {
                seen.last_mut().expect("Non-empty").push(pulse);
                None
            },
            Module::Rx => None,
        }
    }
}

#[derive(Debug)]
struct Configuration {
    presses: u64,
    button: Rc<str>,
    broadcaster: Rc<str>,
    output: Rc<str>,
    dest: BTreeMap<Rc<str>, Vec<Rc<str>>>,
    modules: BTreeMap<Rc<str>, Module>,
    counts: HashMap<Pulse, u64>,
}

impl Configuration {
    fn press_button(&mut self) -> Result<HashMap<Pulse, u64>> {
        self.presses += 1;
        if let Some(Module::Output(output)) = self.modules.get_mut(&self.output) {
            output.push(Vec::new());
        }
        let mut counts = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back((self.button.clone(), self.broadcaster.clone(), Pulse::Low));
        while !queue.is_empty() {
            let (source, dest, pulse) = queue.pop_front().expect("Non-empty");
            // println!("{} -{:?}-> {}", source, pulse, dest);
            *counts.entry(pulse).or_insert(0) += 1;
            // entry won't be present for untyped modules that are just sinks
            if let Some(module) = self.modules.get_mut(&dest) {
                if let Some(output) = module.receive(&source, pulse, self.presses) {
                    if let Some(next) = self.dest.get(&dest) {
                        for next_dest in next {
                            queue.push_back((dest.clone(), next_dest.clone(), output));
                        }
                    }
                }
            }
        }

        for (pulse, count) in counts.iter() {
            *self.counts.entry(*pulse).or_insert(0) += count;
        }
        Ok(counts)
    }

    #[cfg(test)]
    fn flip_flops(&self) -> Vec<(&str, bool)> {
        use itertools::Itertools;

        self.modules.iter()
            .flat_map(|(n, m)| if let Module::FlipFlop(state) = m {
                    Some((n.as_ref(), *state))
                } else { None })
            .sorted_by(|(a,_), (b,_)| a.cmp(b))
            .collect()
    }

    fn conjunction_cycles(&self) -> Vec<u64> {
        self.modules.values()
            .filter_map(|m| if let Module::Conjunction(_, Some(cycle)) = m { Some(*cycle) } else { None })
            .collect()
    }

    #[cfg(test)]
    fn output(&self) -> &[Vec<Pulse>] {
        if let Some(Module::Output(output)) = self.modules.get(&self.output) {
            &output
        } else { panic!("Missing/invalid output module") }
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut intern: BTreeMap<Rc<str>, Rc<str>> = BTreeMap::new();
        let mut intern = |s: &str| -> Rc<str> {
            if let Some(ret) = intern.get(s) {
                return ret.clone();
            }
            let rc: Rc<str> = Rc::from(s);
            intern.insert(rc.clone(), rc.clone());
            rc
        };

        let button = intern("button");
        let broadcaster = intern("broadcaster");
        let output = intern("output");
        let rx = intern("rx");
        let mut dest = BTreeMap::new();
        let mut source = BTreeMap::new();
        let mut modules = BTreeMap::new();

        for line in s.lines() {
            let (_, marker, name, dests) = regex_captures!(r"([%&]?)(.*) -> (.*)", line).with_context(|| format!("Invalid: {}", s))?;
            let name = intern(name);
            ensure!(!marker.is_empty() || name == broadcaster, "Unexpected module marker for {}", line);
            let module = match marker {
                "" => Ok(Module::Broadcaster),
                "%" => Ok(Module::FlipFlop(false)),
                "&" => Ok(Module::Conjunction(BTreeMap::new(), None)),
                _ => bail!("Invalid marker {}", marker),
            }?;

            let dests: Vec<_> = dests.split(", ").map(|s| intern(s)).collect();
            for d in dests.iter() {
                source.entry(d.clone()).or_insert_with(Vec::new).push(name.clone());
            }
            dest.insert(name.clone(), dests);
            modules.insert(name, module);
        }

        for (name, module) in modules.iter_mut() {
            if let Module::Conjunction(map, _) = module {
                for s in source.get(name).expect("Must be present").iter() {
                    map.insert(s.clone(), Pulse::Low);
                }
            }
        }

        if source.contains_key(&output) {
            modules.insert(output.clone(), Module::Output(Vec::new()));
        }
        if source.contains_key(&rx) {
            modules.insert(rx.clone(), Module::Rx);
        }

        Ok(Configuration{ presses: 0, button, broadcaster, output, dest, modules, counts: HashMap::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_all_flip_flops(config: &Configuration, expected: bool) {
        for (name, module) in config.modules.iter() {
            if let Module::FlipFlop(state) = module {
                assert_eq!(*state, expected, "{} is not {}", name, if expected { "on" } else { "off" });
            }
        }
    }

    #[test]
    fn check_input() { include_str!("input.txt").parse::<Configuration>().unwrap(); }

    #[test]
    fn example1() {
        let mut config: Configuration = include_str!("example1.txt").parse().unwrap();
        let pulses1 = config.press_button().unwrap();
        assert_eq!(pulses1, [(Pulse::Low, 8), (Pulse::High, 4)].into_iter().collect());
        // After this sequence, the flip-flop modules all end up off
        assert_all_flip_flops(&config, false);

        // so pushing the button again repeats the same sequence.
        let pulses2 = config.press_button().unwrap();
        assert_all_flip_flops(&config, false);
        assert_eq!(pulses1, pulses2);
    }

    #[test]
    fn example2() {
        let mut config: Configuration = include_str!("example2.txt").parse().unwrap();
        let _pulses1 = config.press_button().unwrap();
        assert_all_flip_flops(&config, true);
        assert_eq!(config.flip_flops(), [("a", true), ("b", true)]);
        assert_eq!(config.output(), [vec![Pulse::High, Pulse::Low]]);

        let _pulses2 = config.press_button().unwrap();
        assert_eq!(config.flip_flops(), [("a", false), ("b", true)]);
        assert_eq!(config.output(), [vec![Pulse::High, Pulse::Low], vec![Pulse::High]]);

        let _pulses3 = config.press_button().unwrap();
        assert_eq!(config.flip_flops(), [("a", true), ("b", false)]);
        assert_eq!(config.output(), [vec![Pulse::High, Pulse::Low], vec![Pulse::High], vec![Pulse::Low, Pulse::High]]);

        let _pulses4 = config.press_button().unwrap();
        assert_eq!(config.flip_flops(), [("a", false), ("b", false)]);
        assert_eq!(config.output(), [vec![Pulse::High, Pulse::Low], vec![Pulse::High], vec![Pulse::Low, Pulse::High], vec![Pulse::High]]);

    }

    parameterized_test::create!{ example, (input, low, high), {
        let mut config: Configuration = input.parse().unwrap();
        for _ in 0..1000 {
            config.press_button().unwrap();
        }
        assert_eq!(config.counts, [(Pulse::Low, low), (Pulse::High, high)].into_iter().collect());
    } }
    example! {
        a: (include_str!("example1.txt"), 8000, 4000),
        b: (include_str!("example2.txt"), 4250, 2750),
        // same as example2 but "output" is named "rx"
        b_rx: (include_str!("example2.txt").replace("output", "rx"), 4250, 2750),
    }
}
