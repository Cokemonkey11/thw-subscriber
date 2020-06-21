use crate::util::{RandomSignal, SinSignal, StatefulList, TabsState};

use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use clipboard::{ClipboardContext, ClipboardProvider};

pub struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        for _ in 0..self.tick_rate {
            self.points.remove(0);
        }
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

pub struct Signals {
    pub sin1: Signal<SinSignal>,
    pub sin2: Signal<SinSignal>,
    pub window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        self.sin1.on_tick();
        self.sin2.on_tick();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

pub struct Server<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub coords: (f64, f64),
    pub status: &'a str,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ThwDatum {
    pub title: String,
    pub forum: String,
    pub href: String,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_chart: bool,
    pub next_update: Instant, // seconds
    pub progress: f64,
    pub refresh_sender: mpsc::Sender<()>,
    pub results_receiver: mpsc::Receiver<ThwDatum>,
    pub sparkline: Signal<RandomSignal>,
    pub tasks: StatefulList<ThwDatum>,
    pub logs: StatefulList<(&'a str, &'a str)>,
    pub signals: Signals,
    pub barchart: Vec<(&'a str, u64)>,
    pub servers: Vec<Server<'a>>,
    pub enhanced_graphics: bool,
    pub errors: Vec<String>,
    pub filters: Vec<String>,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        enhanced_graphics: bool,
        refresh_sender: mpsc::Sender<()>,
        results_receiver: mpsc::Receiver<ThwDatum>,
    ) -> App<'a> {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 3.0, 18.0);
        let sin1_points = sin_signal.by_ref().take(100).collect();
        let mut sin_signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let sin2_points = sin_signal2.by_ref().take(200).collect();
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["New posts", "Filters"]),
            show_chart: true,
            next_update: Instant::now() + Duration::from_secs(60),
            progress: 0.0,
            refresh_sender,
            results_receiver,
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
                tick_rate: 1,
            },
            tasks: StatefulList::new(),
            logs: StatefulList::new(),

            signals: Signals {
                sin1: Signal {
                    source: sin_signal,
                    points: sin1_points,
                    tick_rate: 5,
                },
                sin2: Signal {
                    source: sin_signal2,
                    points: sin2_points,
                    tick_rate: 10,
                },
                window: [0.0, 20.0],
            },
            barchart: vec![],
            servers: vec![
                Server {
                    name: "NorthAmerica-1",
                    location: "New York City",
                    coords: (40.71, -74.00),
                    status: "Up",
                },
                Server {
                    name: "Europe-1",
                    location: "Paris",
                    coords: (48.85, 2.35),
                    status: "Failure",
                },
                Server {
                    name: "SouthAmerica-1",
                    location: "São Paulo",
                    coords: (-23.54, -46.62),
                    status: "Up",
                },
                Server {
                    name: "Asia-1",
                    location: "Singapore",
                    coords: (1.35, 103.86),
                    status: "Up",
                },
            ],
            enhanced_graphics,
            errors: vec![],
            filters: vec![
                "Maps",
                "Models",
                "Site Discussion",
                "Multiplayer LFG",
                "Skins",
                "Something Else",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
        }
    }

    pub fn on_up(&mut self) {
        self.tasks.previous();
    }

    pub fn get_uri(&mut self) -> Option<String> {
        self.tasks.state.selected().and_then(|idx| {
            self.tasks
                .items
                .iter()
                .nth(idx)
                .map(|thw| format!("https://www.hiveworkshop.com/{}", thw.href))
        })
    }

    pub fn on_down(&mut self) {
        self.tasks.next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            't' => {
                self.show_chart = !self.show_chart;
            }
            'c' => {
                if let Some(uri) = self.get_uri() {
                    let cb: Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                    match cb {
                        Ok(mut cb) => {
                            cb.set_contents(uri).expect("failed to set clipboard");
                        }
                        Err(e) => {
                            self.errors.push(format!("{:?} - {:?}\n", e, e.to_string()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.progress =
            (Instant::now() - (self.next_update - Duration::from_secs(60))).as_secs() as f64 / 60.0;
        if self.progress >= 1.0 {
            self.next_update = Instant::now() + Duration::from_secs(60);
            self.progress = 0.0;
            self.refresh_sender
                .send(())
                .expect("Failed to send a refresh");
        }

        let new_res = self.results_receiver.try_recv();
        if let Ok(res) = new_res {
            if !self.filters.contains(&res.forum) {
                self.tasks.items.insert(res);
            }
        }

        self.sparkline.on_tick();
        self.signals.on_tick();
    }
}
