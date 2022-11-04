use std::ops::Range;

use chrono::{Duration, NaiveDate};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use wasm_rs_dbg::dbg;

use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use common::TestsuiteResult;

#[derive(Debug, Clone)]
enum Error {
    CacheAPI,
}

impl From<reqwasm::Error> for Error {
    fn from(_: reqwasm::Error) -> Self {
        Error::CacheAPI
    }
}

async fn fetch_testsuites(base_url: &str) -> Result<Vec<String>, reqwasm::Error> {
    let url = format!("{base_url}/api/testsuites");
    let response = reqwasm::http::Request::get(&url).send().await?;
    let testsuites: Vec<String> = response.json().await?;

    Ok(testsuites)
}

async fn fetch_results(base_url: &str, key: &str) -> Result<Vec<TestsuiteResult>, reqwasm::Error> {
    let url = format!("{base_url}/api/testsuites/{key}");
    let response = reqwasm::http::Request::get(&url).send().await?;
    let testsuites: Vec<TestsuiteResult> = response.json().await?;

    Ok(testsuites)
}

enum CacheMsg {
    FetchKeys,
    UpdateKeys(Vec<String>),
    FetchKeyResult(String),
    UpdateResults(String, Vec<TestsuiteResult>),
}

struct CacheModel {
    url: &'static str,
    canvas: NodeRef,
    keys: Vec<String>,
    current_key: String,
    results: Vec<TestsuiteResult>,
}

impl CacheModel {
    fn graph(&self, backend: CanvasBackend) {
        let testsuites = &self.results;

        dbg!(testsuites);

        if testsuites.is_empty() {
            dbg!("empty testsuites");
            return;
        }

        let root = backend.into_drawing_area();
        root.fill(&WHITE).unwrap();

        let range = get_date_range(testsuites);
        let limits = get_limits(testsuites);

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.current_key, ("sans-serif", 20).into_font())
            .margin(5u32)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(range.0..range.1 + Duration::days(1), limits)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(
                (range.clone()).filter_map(|date| {
                    // This skips days which do not exist
                    let to_show = testsuites.iter().find(|run| run.date == date)?;

                    Some((to_show.date, to_show.results.passes))
                }),
                GREEN,
            ))
            .unwrap()
            .label("passes")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

        chart
            .draw_series(LineSeries::new(
                (range.clone()).filter_map(|date| {
                    // This skips days which do not exist
                    let to_show = testsuites.iter().find(|run| run.date == date)?;

                    Some((to_show.date, to_show.results.failures))
                }),
                RED,
            ))
            .unwrap()
            .label("failures")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .draw_series(LineSeries::new(
                (range).filter_map(|date| {
                    // This skips days which do not exist
                    let to_show = testsuites.iter().find(|run| run.date == date)?;

                    Some((to_show.date, to_show.results.tests))
                }),
                BLACK,
            ))
            .unwrap()
            .label("number of tests")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();
    }
}

impl Component for CacheModel {
    type Message = CacheMsg;

    // FIXME: Should we store anything here?
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(CacheMsg::FetchKeys);

        CacheModel {
            // FIXME: Use environment variable instead?
            url: "http://127.0.0.1:8000",
            canvas: NodeRef::default(),
            keys: vec![],
            current_key: String::new(),
            results: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CacheMsg::FetchKeys => {
                let url = String::from(self.url);
                ctx.link().send_future(async move {
                    // FIXME: No unwrap
                    let keys = fetch_testsuites(&url).await.unwrap();
                    dbg!(&keys);
                    CacheMsg::UpdateKeys(keys)
                });
                true
            }
            CacheMsg::FetchKeyResult(key) => {
                let url = String::from(self.url);
                ctx.link().send_future(async move {
                    // FIXME: No unwrap
                    let results = fetch_results(&url, &key).await.unwrap();
                    dbg!(&results);
                    CacheMsg::UpdateResults(key, results)
                });
                true
            }
            CacheMsg::UpdateKeys(keys) => {
                self.keys = keys;
                true
            }
            CacheMsg::UpdateResults(key, results) => {
                self.results = results;
                self.current_key = key;
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        canvas.set_width(720);
        canvas.set_height(480);

        let backend: CanvasBackend = CanvasBackend::with_canvas_object(canvas).unwrap();
        self.graph(backend);
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let items: Html = self.keys.iter().map(|key| {
            let key = String::from(key);
            let button_text = key.clone();
            html! {
                <button onclick={ctx.link().callback(move |_| CacheMsg::FetchKeyResult(key.clone()))}> { button_text } </button>

            }
        }).collect();

        html! {
            <div>
                <h1> { "Rust-GCC testing dashboard" } </h1>
                <p> { "Welcome! Here you'll find a collection of the results we're accumulating while testing our compiler on various test-suites" } </p>
                <ul class="item-list">
                { items }
                </ul>
                <canvas ref={ self.canvas.clone() } />
                <div class="footer">
                <p>{"Made in Rust with"}<a href="https://github.com/yewstack/yew" target="_blank">{" Yew"}</a></p>
                </div>
            </div>
        }
    }
}

#[derive(Clone)]
struct DateRange(NaiveDate, NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0;
            self.0 = next.succ_opt()?;

            Some(next)
        } else {
            None
        }
    }
}

fn get_date_range(testsuites: &[TestsuiteResult]) -> DateRange {
    // FIXME: This can panic if the array is empty
    let mut lo: NaiveDate = testsuites[0].date;
    let mut hi: NaiveDate = testsuites[0].date;

    for run in testsuites.iter() {
        if run.date < lo {
            lo = run.date;
        }
        if run.date > hi {
            hi = run.date;
        }
    }

    DateRange(lo, hi)
}

fn get_limits(testsuites: &[TestsuiteResult]) -> Range<u64> {
    // FIXME: Don't unwrap here
    0..testsuites
        .iter()
        .map(|run| run.results.tests)
        .max()
        .unwrap()
        + 1
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    yew::Renderer::<CacheModel>::new().render();
}
