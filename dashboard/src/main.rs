use std::ops::Range;

use chrono::NaiveDate;
use plotters::{coord::Shift, prelude::*};
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_rs_dbg::dbg;

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
    let url = format!("{}/api/testsuites", base_url);
    let response = reqwasm::http::Request::get(&url).send().await?;
    let testsuites: Vec<String> = response.json().await?;

    Ok(testsuites)
}

async fn fetch_results(base_url: &str, key: &str) -> Result<Vec<TestsuiteResult>, reqwasm::Error> {
    let url = format!("{}/api/testsuites/{}", base_url, key);
    let response = reqwasm::http::Request::get(&url).send().await?;
    let testsuites: Vec<TestsuiteResult> = response.json().await?;

    Ok(testsuites)
}

enum CacheMsg {
    FetchKeys,
    UpdateKeys(Vec<String>),
    FetchKeyResult(String),
    UpdateResults(Vec<TestsuiteResult>),
}

struct CacheModel {
    url: &'static str,
    keys: Vec<String>,
    results: Vec<TestsuiteResult>,
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
            keys: vec![],
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
                    CacheMsg::UpdateResults(results)
                });
                true
            }
            CacheMsg::UpdateKeys(keys) => {
                self.keys = keys;
                true
            }
            CacheMsg::UpdateResults(results) => {
                self.results = results;
                true
            }
        }
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
            <ul class="item-list">
            { items }
            </ul>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <h3>{ "Hello there!" }</h3>
            <canvas id="canvas"
            width="500" height="300"></canvas>
        </div>
    }
}

fn get_root() -> DrawingArea<CanvasBackend, Shift> {
    // FIXME: Don't unwrap
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area()
}

#[derive(Clone, Copy)]
struct DateRange(NaiveDate, NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0;
            self.0 = next.succ();

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

    testsuites.iter().for_each(|run| {
        if run.date < lo {
            lo = run.date
        };
        if run.date > hi {
            hi = run.date
        }
    });

    DateRange(lo, hi)
}

fn get_limits(testsuites: &[TestsuiteResult]) -> Range<u64> {
    // FIXME: Don't unwrap here
    0..testsuites
        .iter()
        .map(|run| run.results.tests)
        .max()
        .unwrap()
        + 1 // for extra comfortable viewing
}

fn graph(testsuites: &[TestsuiteResult]) {
    let root = get_root();
    root.fill(&WHITE).unwrap();

    let testsuites: Vec<TestsuiteResult> = testsuites
        .iter()
        .filter(|run| run.name == "blake3")
        .cloned()
        .collect();

    let range = get_date_range(&testsuites);
    let limits = get_limits(&testsuites);

    let mut chart = ChartBuilder::on(&root)
        .caption("testsuites", ("sans-serif", 50).into_font())
        .margin(5u32)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(0..testsuites.len(), limits)
        .unwrap();

    chart.configure_mesh().draw().unwrap();
    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.passes)
            }),
            &GREEN,
        ))
        .unwrap()
        .label("passes")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.failures)
            }),
            &RED,
        ))
        .unwrap()
        .label("failures")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(LineSeries::new(
            (range).enumerate().map(|(i, date)| {
                // There will always be a unique run per day
                // FIXME: Right?
                let to_show = testsuites.iter().find(|run| run.date == date).unwrap();

                (i, to_show.results.tests)
            }),
            &BLACK,
        ))
        .unwrap()
        .label("total test cases")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    yew::start_app::<CacheModel>();
}
